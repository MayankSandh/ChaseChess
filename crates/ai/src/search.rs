use engine::{Board, Move, piece_type, is_empty, PAWN};
use crate::{evaluation::*, types::*};
use crate::transposition::*;
use crate::piece_square_tables::initialize_pst;

pub struct SearchEngine {
    pub nodes_searched: u64,
    transposition_table: TranspositionTable,
    advanced_logging_enabled: bool,
    log_callback: Option<Box<dyn FnMut(&str)>>,  // ADD THIS MISSING FIELD
}

impl SearchEngine {
    pub fn new() -> Self {
        initialize_pst();
        Self {
            nodes_searched: 0,
            transposition_table: TranspositionTable::new(64),
            advanced_logging_enabled: false,
            log_callback: None,  // ADD THIS INITIALIZATION
        }
    }
    
    pub fn set_advanced_logging<F>(&mut self, enabled: bool, callback: Option<F>) 
    where 
        F: FnMut(&str) + 'static,
    {
        self.advanced_logging_enabled = enabled;
        self.log_callback = callback.map(|f| Box::new(f) as Box<dyn FnMut(&str)>);
    }
    
    pub fn search(&mut self, board: &mut Board, depth: u32) -> SearchResult {
        self.nodes_searched = 0;
        self.transposition_table.new_search();
        
        if self.advanced_logging_enabled {
            self.log(&format!("🚀 Starting search at depth {} (Advanced logging enabled)", depth));
        }
        
        let (best_move, evaluation) = self.alphabeta_root(board, depth as i32);
        
        if self.advanced_logging_enabled {
            self.log(&format!("🏆 Search completed. Best move: {:?}, Evaluation: {}", 
                best_move.map(|m| self.move_to_notation(m)).unwrap_or("None".to_string()), 
                evaluation));
        }
        
        SearchResult {
            best_move,
            evaluation,
            depth,
            nodes_searched: self.nodes_searched,
        }
    }
    
    fn alphabeta_root(&mut self, board: &mut Board, depth: i32) -> (Option<Move>, i32) {
        let mut moves = board.get_all_legal_moves();
        if moves.is_empty() {
            let eval = if board.is_in_check() { -MATE_SCORE } else { 0 };
            return (None, eval);
        }

        self.order_moves(board, &mut moves);
        
        let mut best_score = -MATE_SCORE - 1;
        let mut best_move = None;
        let mut alpha = -MATE_SCORE - 1;
        let beta = MATE_SCORE + 1;
        
        if self.advanced_logging_enabled {
            let game_phase = crate::piece_square_tables::calculate_game_phase(board);
            let pattern = crate::piece_square_tables::detect_endgame_pattern(board);
            self.log(&format!("📊 Root Analysis - {} candidate moves, Game phase: {} ({:?})", 
                moves.len(), game_phase, pattern));
            
            // Log position evaluation breakdown
            let (material, pst, total) = evaluate_position_detailed(board);
            self.log(&format!("📊 Position Evaluation: Material={}, PST={}, Total={}", 
                material, pst, total));
        }
        
        for (move_index, &mv) in moves.iter().enumerate() {
            if let Ok(_) = board.try_make_move(mv) {
                if self.advanced_logging_enabled {
                    self.log(&format!("🎯 [{}/{}] Evaluating move: {} ({})", 
                        move_index + 1, moves.len(), 
                        self.move_to_notation(mv),
                        self.move_description(board, mv)));
                }
                
                let score = -self.alphabeta(board, depth - 1, -beta, -alpha, 1);
                
                if let Err(_) = board.undo_move() { break; }
                
                if score > best_score {
                    let old_best = best_score;
                    best_score = score;
                    best_move = Some(mv);
                    
                    if self.advanced_logging_enabled {
                        self.log(&format!("    ⭐ NEW BEST! {} → {} (Δ: +{})", 
                            old_best, best_score, best_score - old_best));
                    }
                }
                
                if score > alpha {
                    alpha = score;
                    if self.advanced_logging_enabled {
                        self.log(&format!("    📈 Alpha raised to {}", alpha));
                    }
                }
                
                if self.advanced_logging_enabled {
                    self.log(&format!("    📊 Move {} scored: {}", 
                        self.move_to_notation(mv), score));
                }
            }
        }
        
        (best_move, best_score)
    }

    fn alphabeta(&mut self, board: &mut Board, depth: i32, mut alpha: i32, beta: i32, ply: u32) -> i32 {
        self.nodes_searched += 1;
        
        let indent = "  ".repeat(ply as usize);
        
        // Probe transposition table
        let hash = self.transposition_table.get_hash(board);
        if let Some((tt_score, tt_move)) = self.transposition_table.probe(hash, depth, alpha, beta) {
            if self.advanced_logging_enabled {
                self.log(&format!("{}🔍 TT HIT! Depth:{}, Score:{}, Move:{}", 
                    indent, depth, tt_score, 
                    tt_move.map(|m| self.move_to_notation(m)).unwrap_or("None".to_string())));
            }
            return tt_score;
        }

        if depth <= 0 {
            if self.advanced_logging_enabled {
                self.log(&format!("{}⚡ Entering quiescence search (depth exhausted)", indent));
            }
            let eval = self.quiescence_search(board, alpha, beta, ply + 1);
            self.transposition_table.store(hash, depth, eval, None, NodeType::Exact);
            return eval;
        }
        
        let mut moves = board.get_all_legal_moves();
        if moves.is_empty() {
            let eval = if board.is_in_check() { -MATE_SCORE } else { 0 };
            if self.advanced_logging_enabled {
                self.log(&format!("{}🔚 Terminal: {} (eval: {})", indent, 
                    if board.is_in_check() { "CHECKMATE" } else { "STALEMATE" }, eval));
            }
            self.transposition_table.store(hash, depth, eval, None, NodeType::Exact);
            return eval;
        }

        self.order_moves(board, &mut moves);
        let original_alpha = alpha;
        let mut best_move = None;
        
        if self.advanced_logging_enabled && moves.len() > 1 {
            self.log(&format!("{}📋 Searching {} moves at depth {}", indent, moves.len(), depth));
        }

        for (i, &mv) in moves.iter().enumerate() {
            if let Ok(_) = board.try_make_move(mv) {
                let is_capture = !is_empty(board.get_piece(mv.to));
                let is_check = board.is_in_check();
                
                if self.advanced_logging_enabled {
                    let mut move_info = format!("{}├─ [{}/{}] {}", 
                        indent, i + 1, moves.len(), self.move_to_notation(mv));
                    
                    if is_capture {
                        move_info.push_str(" [CAPTURE]");
                    }
                    if is_check {
                        move_info.push_str(" [CHECK]");
                    }
                    move_info.push_str(&format!(" (α:{}, β:{})", alpha, beta));
                    
                    self.log(&move_info);
                }
                
                let score = -self.alphabeta(board, depth - 1, -beta, -alpha, ply + 1);
                
                if let Err(_) = board.undo_move() { break; }
                
                if score > alpha {
                    alpha = score;
                    best_move = Some(mv);
                    if self.advanced_logging_enabled {
                        self.log(&format!("{}    ↗️ Alpha improved: {} → {}", 
                            indent, original_alpha, alpha));
                    }
                }
                
                if alpha >= beta {
                    if self.advanced_logging_enabled {
                        self.log(&format!("{}    ✂️ BETA CUTOFF! (α:{} ≥ β:{})", 
                            indent, alpha, beta));
                    }
                    self.transposition_table.store(hash, depth, beta, best_move, NodeType::LowerBound);
                    return beta;
                }
            }
        }
        
        let node_type = if alpha <= original_alpha {
            NodeType::UpperBound
        } else {
            NodeType::Exact
        };
        
        self.transposition_table.store(hash, depth, alpha, best_move, node_type);
        alpha
    }

    fn quiescence_search(&mut self, board: &mut Board, mut alpha: i32, beta: i32, ply: u32) -> i32 {
        self.nodes_searched += 1;
        let indent = "  ".repeat(ply as usize);
        
        // Stand pat evaluation with detailed breakdown
        let (material, pst, stand_pat) = evaluate_position_detailed(board);
        
        if self.advanced_logging_enabled {
            self.log(&format!("{}🎯 Quiescence stand-pat: {} (Material:{}, PST:{})", 
                indent, stand_pat, material, pst));
        }
        
        if stand_pat >= beta {
            if self.advanced_logging_enabled {
                self.log(&format!("{}✂️ Stand-pat β-cutoff ({} ≥ {})", indent, stand_pat, beta));
            }
            return beta;
        }
        
        if stand_pat > alpha {
            alpha = stand_pat;
            if self.advanced_logging_enabled {
                self.log(&format!("{}📈 Stand-pat α-raise: {}", indent, alpha));
            }
        }
        
        let captures = self.get_capture_moves(board);
        if captures.is_empty() {
            if self.advanced_logging_enabled {
                self.log(&format!("{}🔚 No captures, returning stand-pat", indent));
            }
            return alpha;
        }
        
        if self.advanced_logging_enabled {
            self.log(&format!("{}⚔️ Quiescence: {} captures to analyze", indent, captures.len()));
        }

        for &mv in &captures {
            if let Ok(_) = board.try_make_move(mv) {
                if self.advanced_logging_enabled {
                    self.log(&format!("{}  ⚔️ Capture: {}", indent, self.move_to_notation(mv)));
                }
                
                let score = -self.quiescence_search(board, -beta, -alpha, ply + 1);
                
                if let Err(_) = board.undo_move() { break; }
                
                if score >= beta {
                    if self.advanced_logging_enabled {
                        self.log(&format!("{}  ✂️ Capture β-cutoff", indent));
                    }
                    return beta;
                }
                
                if score > alpha {
                    alpha = score;
                    if self.advanced_logging_enabled {
                        self.log(&format!("{}  📈 Capture α-raise: {}", indent, alpha));
                    }
                }
            }
        }
        
        alpha
    }

    fn order_moves(&self, board: &Board, moves: &mut Vec<Move>) {
        moves.sort_by_key(|&mv| {
            let from_piece = board.get_piece(mv.from);
            let to_piece = board.get_piece(mv.to);
            let mut score = 0;

            // Captures: MVV-LVA
            if !is_empty(to_piece) {
                let victim_value = PIECE_VALUES[piece_type(to_piece) as usize];
                let attacker_value = PIECE_VALUES[piece_type(from_piece) as usize];
                score += 10000 + victim_value - attacker_value;
            }

            // Promotions
            if piece_type(from_piece) == PAWN {
                let from_rank = mv.from.rank();
                let to_rank = mv.to.rank();
                if (from_rank == 6 && to_rank == 7) || (from_rank == 1 && to_rank == 0) {
                    score += 9000;
                }
            }

            -score
        });
    }

    fn get_capture_moves(&self, board: &Board) -> Vec<Move> {
        board.get_all_legal_moves()
            .into_iter()
            .filter(|&mv| !is_empty(board.get_piece(mv.to)))
            .collect()
    }
    
    fn move_to_notation(&self, mv: Move) -> String {
        let from_file = (b'a' + mv.from.file()) as char;
        let from_rank = (b'1' + mv.from.rank()) as char;
        let to_file = (b'a' + mv.to.file()) as char;
        let to_rank = (b'1' + mv.to.rank()) as char;
        format!("{}{}{}{}", from_file, from_rank, to_file, to_rank)
    }
    
    fn move_description(&self, board: &Board, mv: Move) -> String {
        let piece = board.get_piece(mv.from);
        let piece_name = match piece_type(piece) {
            PAWN => "Pawn",
            2 => "Knight", // KNIGHT
            3 => "Bishop", // BISHOP  
            4 => "Rook",   // ROOK
            5 => "Queen",  // QUEEN
            6 => "King",   // KING
            _ => "Unknown"
        };
        
        let target_piece = board.get_piece(mv.to);
        if is_empty(target_piece) {
            piece_name.to_string()
        } else {
            format!("{} captures", piece_name)
        }
    }
    
    fn log(&mut self, message: &str) {
        if let Some(ref mut callback) = self.log_callback {
            callback(message);
        }
    }
}

impl Default for SearchEngine {
    fn default() -> Self {
        Self::new()
    }
}

use engine::{Board, Move};
use crate::{evaluation::*, types::*};
use crate::transposition::*;
use crate::piece_square_tables::get_pst;
use crate::AILoggerExtensions;

pub struct SearchEngine {
    pub nodes_searched: u64,
    transposition_table: TranspositionTable,
    logger: Option<std::rc::Rc<std::cell::RefCell<engine::ChessLogger>>>,
}

impl SearchEngine {
    pub fn new() -> Self {
        get_pst();
        Self { 
            nodes_searched: 0,
            transposition_table: TranspositionTable::new(64), 
            logger: None,
        }
    }

    pub fn search(&mut self, board: &mut Board, depth: u32) -> SearchResult {
        self.nodes_searched = 0;
        self.transposition_table.new_search(); // Age increment for new search
        let (best_move, evaluation) = self.alphabeta_root(board, depth as i32);
        
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
    
        // LOG: Show all legal moves before ordering
        if let Some(logger) = &self.logger {
            logger.borrow_mut().log_search_root_start(depth as u32, &moves);
        }
    
        // Order moves for better alpha-beta efficiency
        self.order_moves(board, &mut moves);
        
        // LOG: Show move ordering results
        if let Some(logger) = &self.logger {
            logger.borrow_mut().log_move_ordering_result(&moves, board);
        }
    
        let mut best_score = -MATE_SCORE - 1;
        let mut best_move = None;
        let mut alpha = -MATE_SCORE - 1;
        let beta = MATE_SCORE + 1;
    
        for (move_num, &mv) in moves.iter().enumerate() {
            if let Ok(_) = board.try_make_move(mv) {
                // LOG: Start analyzing this move
                if let Some(logger) = &self.logger {
                    logger.borrow_mut().log_root_move_start(mv, move_num + 1, moves.len(), alpha, beta);
                }
    
                let score = -self.alphabeta(board, depth - 1, -beta, -alpha);
                if let Err(_) = board.undo_move() { break; }
    
                // LOG: Root move result
                if let Some(logger) = &self.logger {
                    logger.borrow_mut().log_root_move_result(mv, score, alpha, beta);
                }
    
                if score > best_score {
                    best_score = score;
                    best_move = Some(mv);
                    
                    if let Some(logger) = &self.logger {
                        if score > alpha {
                            logger.borrow_mut().log_root_alpha_change(alpha, score, mv);
                        }
                    }
                }
                
                alpha = alpha.max(score);
            }
        }
    
        // LOG: Search complete
        if let Some(logger) = &self.logger {
            logger.borrow_mut().log_search_complete(best_move, best_score, self.nodes_searched);
        }
    
        (best_move, best_score)
    }
    
    

    fn alphabeta(&mut self, board: &mut Board, depth: i32, mut alpha: i32, beta: i32) -> i32 {
        self.nodes_searched += 1;
        
        // LOG: Enter depth
        if let Some(logger) = &self.logger {
            logger.borrow_mut().log_depth_enter(depth, alpha, beta, None);
        }
    
        // Probe transposition table
        let hash = self.transposition_table.get_hash(board);
        if let Some((tt_score, tt_move)) = self.transposition_table.probe(hash, depth, alpha, beta) {
            if let Some(logger) = &self.logger {
                logger.borrow_mut().log_tt_hit(depth, depth, tt_score, tt_move);
                logger.borrow_mut().log_depth_exit(depth, tt_score, tt_move);
            }
            return tt_score;
        }
    
        // Check for terminal conditions
        if depth <= 0 {
            if let Some(logger) = &self.logger {
                logger.borrow_mut().log_quiescence_enter(alpha, beta);
            }
            
            let eval = self.quiescence_search(board, alpha, beta);
            
            if let Some(logger) = &self.logger {
                logger.borrow_mut().log_quiescence_exit(eval);
                logger.borrow_mut().log_depth_exit(depth, eval, None);
            }
            
            self.transposition_table.store(hash, depth, eval, None, crate::transposition::NodeType::Exact);
            return eval;
        }
    
        // Generate moves
        let mut moves = board.get_all_legal_moves();
        if moves.is_empty() {
            let eval = if board.is_in_check() { -MATE_SCORE + depth } else { 0 };
            
            if let Some(logger) = &self.logger {
                logger.borrow_mut().log_leaf_evaluation(depth, eval);
                logger.borrow_mut().log_depth_exit(depth, eval, None);
            }
            
            self.transposition_table.store(hash, depth, eval, None, crate::transposition::NodeType::Exact);
            return eval;
        }
    
        // LOG: Show available moves
        if let Some(logger) = &self.logger {
            logger.borrow_mut().log_available_moves_at_depth(depth, &moves);
        }
    
        // Order moves
        self.order_moves(board, &mut moves);
        
        // LOG: Show move ordering result
        if let Some(logger) = &self.logger {
            logger.borrow_mut().log_move_ordering_result(&moves, board);
        }
    
        let original_alpha = alpha;
        let mut best_score = -MATE_SCORE - 1;
        let mut best_move = None;
    
        // Explore each move
        for (move_index, &mv) in moves.iter().enumerate() {
            if let Ok(_) = board.try_make_move(mv) {
                // LOG: Start exploring this move
                if let Some(logger) = &self.logger {
                    logger.borrow_mut().log_move_exploration_start(mv, move_index + 1, moves.len(), depth, alpha, beta);
                }
    
                // Recursive search
                let score = -self.alphabeta(board, depth - 1, -beta, -alpha);
                
                if let Err(_) = board.undo_move() { break; }
    
                // LOG: Move exploration result
                if let Some(logger) = &self.logger {
                    logger.borrow_mut().log_move_exploration_result(mv, score, alpha, beta, depth);
                }
    
                // Update best score
                if score > best_score {
                    best_score = score;
                    best_move = Some(mv);
                }
    
                // Update alpha
                if score > alpha {
                    alpha = score;
                    
                    if let Some(logger) = &self.logger {
                        logger.borrow_mut().log_alpha_change(original_alpha, alpha, mv);
                    }
                }
    
                // Beta cutoff
                if alpha >= beta {
                    if let Some(logger) = &self.logger {
                        logger.borrow_mut().log_beta_cutoff(beta, score, mv);
                        logger.borrow_mut().log_depth_exit(depth, beta, Some(mv));
                    }
                    
                    self.transposition_table.store(hash, depth, beta, best_move, crate::transposition::NodeType::LowerBound);
                    return beta;
                }
            }
        }
    
        // Determine node type
        let node_type = if alpha <= original_alpha {
            crate::transposition::NodeType::UpperBound
        } else {
            crate::transposition::NodeType::Exact
        };
    
        // LOG: Exit depth
        if let Some(logger) = &self.logger {
            logger.borrow_mut().log_alphabeta_node_complete(depth, alpha, node_type);
            logger.borrow_mut().log_depth_exit(depth, alpha, best_move);
        }
    
        self.transposition_table.store(hash, depth, alpha, best_move, node_type);
        alpha
    }
    

    
    
    fn order_moves(&self, board: &Board, moves: &mut Vec<Move>) {
        moves.sort_by_key(|&mv| {
            let from_piece = board.get_piece(mv.from);
            let to_piece = board.get_piece(mv.to);
            
            let mut score = 0;
            
            // Captures: MVV-LVA (Most Valuable Victim - Least Valuable Attacker)
            if !engine::types::is_empty(to_piece) {
                let victim_value = PIECE_VALUES[engine::types::piece_type(to_piece) as usize];
                let attacker_value = PIECE_VALUES[engine::types::piece_type(from_piece) as usize];
                score += 10000 + victim_value - attacker_value; // Higher score = better
            }
            
            // Promotions
            if engine::types::piece_type(from_piece) == engine::types::PAWN {
                let from_rank = mv.from.rank();
                let to_rank = mv.to.rank();
                if (from_rank == 6 && to_rank == 7) || (from_rank == 1 && to_rank == 0) {
                    score += 9000; // High priority for promotions
                }
            }
            
            // TODO: Add check detection bonus later
            
            -score // Negative because sort_by_key sorts ascending, we want descending
        });
    }
    
    fn quiescence_search(&mut self, board: &mut Board, mut alpha: i32, beta: i32) -> i32 {
        self.nodes_searched += 1;
        
        // Stand pat - evaluate current position
        let stand_pat = evaluate_position(board);
        
        if stand_pat >= beta {
            return beta;
        }
        
        if stand_pat > alpha {
            alpha = stand_pat;
        }
        
        // Get only capture moves
        let captures = self.get_capture_moves(board);
        
        // Delta pruning - skip captures that can't improve position significantly
        let big_delta = 900; // Queen value - largest possible material gain
        if stand_pat + big_delta < alpha {
            return alpha;
        }
        
        // Search captures
        for &mv in &captures {
            if let Ok(_) = board.try_make_move(mv) {
                let score = -self.quiescence_search(board, -beta, -alpha);
                if let Err(_) = board.undo_move() { break; }
                
                if score >= beta {
                    return beta;
                }
                
                if score > alpha {
                    alpha = score;
                }
            }
        }
        
        alpha
    }
    
    fn get_capture_moves(&self, board: &Board) -> Vec<Move> {
        board.get_all_legal_moves()
            .into_iter()
            .filter(|&mv| {
                let to_piece = board.get_piece(mv.to);
                !engine::types::is_empty(to_piece) // Only captures
            })
            .collect()
    }
    
    pub fn set_logger(&mut self, logger: std::rc::Rc<std::cell::RefCell<engine::ChessLogger>>) {
        self.logger = Some(logger);
    }
}

impl Default for SearchEngine {
    fn default() -> Self {
        Self::new()
    }
}

fn move_to_string(mv: engine::Move) -> String {
    let files = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
    let from_file = files[mv.from.file() as usize];
    let from_rank = mv.from.rank() + 1;
    let to_file = files[mv.to.file() as usize];
    let to_rank = mv.to.rank() + 1;
    
    if mv.is_promotion() {
        let promotion_char = match mv.promotion.unwrap() {
            engine::QUEEN => 'Q', engine::ROOK => 'R', 
            engine::BISHOP => 'B', engine::KNIGHT => 'N', _ => '?',
        };
        format!("{}{}-{}{}={}", from_file, from_rank, to_file, to_rank, promotion_char)
    } else {
        format!("{}{}-{}{}", from_file, from_rank, to_file, to_rank)
    }
}

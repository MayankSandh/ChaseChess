use engine::{Board, Move};
use crate::{evaluation::*, types::*};
use crate::transposition::*;
use crate::piece_square_tables::get_pst;

pub struct SearchEngine {
    pub nodes_searched: u64,
    transposition_table: TranspositionTable,
    logger: Option<std::rc::Rc<std::cell::RefCell<engine::ChessLogger>>>,
    killer_moves: [[Option<Move>; 2]; 128],
}

impl SearchEngine {
    pub fn new() -> Self {
        get_pst();
        Self {
            nodes_searched: 0,
            transposition_table: TranspositionTable::new(64),
            logger: None,
            killer_moves: [[None; 2]; 128],
        }
    }

    const MAX_EXTENSIONS: i32 = 1;
    const MAX_QS_DEPTH: i32 = 4;

    /// Static Exchange Evaluation stub (simple placeholder - expand for full implementation)
    fn see(&self, board: &Board, mv: Move) -> i32 {
        // TODO: Implement full SEE by simulating exchanges on mv.to
        // For now, assume all are neutral (0) to avoid pruning everything
        let target_piece = board.get_piece(mv.to);
        if engine::types::is_empty(target_piece) {
            return 0; // Quiet moves
        }
        let victim_value = PIECE_VALUES[engine::types::piece_type(target_piece) as usize];
        let attacker_value = PIECE_VALUES[engine::types::piece_type(board.get_piece(mv.from)) as usize];
        victim_value - attacker_value / 2 // Rough estimate
    }

    /// Calculates how many plies to extend the search based on the move and position
    fn calculate_extensions(
        &self,
        mv: Move,
        board: &Board,
        was_in_check: bool,
        extensions_used: i32,
        depth: i32,
    ) -> i32 {
        if extensions_used >= Self::MAX_EXTENSIONS || depth < 2 {
            return 0;
        }
        let mut extension = 0;
        let gives_check = board.is_in_check(); // After move
        if gives_check && self.see(board, mv) >= 0 {
            extension += 1;
        }
        if was_in_check {
            extension += 1;
        }
        extension.min(Self::MAX_EXTENSIONS - extensions_used)
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

        // Order moves for better alpha-beta efficiency
        self.order_moves(board, &mut moves, depth);
        if let Some(logger) = &self.logger {
            logger.borrow_mut().log_search_start(depth as u32, moves.len());
        }

        let mut best_score = -MATE_SCORE - 1;
        let mut best_move = None;
        let mut alpha = -MATE_SCORE - 1;
        let beta = MATE_SCORE + 1;

        for (move_num, &mv) in moves.iter().enumerate() {
            if let Ok(_) = board.try_make_move(mv) {
                let score = -self.alphabeta(board, depth - 1, -beta, -alpha, 0);
                if let Err(_) = board.undo_move() {
                    break;
                }

                // LOG: Move analysis
                if let Some(logger) = &self.logger {
                    logger.borrow_mut().log_move_analysis(mv, move_num + 1, moves.len(), score);
                }

                if score > best_score {
                    best_score = score;
                    best_move = Some(mv);

                    // LOG: Alpha improvement
                    if let Some(logger) = &self.logger {
                        if score > alpha {
                            logger.borrow_mut().log_alpha_change(alpha, score, mv);
                        }
                    }
                    alpha = alpha.max(score);
                }
            }
        }

        // LOG: Search complete
        if let Some(logger) = &self.logger {
            logger.borrow_mut().log_search_complete(best_move, best_score, self.nodes_searched);
        }

        (best_move, best_score)
    }

    fn alphabeta(&mut self, board: &mut Board, depth: i32, mut alpha: i32, beta: i32, extensions_used: i32) -> i32 {
        self.nodes_searched += 1;

        // Probe transposition table
        let hash = self.transposition_table.get_hash(board);
        if let Some((tt_score, tt_move)) = self.transposition_table.probe(hash, depth, alpha, beta) {
            if let Some(logger) = &self.logger {
                logger.borrow_mut().log_tt_hit(depth, depth, tt_score, tt_move);
            }
            return tt_score;
        }

        // CRITICAL FIX: Never enter quiescence while in check
        let in_check = board.is_in_check();
        if depth <= 0 && !in_check {
            // Only enter QS if NOT in check
            let eval = self.quiescence_search(board, alpha, beta, 0);
            self.transposition_table.store(hash, depth, eval, None, NodeType::Exact);
            return eval;
        }

        let mut moves = board.get_all_legal_moves();
        if moves.is_empty() {
            let eval = if in_check { -MATE_SCORE } else { 0 };
            self.transposition_table.store(hash, depth, eval, None, NodeType::Exact);
            return eval;
        }

        self.order_moves(board, &mut moves, depth);

        let original_alpha = alpha;
        let mut best_move = None;
        let mut best_score = -MATE_SCORE - 1;

        for &mv in &moves {
            if let Ok(_) = board.try_make_move(mv) {
                // Calculate extensions for this move
                let extension = self.calculate_extensions(mv, board, in_check, extensions_used, depth);
                let new_depth = depth - 1 + extension;
                let new_extensions = extensions_used + extension;

                // Recursive call with updated parameters
                let score = -self.alphabeta(board, new_depth, -beta, -alpha, new_extensions);
                if let Err(_) = board.undo_move() {
                    break;
                }

                if score > best_score {
                    best_score = score;
                    best_move = Some(mv);
                }

                if score > alpha {
                    alpha = score;
                }

                if alpha >= beta {
                    if let Some(logger) = &self.logger {
                        logger.borrow_mut().log_beta_cutoff(beta, score, mv);
                    }

                    let to_piece = board.get_piece(mv.to);
                    if engine::types::is_empty(to_piece) {
                        self.store_killer_move(mv, depth);
                    }

                    self.transposition_table.store(hash, depth, best_score, best_move, NodeType::LowerBound);
                    return best_score;
                }
            }
        }

        let node_type = if alpha <= original_alpha {
            NodeType::UpperBound
        } else {
            NodeType::Exact
        };
        self.transposition_table.store(hash, depth, best_score, best_move, node_type);
        best_score
    }

    fn quiescence_search(&mut self, board: &mut Board, mut alpha: i32, beta: i32, qs_depth: i32) -> i32 {
        self.nodes_searched += 1;

        // Cap recursion
        if qs_depth > Self::MAX_QS_DEPTH {
            return evaluate_position(board);
        }

        let in_check = board.is_in_check();
        let stand_pat = if in_check { -MATE_SCORE } else { evaluate_position(board) };

        if !in_check {
            if stand_pat >= beta {
                return stand_pat;
            }
            if stand_pat > alpha {
                alpha = stand_pat;
            }
        }

        let mut tactical_moves = if in_check {
            board.get_all_legal_moves() // All evasions in check
        } else {
            self.get_capture_moves(board) // Captures otherwise
        };

        if !in_check && qs_depth == 0 {
            let checking_moves = self.get_safe_checking_moves(board);
            let filtered_checks: Vec<Move> = checking_moves.into_iter()
                .filter(|m| !tactical_moves.contains(m))
                .collect();
            tactical_moves.extend(filtered_checks);
        }

        // Delta pruning (skip if in check)
        let big_delta = 900;
        if !in_check && stand_pat + big_delta < alpha && tactical_moves.is_empty() {
            return stand_pat;
        }

        // Order moves
        self.order_moves(board, &mut tactical_moves, 0);

        let mut best_score = stand_pat;

        for &mv in &tactical_moves {
            if self.see(board, mv) < 0 {
                continue; // Prune bad SEE
            }
            if let Ok(_) = board.try_make_move(mv) {
                let score = -self.quiescence_search(board, -beta, -alpha, qs_depth + 1);
                if let Err(_) = board.undo_move() {
                    break;
                }
                if score > best_score {
                    best_score = score;
                }
                if score >= beta {
                    return score;
                }
                if score > alpha {
                    alpha = score;
                }
            }
        }
        best_score
    }

    fn order_moves(&self, board: &Board, moves: &mut Vec<Move>, depth: i32) {
        moves.sort_by_key(|&mv| {
            let from_piece = board.get_piece(mv.from);
            let to_piece = board.get_piece(mv.to);
            let mut score = 0;

            // 1. Hash move (handled separately in search - highest priority)

            // 2. Captures: MVV-LVA (Most Valuable Victim - Least Valuable Attacker)
            if !engine::types::is_empty(to_piece) {
                let victim_value = PIECE_VALUES[engine::types::piece_type(to_piece) as usize];
                let attacker_value = PIECE_VALUES[engine::types::piece_type(from_piece) as usize];
                score += 10000 + victim_value - attacker_value;
            }

            // 3. Promotions
            else if engine::types::piece_type(from_piece) == engine::types::PAWN {
                let from_rank = mv.from.rank();
                let to_rank = mv.to.rank();
                if (from_rank == 6 && to_rank == 7) || (from_rank == 1 && to_rank == 0) {
                    score += 9000;
                }
            }

            // 4. Killer moves (for non-captures)
            else if depth >= 0 && depth < 128 {
                let depth_idx = depth as usize;
                if let Some(killer1) = self.killer_moves[depth_idx][0] {
                    if killer1 == mv {
                        score += 8000; // First killer gets higher priority
                    }
                }
                if let Some(killer2) = self.killer_moves[depth_idx][1] {
                    if killer2 == mv {
                        score += 7000; // Second killer gets lower priority
                    }
                }
            }
            -score // Negative because sort_by_key sorts ascending, we want descending
        });
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

    fn get_safe_checking_moves(&self, board: &Board) -> Vec<Move> {
        board.get_all_legal_moves()
            .into_iter()
            .filter(|&mv| {
                let to_piece = board.get_piece(mv.to);
                engine::types::is_empty(to_piece) && { // Quiet moves only
                    let mut test_board = board.clone();
                    if test_board.try_make_move(mv).is_ok() {
                        test_board.is_in_check() && self.see(&test_board, mv) >= 0
                    } else {
                        false
                    }
                }
            })
            .collect()
    }

    /// Helper function to find all checking moves
    fn get_checking_moves(&self, board: &Board) -> Vec<Move> {
        board.get_all_legal_moves()
            .into_iter()
            .filter(|&mv| {
                // Test if this move gives check by trying it
                let mut test_board = board.clone();
                if let Ok(_) = test_board.try_make_move(mv) {
                    test_board.is_in_check()
                } else {
                    false
                }
            })
            .collect()
    }

    pub fn set_logger(&mut self, logger: std::rc::Rc<std::cell::RefCell<engine::ChessLogger>>) {
        self.logger = Some(logger);
    }

    fn store_killer_move(&mut self, mv: Move, depth: i32) {
        if depth < 0 || depth >= 128 {
            return;
        }

        let depth_idx = depth as usize;
        // Don't store if it's already the first killer
        if let Some(first_killer) = self.killer_moves[depth_idx][0] {
            if first_killer == mv {
                return;
            }
        }

        // Shift killers: second becomes first, new move becomes first
        self.killer_moves[depth_idx][1] = self.killer_moves[depth_idx][0];
        self.killer_moves[depth_idx][0] = Some(mv);
    }
}

impl Default for SearchEngine {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SearchResult {
    pub best_move: Option<Move>,
    pub evaluation: i32,
    pub depth: u32,
    pub nodes_searched: u64,
}

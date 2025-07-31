use engine::{Board, Move};
use crate::{evaluation::*, types::*};

pub struct SearchEngine {
    pub nodes_searched: u64,
}

impl SearchEngine {
    pub fn new() -> Self {
        Self { nodes_searched: 0 }
    }

    pub fn search(&mut self, board: &mut Board, depth: u32) -> SearchResult {
        self.nodes_searched = 0;
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
        self.order_moves(board, &mut moves);
    
        let mut best_score = -MATE_SCORE - 1;
        let mut best_move = None;
        let mut alpha = -MATE_SCORE - 1;
        let beta = MATE_SCORE + 1;
    
        for &mv in &moves {
            if let Ok(_) = board.try_make_move(mv) {
                let score = -self.alphabeta(board, depth - 1, -beta, -alpha);
                if let Err(_) = board.undo_move() { break; }
                
                if score > best_score {
                    best_score = score;
                    best_move = Some(mv);
                }
                
                alpha = alpha.max(score);
            }
        }
    
        (best_move, best_score)
    }
    

    fn alphabeta(&mut self, board: &mut Board, depth: i32, alpha: i32, beta: i32) -> i32 {
        self.nodes_searched += 1;
    
        if depth <= 0 {
            return evaluate_position(board);
        }
    
        let mut moves = board.get_all_legal_moves();
        if moves.is_empty() {
            return if board.is_in_check() { -MATE_SCORE } else { 0 };
        }
    
        // Order moves for better alpha-beta efficiency
        self.order_moves(board, &mut moves);
    
        let mut alpha = alpha;
    
        for &mv in &moves {
            if let Ok(_) = board.try_make_move(mv) {
                let score = -self.alphabeta(board, depth - 1, -beta, -alpha);
                if let Err(_) = board.undo_move() { break; }
                
                if score >= beta {
                    return beta; // Beta cutoff
                }
                
                alpha = alpha.max(score);
            }
        }
    
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
    
}

impl Default for SearchEngine {
    fn default() -> Self {
        Self::new()
    }
}

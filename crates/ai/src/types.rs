use engine::Move; // Remove unused Square import

/// Search result containing best move and evaluation
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub best_move: Option<Move>,
    pub evaluation: i32,
    pub depth: u32,
    pub nodes_searched: u64,
}

/// Basic piece values for evaluation
pub const PIECE_VALUES: [i32; 7] = [
    0,    // Empty
    100,  // Pawn
    320,  // Knight
    330,  // Bishop
    500,  // Rook
    900,  // Queen
    20000, // King
];

/// Evaluation constants
pub const MATE_SCORE: i32 = 100000;
pub const DRAW_SCORE: i32 = 0;

/// Maximum search depth
pub const MAX_DEPTH: u32 = 8;

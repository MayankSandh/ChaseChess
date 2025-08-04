use engine::{Board, types::*};

use std::sync::OnceLock;

static PST: OnceLock<PreCalculatedPST> = OnceLock::new();

pub fn get_pst() -> &'static PreCalculatedPST {
    PST.get_or_init(|| PreCalculatedPST::new())
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EndgamePattern {
    Opening = 0,
    Middlegame = 1,
    KQvsK = 2, // King + Queen vs King
    KRvsK = 3, // King + Rook vs King
    KPvsK = 4, // King + Pawns vs King
    RookEndgame = 5, // Rook endgames
    QueenEndgame = 6, // Queen endgames
    PawnEndgame = 7, // Pure pawn endgames
    GeneralEndgame = 8, // General simplified endgame
}

const OPENING_PAWN_PST: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, // Rank 1 (index 0-7)
    50, 50, 50, 50, 50, 50, 50, 50, // Rank 2 - encourage pawn moves
    10, 10, 20, 30, 30, 20, 10, 10, // Rank 3 - good development
    5, 5, 10, 25, 25, 10, 5, 5, // Rank 4 - central control
    0, 0, 0, 20, 20, 0, 0, 0, // Rank 5 - central pawns
    5, -5,-10, 0, 0,-10, -5, 5, // Rank 6 - your extreme values
    5, 10, 10,-20,-20, 10, 10, 5, // Rank 7 - penalize premature advances
    0, 0, 0, 0, 0, 0, 0, 0, // Rank 8 (index 56-63)
];

const OPENING_KNIGHT_PST: [i32; 64] = [
    -50,-40,-30,-30,-30,-30,-40,-50, // Rank 1 - knights belong in center
    -40,-20, 0, 0, 0, 0,-20,-40, // Rank 2
    -30, 0, 10, 15, 15, 10, 0,-30, // Rank 3 - good development squares
    -30, 5, 15, 20, 20, 15, 5,-30, // Rank 4 - excellent central squares
    -30, 0, 15, 20, 20, 15, 0,-30, // Rank 5 - strong central outposts
    -30, 5, 10, 15, 15, 10, 5,-30, // Rank 6
    -40,-20, 0, 5, 5, 0,-20,-40, // Rank 7
    -50,-40,-30,-30,-30,-30,-40,-50, // Rank 8
];

const OPENING_BISHOP_PST: [i32; 64] = [
    -20,-10,-10,-10,-10,-10,-10,-20, // Rank 1
    -10, 0, 0, 0, 0, 0, 0,-10, // Rank 2
    -10, 0, 5, 10, 10, 5, 0,-10, // Rank 3 - good development
    -10, 5, 5, 10, 10, 5, 5,-10, // Rank 4 - active bishops
    -10, 0, 10, 10, 10, 10, 0,-10, // Rank 5 - long diagonals
    -10, 10, 10, 10, 10, 10, 10,-10, // Rank 6
    -10, 5, 0, 0, 0, 0, 5,-10, // Rank 7
    -20,-10,-10,-10,-10,-10,-10,-20, // Rank 8
];

const OPENING_ROOK_PST: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, // Rank 1 - rooks develop later
    5, 10, 10, 10, 10, 10, 10, 5, // Rank 2 - 2nd rank is strong
    -5, 0, 0, 0, 0, 0, 0, -5, // Rank 3
    -5, 0, 0, 0, 0, 0, 0, -5, // Rank 4
    -5, 0, 0, 0, 0, 0, 0, -5, // Rank 5
    -5, 0, 0, 0, 0, 0, 0, -5, // Rank 6
    -5, 0, 0, 0, 0, 0, 0, -5, // Rank 7
    0, 0, 0, 5, 5, 0, 0, 0, // Rank 8 - central files preferred
];

const OPENING_QUEEN_PST: [i32; 64] = [
    -20,-10,-10, -5, -5,-10,-10,-20, // Rank 1 - don't develop queen early
    -10, 0, 0, 0, 0, 0, 0,-10, // Rank 2
    -10, 0, 5, 5, 5, 5, 0,-10, // Rank 3
    -5, 0, 5, 5, 5, 5, 0, -5, // Rank 4
    0, 0, 5, 5, 5, 5, 0, -5, // Rank 5
    -10, 5, 5, 5, 5, 5, 0,-10, // Rank 6
    -10, 0, 5, 0, 0, 0, 0,-10, // Rank 7
    -20,-10,-10, -5, -5,-10,-10,-20, // Rank 8
];

const OPENING_KING_PST: [i32; 64] = [
    -30,-40,-40,-50,-50,-40,-40,-30, // Rank 1 - king safety crucial
    -30,-40,-40,-50,-50,-40,-40,-30, // Rank 2
    -30,-40,-40,-50,-50,-40,-40,-30, // Rank 3
    -30,-40,-40,-50,-50,-40,-40,-30, // Rank 4
    -20,-30,-30,-40,-40,-30,-30,-20, // Rank 5
    -10,-20,-20,-20,-20,-20,-20,-10, // Rank 6
    20, 20, 0, 0, 0, 0, 20, 20, // Rank 7 - encourage castling
    20, 30, 10, 0, 0, 10, 30, 20, // Rank 8 - corners after castling
];

// Endgame PST
const ENDGAME_PAWN_PST: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0,
    80, 80, 80, 80, 80, 80, 80, 80, // Pawn promotion is huge
    60, 60, 60, 60, 60, 60, 60, 60,
    40, 40, 40, 40, 40, 40, 40, 40,
    20, 20, 20, 20, 20, 20, 20, 20,
    10, 10, 10, 10, 10, 10, 10, 10,
    10, 10, 10, 10, 10, 10, 10, 10,
    0, 0, 0, 0, 0, 0, 0, 0,
];

const ENDGAME_KING_PST: [i32; 64] = [
    -50,-40,-30,-20,-20,-30,-40,-50, // King activity crucial in endgame
    -30,-20,-10, 0, 0,-10,-20,-30,
    -30,-10, 20, 30, 30, 20,-10,-30,
    -30,-10, 30, 40, 40, 30,-10,-30,
    -30,-10, 30, 40, 40, 30,-10,-30,
    -30,-10, 20, 30, 30, 20,-10,-30,
    -30,-30, 0, 0, 0, 0,-30,-30,
    -50,-30,-30,-30,-30,-30,-30,-50,
];

// KQ vs K - Force enemy king to edge
const KQ_VS_K_ENEMY_KING: [i32; 64] = [
    -200,-160,-120, -80, -80,-120,-160,-200,
    -160, -80, -40, -20, -20, -40, -80,-160,
    -120, -40, -20, 0, 0, -20, -40,-120,
    -80, -20, 0, 20, 20, 0, -20, -80,
    -80, -20, 0, 20, 20, 0, -20, -80,
    -120, -40, -20, 0, 0, -20, -40,-120,
    -160, -80, -40, -20, -20, -40, -80,-160,
    -200,-160,-120, -80, -80,-120,-160,-200,
];

const KQ_VS_K_OUR_KING: [i32; 64] = [
    20, 30, 40, 50, 50, 40, 30, 20,
    30, 40, 50, 60, 60, 50, 40, 30,
    40, 50, 60, 70, 70, 60, 50, 40,
    50, 60, 70, 80, 80, 70, 60, 50,
    50, 60, 70, 80, 80, 70, 60, 50,
    40, 50, 60, 70, 70, 60, 50, 40,
    30, 40, 50, 60, 60, 50, 40, 30,
    20, 30, 40, 50, 50, 40, 30, 20,
];

const KQ_VS_K_QUEEN: [i32; 64] = [
    10, 20, 30, 40, 40, 30, 20, 10,
    20, 30, 40, 50, 50, 40, 30, 20,
    30, 40, 60, 70, 70, 60, 40, 30,
    40, 50, 70, 80, 80, 70, 50, 40,
    40, 50, 70, 80, 80, 70, 50, 40,
    30, 40, 60, 70, 70, 60, 40, 30,
    20, 30, 40, 50, 50, 40, 30, 20,
    10, 20, 30, 40, 40, 30, 20, 10,
];

// KR vs K
const KR_VS_K_ENEMY_KING: [i32; 64] = [
    -150,-120, -90, -60, -60, -90,-120,-150,
    -120, -60, -30, -10, -10, -30, -60,-120,
    -90, -30, -10, 10, 10, -10, -30, -90,
    -60, -10, 10, 30, 30, 10, -10, -60,
    -60, -10, 10, 30, 30, 10, -10, -60,
    -90, -30, -10, 10, 10, -10, -30, -90,
    -120, -60, -30, -10, -10, -30, -60,-120,
    -150,-120, -90, -60, -60, -90,-120,-150,
];

const KR_VS_K_OUR_KING: [i32; 64] = [
    10, 20, 30, 40, 40, 30, 20, 10,
    20, 30, 40, 50, 50, 40, 30, 20,
    30, 40, 50, 60, 60, 50, 40, 30,
    40, 50, 60, 70, 70, 60, 50, 40,
    40, 50, 60, 70, 70, 60, 50, 40,
    30, 40, 50, 60, 60, 50, 40, 30,
    20, 30, 40, 50, 50, 40, 30, 20,
    10, 20, 30, 40, 40, 30, 20, 10,
];

const KR_VS_K_ROOK: [i32; 64] = [
    20, 30, 40, 50, 50, 40, 30, 20,
    30, 40, 50, 60, 60, 50, 40, 30,
    40, 50, 60, 70, 70, 60, 50, 40,
    50, 60, 70, 80, 80, 70, 60, 50,
    50, 60, 70, 80, 80, 70, 60, 50,
    40, 50, 60, 70, 70, 60, 50, 40,
    30, 40, 50, 60, 60, 50, 40, 30,
    20, 30, 40, 50, 50, 40, 30, 20,
];

// Simplified PST structure - no more complex 4D arrays!
pub struct PreCalculatedPST {
    // No fields needed - all calculation is real-time
}

impl PreCalculatedPST {
    pub fn new() -> Self {
        PreCalculatedPST {
            // No initialization needed
        }
    }

    // Real-time PST calculation - much simpler!
    pub fn get_value(&self, piece_type: usize, pattern: EndgamePattern, phase: u8, square: usize) -> i32 {
        // Convert phase (0-255) to interpolation factor (0.0-1.0)
        let phase_factor = phase as f32 / 255.0;
        
        // Get opening and endgame values
        let opening_val = self.get_opening_pst_value(piece_type, square);
        let endgame_val = self.get_endgame_pst_value(piece_type, pattern as usize, square);
        
        // Real-time linear interpolation
        let interpolated = opening_val as f32 * (1.0 - phase_factor) + 
                          endgame_val as f32 * phase_factor;
        
        
        interpolated as i32
    }

    fn get_opening_pst_value(&self, piece: usize, square: usize) -> i32 {
        match piece {
            0 => OPENING_PAWN_PST[square],   // PAWN - 1
            1 => OPENING_KNIGHT_PST[square], // KNIGHT - 1
            2 => OPENING_BISHOP_PST[square], // BISHOP - 1
            3 => OPENING_ROOK_PST[square],   // ROOK - 1
            4 => OPENING_QUEEN_PST[square],  // QUEEN - 1
            5 => OPENING_KING_PST[square],   // KING - 1
            _ => 0,
        }
    }

    fn get_endgame_pst_value(&self, piece: usize, pattern: usize, square: usize) -> i32 {
        match EndgamePattern::from_usize(pattern) {
            EndgamePattern::KQvsK => {
                match piece {
                    4 => KQ_VS_K_QUEEN[square],     // Queen active
                    5 => KQ_VS_K_OUR_KING[square],  // King supports
                    _ => self.get_general_endgame_value(piece, square),
                }
            },
            EndgamePattern::KRvsK => {
                match piece {
                    3 => KR_VS_K_ROOK[square],      // Rook active
                    5 => KR_VS_K_OUR_KING[square],  // King supports
                    _ => self.get_general_endgame_value(piece, square),
                }
            },
            _ => self.get_general_endgame_value(piece, square),
        }
    }

    fn get_general_endgame_value(&self, piece: usize, square: usize) -> i32 {
        match piece {
            0 => ENDGAME_PAWN_PST[square],
            5 => ENDGAME_KING_PST[square],
            _ => self.get_opening_pst_value(piece, square), // Use opening values for other pieces
        }
    }
}

impl EndgamePattern {
    fn from_usize(value: usize) -> EndgamePattern {
        match value {
            0 => EndgamePattern::Opening,
            1 => EndgamePattern::Middlegame,
            2 => EndgamePattern::KQvsK,
            3 => EndgamePattern::KRvsK,
            4 => EndgamePattern::KPvsK,
            5 => EndgamePattern::RookEndgame,
            6 => EndgamePattern::QueenEndgame,
            7 => EndgamePattern::PawnEndgame,
            8 => EndgamePattern::GeneralEndgame,
            _ => EndgamePattern::GeneralEndgame,
        }
    }
}

pub fn detect_endgame_pattern(board: &Board) -> EndgamePattern {
    let (white_pieces, black_pieces) = count_pieces(board);
    
    // KQ vs K patterns
    if (white_pieces.0 == 0 && white_pieces.4 == 1 && total_pieces(&white_pieces) == 2) &&
       (total_pieces(&black_pieces) == 1) {
        return EndgamePattern::KQvsK;
    }
    if (black_pieces.0 == 0 && black_pieces.4 == 1 && total_pieces(&black_pieces) == 2) &&
       (total_pieces(&white_pieces) == 1) {
        return EndgamePattern::KQvsK;
    }
    
    // KR vs K patterns
    if (white_pieces.0 == 0 && white_pieces.3 == 1 && total_pieces(&white_pieces) == 2) &&
       (total_pieces(&black_pieces) == 1) {
        return EndgamePattern::KRvsK;
    }
    if (black_pieces.0 == 0 && black_pieces.3 == 1 && total_pieces(&black_pieces) == 2) &&
       (total_pieces(&white_pieces) == 1) {
        return EndgamePattern::KRvsK;
    }
    
    // General phase detection
    let total_material = calculate_total_material(board);
    if total_material > 6000 {
        EndgamePattern::Opening
    } else if total_material > 2500 {
        EndgamePattern::Middlegame
    } else {
        EndgamePattern::GeneralEndgame
    }
}

pub fn calculate_game_phase(board: &Board) -> u8 {
    let total_material = calculate_total_material(board);
    let max_material = 2 * (8 * 100 + 2 * 320 + 2 * 330 + 2 * 500 + 900); // Starting material = 7800
    let phase_raw = 255 - ((total_material * 255) / max_material);
    phase_raw.min(255).max(0) as u8
}

fn count_pieces(board: &Board) -> ((u8, u8, u8, u8, u8, u8), (u8, u8, u8, u8, u8, u8)) {
    let white_pieces = (
        board.bitboards.count_pieces(WHITE, PAWN) as u8,
        board.bitboards.count_pieces(WHITE, KNIGHT) as u8,
        board.bitboards.count_pieces(WHITE, BISHOP) as u8,
        board.bitboards.count_pieces(WHITE, ROOK) as u8,
        board.bitboards.count_pieces(WHITE, QUEEN) as u8,
        board.bitboards.count_pieces(WHITE, KING) as u8,
    );
    let black_pieces = (
        board.bitboards.count_pieces(BLACK, PAWN) as u8,
        board.bitboards.count_pieces(BLACK, KNIGHT) as u8,
        board.bitboards.count_pieces(BLACK, BISHOP) as u8,
        board.bitboards.count_pieces(BLACK, ROOK) as u8,
        board.bitboards.count_pieces(BLACK, QUEEN) as u8,
        board.bitboards.count_pieces(BLACK, KING) as u8,
    );
    (white_pieces, black_pieces)
}

fn total_pieces(pieces: &(u8, u8, u8, u8, u8, u8)) -> u8 {
    pieces.0 + pieces.1 + pieces.2 + pieces.3 + pieces.4 + pieces.5
}

fn calculate_total_material(board: &Board) -> i32 {
    board.bitboards.count_pieces(WHITE, PAWN) as i32 * 100 +
    board.bitboards.count_pieces(WHITE, KNIGHT) as i32 * 320 +
    board.bitboards.count_pieces(WHITE, BISHOP) as i32 * 330 +
    board.bitboards.count_pieces(WHITE, ROOK) as i32 * 500 +
    board.bitboards.count_pieces(WHITE, QUEEN) as i32 * 900 +
    board.bitboards.count_pieces(WHITE, KING) as i32 * 20000 +
    board.bitboards.count_pieces(BLACK, PAWN) as i32 * 100 +
    board.bitboards.count_pieces(BLACK, KNIGHT) as i32 * 320 +
    board.bitboards.count_pieces(BLACK, BISHOP) as i32 * 330 +
    board.bitboards.count_pieces(BLACK, ROOK) as i32 * 500 +
    board.bitboards.count_pieces(BLACK, QUEEN) as i32 * 900 +
    board.bitboards.count_pieces(BLACK, KING) as i32 * 20000
}

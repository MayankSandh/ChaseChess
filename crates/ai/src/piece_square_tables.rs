use engine::{Board, Square, types::*};
use std::sync::Once;

static INIT: Once = Once::new();

pub fn initialize_pst() {
    INIT.call_once(|| {
        // This ensures PST is initialized only once
        // The actual initialization happens when PreCalculatedPST::new() is called
    });
}

pub fn get_pst() -> &'static PreCalculatedPST {
    unsafe {
        static mut PST: Option<PreCalculatedPST> = None;
        INIT.call_once(|| {
            PST = Some(PreCalculatedPST::new());
        });
        PST.as_ref().unwrap()
    }
}

// Game phase buckets - 16 for higher precision (0-255 scale)
const PHASE_BUCKETS: [u8; 16] = [0, 16, 32, 48, 64, 80, 96, 112, 128, 144, 160, 176, 192, 208, 224, 240];

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EndgamePattern {
    Opening = 0,
    Middlegame = 1,
    KQvsK = 2,        // King + Queen vs King
    KRvsK = 3,        // King + Rook vs King  
    KPvsK = 4,        // King + Pawns vs King
    RookEndgame = 5,  // Rook endgames
    QueenEndgame = 6, // Queen endgames
    PawnEndgame = 7,  // Pure pawn endgames
    GeneralEndgame = 8, // General simplified endgame
}

// Opening PST - Scaled to 5-15% of piece material values
const OPENING_PAWN_PST: [i32; 64] = [
     0,  0,  0,  0,  0,  0,  0,  0,
    12, 12, 12, 12, 12, 12, 12, 12,  // Reduced from 50 to 12 (12% of 100)
     2,  2,  4,  6,  6,  4,  2,  2,  // Reduced from 10-30 to 2-6
     1,  1,  2,  5,  5,  2,  1,  1,  // Reduced from 5-25 to 1-5
     0,  0,  0,  4,  4,  0,  0,  0,  // Reduced from 20 to 4
     1, -1, -2,  0,  0, -2, -1,  1,
     1,  2,  2, -4, -4,  2,  2,  1,  // Reduced penalty from -20 to -4
     0,  0,  0,  0,  0,  0,  0,  0,
];

const OPENING_KNIGHT_PST: [i32; 64] = [
   -12,-10, -6, -6, -6, -6,-10,-12,  // Reduced from -50 to -12 (4% of 320)
   -10, -4,  0,  0,  0,  0, -4,-10,  // Reduced from -20 to -4
    -6,  0,  2,  4,  4,  2,  0, -6,  // Reduced from 10-15 to 2-4
    -6,  1,  4,  5,  5,  4,  1, -6,  // Reduced from 15-20 to 4-5 (1.5% of 320)
    -6,  0,  4,  5,  5,  4,  0, -6,
    -6,  1,  2,  4,  4,  2,  1, -6,
   -10, -4,  0,  1,  1,  0, -4,-10,
   -12,-10, -6, -6, -6, -6,-10,-12,
];

const OPENING_BISHOP_PST: [i32; 64] = [
    -4, -2, -2, -2, -2, -2, -2, -4,  // Reduced from -20 to -4 (1% of 330)
    -2,  0,  0,  0,  0,  0,  0, -2,
    -2,  0,  1,  2,  2,  1,  0, -2,  // Reduced from 5-10 to 1-2
    -2,  1,  1,  2,  2,  1,  1, -2,
    -2,  0,  2,  2,  2,  2,  0, -2,
    -2,  2,  2,  2,  2,  2,  2, -2,
    -2,  1,  0,  0,  0,  0,  1, -2,
    -4, -2, -2, -2, -2, -2, -2, -4,
];

const OPENING_ROOK_PST: [i32; 64] = [
     0,  0,  0,  0,  0,  0,  0,  0,
     1,  2,  2,  2,  2,  2,  2,  1,  // Reduced from 5-10 to 1-2 (0.4% of 500)
    -1,  0,  0,  0,  0,  0,  0, -1,  // Reduced from -5 to -1
    -1,  0,  0,  0,  0,  0,  0, -1,
    -1,  0,  0,  0,  0,  0,  0, -1,
    -1,  0,  0,  0,  0,  0,  0, -1,
    -1,  0,  0,  0,  0,  0,  0, -1,
     0,  0,  0,  1,  1,  0,  0,  0,  // Reduced from 5 to 1
];

const OPENING_QUEEN_PST: [i32; 64] = [
    -4, -2, -2, -1, -1, -2, -2, -4,  // Reduced from -20 to -4 (0.4% of 900)
    -2,  0,  0,  0,  0,  0,  0, -2,
    -2,  0,  1,  1,  1,  1,  0, -2,  // Reduced from 5 to 1
    -1,  0,  1,  1,  1,  1,  0, -1,
     0,  0,  1,  1,  1,  1,  0, -1,
    -2,  1,  1,  1,  1,  1,  0, -2,
    -2,  0,  1,  0,  0,  0,  0, -2,
    -4, -2, -2, -1, -1, -2, -2, -4,
];

const OPENING_KING_PST: [i32; 64] = [
    -6, -8, -8,-10,-10, -8, -8, -6,  // Reduced from -30 to -6
    -6, -8, -8,-10,-10, -8, -8, -6,  // King safety still important but scaled
    -6, -8, -8,-10,-10, -8, -8, -6,
    -6, -8, -8,-10,-10, -8, -8, -6,
    -4, -6, -6, -8, -8, -6, -6, -4,
    -2, -4, -4, -4, -4, -4, -4, -2,
     4,  4,  0,  0,  0,  0,  4,  4,  // Reduced from 20 to 4
     4,  6,  2,  0,  0,  2,  6,  4,  // Reduced from 30 to 6
];

// Endgame PST - Keep higher values as endgame positioning is more critical
const ENDGAME_PAWN_PST: [i32; 64] = [
     0,  0,  0,  0,  0,  0,  0,  0,
    80, 80, 80, 80, 80, 80, 80, 80,  // Keep high - pawn promotion is huge
    60, 60, 60, 60, 60, 60, 60, 60,
    40, 40, 40, 40, 40, 40, 40, 40,
    20, 20, 20, 20, 20, 20, 20, 20,
    10, 10, 10, 10, 10, 10, 10, 10,
    10, 10, 10, 10, 10, 10, 10, 10,
     0,  0,  0,  0,  0,  0,  0,  0,
];

const ENDGAME_KING_PST: [i32; 64] = [
   -50,-40,-30,-20,-20,-30,-40,-50,  // Keep high - king activity crucial in endgame
   -30,-20,-10,  0,  0,-10,-20,-30,
   -30,-10, 20, 30, 30, 20,-10,-30,
   -30,-10, 30, 40, 40, 30,-10,-30,
   -30,-10, 30, 40, 40, 30,-10,-30,
   -30,-10, 20, 30, 30, 20,-10,-30,
   -30,-30,  0,  0,  0,  0,-30,-30,
   -50,-30,-30,-30,-30,-30,-30,-50,
];

// KQ vs K - Force enemy king to edge (keep higher values for mating patterns)
const KQ_VS_K_ENEMY_KING: [i32; 64] = [
   -200,-160,-120, -80, -80,-120,-160,-200,
   -160, -80, -40, -20, -20, -40, -80,-160,
   -120, -40, -20,   0,   0, -20, -40,-120,
    -80, -20,   0,  20,  20,   0, -20, -80,
    -80, -20,   0,  20,  20,   0, -20, -80,
   -120, -40, -20,   0,   0, -20, -40,-120,
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

// KR vs K - Similar concept
const KR_VS_K_ENEMY_KING: [i32; 64] = [
   -150,-120, -90, -60, -60, -90,-120,-150,
   -120, -60, -30, -10, -10, -30, -60,-120,
    -90, -30, -10,  10,  10, -10, -30, -90,
    -60, -10,  10,  30,  30,  10, -10, -60,
    -60, -10,  10,  30,  30,  10, -10, -60,
    -90, -30, -10,  10,  10, -10, -30, -90,
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

pub struct PreCalculatedPST {
    // [piece_type][pattern][phase_bucket][square]
    tables: [[[[i32; 64]; 16]; 9]; 6],
}

impl PreCalculatedPST {
    pub fn new() -> Self {
        let mut pst = PreCalculatedPST {
            tables: [[[[0; 64]; 16]; 9]; 6],
        };
        
        pst.calculate_all_tables();
        pst
    }
    
    fn calculate_all_tables(&mut self) {
        for piece_type in 0..6 {
            for pattern in 0..9 {
                for bucket in 0..16 {
                    self.calculate_bucket_values(piece_type, pattern, bucket);
                }
            }
        }
    }
    
    fn calculate_bucket_values(&mut self, piece: usize, pattern: usize, bucket: usize) {
        let phase = PHASE_BUCKETS[bucket] as f32 / 255.0;
        
        for square in 0..64 {
            let opening_val = self.get_opening_pst_value(piece, square);
            let endgame_val = self.get_endgame_pst_value(piece, pattern, square);
            
            // Linear interpolation
            let interpolated = opening_val as f32 * (1.0 - phase) + 
                              endgame_val as f32 * phase;
            
            self.tables[piece][pattern][bucket][square] = interpolated as i32;
        }
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
                    4 => KQ_VS_K_QUEEN[square],      // Queen active
                    5 => KQ_VS_K_OUR_KING[square],   // King supports
                    _ => self.get_general_endgame_value(piece, square),
                }
            }
            EndgamePattern::KRvsK => {
                match piece {
                    3 => KR_VS_K_ROOK[square],       // Rook active
                    5 => KR_VS_K_OUR_KING[square],   // King supports  
                    _ => self.get_general_endgame_value(piece, square),
                }
            }
            _ => self.get_general_endgame_value(piece, square),
        }
    }
    
    fn get_general_endgame_value(&self, piece: usize, square: usize) -> i32 {
        match piece {
            0 => ENDGAME_PAWN_PST[square],
            5 => ENDGAME_KING_PST[square],
            _ => self.get_opening_pst_value(piece, square), // Use opening values
        }
    }
    
    pub fn get_value(&self, piece_type: usize, pattern: EndgamePattern, phase: u8, square: usize) -> i32 {
        let bucket = self.phase_to_bucket(phase);
        self.tables[piece_type][pattern as usize][bucket][square]
    }
    
    fn phase_to_bucket(&self, phase: u8) -> usize {
        ((phase as usize) / 16).min(15) // Maps 0-255 to 0-15 buckets
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
    let max_material = 2 * (8 * 100 + 2 * 320 + 2 * 330 + 2 * 500 + 900); // Starting material
    
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
    let material = 
        (board.bitboards.count_pieces(WHITE, PAWN) as i32) * 100 +
        (board.bitboards.count_pieces(WHITE, KNIGHT) as i32) * 320 +
        (board.bitboards.count_pieces(WHITE, BISHOP) as i32) * 330 +
        (board.bitboards.count_pieces(WHITE, ROOK) as i32) * 500 +
        (board.bitboards.count_pieces(WHITE, QUEEN) as i32) * 900 +
        (board.bitboards.count_pieces(WHITE, KING) as i32) * 20000 +
        
        (board.bitboards.count_pieces(BLACK, PAWN) as i32) * 100 +
        (board.bitboards.count_pieces(BLACK, KNIGHT) as i32) * 320 +
        (board.bitboards.count_pieces(BLACK, BISHOP) as i32) * 330 +
        (board.bitboards.count_pieces(BLACK, ROOK) as i32) * 500 +
        (board.bitboards.count_pieces(BLACK, QUEEN) as i32) * 900 +
        (board.bitboards.count_pieces(BLACK, KING) as i32) * 20000;
        
    material
}


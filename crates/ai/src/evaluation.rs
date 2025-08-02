use engine::{Board, types::*};
use crate::types::*;
use crate::piece_square_tables::*;

static mut PST: Option<PreCalculatedPST> = None;

pub fn initialize_pst() {
    unsafe {
        if PST.is_none() {
            PST = Some(PreCalculatedPST::new());
        }
    }
}

pub fn evaluate_position(board: &Board) -> i32 {
    let (_, _, total) = evaluate_position_detailed(board);
    total
}

/// Returns (material_score, pst_score, total_score) for detailed logging
pub fn evaluate_position_detailed(board: &Board) -> (i32, i32, i32) {
    unsafe {
        if PST.is_none() {
            initialize_pst();
        }
    }

    let legal_moves = board.get_all_legal_moves();
    if legal_moves.is_empty() {
        let terminal_score = if board.is_in_check() { -MATE_SCORE } else { DRAW_SCORE };
        return (0, 0, terminal_score);
    }

    let material_score = evaluate_material(board);
    let pst_score = evaluate_position_with_pst_detailed(board);
    let total_score = material_score + pst_score;
    
    (material_score, pst_score, total_score)
}

fn evaluate_material(board: &Board) -> i32 {
    let mut white_material = 0;
    let mut black_material = 0;

    // OPTIMIZED: Use bitboard counting instead of nested loops
    white_material += (board.bitboards.count_pieces(WHITE, PAWN) as i32) * PIECE_VALUES[PAWN as usize];
    white_material += (board.bitboards.count_pieces(WHITE, KNIGHT) as i32) * PIECE_VALUES[KNIGHT as usize];
    white_material += (board.bitboards.count_pieces(WHITE, BISHOP) as i32) * PIECE_VALUES[BISHOP as usize];
    white_material += (board.bitboards.count_pieces(WHITE, ROOK) as i32) * PIECE_VALUES[ROOK as usize];
    white_material += (board.bitboards.count_pieces(WHITE, QUEEN) as i32) * PIECE_VALUES[QUEEN as usize];

    black_material += (board.bitboards.count_pieces(BLACK, PAWN) as i32) * PIECE_VALUES[PAWN as usize];
    black_material += (board.bitboards.count_pieces(BLACK, KNIGHT) as i32) * PIECE_VALUES[KNIGHT as usize];
    black_material += (board.bitboards.count_pieces(BLACK, BISHOP) as i32) * PIECE_VALUES[BISHOP as usize];
    black_material += (board.bitboards.count_pieces(BLACK, ROOK) as i32) * PIECE_VALUES[ROOK as usize];
    black_material += (board.bitboards.count_pieces(BLACK, QUEEN) as i32) * PIECE_VALUES[QUEEN as usize];

    // Use elegant mathematical approach
    (2 * (board.current_turn == WHITE) as i32 - 1) * (white_material - black_material)
}

fn evaluate_position_with_pst_detailed(board: &Board) -> i32 {
    unsafe {
        let pst = PST.as_ref().unwrap();
        let pattern = detect_endgame_pattern(board);
        let phase = calculate_game_phase(board);
        let mut score = 0;

        let piece_types = [PAWN, KNIGHT, BISHOP, ROOK, QUEEN, KING];
        
        for piece_type in piece_types {
            if piece_type >= 1 && piece_type <= 6 {
                let piece_index = (piece_type - 1) as usize;

                // Process white pieces
                let white_pieces = board.bitboards.find_pieces(WHITE, piece_type);
                for square in white_pieces {
                    let rank = square.0 / 8;
                    let file = square.0 % 8;
                    let square_index = (rank * 8 + file) as usize;
                    let pst_value = pst.get_value(piece_index, pattern, phase, square_index);
                    score += (2 * (board.current_turn == WHITE) as i32 - 1) * pst_value;
                }

                // Process black pieces
                let black_pieces = board.bitboards.find_pieces(BLACK, piece_type);
                for square in black_pieces {
                    let rank = square.0 / 8;
                    let file = square.0 % 8;
                    let square_index = ((7 - rank) * 8 + file) as usize; // Flip vertically for black
                    let pst_value = pst.get_value(piece_index, pattern, phase, square_index);
                    score -= (2 * (board.current_turn == WHITE) as i32 - 1) * pst_value;
                }
            }
        }

        score
    }
}

/// Advanced evaluation with detailed breakdown for logging
pub fn evaluate_with_breakdown(board: &Board) -> EvaluationBreakdown {
    let (material, pst, total) = evaluate_position_detailed(board);
    let game_phase = calculate_game_phase(board);
    let pattern = detect_endgame_pattern(board);
    
    let piece_breakdown = get_piece_breakdown(board);
    
    EvaluationBreakdown {
        material_score: material,
        pst_score: pst,
        total_score: total,
        game_phase,
        endgame_pattern: pattern,
        piece_breakdown,
    }
}

fn get_piece_breakdown(board: &Board) -> PieceBreakdown {
    let mut breakdown = PieceBreakdown::default();
    
    // Count pieces for each side
    breakdown.white_pawns = board.bitboards.count_pieces(WHITE, PAWN) as u8;
    breakdown.white_knights = board.bitboards.count_pieces(WHITE, KNIGHT) as u8;
    breakdown.white_bishops = board.bitboards.count_pieces(WHITE, BISHOP) as u8;
    breakdown.white_rooks = board.bitboards.count_pieces(WHITE, ROOK) as u8;
    breakdown.white_queens = board.bitboards.count_pieces(WHITE, QUEEN) as u8;
    
    breakdown.black_pawns = board.bitboards.count_pieces(BLACK, PAWN) as u8;
    breakdown.black_knights = board.bitboards.count_pieces(BLACK, KNIGHT) as u8;
    breakdown.black_bishops = board.bitboards.count_pieces(BLACK, BISHOP) as u8;
    breakdown.black_rooks = board.bitboards.count_pieces(BLACK, ROOK) as u8;
    breakdown.black_queens = board.bitboards.count_pieces(BLACK, QUEEN) as u8;
    
    breakdown
}

/// Detailed evaluation data for advanced logging
#[derive(Debug, Clone)]
pub struct EvaluationBreakdown {
    pub material_score: i32,
    pub pst_score: i32,
    pub total_score: i32,
    pub game_phase: u8,
    pub endgame_pattern: EndgamePattern,
    pub piece_breakdown: PieceBreakdown,
}

#[derive(Debug, Clone, Default)]
pub struct PieceBreakdown {
    pub white_pawns: u8,
    pub white_knights: u8,
    pub white_bishops: u8,
    pub white_rooks: u8,
    pub white_queens: u8,
    pub black_pawns: u8,
    pub black_knights: u8,
    pub black_bishops: u8,
    pub black_rooks: u8,
    pub black_queens: u8,
}

use engine::{Board, types::*};
use crate::types::*;
use crate::piece_square_tables::*;
use engine::bitboard::*;

static mut PST: Option<PreCalculatedPST> = None;

// Initialize PST once
pub fn initialize_pst() {
    unsafe {
        if PST.is_none() {
            PST = Some(PreCalculatedPST::new());
        }
    }
}

pub fn evaluate_position(board: &Board) -> i32 {
    // Ensure PST is initialized
    unsafe {
        if PST.is_none() {
            initialize_pst();
        }
    }
    
    let legal_moves = board.get_all_legal_moves();
    if legal_moves.is_empty() {
        return if board.is_in_check() {
            -MATE_SCORE
        } else {
            DRAW_SCORE
        };
    }

    let mut score = 0;

    // Material evaluation
    score += evaluate_material(board);
    
    // Positional evaluation with PST
    score += evaluate_position_with_pst(board);

    score
}

fn evaluate_material(board: &Board) -> i32 {
    let mut white_material = 0;
    let mut black_material = 0;
    
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
    
    (2 * (board.current_turn == WHITE) as i32 - 1) * (white_material - black_material)
}

fn evaluate_position_with_pst(board: &Board) -> i32 {
    unsafe {
        let pst = PST.as_ref().unwrap();
        let pattern = detect_endgame_pattern(board);
        let phase = calculate_game_phase(board);
        let mut score = 0;
        
        // REPLACE the nested loops with bitboard iteration
        let piece_types = [PAWN, KNIGHT, BISHOP, ROOK, QUEEN, KING];
        
        for piece_type in piece_types {
            if piece_type >= 1 && piece_type <= 6 {
                let piece_index = (piece_type - 1) as usize;
                
                // Process white pieces of this type
                let white_pieces = board.bitboards.find_pieces(WHITE, piece_type);
                for square in white_pieces {
                    let rank = square.0 / 8;
                    let file = square.0 % 8;
                    let square_index = (rank * 8 + file) as usize;
                    let pst_value = pst.get_value(piece_index, pattern, phase, square_index);
                    
                    score += (2 * (board.current_turn == WHITE) as i32 - 1) * (pst_value)
                }
                
                // Process black pieces of this type
                let black_pieces = board.bitboards.find_pieces(BLACK, piece_type);
                for square in black_pieces {
                    let rank = square.0 / 8;
                    let file = square.0 % 8;
                    let square_index = ((7 - rank) * 8 + file) as usize; // Flip vertically for black
                    let pst_value = pst.get_value(piece_index, pattern, phase, square_index);
                    
                    score -= (2 * (board.current_turn == WHITE) as i32 - 1) * (pst_value)
                }
            }
        }
        
        score
    }
}

fn get_enemy_king_penalty(_board: &Board, pattern: EndgamePattern, enemy_king_square: usize) -> i32 {
    match pattern {
        EndgamePattern::KQvsK => {
            // Force enemy king to edge in KQ vs K
            let file = enemy_king_square % 8;
            let rank = enemy_king_square / 8;
            let distance_to_edge = std::cmp::min(
                std::cmp::min(file, 7 - file),
                std::cmp::min(rank, 7 - rank)
            );
            -(50 * (3 - distance_to_edge as i32)) // Penalty increases near center
        }
        EndgamePattern::KRvsK => {
            // Similar logic for KR vs K
            let file = enemy_king_square % 8;
            let rank = enemy_king_square / 8;
            let distance_to_edge = std::cmp::min(
                std::cmp::min(file, 7 - file),
                std::cmp::min(rank, 7 - rank)
            );
            -(30 * (3 - distance_to_edge as i32))
        }
        _ => 0,
    }
}


use engine::{Board, types::*};
use crate::types::*;
use crate::piece_square_tables::*;

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

    for rank in 0..8 {
        for file in 0..8 {
            let square = Square::new(file, rank);
            let piece = board.get_piece(square);

            if !is_empty(piece) {
                let piece_type = piece_type(piece);
                let piece_value = PIECE_VALUES[piece_type as usize];

                if piece_color(piece) == WHITE {
                    white_material += piece_value;
                } else {
                    black_material += piece_value;
                }
            }
        }
    }

    if board.current_turn == WHITE {
        white_material - black_material
    } else {
        black_material - white_material
    }
}

fn evaluate_position_with_pst(board: &Board) -> i32 {
    unsafe {
        let pst = PST.as_ref().unwrap();
        let pattern = detect_endgame_pattern(board);
        let phase = calculate_game_phase(board);
        let mut score = 0;
        
        for rank in 0..8 {
            for file in 0..8 {
                let square = Square::new(file, rank);
                let piece = board.get_piece(square);
                
                if !is_empty(piece) {
                    let piece_type = piece_type(piece);
                    let piece_color = piece_color(piece);
                    
                    if piece_type >= 1 && piece_type <= 6 {
                        let piece_index = (piece_type - 1) as usize; // Convert to 0-indexed
                        
                        // Calculate square index (flip for black pieces)
                        let square_index = if piece_color == WHITE {
                            (rank * 8 + file) as usize
                        } else {
                            ((7 - rank) * 8 + file) as usize // Flip vertically for black
                        };
                        
                        let pst_value = pst.get_value(piece_index, pattern, phase, square_index);
                        
                        if piece_color == board.current_turn {
                            score += pst_value;
                        } else {
                            score -= pst_value;
                        }
                    }
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


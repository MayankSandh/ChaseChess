use engine::{Board, types::*};
use crate::types::*;

/// Evaluate a position from the perspective of the current player
/// Positive = good for current player, Negative = bad for current player
pub fn evaluate_position(board: &Board) -> i32 {
    // Check for mate/stalemate first
    let legal_moves = board.get_all_legal_moves();
    if legal_moves.is_empty() {
        return if board.is_in_check() {
            -MATE_SCORE // Current player is in checkmate
        } else {
            DRAW_SCORE  // Stalemate
        };
    }

    let mut score = 0;

    // Material evaluation
    score += evaluate_material(board);

    score
}

/// Count material for both sides
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

    // Return score relative to current player
    if board.current_turn == WHITE {
        white_material - black_material
    } else {
        black_material - white_material
    }
}

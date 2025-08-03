use engine::{ChessLogger, Board, Move};
use crate::transposition::NodeType;

// Define an extension trait for AI-specific logging
pub trait AILoggerExtensions {
    // PST and evaluation logging
    fn log_detailed_pst_evaluation(&mut self, board: &Board, pst_score: i32);
    
    // Search flow logging
    fn log_depth_enter(&mut self, depth: i32, alpha: i32, beta: i32, move_context: Option<Move>);
    fn log_depth_exit(&mut self, depth: i32, final_score: i32, best_move: Option<Move>);
    fn log_available_moves_at_depth(&mut self, depth: i32, moves: &[Move]);
    fn log_move_ordering_at_depth(&mut self, depth: i32, ordered_moves: &[Move], board: &Board);
    fn log_move_exploration_start(&mut self, mv: Move, move_num: usize, total_moves: usize, depth: i32, alpha: i32, beta: i32);
    fn log_move_exploration_result(&mut self, mv: Move, score: i32, alpha: i32, beta: i32, depth: i32);
    fn log_leaf_evaluation(&mut self, depth: i32, eval_score: i32);
    
    // Use existing methods with corrected names
    fn log_quiescence_enter(&mut self, alpha: i32, beta: i32) {
        // Call the existing method in engine logger
        if self.should_log_advanced() {
            self.log_with_indent(&format!("⚡ QUIESCENCE SEARCH | α={}, β={}", alpha, beta));
            self.increase_indent();
        }
    }
    
    fn log_quiescence_exit(&mut self, score: i32) {
        if self.should_log_advanced() {
            self.decrease_indent();
            self.log_with_indent(&format!("⚡ QUIESCENCE RESULT: {}", score));
        }
    }

    fn log_alphabeta_node_complete(&mut self, depth: i32, final_alpha: i32, node_type: crate::transposition::NodeType);
}

impl AILoggerExtensions for ChessLogger {
    fn log_detailed_pst_evaluation(&mut self, board: &Board, pst_score: i32) {
        if self.should_log_advanced() && !self.in_evaluation {
            self.in_evaluation = true;
            
            self.log_with_indent("🔬 DETAILED PST EVALUATION:");
            self.increase_indent();
            
            let pst = crate::piece_square_tables::get_pst();
            let pattern = crate::piece_square_tables::detect_endgame_pattern(board);
            let phase = crate::piece_square_tables::calculate_game_phase(board);
            
            let piece_types = [engine::PAWN, engine::KNIGHT, engine::BISHOP, engine::ROOK, engine::QUEEN, engine::KING];
            let piece_names = ["Pawn", "Knight", "Bishop", "Rook", "Queen", "King"];
            let files = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
            
            for (idx, &piece_type) in piece_types.iter().enumerate() {
                if piece_type >= 1 && piece_type <= 6 {
                    let piece_index = (piece_type - 1) as usize;
                    let piece_name = piece_names[idx];
                    
                    // White pieces
                    let white_pieces = board.bitboards.find_pieces(engine::WHITE, piece_type);
                    if !white_pieces.is_empty() {
                        self.log_with_indent(&format!("├─ White {}s:", piece_name));
                        for square in white_pieces {
                            let rank = square.0 / 8;
                            let file = square.0 % 8;
                            let square_index = (rank * 8 + file) as usize;
                            let pst_value = pst.get_value(piece_index, pattern, phase, square_index);
                            let adjusted_value = (2 * (board.current_turn == engine::WHITE) as i32 - 1) * pst_value;
                            
                            let square_name = format!("{}{}", files[file as usize], rank + 1);
                            self.log_with_indent(&format!("   {} {} - PST value = {:+}", 
                                piece_name, square_name, adjusted_value));
                        }
                    }
                    
                    // Black pieces
                    let black_pieces = board.bitboards.find_pieces(engine::BLACK, piece_type);
                    if !black_pieces.is_empty() {
                        self.log_with_indent(&format!("├─ Black {}s:", piece_name));
                        for square in black_pieces {
                            let rank = square.0 / 8;
                            let file = square.0 % 8;
                            let square_index = ((7 - rank) * 8 + file) as usize;
                            let pst_value = pst.get_value(piece_index, pattern, phase, square_index);
                            let adjusted_value = -(2 * (board.current_turn == engine::WHITE) as i32 - 1) * pst_value;
                            
                            let square_name = format!("{}{}", files[file as usize], rank + 1);
                            self.log_with_indent(&format!("   {} {} - PST value = {:+}", 
                                piece_name, square_name, adjusted_value));
                        }
                    }
                }
            }
            
            self.log_with_indent(&format!("└─ PST Total Score: {:+}", pst_score));
            self.decrease_indent();
            self.in_evaluation = false;
        }
    }

    fn log_depth_enter(&mut self, depth: i32, alpha: i32, beta: i32, move_context: Option<Move>) {
        if self.should_log_advanced() {
            let move_str = if let Some(mv) = move_context {
                format!(" after {}", move_to_string(mv))
            } else {
                " (ROOT)".to_string()
            };
            
            self.log(&format!("🔍 === ENTERING DEPTH {} === α={}, β={}{}", 
                depth, alpha, beta, move_str));
            self.increase_indent();
        }
    }

    fn log_depth_exit(&mut self, depth: i32, final_score: i32, best_move: Option<Move>) {
        if self.should_log_advanced() {
            self.decrease_indent();
            let best_move_str = if let Some(mv) = best_move {
                move_to_string(mv)
            } else {
                "None".to_string()
            };
            self.log(&format!("🏁 === EXITING DEPTH {} === Score: {}, Best: {}", 
                depth, final_score, best_move_str));
        }
    }

    fn log_available_moves_at_depth(&mut self, depth: i32, moves: &[Move]) {
        if self.should_log_advanced() {
            self.log_with_indent(&format!("📋 Available moves at depth {} ({} moves):", depth, moves.len()));
            self.increase_indent();
            for (i, &mv) in moves.iter().enumerate() {
                self.log_with_indent(&format!("{:2}. {}", i + 1, move_to_string(mv)));
            }
            self.decrease_indent();
        }
    }

    fn log_move_ordering_at_depth(&mut self, depth: i32, ordered_moves: &[Move], board: &Board) {
        if self.should_log_advanced() {
            self.log_with_indent(&format!("🎯 Move ordering at depth {}:", depth));
            self.increase_indent();
            for (i, &mv) in ordered_moves.iter().enumerate() {
                let to_piece = board.get_piece(mv.to);
                let capture = if !engine::is_empty(to_piece) { " [CAPTURE]" } else { "" };
                self.log_with_indent(&format!("{:2}. {}{}", i + 1, move_to_string(mv), capture));
            }
            self.decrease_indent();
        }
    }

    fn log_move_exploration_start(&mut self, mv: Move, move_num: usize, total_moves: usize, depth: i32, alpha: i32, beta: i32) {
        if self.should_log_advanced() {
            self.log(&format!("🔄 === EXPLORING MOVE {}/{}: {} (depth {}) ===", 
                move_num, total_moves, move_to_string(mv), depth));
            self.log(&format!("   Window: α={}, β={}", alpha, beta));
            self.increase_indent();
        }
    }

    fn log_move_exploration_result(&mut self, mv: Move, score: i32, alpha: i32, beta: i32, depth: i32) {
        if self.should_log_advanced() {
            let status = if score >= beta {
                "✂️ CUTOFF!"
            } else if score > alpha {
                "📈 IMPROVED"
            } else {
                "📊 NORMAL"
            };
            
            self.decrease_indent();
            self.log(&format!("✅ {} → Score: {} | {} (depth {})", 
                move_to_string(mv), score, status, depth));
        }
    }

    fn log_leaf_evaluation(&mut self, depth: i32, eval_score: i32) {
        if self.should_log_advanced() {
            self.log_with_indent(&format!("🍃 LEAF NODE (depth {}) → Evaluation: {}", depth, eval_score));
        }
    }

    fn log_alphabeta_node_complete(&mut self, depth: i32, final_alpha: i32, node_type: crate::transposition::NodeType) {
        if self.should_log_advanced() && depth >= 2 {
            self.decrease_indent();
            let node_type_str = match node_type {
                crate::transposition::NodeType::Exact => "EXACT",
                crate::transposition::NodeType::LowerBound => "LOWER",
                crate::transposition::NodeType::UpperBound => "UPPER",
            };
            self.log_with_indent(&format!("└─ Node complete: α={} [{}]", final_alpha, node_type_str));
        }
    }
}

fn move_to_string(mv: Move) -> String {
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

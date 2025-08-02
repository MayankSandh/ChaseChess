use crate::types::*;
use super::Board;

impl Board {
    /// Execute a move and update game state
    pub fn try_make_move(&mut self, mv: Move) -> Result<GameMove, String> {
        if !self.is_valid_move(mv) {
            return Err("Invalid move".to_string());
        }
    
        match self.game_status {
            GameStatus::InProgress | GameStatus::Check(_) => {
                // Game can continue
            },
            _ => {
                return Err("Game is over".to_string());
            }
        }
    
        let captured_piece = self.get_piece(mv.to);
        let moving_piece = self.get_piece(mv.from);
    
        // CHECK FOR SPECIAL MOVES FIRST (before clearing en passant)
        let is_castling = self.is_castling_move(mv).is_some();
        let is_en_passant = self.is_en_passant_move(mv);
    
        // THEN clear en passant target for next move

        self.en_passant_target = None;
        self.en_passant_pawn = None;
    
        let mut game_move = if is_en_passant {
            let captured_pawn = self.get_piece(self.en_passant_pawn.unwrap_or(mv.to));
            GameMove::with_capture_and_state(mv, captured_pawn, self)
        } else if is_empty(captured_piece) {
            GameMove::new_with_state(mv, self)
        } else {
            GameMove::with_capture_and_state(mv, captured_piece, self)
        };
    
        game_move.is_castling = is_castling;
        game_move.is_en_passant = is_en_passant;
        game_move.promotion = mv.promotion;
    
        if !is_castling && !is_en_passant {
            self.update_castling_rights_fixed(mv, moving_piece, captured_piece);
        }
    
        // Execute the move
        if is_castling {
            let kingside = self.is_castling_move(mv).unwrap();
            self.execute_castling(piece_color(moving_piece), kingside);
        } else if is_en_passant {
            self.execute_en_passant(mv);
        } else {
            if mv.is_promotion() {
                let promoted_piece = make_piece(mv.promotion.unwrap(), piece_color(moving_piece));
                self.set_piece(mv.to, promoted_piece);
            } else {
                self.set_piece(mv.to, moving_piece);
            }
            self.set_piece(mv.from, EMPTY);
        }
    
        // NEW EN PASSANT LOGIC: Only set if current move is double pawn push
        if !is_castling && !is_en_passant {
            self.setup_en_passant_fixed(mv);
        }
    
        self.move_history.push(game_move.clone());
        self.current_turn = opposite_color(self.current_turn);
    
        if piece_type(moving_piece) == PAWN || !is_empty(captured_piece) || is_en_passant {
            self.half_move_clock = 0;
        } else {
            self.half_move_clock += 1;
        }
    
        if self.current_turn == WHITE {
            self.full_move_number += 1;
        }
        self.update_game_status();
        Ok(game_move)
    }
    
    
    pub fn update_castling_rights_fixed(&mut self, mv: Move, moving_piece: Piece, captured_piece: Piece) {
        let piece_color_val = piece_color(moving_piece);
    
        if piece_type(moving_piece) == KING {
            if piece_color_val == WHITE {
                remove_castling_right(&mut self.castling_rights, WHITE_KINGSIDE);
                remove_castling_right(&mut self.castling_rights, WHITE_QUEENSIDE);
            } else {
                remove_castling_right(&mut self.castling_rights, BLACK_KINGSIDE);
                remove_castling_right(&mut self.castling_rights, BLACK_QUEENSIDE);
            }
        }
    
        // Handle rook moves (from square)
        if piece_type(moving_piece) == ROOK {
            match (mv.from.file(), mv.from.rank()) {
                (0, 0) => remove_castling_right(&mut self.castling_rights, WHITE_QUEENSIDE), // a1
                (7, 0) => remove_castling_right(&mut self.castling_rights, WHITE_KINGSIDE),  // h1
                (0, 7) => remove_castling_right(&mut self.castling_rights, BLACK_QUEENSIDE), // a8
                (7, 7) => remove_castling_right(&mut self.castling_rights, BLACK_KINGSIDE),  // h8
                _ => {}
            }
        }
    
        // Handle captured rooks (to square) - ✅ NOW we have the captured piece!
        if piece_type(captured_piece) == ROOK {
            match (mv.to.file(), mv.to.rank()) {
                (0, 0) => remove_castling_right(&mut self.castling_rights, WHITE_QUEENSIDE), // a1
                (7, 0) => remove_castling_right(&mut self.castling_rights, WHITE_KINGSIDE),  // h1
                (0, 7) => remove_castling_right(&mut self.castling_rights, BLACK_QUEENSIDE), // a8
                (7, 7) => remove_castling_right(&mut self.castling_rights, BLACK_KINGSIDE),  // h8
                _ => {}
            }
        }
    }
    

    /// Undo the last move made
    pub fn undo_move(&mut self) -> Result<GameMove, String> {
        // Get the last move from history
        let last_move = match self.move_history.pop() {
            Some(mv) => mv,
            None => return Err("No moves to undo".to_string()),
        };

        // Restore the pieces on the board
        self.restore_pieces(&last_move);

        // Restore all board state
        self.castling_rights = last_move.previous_castling_rights;
        self.en_passant_target = last_move.previous_en_passant_target;
        self.en_passant_pawn = last_move.previous_en_passant_pawn;
        self.half_move_clock = last_move.previous_half_move_clock;
        self.full_move_number = last_move.previous_full_move_number;

        // Switch turn back
        self.current_turn = opposite_color(self.current_turn);

        Ok(last_move)
    }

    /// Restore pieces after undoing a move
    fn restore_pieces(&mut self, game_move: &GameMove) {
        let mv = game_move.mv;

        if game_move.is_castling {
            self.undo_castling(mv);
        } else if game_move.is_en_passant {
            self.undo_en_passant(game_move);
        } else if game_move.promotion.is_some() {
            // PROMOTION UNDO: Restore original pawn, not promoted piece
            let original_pawn_color = if mv.from.rank() == 6 { WHITE } else { BLACK };
            let original_pawn = make_piece(PAWN, original_pawn_color);
            
            self.set_piece(mv.from, original_pawn);
            self.set_piece(mv.to, game_move.captured_piece);
        } else {
            // Regular move - move piece back and restore captured piece
            let moving_piece = self.get_piece(mv.to);
            self.set_piece(mv.from, moving_piece);
            self.set_piece(mv.to, game_move.captured_piece);
        }
    }


    /// Check if castling is possible for a given color and side
    pub fn can_castle(&self, color: u8, kingside: bool) -> bool {
        // Determine castling right to check
        let castling_right = match (color, kingside) {
            (WHITE, true) => WHITE_KINGSIDE,
            (WHITE, false) => WHITE_QUEENSIDE,
            (BLACK, true) => BLACK_KINGSIDE,
            (BLACK, false) => BLACK_QUEENSIDE,
            _ => return false,
        };

        // Check if we have castling rights
        if !has_castling_right(self.castling_rights, castling_right) {
            return false;
        }

        // Determine squares involved
        let king_rank = if color == WHITE { 0 } else { 7 };
        let king_start = Square::new(4, king_rank); // e1 or e8

        let (king_end, rook_start, squares_to_check) = if kingside {
            // Kingside castling
            let king_end = Square::new(6, king_rank); // g1 or g8
            let rook_start = Square::new(7, king_rank); // h1 or h8
            let squares = vec![
                Square::new(5, king_rank), // f1 or f8
                Square::new(6, king_rank), // g1 or g8
            ];
            (king_end, rook_start, squares)
        } else {
            // Queenside castling
            let king_end = Square::new(2, king_rank); // c1 or c8
            let rook_start = Square::new(0, king_rank); // a1 or a8
            let squares = vec![
                Square::new(1, king_rank), // b1 or b8
                Square::new(2, king_rank), // c1 or c8
                Square::new(3, king_rank), // d1 or d8
            ];
            (king_end, rook_start, squares)
        };

        // Check if king and rook are in correct positions
        let king_piece = self.get_piece(king_start);
        let rook_piece = self.get_piece(rook_start);

        if piece_type(king_piece) != KING || piece_color(king_piece) != color {
            return false;
        }

        if piece_type(rook_piece) != ROOK || piece_color(rook_piece) != color {
            return false;
        }

        // Check if path is clear
        for &square in &squares_to_check {
            if !is_empty(self.get_piece(square)) {
                return false;
            }
        }

        // Check if king is currently in check
        let opponent_color = opposite_color(color);
        if self.is_under_threat(king_start, opponent_color) {
            return false;
        }

        // Check if king passes through or ends in check
        if self.is_under_threat(king_end, opponent_color) {
            return false;
        }

        // For castling, also check the square king passes through
        let king_path_square = if kingside {
            Square::new(5, king_rank) // f1 or f8
        } else {
            Square::new(3, king_rank) // d1 or d8
        };

        if self.is_under_threat(king_path_square, opponent_color) {
            return false;
        }

        true
    }

    /// Execute a castling move
    fn execute_castling(&mut self, color: u8, kingside: bool) {
        let king_rank = if color == WHITE { 0 } else { 7 };
        let king_start = Square::new(4, king_rank);

        let (king_end, rook_start, rook_end) = if kingside {
            (
                Square::new(6, king_rank), // King to g1/g8
                Square::new(7, king_rank), // Rook from h1/h8
                Square::new(5, king_rank), // Rook to f1/f8
            )
        } else {
            (
                Square::new(2, king_rank), // King to c1/c8
                Square::new(0, king_rank), // Rook from a1/a8
                Square::new(3, king_rank), // Rook to d1/d8
            )
        };

        // Move the pieces
        let king_piece = self.get_piece(king_start);
        let rook_piece = self.get_piece(rook_start);

        self.set_piece(king_end, king_piece);
        self.set_piece(rook_end, rook_piece);
        self.set_piece(king_start, EMPTY);
        self.set_piece(rook_start, EMPTY);

        // Remove all castling rights for this color
        if color == WHITE {
            remove_castling_right(&mut self.castling_rights, WHITE_KINGSIDE);
            remove_castling_right(&mut self.castling_rights, WHITE_QUEENSIDE);
        } else {
            remove_castling_right(&mut self.castling_rights, BLACK_KINGSIDE);
            remove_castling_right(&mut self.castling_rights, BLACK_QUEENSIDE);
        }
    }

    /// Check if a move is a castling move
    pub fn is_castling_move(&self, mv: Move) -> Option<bool> {
        let from_piece = self.get_piece(mv.from);

        // Must be a king move
        if piece_type(from_piece) != KING {
            return None;
        }

        let from_file = mv.from.file();
        let to_file = mv.to.file();
        let from_rank = mv.from.rank();
        let to_rank = mv.to.rank();

        // Must be on same rank
        if from_rank != to_rank {
            return None;
        }

        // Must be from e-file
        if from_file != 4 {
            return None;
        }

        // Check for castling pattern
        match to_file {
            6 => Some(true),  // Kingside (g-file)
            2 => Some(false), // Queenside (c-file)
            _ => None,
        }
    }

    /// Undo castling move
    fn undo_castling(&mut self, mv: Move) {
        let king_rank = mv.to.rank();

        // Determine if it was kingside or queenside castling
        let is_kingside = mv.to.file() == 6; // g-file

        if is_kingside {
            // Undo kingside castling
            let king = self.get_piece(Square::new(6, king_rank));
            let rook = self.get_piece(Square::new(5, king_rank));

            self.set_piece(Square::new(4, king_rank), king); // King back to e-file
            self.set_piece(Square::new(7, king_rank), rook); // Rook back to h-file
            self.set_piece(Square::new(6, king_rank), EMPTY);
            self.set_piece(Square::new(5, king_rank), EMPTY);
        } else {
            // Undo queenside castling
            let king = self.get_piece(Square::new(2, king_rank));
            let rook = self.get_piece(Square::new(3, king_rank));

            self.set_piece(Square::new(4, king_rank), king); // King back to e-file
            self.set_piece(Square::new(0, king_rank), rook); // Rook back to a-file
            self.set_piece(Square::new(2, king_rank), EMPTY);
            self.set_piece(Square::new(3, king_rank), EMPTY);
        }
    }

    /// Update castling rights after a move
    pub fn update_castling_rights(&mut self, mv: Move) {
        let from_piece = self.get_piece(mv.from);
        let to_piece = self.get_piece(mv.to);
        let piece_color_val = piece_color(from_piece);

        // If king moves, remove all castling rights for that color
        if piece_type(from_piece) == KING {
            if piece_color_val == WHITE {
                remove_castling_right(&mut self.castling_rights, WHITE_KINGSIDE);
                remove_castling_right(&mut self.castling_rights, WHITE_QUEENSIDE);
            } else {
                remove_castling_right(&mut self.castling_rights, BLACK_KINGSIDE);
                remove_castling_right(&mut self.castling_rights, BLACK_QUEENSIDE);
            }
        }

        // If rook moves or is captured, remove corresponding castling right
        if piece_type(from_piece) == ROOK || piece_type(to_piece) == ROOK {
            let squares_to_check = [mv.from, mv.to];

            for square in squares_to_check {
                match (square.file(), square.rank()) {
                    (0, 0) => remove_castling_right(&mut self.castling_rights, WHITE_QUEENSIDE), // a1
                    (7, 0) => remove_castling_right(&mut self.castling_rights, WHITE_KINGSIDE),  // h1
                    (0, 7) => remove_castling_right(&mut self.castling_rights, BLACK_QUEENSIDE), // a8
                    (7, 7) => remove_castling_right(&mut self.castling_rights, BLACK_KINGSIDE),  // h8
                    _ => {}
                }
            }
        }
    }

    /// Set up en passant target after a double pawn push 
    pub fn setup_en_passant_fixed(&mut self, mv: Move) {
        let moving_piece = self.get_piece(mv.to);
        
        // Must be a pawn
        if piece_type(moving_piece) != PAWN {
            return;
        }
        
        let from_rank = mv.from.rank();
        let to_rank = mv.to.rank();
        let color = piece_color(moving_piece);
        
        // Check if this is a double pawn push
        let rank_diff = (to_rank as i8 - from_rank as i8).abs();
        if rank_diff == 2 {
            let starting_rank = if color == WHITE { 1 } else { 6 };
            if from_rank == starting_rank {
                // Set en passant target square (the square the pawn "jumped over")
                let en_passant_rank = if color == WHITE { 2 } else { 5 };
                self.en_passant_target = Some(Square::new(mv.from.file(), en_passant_rank));
                self.en_passant_pawn = Some(mv.to); // The pawn that can be captured
            }
        }
        // If not a double pawn push, en passant stays cleared (already cleared above)
    }


    /// Check if a move is a double pawn push that enables en passant
    fn is_double_pawn_push(&self, mv: Move) -> bool {
        let moving_piece = self.get_piece(mv.to);

        // Must be a pawn
        if piece_type(moving_piece) != PAWN {
            return false;
        }

        let from_rank = mv.from.rank();
        let to_rank = mv.to.rank();
        let color = piece_color(moving_piece);

        // Must move exactly 2 squares
        let rank_diff = (to_rank as i8 - from_rank as i8).abs();
        if rank_diff != 2 {
            return false;
        }

        // Must be from starting position
        let starting_rank = if color == WHITE { 1 } else { 6 };
        if from_rank != starting_rank {
            return false;
        }

        // Must be moving in correct direction
        let expected_direction = if color == WHITE { 1 } else { -1 };
        let actual_direction = to_rank as i8 - from_rank as i8;
        actual_direction == expected_direction * 2
    }

    /// Check if a move is an en passant capture
    /// Check if a move is an en passant capture
    pub fn is_en_passant_move(&self, mv: Move) -> bool {
        
        // Must be a pawn move
        let moving_piece = self.get_piece(mv.from);
        
        if piece_type(moving_piece) != PAWN {
            return false;
        }

        // Must have an en passant target
        let target = match self.en_passant_target {
            Some(square) => {
                square
            },
            None => {
                return false;
            }
        };

        
        if mv.to != target {
            return false;
        }

        // Target square must be empty (pawn moves diagonally to empty square)
        let target_piece = self.get_piece(mv.to);
        
        if !is_empty(target_piece) {
            return false;
        }

        // Must be a diagonal move
        let file_diff = (mv.to.file() as i8 - mv.from.file() as i8).abs();
        let rank_diff = (mv.to.rank() as i8 - mv.from.rank() as i8).abs();
        
        if !(file_diff == 1 && rank_diff == 1) {
            return false;
        }
        
        true
    }


    /// Execute an en passant capture
    fn execute_en_passant(&mut self, mv: Move) {
        let moving_piece = self.get_piece(mv.from);
        let moving_color = piece_color(moving_piece);
        
        // Move the capturing pawn to the target square
        self.set_piece(mv.to, moving_piece);
        self.set_piece(mv.from, EMPTY);
        
        // CRITICAL FIX: Remove the captured pawn from its original square
        let captured_pawn_square = if moving_color == WHITE {
            Square::new(mv.to.file(), mv.to.rank() - 1) // Remove Black pawn below
        } else {
            Square::new(mv.to.file(), mv.to.rank() + 1) // Remove White pawn above
        };
        
        // ← THIS IS THE KEY LINE THAT REMOVES THE CAPTURED PAWN
        self.set_piece(captured_pawn_square, EMPTY);
        
        // Clear en passant state
        self.en_passant_target = None;
        self.en_passant_pawn = None;
        
        // Update castling rights if needed
        self.update_castling_rights(mv);
    }

    
    

    /// Undo an en passant capture
    fn undo_en_passant(&mut self, game_move: &GameMove) {
        let mv = game_move.mv;

        // Move our pawn back
        let our_pawn = self.get_piece(mv.to);
        self.set_piece(mv.from, our_pawn);
        self.set_piece(mv.to, EMPTY);

        // Restore the captured pawn
        let captured_pawn_square = if piece_color(our_pawn) == WHITE {
            Square::new(mv.to.file(), mv.to.rank() - 1) // Black pawn was one rank below
        } else {
            Square::new(mv.to.file(), mv.to.rank() + 1) // White pawn was one rank above
        };

        self.set_piece(captured_pawn_square, game_move.captured_piece);
    }

    /// Update game status (basic implementation for now)
    pub fn update_game_status(&mut self) {
        // For now, just set to InProgress
        // In a more complete implementation, we'd add proper check/checkmate detection
        self.game_status = GameStatus::InProgress;
    }

    /// Get the last move made
    pub fn get_last_move(&self) -> Option<&GameMove> {
        self.move_history.last()
    }
}

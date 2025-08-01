use crate::types::*;
use super::Board;
use crate::bitboard::{iterate_bits, index_to_square, get_knight_attacks, get_king_attacks};



impl Board {
    /// Generate all legal moves for the current player
    /// OPTIMIZED: Uses bitboards to iterate only over squares with our pieces instead of all 64 squares
    pub fn get_all_legal_moves(&self) -> Vec<Move> {
        let mut all_moves = Vec::new();

        // OPTIMIZATION: Get all pieces of current color using bitboards - O(1) operation
        let our_pieces = self.bitboards.get_all_pieces(self.current_turn);
        
        // OPTIMIZATION: Iterate only over squares with our pieces - O(actual_pieces) instead of O(64)
        for square_index in iterate_bits(our_pieces) {
            let square = index_to_square(square_index);
            let piece = self.get_piece(square);
            
            // We know this square has our piece, so no empty check needed
            let piece_moves = self.get_legal_moves(square);
            let piece_type_val = piece_type(piece);
            
            for target_square in piece_moves {
                // Check if this is a pawn promotion
                if piece_type_val == PAWN {
                    let promotion_rank = if piece_color(piece) == WHITE { 7 } else { 0 };
                    if target_square.rank() == promotion_rank {
                        // Generate 4 promotion moves - ✅ REMOVE DOUBLE VALIDATION
                        for &promotion_piece in &[QUEEN, ROOK, BISHOP, KNIGHT] {
                            let promotion_move = Move::new_promotion(square, target_square, promotion_piece);
                            all_moves.push(promotion_move); // ✅ No extra validation needed
                        }
                    } else {
                        // Regular pawn move
                        let regular_move = Move::new(square, target_square);
                        all_moves.push(regular_move); // ✅ No extra validation needed
                    }
                } else {
                    // Non-pawn move
                    let regular_move = Move::new(square, target_square);
                    all_moves.push(regular_move); // ✅ No extra validation needed
                }
            }
        }
        
        all_moves
    }

    /// Get legal moves for a piece at the given square
    pub fn get_legal_moves(&self, square: Square) -> Vec<Square> {
        // Get pseudo-legal moves first
        let pseudo_moves = self.get_pseudo_legal_moves(square);
        
        // Check if our king is in check
        let our_color = self.current_turn;
        let king_square = match self.find_king(our_color) {
            Some(square) => square,
            None => return Vec::new(), // No king found
        };
        
        let opponent_color = opposite_color(our_color);
        let checking_pieces = self.find_checking_pieces(king_square, opponent_color);
        
        match checking_pieces.len() {
            0 => {
                // Not in check, but still need to validate king moves
                let piece = self.get_piece(square);
                if piece_type(piece) == KING {
                    *self.ignore_square_for_threats.borrow_mut() = Some(square);
                    let filtered_moves = self.filter_king_moves_in_check(pseudo_moves, opponent_color);
                    *self.ignore_square_for_threats.borrow_mut() = None;
                    filtered_moves
                } else {
                    pseudo_moves
                }
            }
            1 => {
                // Single check - can block or capture
                let checking_piece_square = checking_pieces[0];
                let blocking_squares = self.get_blocking_squares(king_square, checking_piece_square);
                let piece = self.get_piece(square);
                
                if piece_type(piece) == KING {
                    *self.ignore_square_for_threats.borrow_mut() = Some(square);
                    let filtered_moves = self.filter_king_moves_in_check(pseudo_moves, opponent_color);
                    *self.ignore_square_for_threats.borrow_mut() = None;
                    filtered_moves
                } else {
                    // ✅ FIX: Handle en passant moves specially during check resolution
                    pseudo_moves.into_iter()
                        .filter(|&mv| {
                            // Normal case: move blocks or captures checking piece
                            if blocking_squares.contains(&mv) {
                                return true;
                            }
                            
                            // ✅ SPECIAL CASE: En passant that removes the checking piece
                            if self.is_en_passant_move(Move::new(square, mv)) {
                                // Check if this en passant removes the checking piece
                                if let Some(en_passant_pawn_square) = self.en_passant_pawn {
                                    return en_passant_pawn_square == checking_piece_square;
                                }
                            }
                            
                            false
                        })
                        .collect()
                }
            }
            _ => {
                // Double check - only king moves are legal
                let piece = self.get_piece(square);
                if piece_type(piece) == KING {
                    *self.ignore_square_for_threats.borrow_mut() = Some(square);
                    let filtered_moves = self.filter_king_moves_in_check(pseudo_moves, opponent_color);
                    *self.ignore_square_for_threats.borrow_mut() = None;
                    filtered_moves
                } else {
                    Vec::new()
                }
            }
        }
    }
    
    /// Get pseudo-legal moves (before checking for check/pins)
    pub fn get_pseudo_legal_moves(&self, square: Square) -> Vec<Square> {
        let piece = self.get_piece(square);
        if is_empty(piece) {
            return Vec::new();
        }

        // Only generate moves for current player's pieces
        if !is_piece_color(piece, self.current_turn) {
            return Vec::new();
        }

        // Check if piece is pinned
        if let Some(pin_direction) = self.is_piece_pinned(square) {
            let pinned_moves = self.get_pinned_piece_moves(square, pin_direction);
            return pinned_moves;
        }


        // Generate normal moves for non-pinned pieces
        match piece_type(piece) {
            KNIGHT => self.get_knight_moves(square),
            ROOK => self.get_rook_moves(square),
            BISHOP => self.get_bishop_moves(square),
            QUEEN => self.get_queen_moves(square),
            KING => self.get_king_moves(square),
            PAWN => self.get_pawn_moves(square, piece_color(piece)),
            _ => Vec::new(),
        }
    }

    /// Generate pawn moves 
    pub fn get_pawn_moves(&self, square: Square, color: u8) -> Vec<Square> {
        let mut moves = Vec::new();
        let file = square.file();
        let rank = square.rank();
        
        // Determine direction based on color (White moves up, Black moves down)
        let direction = if color == WHITE { 1 } else { -1 };
        
        // Forward moves
        let new_rank = rank as i8 + direction;
        if new_rank >= 0 && new_rank < 8 {
            let forward_square = Square::new(file, new_rank as u8);
            let forward_piece = self.get_piece(forward_square);
            
            // Single forward move (only if square is empty)
            if is_empty(forward_piece) {
                moves.push(forward_square);
                
                // Double forward move from starting position (only if both squares are empty)
                let starting_rank = if color == WHITE { 1 } else { 6 };
                if rank == starting_rank {
                    let double_forward_rank = new_rank + direction;
                    if double_forward_rank >= 0 && double_forward_rank < 8 {
                        let double_forward_square = Square::new(file, double_forward_rank as u8);
                        let double_forward_piece = self.get_piece(double_forward_square);
                        
                        if is_empty(double_forward_piece) {
                            moves.push(double_forward_square);
                        }
                    }
                }
            }
        }
        
        // Diagonal captures
        for df in [-1, 1] {
            let new_file = file as i8 + df;
            let new_rank = rank as i8 + direction;
            
            if new_file >= 0 && new_file < 8 && new_rank >= 0 && new_rank < 8 {
                let capture_square = Square::new(new_file as u8, new_rank as u8);
                let target_piece = self.get_piece(capture_square);
                
                // Only generate diagonal capture if enemy piece exists
                if !is_empty(target_piece) && piece_color(target_piece) != color {
                    moves.push(capture_square);
                }
            }
        }
        
        // ENHANCED EN PASSANT VALIDATION
        if let Some(en_passant_square) = self.en_passant_target {
            let en_passant_rank = en_passant_square.rank() as i8;
            let en_passant_file = en_passant_square.file() as i8;
            
            // Check if we can capture en passant (diagonally adjacent)
            for df in [-1, 1] {
                let pawn_file = file as i8;
                if pawn_file + df == en_passant_file && (rank as i8 + direction) == en_passant_rank {
                    // ENHANCED VALIDATION:
                    
                    // 1. Check if we're on the correct rank for en passant
                    let correct_rank = if color == WHITE { 4 } else { 3 }; // 5th rank for White, 4th for Black
                    if rank != correct_rank {
                        continue;
                    }
                    
                    // 2. Check if there's actually an enemy pawn to capture
                    let enemy_pawn_square = Square::new(en_passant_file as u8, rank);
                    let enemy_piece = self.get_piece(enemy_pawn_square);
                    
                    // 3. CRITICAL: Validate enemy pawn exists and is correct color/type
                    if is_empty(enemy_piece) || 
                    piece_type(enemy_piece) != PAWN || 
                    piece_color(enemy_piece) == color {
                        continue;
                    }
                    
                    // 4. ADDITIONAL: Validate en passant square is empty
                    let en_passant_piece = self.get_piece(en_passant_square);
                    if !is_empty(en_passant_piece) {
                        continue;
                    }
                    
                    moves.push(en_passant_square);
                }
            }
        }
        
        moves
    }

    /// Generate knight moves
    fn get_knight_moves(&self, square: Square) -> Vec<Square> {
        // OPTIMIZED: Use pre-computed knight attack mask instead of manual offsets
        let knight_attack_mask = get_knight_attacks(square.0);
        
        // Can't capture our own pieces
        let our_pieces = self.bitboards.get_all_pieces(self.current_turn);
        let valid_moves = knight_attack_mask & !our_pieces;
        
        // Convert bitboard to squares
        let mut moves = Vec::new();
        let mut remaining_moves = valid_moves;
        
        while remaining_moves != 0 {
            let square_index = remaining_moves.trailing_zeros() as u8;
            moves.push(index_to_square(square_index));
            remaining_moves &= remaining_moves - 1; // Remove the processed bit
        }
        
        moves
    }
    

    /// Generate bishop moves
    fn get_bishop_moves(&self, square: Square) -> Vec<Square> {
        let directions = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
        self.get_sliding_moves(square, &directions)
    }

    /// Generate rook moves
    fn get_rook_moves(&self, square: Square) -> Vec<Square> {
        let directions = [(0, 1), (0, -1), (1, 0), (-1, 0)];
        self.get_sliding_moves(square, &directions)
    }

    /// Generate queen moves
    fn get_queen_moves(&self, square: Square) -> Vec<Square> {
        let mut moves = self.get_rook_moves(square);
        moves.extend(self.get_bishop_moves(square));
        moves
    }

    /// Generate king moves - OPTIMIZED with bitboard lookups
    fn get_king_moves(&self, square: Square) -> Vec<Square> {
        let source_color = piece_color(self.get_piece(square));
        
        // OPTIMIZED: Use pre-computed king attack mask instead of nested loops
        let king_attack_mask = get_king_attacks(square.0);
        
        // Filter out squares occupied by our own pieces
        let our_pieces = self.bitboards.get_all_pieces(source_color);
        let valid_moves = king_attack_mask & !our_pieces;
        
        // Convert bitboard to squares
        let mut moves = Vec::new();
        let mut remaining_moves = valid_moves;
        
        while remaining_moves != 0 {
            let square_index = remaining_moves.trailing_zeros() as u8;
            moves.push(index_to_square(square_index));
            remaining_moves &= remaining_moves - 1; // Remove the processed bit
        }
        
        // Add castling moves (unchanged - castling logic remains the same)
        if self.can_castle(source_color, true) {
            // Kingside castling
            let king_rank = square.rank();
            moves.push(Square::new(6, king_rank)); // g1 or g8
        }

        if self.can_castle(source_color, false) {
            // Queenside castling
            let king_rank = square.rank();
            moves.push(Square::new(2, king_rank)); // c1 or c8
        }

        moves
    }


    /// Generate sliding piece moves in given directions
    fn get_sliding_moves(&self, square: Square, directions: &[(i8, i8)]) -> Vec<Square> {
        let mut moves = Vec::new();
        let file = square.file() as i8;
        let rank = square.rank() as i8;
        let source_color = piece_color(self.get_piece(square));

        for &(df, dr) in directions {
            for distance in 1..8 {
                let new_file = file + df * distance;
                let new_rank = rank + dr * distance;

                if new_file < 0 || new_file >= 8 || new_rank < 0 || new_rank >= 8 {
                    break; // Off the board
                }

                let target_square = Square::new(new_file as u8, new_rank as u8);
                let target_piece = self.get_piece(target_square);

                if is_empty(target_piece) {
                    moves.push(target_square); // Empty square, can move
                } else if piece_color(target_piece) != source_color {
                    moves.push(target_square); // Enemy piece, can capture
                    break; // Can't continue beyond this piece
                } else {
                    break; // Own piece, can't move here or beyond
                }
            }
        }

        moves
    }

    /// Generate moves for a pinned piece (only along pin line)
    pub fn get_pinned_piece_moves(&self, square: Square, pin_direction: (i8, i8)) -> Vec<Square> {
        let piece = self.get_piece(square);
        let piece_type_val = piece_type(piece);
        let source_color = piece_color(piece);

        // Knights can't move when pinned
        if piece_type_val == KNIGHT {
            return Vec::new();
        }

        // Pawns have special pinning rules
        if piece_type_val == PAWN {
            return self.get_pinned_pawn_moves(square, pin_direction, source_color);
        }

        // FIX: Check if the piece can actually move in the pin direction
        let can_move_in_pin_direction = match piece_type_val {
            ROOK => {
                // Rook can only move horizontally/vertically
                pin_direction.0 == 0 || pin_direction.1 == 0
            }
            BISHOP => {
                // Bishop can only move diagonally
                pin_direction.0.abs() == pin_direction.1.abs() && pin_direction.0 != 0
            }
            QUEEN => {
                // Queen can move in any direction
                true
            }
            _ => false,
        };

        if !can_move_in_pin_direction {
            return Vec::new();
        }

        // For sliding pieces that CAN move in pin direction, generate moves along pin line
        let mut moves = Vec::new();
        for direction in [pin_direction, (-pin_direction.0, -pin_direction.1)] {
            moves.extend(self.get_moves_in_direction(square, direction, source_color));
        }
        moves
    }


    /// Generate moves in a specific direction
    fn get_moves_in_direction(&self, square: Square, direction: (i8, i8), source_color: u8) -> Vec<Square> {
        let mut moves = Vec::new();
        let mut file = square.file() as i8 + direction.0;
        let mut rank = square.rank() as i8 + direction.1;

        while file >= 0 && file < 8 && rank >= 0 && rank < 8 {
            let target_square = Square::new(file as u8, rank as u8);
            let target_piece = self.get_piece(target_square);

            if is_empty(target_piece) {
                moves.push(target_square);
            } else if piece_color(target_piece) != source_color {
                moves.push(target_square); // Can capture
                break;
            } else {
                break; // Hit own piece
            }

            file += direction.0;
            rank += direction.1;
        }

        moves
    }

    /// Generate moves for a pinned pawn 
    fn get_pinned_pawn_moves(&self, square: Square, pin_direction: (i8, i8), color: u8) -> Vec<Square> {
        let mut moves = Vec::new();
        let file = square.file();
        let rank = square.rank();
        let direction = if color == WHITE { 1 } else { -1 };
    
        // Case 1: Vertically pinned (along the same file)
        if pin_direction.0 == 0 {
            // Pawn can ONLY move forward along the pin line, NEVER backward
            let forward_rank = rank as i8 + direction;
            if forward_rank >= 0 && forward_rank < 8 {
                let target_square = Square::new(file, forward_rank as u8);
                let target_piece = self.get_piece(target_square);
                
                // Only forward moves, only if empty
                if is_empty(target_piece) {
                    moves.push(target_square);
                    
                    // Check double move from starting position
                    let starting_rank = if color == WHITE { 1 } else { 6 };
                    if rank == starting_rank {
                        let double_rank = forward_rank + direction;
                        if double_rank >= 0 && double_rank < 8 {
                            let double_square = Square::new(file, double_rank as u8);
                            if is_empty(self.get_piece(double_square)) {
                                moves.push(double_square);
                            }
                        }
                    }
                }
            }
        }
        // Case 2: Diagonally pinned
        else if pin_direction.0.abs() == pin_direction.1.abs() && pin_direction.0 != 0 {
            // Check if the diagonal move is in the forward direction for the pawn
            let forward_rank = rank as i8 + direction;
            
            // Only check the diagonal that's in the pin direction AND forward for the pawn
            for direction_multiplier in [-1, 1] {
                let new_file = file as i8 + (pin_direction.0 * direction_multiplier);
                let new_rank = rank as i8 + (pin_direction.1 * direction_multiplier);
                
                // CRITICAL: Only allow forward moves for pawns
                if new_rank == forward_rank && new_file >= 0 && new_file < 8 {
                    let target_square = Square::new(new_file as u8, new_rank as u8);
                    let target_piece = self.get_piece(target_square);
                    
                    // Only captures on diagonal
                    if !is_empty(target_piece) && piece_color(target_piece) != color {
                        moves.push(target_square);
                    }
                }
            }
            
            // Handle en passant for diagonally pinned pawns (same validation as before)
            if let Some(en_passant_square) = self.en_passant_target {
                let correct_rank = if color == WHITE { 4 } else { 3 };
                if rank == correct_rank {
                    for direction_multiplier in [-1, 1] {
                        let new_file = file as i8 + (pin_direction.0 * direction_multiplier);
                        let new_rank = rank as i8 + (pin_direction.1 * direction_multiplier);
                        
                        if en_passant_square.file() as i8 == new_file && 
                           en_passant_square.rank() as i8 == new_rank &&
                           new_rank == forward_rank { // Must be forward move
                            
                            let enemy_pawn_square = Square::new(en_passant_square.file(), rank);
                            let enemy_piece = self.get_piece(enemy_pawn_square);
                            
                            if !is_empty(enemy_piece) &&
                               piece_type(enemy_piece) == PAWN &&
                               piece_color(enemy_piece) != color &&
                               is_empty(self.get_piece(en_passant_square)) {
                                moves.push(en_passant_square);
                            }
                        }
                    }
                }
            }
        }
        
        moves
    }
    




}

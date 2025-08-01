use crate::types::*;
use super::{Board};
use std::collections::HashSet;
use crate::bitboard::{get_knight_attacks, index_to_square, iterate_bits};


impl Board {
    /// Check if a move is valid
    pub fn is_valid_move(&self, mv: Move) -> bool {
        let from_piece = self.get_piece(mv.from);
        let to_piece = self.get_piece(mv.to);
        
        // Basic validations
        if is_empty(from_piece) {
            return false; // No piece to move
        }
        
        if !is_piece_color(from_piece, self.current_turn) {
            return false; // Not your piece
        }
        
        // For en passant, destination square is empty but it's still a valid capture
        if self.is_en_passant_move(mv) {
            // En passant has its own validation
            return self.is_en_passant_legal(mv);
        }
        
        if is_piece_color(to_piece, self.current_turn) {
            return false; // Can't capture your own piece
        }
        
        // Check if the move is in the piece's legal moves
        let legal_moves = self.get_legal_moves(mv.from);
        if !legal_moves.contains(&mv.to) {
            return false;
        }
        
        true
    }

    /// Check if a square is under threat by the specified color using ray tracing
    pub fn is_under_threat(&self, square: Square, by_color: u8) -> bool {
        
        // Run normal threat detection with king visible (preserves pin detection)
        self.check_sliding_threats(square, by_color) ||
        self.check_knight_threats(square, by_color) ||
        self.check_pawn_threats(square, by_color) ||
        self.check_king_threats(square, by_color)
    }

    /// Check for sliding piece threats (queen, rook, bishop)
    fn check_sliding_threats(&self, square: Square, by_color: u8) -> bool {
        let file = square.file() as i8;
        let rank = square.rank() as i8;
        
        // All 8 directions: 4 rook directions + 4 bishop directions
        let directions = [
            (0, 1), (0, -1), (1, 0), (-1, 0), // Rook directions
            (1, 1), (1, -1), (-1, 1), (-1, -1) // Bishop directions
        ];
        
        for (i, &(df, dr)) in directions.iter().enumerate() {
            let _direction_name = match i {
                0 => "up", 1 => "down", 2 => "right", 3 => "left",
                4 => "up-right", 5 => "down-right", 6 => "up-left", 7 => "down-left",
                _ => "unknown"
            };
            
            
            if let Some(attacking_piece) = self.cast_ray(file, rank, df, dr) {
                let piece_type_val = piece_type(attacking_piece);
                let piece_color_val = piece_color(attacking_piece);
                
                if piece_color_val == by_color {
                    
                    // Check if this piece can attack in this direction
                    if piece_type_val == QUEEN {
                        return true; // Queen attacks in all directions
                    } else if i < 4 && piece_type_val == ROOK {
                        return true; // Rook attacks in first 4 directions (rank/file)
                    } else if i >= 4 && piece_type_val == BISHOP {
                        return true; // Bishop attacks in last 4 directions (diagonal)
                    } 
                } 
            } 
        }

        false
    }



    /// Cast a ray in a direction and return the first piece encountered
    fn cast_ray(&self, start_file: i8, start_rank: i8, df: i8, dr: i8) -> Option<Piece> {
        let mut file = start_file + df;
        let mut rank = start_rank + dr;
        
        // Special debug for the problematic case
        if start_file == 7 && start_rank == 3 && df == -1 && dr == 0 {
            let g4_square = Square::new(6, 3);
            let _g4_piece = self.get_piece(g4_square);
        }
        
        while file >= 0 && file < 8 && rank >= 0 && rank < 8 {
            let target_square = Square::new(file as u8, rank as u8);
            let piece = self.get_piece(target_square);
            
            if !is_empty(piece) {
                return Some(piece);
            }
            
            file += df;
            rank += dr;
        }
        
        None
    }
    

    /// Check for knight threats
    fn check_knight_threats(&self, square: Square, by_color: u8) -> bool {
        let file = square.file() as i8;
        let rank = square.rank() as i8;
        
        let knight_offsets = [
            (-2, -1), (-2, 1), (-1, -2), (-1, 2),
            (1, -2), (1, 2), (2, -1), (2, 1)
        ];
        
        for (df, dr) in knight_offsets {
            let new_file = file + df;
            let new_rank = rank + dr;
            
            if new_file >= 0 && new_file < 8 && new_rank >= 0 && new_rank < 8 {
                let target_square = Square::new(new_file as u8, new_rank as u8);
                let piece = self.get_piece(target_square);
                
                if !is_empty(piece) &&
                   piece_color(piece) == by_color &&
                   piece_type(piece) == KNIGHT {
                    return true;
                }
            }
        }
        
        false
    }

    /// Check for pawn threats
    fn check_pawn_threats(&self, square: Square, by_color: u8) -> bool {
        let file = square.file() as i8;
        let rank = square.rank() as i8;
        
        // Pawn attack direction (opposite of movement direction)
        let attack_direction = if by_color == WHITE { -1 } else { 1 };
        
        // Check both diagonal attack squares
        for df in [-1, 1] {
            let pawn_file = file + df;
            let pawn_rank = rank + attack_direction;
            
            if pawn_file >= 0 && pawn_file < 8 && pawn_rank >= 0 && pawn_rank < 8 {
                let pawn_square = Square::new(pawn_file as u8, pawn_rank as u8);
                let piece = self.get_piece(pawn_square);
                
                if !is_empty(piece) &&
                   piece_color(piece) == by_color &&
                   piece_type(piece) == PAWN {
                    return true;
                }
            }
        }
        
        false
    }

    /// Check for king threats (adjacent squares)
    fn check_king_threats(&self, square: Square, by_color: u8) -> bool {
        let file = square.file() as i8;
        let rank = square.rank() as i8;
        
        for df in -1..=1 {
            for dr in -1..=1 {
                if df == 0 && dr == 0 { continue; }
                
                let king_file = file + df;
                let king_rank = rank + dr;
                
                if king_file >= 0 && king_file < 8 && king_rank >= 0 && king_rank < 8 {
                    let king_square = Square::new(king_file as u8, king_rank as u8);
                    let piece = self.get_piece(king_square);
                    
                    if !is_empty(piece) &&
                       piece_color(piece) == by_color &&
                       piece_type(piece) == KING {
                        return true;
                    }
                }
            }
        }
        
        false
    }

    pub fn find_king(&self, color: u8) -> Option<Square> {
        println!("🔍 DEBUG find_king: Looking for {} king", if color == WHITE { "WHITE" } else { "BLACK" });
        
        // Add this debug line to see what bitboard you're actually getting
        let king_bb = self.bitboards.get_pieces(color, KING);
        println!("🔍 DEBUG find_king: Requested color={}, KING={}, bitboard=0x{:016x}", color, KING, king_bb);
        
        // Also debug what pieces are actually in the bitboards
        let white_king_bb = self.bitboards.get_pieces(WHITE, KING);
        let black_king_bb = self.bitboards.get_pieces(BLACK, KING);
        println!("🔍 DEBUG find_king: WHITE king bitboard = 0x{:016x}", white_king_bb);
        println!("🔍 DEBUG find_king: BLACK king bitboard = 0x{:016x}", black_king_bb);
        
        if king_bb == 0 {
            println!("❌ DEBUG find_king: No king found in bitboards for color {}!", color);
            return None;
        }
        
        let king_square = Square(king_bb.trailing_zeros() as u8);
        println!("✅ DEBUG find_king: Found {} king at {:?}", if color == WHITE { "WHITE" } else { "BLACK" }, king_square);
        
        Some(king_square)
    }
    
    

    /// Find all pieces that are checking the king using optimized algorithm
    pub fn find_checking_pieces(&self, king_square: Square, king_color: u8) -> Vec<Square> {
        let opponent_color = if king_color == WHITE { BLACK } else { WHITE };
        println!("🔍 DEBUG find_checking_pieces: Looking for {} pieces checking {} king at {:?}",
                if opponent_color == WHITE { "WHITE" } else { "BLACK" },
                if king_color == WHITE { "WHITE" } else { "BLACK" },
                king_square);

        let mut checking_pieces = Vec::new();
        
        // Phase 1: Check pawn threats - if found, return immediately (only one pawn check possible)
        if let Some(pawn_check) = self.find_pawn_check(king_square, opponent_color) {
            println!("🔍 DEBUG: Found pawn check at {:?}, returning immediately", pawn_check);
            return vec![pawn_check];
        }
        println!("🔍 DEBUG: No pawn checks found");
        
        // Phase 2: Maintain count variable for other pieces
        let mut count = 0;
        
        // Phase 3: Check knight threats using bitmask AND and trailing_zeros
        if let Some(knight_check) = self.find_knight_check(king_square, opponent_color) {
            println!("🔍 DEBUG: Found knight check at {:?}", knight_check);
            checking_pieces.push(knight_check);
            count += 1;
        } else {
            println!("🔍 DEBUG: No knight checks found");
        }
        
        // Phase 4: Check diagonal directions for enemy bishop/queen
        if let Some(diagonal_check) = self.find_diagonal_check(king_square, opponent_color) {
            println!("🔍 DEBUG: Found diagonal check at {:?}", diagonal_check);
            checking_pieces.push(diagonal_check);
            count += 1;
            
            // If count == 2, return both checks
            if count == 2 {
                println!("🔍 DEBUG: Found 2 checks, returning early: {:?}", checking_pieces);
                return checking_pieces;
            }
        } else {
            println!("🔍 DEBUG: No diagonal checks found");
        }
        
        // Phase 5: Check axial directions for rook/queen
        if let Some(axial_check) = self.find_axial_check(king_square, opponent_color) {
            println!("🔍 DEBUG: Found axial check at {:?}", axial_check);
            checking_pieces.push(axial_check);
            count += 1;
            
            // If count == 2, return both checks
            if count == 2 {
                println!("🔍 DEBUG: Found 2 checks, returning early: {:?}", checking_pieces);
                return checking_pieces;
            }
        } else {
            println!("🔍 DEBUG: No axial checks found");
        }
        
        // Return all checks found
        println!("✅ DEBUG find_checking_pieces: Returning {} checking pieces: {:?}", checking_pieces.len(), checking_pieces);
        checking_pieces
    }


    // Helper function: Find pawn check (only one possible)
    fn find_pawn_check(&self, king_square: Square, opponent_color: u8) -> Option<Square> {
        let king_file = king_square.file() as i8;
        let king_rank = king_square.rank() as i8;
        
        // Pawn attack direction (where pawns could attack from)
        let attack_direction = if opponent_color == WHITE { -1 } else { 1 };
        
        // Check both diagonal squares where attacking pawns could be
        for df in [-1, 1] {
            let pawn_file = king_file + df;
            let pawn_rank = king_rank + attack_direction;
            
            if pawn_file >= 0 && pawn_file < 8 && pawn_rank >= 0 && pawn_rank < 8 {
                let pawn_square = Square::new(pawn_file as u8, pawn_rank as u8);
                
                if self.bitboards.is_occupied_by(pawn_square, opponent_color) {
                    let piece = self.get_piece(pawn_square);
                    if piece_type(piece) == PAWN {
                        return Some(pawn_square);
                    }
                }
            }
        }
        
        None
    }

    // Helper function: Find knight check using your elegant approach
    fn find_knight_check(&self, king_square: Square, opponent_color: u8) -> Option<Square> {
        // Get pre-computed knight attack mask for king's position
        let knight_attack_mask = get_knight_attacks(king_square.0);
        
        // Get opponent's knights
        let opponent_knights = self.bitboards.get_pieces(opponent_color, KNIGHT);
        
        // AND operation - gives us bits set only at attacking knight positions
        let checking_knights = knight_attack_mask & opponent_knights;
        
        if checking_knights != 0 {
            // Get the bit index - that's our knight square!
            let knight_square_index = checking_knights.trailing_zeros() as u8;
            Some(index_to_square(knight_square_index))
        } else {
            None
        }
    }

    // Helper function: Find diagonal check (bishop/queen)
    fn find_diagonal_check(&self, king_square: Square, opponent_color: u8) -> Option<Square> {
        // 4 diagonal directions
        let diagonal_directions = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
        
        for direction in diagonal_directions {
            if let Some(checking_piece) = self.trace_ray_for_check(king_square, direction, opponent_color, &[BISHOP, QUEEN]) {
                return Some(checking_piece);
            }
        }
        
        None
    }

    // Helper function: Find axial check (rook/queen)
    fn find_axial_check(&self, king_square: Square, opponent_color: u8) -> Option<Square> {
        // 4 axial directions
        let axial_directions = [(0, 1), (0, -1), (1, 0), (-1, 0)];
        
        for direction in axial_directions {
            if let Some(checking_piece) = self.trace_ray_for_check(king_square, direction, opponent_color, &[ROOK, QUEEN]) {
                return Some(checking_piece);
            }
        }
        
        None
    }

    // Core ray tracing function with piece type filtering
    fn trace_ray_for_check(&self, king_square: Square, direction: (i8, i8), opponent_color: u8, valid_piece_types: &[u8]) -> Option<Square> {
        let mut current_file = king_square.file() as i8 + direction.0;
        let mut current_rank = king_square.rank() as i8 + direction.1;
        
        while current_file >= 0 && current_file < 8 && current_rank >= 0 && current_rank < 8 {
            let current_square = Square::new(current_file as u8, current_rank as u8);
            
            if self.bitboards.is_occupied(current_square) {
                let piece = self.get_piece(current_square);
                
                if piece_color(piece) == opponent_color {
                    let piece_type_val = piece_type(piece);
                    
                    // Check if this piece type can attack in this direction
                    if valid_piece_types.contains(&piece_type_val) {
                        return Some(current_square);
                    }
                }
                
                // Hit any piece - ray blocked, stop tracing
                return None;
            }
            
            current_file += direction.0;
            current_rank += direction.1;
        }
        
        None
    }


    /// Check if a piece at 'from' attacks 'to'
    pub fn piece_attacks_square(&self, from: Square, to: Square) -> bool {
        let piece = self.get_piece(from);
        let piece_type_val = piece_type(piece);
        
        match piece_type_val {
            PAWN => self.pawn_attacks_square(from, to, piece_color(piece)),
            KNIGHT => self.knight_attacks_square(from, to),
            BISHOP => self.bishop_attacks_square(from, to),
            ROOK => self.rook_attacks_square(from, to),
            QUEEN => self.queen_attacks_square(from, to),
            KING => self.king_attacks_square(from, to),
            _ => false,
        }
    }

    /// Get squares that can block a check (including capturing the checking piece)
    pub fn get_blocking_squares(&self, king_square: Square, checking_piece_square: Square) -> HashSet<Square> {
        let mut blocking_squares = HashSet::new();
        
        // Can always capture the checking piece
        blocking_squares.insert(checking_piece_square);
        
        // If it's a sliding piece, can also block on squares between
        let checking_piece = self.get_piece(checking_piece_square);
        let piece_type_val = piece_type(checking_piece);
        
        if piece_type_val == QUEEN || piece_type_val == ROOK || piece_type_val == BISHOP {
            let king_file = king_square.file() as i8;
            let king_rank = king_square.rank() as i8;
            let checker_file = checking_piece_square.file() as i8;
            let checker_rank = checking_piece_square.rank() as i8;
            
            let file_diff = checker_file - king_file;
            let rank_diff = checker_rank - king_rank;
            
            // Calculate direction
            let direction = (file_diff.signum(), rank_diff.signum());
            
            // Add all squares between king and checking piece
            let mut file = king_file + direction.0;
            let mut rank = king_rank + direction.1;
            
            while file != checker_file || rank != checker_rank {
                blocking_squares.insert(Square::new(file as u8, rank as u8));
                file += direction.0;
                rank += direction.1;
            }
        }
        
        blocking_squares
    }

    /// Filter king moves when in check
    pub fn filter_king_moves_in_check(&self, moves: Vec<Square>, opponent_color: u8) -> Vec<Square> {
        moves.into_iter()
            .filter(|&square| !self.is_under_threat(square, opponent_color))
            .collect()
    }

    /// Filter moves to escape check (for non-king pieces)
    pub fn filter_moves_to_escape_check(&self, _square: Square, moves: Vec<Square>, checking_piece_square: Square) -> Vec<Square> {
        let our_color = self.current_turn;
        let king_square = match self.find_king(our_color) {
            Some(square) => square,
            None => return Vec::new(),
        };
        
        let blocking_squares = self.get_blocking_squares(king_square, checking_piece_square);
        
        moves.into_iter()
            .filter(|&mv| blocking_squares.contains(&mv))
            .collect()
    }

    // Helper methods for piece attack patterns
    fn pawn_attacks_square(&self, from: Square, to: Square, color: u8) -> bool {
        let file_diff = to.file() as i8 - from.file() as i8;
        let rank_diff = to.rank() as i8 - from.rank() as i8;
        let direction = if color == WHITE { 1 } else { -1 };
        
        file_diff.abs() == 1 && rank_diff == direction
    }

    fn knight_attacks_square(&self, from: Square, to: Square) -> bool {
        let file_diff = (to.file() as i8 - from.file() as i8).abs();
        let rank_diff = (to.rank() as i8 - from.rank() as i8).abs();
        
        (file_diff == 2 && rank_diff == 1) || (file_diff == 1 && rank_diff == 2)
    }

    fn bishop_attacks_square(&self, from: Square, to: Square) -> bool {
        let file_diff = (to.file() as i8 - from.file() as i8).abs();
        let rank_diff = (to.rank() as i8 - from.rank() as i8).abs();
        
        if file_diff != rank_diff {
            return false; // Not on diagonal
        }
        
        // Check if path is clear
        let direction = (
            (to.file() as i8 - from.file() as i8).signum(),
            (to.rank() as i8 - from.rank() as i8).signum(),
        );
        
        self.is_clear_path(from, to, direction)
    }

    fn rook_attacks_square(&self, from: Square, to: Square) -> bool {
        if from.file() != to.file() && from.rank() != to.rank() {
            return false; // Not on same rank or file
        }
        
        let direction = (
            (to.file() as i8 - from.file() as i8).signum(),
            (to.rank() as i8 - from.rank() as i8).signum(),
        );
        
        self.is_clear_path(from, to, direction)
    }

    fn queen_attacks_square(&self, from: Square, to: Square) -> bool {
        self.rook_attacks_square(from, to) || self.bishop_attacks_square(from, to)
    }

    fn king_attacks_square(&self, from: Square, to: Square) -> bool {
        let file_diff = (to.file() as i8 - from.file() as i8).abs();
        let rank_diff = (to.rank() as i8 - from.rank() as i8).abs();
        
        file_diff <= 1 && rank_diff <= 1 && (file_diff != 0 || rank_diff != 0)
    }

    /// Check if a piece at the given square is pinned
    pub fn is_piece_pinned(&self, square: Square) -> Option<(i8, i8)> {
        let piece = self.get_piece(square);
        if is_empty(piece) {
            return None;
        }
    
        let piece_color_val = piece_color(piece);
        let opponent_color = opposite_color(piece_color_val);
    
        // Find our king
        let king_square = self.find_king(piece_color_val)?;

        // Check if this piece is between king and an attacking piece
        let pin_result = self.find_pin_direction(square, king_square, opponent_color);
        
        pin_result
    }
    

    /// Check if a piece is pinned by looking for attacking pieces through it to the king
    fn find_pin_direction(&self, piece_square: Square, king_square: Square, opponent_color: u8) -> Option<(i8, i8)> {
        let piece_file = piece_square.file() as i8;
        let piece_rank = piece_square.rank() as i8;
        let king_file = king_square.file() as i8;
        let king_rank = king_square.rank() as i8;
        
        // Calculate direction from piece to king
        let file_diff = king_file - piece_file;
        let rank_diff = king_rank - piece_rank;
        
        // Normalize to direction vector
        let pin_direction = match (file_diff.signum(), rank_diff.signum()) {
            (0, 1) | (0, -1) => (0, rank_diff.signum()), // Vertical
            (1, 0) | (-1, 0) => (file_diff.signum(), 0), // Horizontal
            (1, 1) | (-1, -1) | (1, -1) | (-1, 1) => { // Diagonal
                if file_diff.abs() == rank_diff.abs() {
                    (file_diff.signum(), rank_diff.signum())
                } else {
                    return None; // Not on same line
                }
            }
            _ => return None, // Not aligned
        };
        
        // Look from piece toward king to see if king is there
        if !self.is_clear_path(piece_square, king_square, pin_direction) {
            return None;
        }
        
        // Look from piece away from king to find attacking piece
        let opposite_direction = (-pin_direction.0, -pin_direction.1);
        if let Some(attacking_piece) = self.find_attacking_piece_in_direction(
            piece_square,
            opposite_direction,
            opponent_color
        ) {
            // Check if attacking piece can actually attack in this direction
            let piece_type_val = piece_type(attacking_piece);
            let is_valid_attacker = match (pin_direction.0, pin_direction.1) {
                (0, _) | (_, 0) => piece_type_val == ROOK || piece_type_val == QUEEN, // Rank/file
                (_, _) => piece_type_val == BISHOP || piece_type_val == QUEEN, // Diagonal
            };
            
            if is_valid_attacker {
                return Some(pin_direction);
            }
        }
        
        None
    }

    /// Check if path between two squares is clear
    pub fn is_clear_path(&self, from: Square, to: Square, direction: (i8, i8)) -> bool {
        let mut file = from.file() as i8 + direction.0;
        let mut rank = from.rank() as i8 + direction.1;
        let to_file = to.file() as i8;
        let to_rank = to.rank() as i8;
        
        // Check squares BETWEEN from and to (not including endpoints)
        while file != to_file || rank != to_rank {
            if file < 0 || file >= 8 || rank < 0 || rank >= 8 {
                return false;
            }
            
            let square = Square::new(file as u8, rank as u8);
            if !is_empty(self.get_piece(square)) {
                return false; // Path is blocked
            }
            
            file += direction.0;
            rank += direction.1;
        }
        
        true // Path is clear
    }
    

    /// Find attacking piece in a given direction
    fn find_attacking_piece_in_direction(&self, from: Square, direction: (i8, i8), color: u8) -> Option<Piece> {
        let mut file = from.file() as i8 + direction.0;
        let mut rank = from.rank() as i8 + direction.1;
        
        while file >= 0 && file < 8 && rank >= 0 && rank < 8 {
            let square = Square::new(file as u8, rank as u8);
            let piece = self.get_piece(square);
            
            if !is_empty(piece) {
                if piece_color(piece) == color {
                    return Some(piece);
                } else {
                    return None; // Hit our own piece first
                }
            }
            
            file += direction.0;
            rank += direction.1;
        }
        
        None
    }

    /// Check if en passant move is legal (doesn't leave king in check)
    pub fn is_en_passant_legal(&self, mv: Move) -> bool {
        // Get the squares involved
        let capturing_pawn_square = mv.from;
        let target_square = mv.to;
        let captured_pawn_square = self.en_passant_pawn.unwrap();
        
        let our_color = piece_color(self.get_piece(capturing_pawn_square));
        let opponent_color = opposite_color(our_color);
        
        // Find our king
        let king_square = match self.find_king(our_color) {
            Some(square) => square,
            None => return false,
        };
        
        // Simulate the en passant capture
        let capturing_pawn = self.get_piece(capturing_pawn_square);
        let _captured_pawn = self.get_piece(captured_pawn_square);
        
        // Create a temporary board state
        let mut temp_board = self.clone();
        temp_board.set_piece(target_square, capturing_pawn); // Move our pawn
        temp_board.set_piece(capturing_pawn_square, EMPTY); // Clear original position
        temp_board.set_piece(captured_pawn_square, EMPTY); // Remove captured pawn
        
        // Check if our king would be in check after this move
        !temp_board.is_under_threat(king_square, opponent_color)
    }

    /// Test if king would be in check after a specific move
    pub fn would_king_be_in_check_after_move(&self, mv: Move) -> bool {
        let mut temp_board = self.clone();
        
        // Make the move temporarily
        if let Ok(_) = temp_board.try_make_move(mv) {
            // Find the king's new position
            let king_color = opposite_color(temp_board.current_turn); // King that just moved
            if let Some(king_square) = temp_board.find_king(king_color) {
                let opponent_color = opposite_color(king_color);
                return temp_board.is_under_threat(king_square, opponent_color);
            }
        }
        
        false
    }
}

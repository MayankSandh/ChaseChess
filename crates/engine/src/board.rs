use crate::types::*;

#[derive(Debug, Clone)]
pub struct Board {
    pub squares: [Piece; 64],
    pub current_turn: u8,
    pub move_history: Vec<GameMove>,
    pub game_status: GameStatus,
    // TODO: can be u8
    pub half_move_clock: u16,
    pub full_move_number: u16,
    pub castling_rights: u8, 
}

impl Board {
    pub fn new() -> Self {
        let mut board = Self {
            squares: [EMPTY; 64],
            current_turn: WHITE,
            move_history: Vec::new(),
            game_status: GameStatus::InProgress,
            half_move_clock: 0,
            full_move_number: 1,
            castling_rights: ALL_CASTLING_RIGHTS,
        };
        board.setup_starting_position();
        board
    }
    
    fn setup_starting_position(&mut self) {
        // Setup white pieces (rank 0)
        self.squares[Square::new(0, 0).0 as usize] = make_piece(ROOK, WHITE);
        self.squares[Square::new(1, 0).0 as usize] = make_piece(KNIGHT, WHITE);
        self.squares[Square::new(2, 0).0 as usize] = make_piece(BISHOP, WHITE);
        self.squares[Square::new(3, 0).0 as usize] = make_piece(QUEEN, WHITE);
        self.squares[Square::new(4, 0).0 as usize] = make_piece(KING, WHITE);
        self.squares[Square::new(5, 0).0 as usize] = make_piece(BISHOP, WHITE);
        self.squares[Square::new(6, 0).0 as usize] = make_piece(KNIGHT, WHITE);
        self.squares[Square::new(7, 0).0 as usize] = make_piece(ROOK, WHITE);
        
        // White pawns (rank 1)
        for file in 0..8 {
            self.squares[Square::new(file, 1).0 as usize] = make_piece(PAWN, WHITE);
        }
        
        // Setup black pieces (rank 7)
        self.squares[Square::new(0, 7).0 as usize] = make_piece(ROOK, BLACK);
        self.squares[Square::new(1, 7).0 as usize] = make_piece(KNIGHT, BLACK);
        self.squares[Square::new(2, 7).0 as usize] = make_piece(BISHOP, BLACK);
        self.squares[Square::new(3, 7).0 as usize] = make_piece(QUEEN, BLACK);
        self.squares[Square::new(4, 7).0 as usize] = make_piece(KING, BLACK);
        self.squares[Square::new(5, 7).0 as usize] = make_piece(BISHOP, BLACK);
        self.squares[Square::new(6, 7).0 as usize] = make_piece(KNIGHT, BLACK);
        self.squares[Square::new(7, 7).0 as usize] = make_piece(ROOK, BLACK);
        
        // Black pawns (rank 6)
        for file in 0..8 {
            self.squares[Square::new(file, 6).0 as usize] = make_piece(PAWN, BLACK);
        }
    }
    
    pub fn get_piece(&self, square: Square) -> Piece {
        self.squares[square.0 as usize]
    }
    
    pub fn set_piece(&mut self, square: Square, piece: Piece) {
        self.squares[square.0 as usize] = piece;
    }
    
    // Enhanced move validation
    pub fn is_valid_move(&self, mv: Move) -> bool {
        let from_piece = self.get_piece(mv.from);
        let to_piece = self.get_piece(mv.to);
        // TODO: can reduce if else here
        // Basic validations
        if is_empty(from_piece) {
            return false; // No piece to move
        }
        
        if !is_piece_color(from_piece, self.current_turn) {
            return false; // Not your piece
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
    
    // Enhanced move making with validation
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
        
        // Check if this is a castling move
        let is_castling = self.is_castling_move(mv).is_some();
        
        // Create game move record
        let mut game_move = if is_empty(captured_piece) {
            GameMove::new(mv)
        } else {
            GameMove::with_capture(mv, captured_piece)
        };
        
        game_move.is_castling = is_castling;
        
        // Execute the move
        if is_castling {
            let kingside = self.is_castling_move(mv).unwrap();
            self.execute_castling(piece_color(moving_piece), kingside);
        } else {
            // Regular move
            self.set_piece(mv.to, moving_piece);
            self.set_piece(mv.from, EMPTY);
            
            // Update castling rights
            self.update_castling_rights(mv);
        }
        
        // Update game state
        self.move_history.push(game_move.clone());
        self.current_turn = opposite_color(self.current_turn);
        
        // Update move counters
        if piece_type(moving_piece) == PAWN || !is_empty(captured_piece) {
            self.half_move_clock = 0;
        } else {
            self.half_move_clock += 1;
        }
        
        if self.current_turn == WHITE {
            self.full_move_number += 1;
        }
        
        // Check for game ending conditions
        self.update_game_status();
        
        Ok(game_move)
    }
    
    
    
    // Improved sliding piece movement with obstacle detection
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
    
    // Enhanced knight moves with collision detection
    fn get_knight_moves(&self, square: Square) -> Vec<Square> {
        let mut moves = Vec::new();
        let file = square.file() as i8;
        let rank = square.rank() as i8;
        let source_color = piece_color(self.get_piece(square));
        
        let knight_offsets = [
            (-2, -1), (-2, 1), (-1, -2), (-1, 2),
            (1, -2), (1, 2), (2, -1), (2, 1)
        ];
        
        for (df, dr) in knight_offsets {
            let new_file = file + df;
            let new_rank = rank + dr;
            
            if new_file >= 0 && new_file < 8 && new_rank >= 0 && new_rank < 8 {
                let target_square = Square::new(new_file as u8, new_rank as u8);
                let target_piece = self.get_piece(target_square);
                
                if is_empty(target_piece) || piece_color(target_piece) != source_color {
                    moves.push(target_square);
                }
            }
        }
        
        moves
    }
    
    fn get_queen_moves(&self, square: Square) -> Vec<Square> {
        let mut moves = self.get_sliding_moves(square, &[(0, 1), (0, -1), (1, 0), (-1, 0)]);
        moves.extend(self.get_sliding_moves(square, &[(1, 1), (1, -1), (-1, 1), (-1, -1)]));
        moves
    }
    
    // Enhanced king moves
    fn get_king_moves(&self, square: Square) -> Vec<Square> {
        let mut moves = Vec::new();
        let file = square.file() as i8;
        let rank = square.rank() as i8;
        let source_color = piece_color(self.get_piece(square));
        
        // Regular king moves
        for df in -1..=1 {
            for dr in -1..=1 {
                if df == 0 && dr == 0 { continue; }
                
                let new_file = file + df;
                let new_rank = rank + dr;
                
                if new_file >= 0 && new_file < 8 && new_rank >= 0 && new_rank < 8 {
                    let target_square = Square::new(new_file as u8, new_rank as u8);
                    let target_piece = self.get_piece(target_square);
                    
                    if is_empty(target_piece) || piece_color(target_piece) != source_color {
                        moves.push(target_square);
                    }
                }
            }
        }
        
        // Add castling moves
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
    
    
    // Enhanced pawn moves with captures
    fn get_pawn_moves(&self, square: Square, color: u8) -> Vec<Square> {
        let mut moves = Vec::new();
        let file = square.file();
        let rank = square.rank();
        
        let direction = if color == WHITE { 1 } else { -1 };
        let starting_rank = if color == WHITE { 1 } else { 6 };
        
        // Forward moves
        let new_rank = rank as i8 + direction;
        if new_rank >= 0 && new_rank < 8 {
            let forward_square = Square::new(file, new_rank as u8);
            
            if is_empty(self.get_piece(forward_square)) {
                moves.push(forward_square);
                
                // Double move from starting position
                if rank == starting_rank {
                    let double_square = Square::new(file, (new_rank + direction) as u8);
                    if is_empty(self.get_piece(double_square)) {
                        moves.push(double_square);
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
                
                if !is_empty(target_piece) && piece_color(target_piece) != color {
                    moves.push(capture_square);
                }
            }
        }
        
        moves
    }
    
    // Basic game status update (simplified for now)
    fn update_game_status(&mut self) {
        // For now, just set to InProgress
        // In stage 3, we'll add proper check/checkmate detection
        self.game_status = GameStatus::InProgress;
    }
    
    // Utility methods
    pub fn can_player_move(&self) -> bool {
        // Check if current player has any legal moves
        for rank in 0..8 {
            for file in 0..8 {
                let square = Square::new(file, rank);
                let piece = self.get_piece(square);
                
                if is_piece_color(piece, self.current_turn) {
                    if !self.get_legal_moves(square).is_empty() {
                        return true;
                    }
                }
            }
        }
        false
    }
    
    /// Check if a square is under threat by the specified color using ray tracing
    pub fn is_under_threat(&self, square: Square, by_color: u8) -> bool {
        // Check sliding piece threats (queen, rook, bishop)
        if self.check_sliding_threats(square, by_color) {
            return true;
        }
        
        // Check knight threats
        if self.check_knight_threats(square, by_color) {
            return true;
        }
        
        // Check pawn threats
        if self.check_pawn_threats(square, by_color) {
            return true;
        }
        
        // Check king threats
        if self.check_king_threats(square, by_color) {
            return true;
        }
        
        false
    }
    
    /// Check for sliding piece threats (queen, rook, bishop)
    fn check_sliding_threats(&self, square: Square, by_color: u8) -> bool {
        let file = square.file() as i8;
        let rank = square.rank() as i8;
        
        // All 8 directions: 4 rook directions + 4 bishop directions
        let directions = [
            (0, 1), (0, -1), (1, 0), (-1, 0),     // Rook directions
            (1, 1), (1, -1), (-1, 1), (-1, -1)   // Bishop directions
        ];
        
        for (i, &(df, dr)) in directions.iter().enumerate() {
            if let Some(attacking_piece) = self.cast_ray(file, rank, df, dr) {
                if piece_color(attacking_piece) == by_color {
                    let piece_type_val = piece_type(attacking_piece);
                    
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
        
        while file >= 0 && file < 8 && rank >= 0 && rank < 8 {
            let target_square = Square::new(file as u8, rank as u8);
            let piece = self.get_piece(target_square);
            
            if !is_empty(piece) {
                return Some(piece); // Found a piece
            }
            
            file += df;
            rank += dr;
        }
        
        None // No piece found in this direction
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
    fn is_castling_move(&self, mv: Move) -> Option<bool> {
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
    
    /// Update castling rights after a move
    fn update_castling_rights(&mut self, mv: Move) {
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
        self.find_pin_direction(square, king_square, opponent_color)
    }
    
    /// Find the king of the specified color
    fn find_king(&self, color: u8) -> Option<Square> {
        for rank in 0..8 {
            for file in 0..8 {
                let square = Square::new(file, rank);
                let piece = self.get_piece(square);
                if piece_type(piece) == KING && piece_color(piece) == color {
                    return Some(square);
                }
            }
        }
        None
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
            (0, 1) | (0, -1) => (0, rank_diff.signum()),   // Vertical
            (1, 0) | (-1, 0) => (file_diff.signum(), 0),   // Horizontal  
            (1, 1) | (-1, -1) | (1, -1) | (-1, 1) => {     // Diagonal
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
                (_, _) => piece_type_val == BISHOP || piece_type_val == QUEEN,        // Diagonal
            };
            
            if is_valid_attacker {
                return Some(pin_direction);
            }
        }
        
        None
    }
    
    /// Check if path between two squares is clear
    fn is_clear_path(&self, from: Square, to: Square, direction: (i8, i8)) -> bool {
        let mut file = from.file() as i8 + direction.0;
        let mut rank = from.rank() as i8 + direction.1;
        let to_file = to.file() as i8;
        let to_rank = to.rank() as i8;
        
        while file != to_file || rank != to_rank {
            if file < 0 || file >= 8 || rank < 0 || rank >= 8 {
                return false;
            }
            
            let square = Square::new(file as u8, rank as u8);
            if !is_empty(self.get_piece(square)) {
                return false;
            }
            
            file += direction.0;
            rank += direction.1;
        }
        
        true
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
                // Not in check, all pseudo-legal moves are legal
                pseudo_moves
            }
            1 => {
                // Single check - can block or capture
                let checking_piece_square = checking_pieces[0];
                let blocking_squares = self.get_blocking_squares(king_square, checking_piece_square);
                
                // For king moves, use normal validation
                let piece = self.get_piece(square);
                if piece_type(piece) == KING {
                    self.filter_king_moves_in_check(pseudo_moves, opponent_color)
                } else {
                    // For other pieces, only moves that block or capture are legal
                    pseudo_moves.into_iter()
                        .filter(|&mv| blocking_squares.contains(&mv))
                        .collect()
                }
            }
            _ => {
                // Double check - only king moves are legal
                let piece = self.get_piece(square);
                if piece_type(piece) == KING {
                    self.filter_king_moves_in_check(pseudo_moves, opponent_color)
                } else {
                    Vec::new() // No legal moves for non-king pieces
                }
            }
        }
    }
    
    /// Find all pieces that are checking the king
    fn find_checking_pieces(&self, king_square: Square, opponent_color: u8) -> Vec<Square> {
        let mut checking_pieces = Vec::new();
        
        // Check all opponent pieces to see if they attack the king
        for rank in 0..8 {
            for file in 0..8 {
                let square = Square::new(file, rank);
                let piece = self.get_piece(square);
                
                if !is_empty(piece) && piece_color(piece) == opponent_color {
                    if self.piece_attacks_square(square, king_square) {
                        checking_pieces.push(square);
                    }
                }
            }
        }
        
        checking_pieces
    }
    
    /// Check if a piece at 'from' attacks 'to'
    fn piece_attacks_square(&self, from: Square, to: Square) -> bool {
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
    fn get_blocking_squares(&self, king_square: Square, checking_piece_square: Square) -> std::collections::HashSet<Square> {
        let mut blocking_squares = std::collections::HashSet::new();
        
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
    fn filter_king_moves_in_check(&self, moves: Vec<Square>, opponent_color: u8) -> Vec<Square> {
        moves.into_iter()
            .filter(|&square| !self.is_under_threat(square, opponent_color))
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
            return self.get_pinned_piece_moves(square, pin_direction);
        }
        
        // Generate normal moves for non-pinned pieces
        match piece_type(piece) {
            KNIGHT => self.get_knight_moves(square),
            ROOK => self.get_sliding_moves(square, &[(0, 1), (0, -1), (1, 0), (-1, 0)]),
            BISHOP => self.get_sliding_moves(square, &[(1, 1), (1, -1), (-1, 1), (-1, -1)]),
            QUEEN => self.get_queen_moves(square),
            KING => self.get_king_moves(square),
            PAWN => self.get_pawn_moves(square, piece_color(piece)),
            _ => Vec::new(),
        }
    }
    
    /// Generate moves for a pinned piece (only along pin line)
    fn get_pinned_piece_moves(&self, square: Square, pin_direction: (i8, i8)) -> Vec<Square> {
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
        
        // For sliding pieces (rook, bishop, queen), generate moves along pin line
        let mut moves = Vec::new();
        
        // Generate moves only along the pin line (both directions)
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
    
    /// Generate moves for pinned pawn
    fn get_pinned_pawn_moves(&self, square: Square, pin_direction: (i8, i8), color: u8) -> Vec<Square> {
        let mut moves = Vec::new();
        let file = square.file();
        let rank = square.rank();
        
        let forward_direction = if color == WHITE { 1 } else { -1 };
        let starting_rank = if color == WHITE { 1 } else { 6 };
        
        // Check if pawn can move forward along pin line (vertical pin)
        if pin_direction == (0, forward_direction) || pin_direction == (0, -forward_direction) {
            // Pawn is pinned vertically, can move forward
            let new_rank = rank as i8 + forward_direction;
            if new_rank >= 0 && new_rank < 8 {
                let forward_square = Square::new(file, new_rank as u8);
                if is_empty(self.get_piece(forward_square)) {
                    moves.push(forward_square);
                    
                    // Double move from starting position
                    if rank == starting_rank {
                        let double_square = Square::new(file, (new_rank + forward_direction) as u8);
                        if is_empty(self.get_piece(double_square)) {
                            moves.push(double_square);
                        }
                    }
                }
            }
        }
        
        // Check diagonal captures along pin line
        for df in [-1, 1] {
            if pin_direction == (df, forward_direction) || pin_direction == (-df, -forward_direction) {
                let new_file = file as i8 + df;
                let new_rank = rank as i8 + forward_direction;
                
                if new_file >= 0 && new_file < 8 && new_rank >= 0 && new_rank < 8 {
                    let capture_square = Square::new(new_file as u8, new_rank as u8);
                    let target_piece = self.get_piece(capture_square);
                    
                    if !is_empty(target_piece) && piece_color(target_piece) != color {
                        moves.push(capture_square);
                    }
                }
            }
        }
        
        moves
    }


    pub fn get_last_move(&self) -> Option<&GameMove> {
        self.move_history.last()
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

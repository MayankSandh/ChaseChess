use crate::types::*;

#[derive(Debug, Clone)]
pub struct Board {
    pub squares: [Piece; 64],
    pub current_turn: u8,
    pub move_history: Vec<GameMove>,
    pub game_status: GameStatus,
    pub half_move_clock: u16,
    pub full_move_number: u16,
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
        
        // Create game move record
        let game_move = if is_empty(captured_piece) {
            GameMove::new(mv)
        } else {
            GameMove::with_capture(mv, captured_piece)
        };
        
        // Make the move
        self.set_piece(mv.to, moving_piece);
        self.set_piece(mv.from, EMPTY);
        
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
    
    // Improved legal move generation with collision detection
    pub fn get_legal_moves(&self, square: Square) -> Vec<Square> {
        let piece = self.get_piece(square);
        if is_empty(piece) {
            return Vec::new();
        }
        
        // Only generate moves for current player's pieces
        if !is_piece_color(piece, self.current_turn) {
            return Vec::new();
        }
        
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
    
    pub fn get_last_move(&self) -> Option<&GameMove> {
        self.move_history.last()
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

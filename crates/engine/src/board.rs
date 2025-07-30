use crate::types::*;

#[derive(Debug, Clone)]
pub struct Board {
    pub squares: [Piece; 64],
    pub current_turn: u8, // WHITE or BLACK
}

impl Board {
    pub fn new() -> Self {
        let mut board = Self {
            squares: [EMPTY; 64],
            current_turn: WHITE,
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
    
    pub fn make_move(&mut self, mv: Move) -> bool {
        let piece = self.get_piece(mv.from);
        if !is_empty(piece) {
            self.set_piece(mv.to, piece);
            self.set_piece(mv.from, EMPTY);
            self.current_turn = if self.current_turn == WHITE { BLACK } else { WHITE };
            true
        } else {
            false
        }
    }
    
    pub fn get_legal_moves(&self, square: Square) -> Vec<Square> {
        let piece = self.get_piece(square);
        if is_empty(piece) {
            return Vec::new();
        }
        
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
    
    fn get_knight_moves(&self, square: Square) -> Vec<Square> {
        let mut moves = Vec::new();
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
                moves.push(Square::new(new_file as u8, new_rank as u8));
            }
        }
        
        moves
    }
    
    fn get_rook_moves(&self, square: Square) -> Vec<Square> {
        let mut moves = Vec::new();
        let file = square.file();
        let rank = square.rank();
        
        // Horizontal and vertical moves (simplified)
        for f in 0..8 {
            if f != file {
                moves.push(Square::new(f, rank));
            }
        }
        for r in 0..8 {
            if r != rank {
                moves.push(Square::new(file, r));
            }
        }
        
        moves
    }
    
    fn get_bishop_moves(&self, square: Square) -> Vec<Square> {
        let mut moves = Vec::new();
        let file = square.file() as i8;
        let rank = square.rank() as i8;
        
        // Diagonal moves (simplified)
        for i in 1..8 {
            for (df, dr) in [(1, 1), (1, -1), (-1, 1), (-1, -1)] {
                let new_file = file + df * i;
                let new_rank = rank + dr * i;
                
                if new_file >= 0 && new_file < 8 && new_rank >= 0 && new_rank < 8 {
                    moves.push(Square::new(new_file as u8, new_rank as u8));
                }
            }
        }
        
        moves
    }
    
    fn get_queen_moves(&self, square: Square) -> Vec<Square> {
        let mut moves = self.get_rook_moves(square);
        moves.extend(self.get_bishop_moves(square));
        moves
    }
    
    fn get_king_moves(&self, square: Square) -> Vec<Square> {
        let mut moves = Vec::new();
        let file = square.file() as i8;
        let rank = square.rank() as i8;
        
        for df in -1..=1 {
            for dr in -1..=1 {
                if df == 0 && dr == 0 { continue; }
                
                let new_file = file + df;
                let new_rank = rank + dr;
                
                if new_file >= 0 && new_file < 8 && new_rank >= 0 && new_rank < 8 {
                    moves.push(Square::new(new_file as u8, new_rank as u8));
                }
            }
        }
        
        moves
    }
    
    fn get_pawn_moves(&self, square: Square, color: u8) -> Vec<Square> {
        let mut moves = Vec::new();
        let file = square.file();
        let rank = square.rank();
        
        let direction = if color == WHITE { 1 } else { -1 };
        
        let new_rank = rank as i8 + direction;
        if new_rank >= 0 && new_rank < 8 {
            moves.push(Square::new(file, new_rank as u8));
            
            // Double move from starting position
            let starting_rank = if color == WHITE { 1 } else { 6 };
            
            if rank == starting_rank {
                let double_rank = rank as i8 + direction * 2;
                if double_rank >= 0 && double_rank < 8 {
                    moves.push(Square::new(file, double_rank as u8));
                }
            }
        }
        
        moves
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

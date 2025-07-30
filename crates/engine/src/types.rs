#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Square(pub u8);

impl Square {
    pub fn new(file: u8, rank: u8) -> Self {
        Self(rank * 8 + file)
    }
    
    pub fn file(&self) -> u8 {
        self.0 % 8
    }
    
    pub fn rank(&self) -> u8 {
        self.0 / 8
    }
    
    pub fn from_coords(x: f32, y: f32, square_size: f32) -> Option<Self> {
        let file = (x / square_size) as u8;
        let rank = 7 - (y / square_size) as u8; // Flip rank for screen coordinates
        
        if file < 8 && rank < 8 {
            Some(Self::new(file, rank))
        } else {
            None
        }
    }
}

// 4-bit piece representation
// Bits 0-2: piece type (0=empty, 1=pawn, 2=knight, 3=bishop, 4=rook, 5=queen, 6=king)
// Bit 3: color (0=black, 1=white)
pub type Piece = u8;

pub const EMPTY: u8 = 0;

// Piece types (bits 0-2)
pub const PAWN: u8 = 1;
pub const KNIGHT: u8 = 2;
pub const BISHOP: u8 = 3;
pub const ROOK: u8 = 4;
pub const QUEEN: u8 = 5;
pub const KING: u8 = 6;

// Colors (bit 3)
pub const BLACK: u8 = 0;
pub const WHITE: u8 = 8; // 1000 in binary

// Helper functions for piece manipulation
pub fn make_piece(piece_type: u8, color: u8) -> Piece {
    piece_type | color
}

pub fn piece_type(piece: Piece) -> u8 {
    piece & 7 // Extract bits 0-2
}

pub fn piece_color(piece: Piece) -> u8 {
    piece & 8 // Extract bit 3
}

pub fn is_white(piece: Piece) -> bool {
    piece_color(piece) == WHITE
}

pub fn is_black(piece: Piece) -> bool {
    piece_color(piece) == BLACK && piece != EMPTY
}

pub fn is_empty(piece: Piece) -> bool {
    piece == EMPTY
}

#[derive(Debug, Clone, Copy)]
pub struct Move {
    pub from: Square,
    pub to: Square,
}

impl Move {
    pub fn new(from: Square, to: Square) -> Self {
        Self { from, to }
    }
}

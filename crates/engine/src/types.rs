#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Square(pub u8);
use crate::Board;

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub promotion: Option<u8>, // Add this field
}

impl Move {
    pub fn new(from: Square, to: Square) -> Self {
        Self { from, to, promotion: None }
    }
    
    pub fn new_promotion(from: Square, to: Square, promotion_piece: u8) -> Self {
        Self { from, to, promotion: Some(promotion_piece) }
    }
    
    pub fn is_promotion(&self) -> bool {
        self.promotion.is_some()
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameStatus {
    InProgress,
    Check(u8), // Which color is in check
    Checkmate(u8), // Which color is checkmated (other color wins)
    Stalemate,
    Draw,
}

#[derive(Debug, Clone)]
pub struct GameMove {
    pub mv: Move,
    pub captured_piece: Piece,
    pub promotion: Option<u8>,
    pub is_castling: bool,
    pub is_en_passant: bool,
    
    // Add these fields for undo functionality:
    pub previous_castling_rights: u8,
    pub previous_en_passant_target: Option<Square>,
    pub previous_en_passant_pawn: Option<Square>,
    pub previous_half_move_clock: u16,
    pub previous_full_move_number: u16,
}

impl GameMove {
    pub fn new(mv: Move) -> Self {
        Self {
            mv,
            captured_piece: EMPTY,
            promotion: None,
            is_castling: false,
            is_en_passant: false,
            previous_castling_rights: 0,
            previous_en_passant_target: None,
            previous_en_passant_pawn: None,
            previous_half_move_clock: 0,
            previous_full_move_number: 0,
        }
    }
    
    pub fn with_capture(mv: Move, captured: Piece) -> Self {
        let mut game_move = Self::new(mv);
        game_move.captured_piece = captured;
        game_move
    }

    pub fn new_with_state(mv: Move, board: &Board) -> Self {
        Self {
            mv,
            captured_piece: EMPTY,
            promotion: None,
            is_castling: false,
            is_en_passant: false,
            previous_castling_rights: board.castling_rights,
            previous_en_passant_target: board.en_passant_target,
            previous_en_passant_pawn: board.en_passant_pawn,
            previous_half_move_clock: board.half_move_clock,
            previous_full_move_number: board.full_move_number,
        }
    }

    pub fn with_capture_and_state(mv: Move, captured: Piece, board: &Board) -> Self {
        let mut game_move = Self::new_with_state(mv, board);
        game_move.captured_piece = captured;
        game_move
    }
}

// Helper function to get opposite color
pub fn opposite_color(color: u8) -> u8 {
    color ^ WHITE
}

// Helper function to check if piece belongs to current player
pub fn is_piece_color(piece: Piece, color: u8) -> bool {
    !is_empty(piece) && piece_color(piece) == color
}

// Add these castling constants to the existing file
// Castling rights bitfield constants
pub const WHITE_KINGSIDE: u8 = 0b0001;
pub const WHITE_QUEENSIDE: u8 = 0b0010;
pub const BLACK_KINGSIDE: u8 = 0b0100;
pub const BLACK_QUEENSIDE: u8 = 0b1000;
pub const ALL_CASTLING_RIGHTS: u8 = 0b1111;

// Helper functions for castling rights
pub fn has_castling_right(castling_rights: u8, right: u8) -> bool {
    castling_rights & right != 0
}

pub fn remove_castling_right(castling_rights: &mut u8, right: u8) {
    *castling_rights &= !right;
}

pub fn get_castling_rights_for_color(castling_rights: u8, color: u8) -> u8 {
    if color == WHITE {
        castling_rights & (WHITE_KINGSIDE | WHITE_QUEENSIDE)
    } else {
        castling_rights & (BLACK_KINGSIDE | BLACK_QUEENSIDE)
    }
}

pub fn piece_type_name(piece_type: u8) -> &'static str {
    match piece_type {
        PAWN => "Pawn",
        ROOK => "Rook", 
        KNIGHT => "Knight",
        BISHOP => "Bishop",
        QUEEN => "Queen",
        KING => "King",
        _ => "Unknown"
    }
}

impl Square {
    // Add this if it doesn't exist
    pub fn from_algebraic(algebraic: &str) -> Self {
        let chars: Vec<char> = algebraic.chars().collect();
        let file = (chars[0] as u8) - b'a';
        let rank = (chars[1] as u8) - b'1';
        Self::new(file, rank)
    }
}
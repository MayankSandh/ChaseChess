use crate::types::*;

pub type Bitboard = u64;

// Constants
pub const EMPTY: Bitboard = 0;
pub const FULL: Bitboard = 0xFFFFFFFFFFFFFFFF;

// File masks
pub const FILE_A: Bitboard = 0x0101010101010101;
pub const FILE_B: Bitboard = 0x0202020202020202;
pub const FILE_C: Bitboard = 0x0404040404040404;
pub const FILE_D: Bitboard = 0x0808080808080808;
pub const FILE_E: Bitboard = 0x1010101010101010;
pub const FILE_F: Bitboard = 0x2020202020202020;
pub const FILE_G: Bitboard = 0x4040404040404040;
pub const FILE_H: Bitboard = 0x8080808080808080;

// Rank masks
pub const RANK_1: Bitboard = 0x00000000000000FF;
pub const RANK_2: Bitboard = 0x000000000000FF00;
pub const RANK_3: Bitboard = 0x0000000000FF0000;
pub const RANK_4: Bitboard = 0x00000000FF000000;
pub const RANK_5: Bitboard = 0x000000FF00000000;
pub const RANK_6: Bitboard = 0x0000FF0000000000;
pub const RANK_7: Bitboard = 0x00FF000000000000;
pub const RANK_8: Bitboard = 0xFF00000000000000;

// Core bitboard operations
pub fn set_bit(bitboard: &mut Bitboard, square: u8) {
    *bitboard |= 1u64 << square;
}

pub fn clear_bit(bitboard: &mut Bitboard, square: u8) {
    *bitboard &= !(1u64 << square);
}

pub fn get_bit(bitboard: Bitboard, square: u8) -> bool {
    (bitboard & (1u64 << square)) != 0
}

pub fn square_to_bitboard(square: u8) -> Bitboard {
    1u64 << square
}

pub fn count_bits(bitboard: Bitboard) -> u32 {
    bitboard.count_ones()
}

pub fn is_bitboard_empty(bitboard: Bitboard) -> bool {
    bitboard == 0
}

// Convert Square to bitboard index (0-63)
pub fn square_to_index(square: Square) -> u8 {
    square.0
}

// Convert bitboard index back to Square
pub fn index_to_square(index: u8) -> Square {
    Square(index)
}

// Pop LSB (remove and return the least significant bit)
pub fn pop_lsb(bitboard: &mut Bitboard) -> Option<u8> {
    if *bitboard == 0 {
        None
    } else {
        let lsb = bitboard.trailing_zeros() as u8;
        *bitboard &= *bitboard - 1; // Remove LSB
        Some(lsb)
    }
}

// Iterator for set bits
pub struct BitboardIterator {
    bitboard: Bitboard,
}

impl BitboardIterator {
    pub fn new(bitboard: Bitboard) -> Self {
        BitboardIterator { bitboard }
    }
}

impl Iterator for BitboardIterator {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        pop_lsb(&mut self.bitboard)
    }
}

pub fn iterate_bits(bitboard: Bitboard) -> BitboardIterator {
    BitboardIterator::new(bitboard)
}

// BitboardManager - manages all piece bitboards
#[derive(Clone, Debug)]
pub struct BitboardManager {
    // Individual piece bitboards
    pub white_pawns: Bitboard,
    pub white_knights: Bitboard,
    pub white_bishops: Bitboard,
    pub white_rooks: Bitboard,
    pub white_queens: Bitboard,
    pub white_king: Bitboard,
    
    pub black_pawns: Bitboard,
    pub black_knights: Bitboard,
    pub black_bishops: Bitboard,
    pub black_rooks: Bitboard,
    pub black_queens: Bitboard,
    pub black_king: Bitboard,
    
    // Aggregate bitboards
    pub white_pieces: Bitboard,
    pub black_pieces: Bitboard,
    pub all_pieces: Bitboard,
}

impl BitboardManager {
    pub fn new() -> Self {
        BitboardManager {
            white_pawns: EMPTY,
            white_knights: EMPTY,
            white_bishops: EMPTY,
            white_rooks: EMPTY,
            white_queens: EMPTY,
            white_king: EMPTY,
            
            black_pawns: EMPTY,
            black_knights: EMPTY,
            black_bishops: EMPTY,
            black_rooks: EMPTY,
            black_queens: EMPTY,
            black_king: EMPTY,
            
            white_pieces: EMPTY,
            black_pieces: EMPTY,
            all_pieces: EMPTY,
        }
    }
    
    // Rebuild all bitboards from the squares array
    pub fn rebuild_from_squares(&mut self, squares: &[Piece; 64]) {
        // Clear all bitboards
        *self = BitboardManager::new();
        
        // Build bitboards by scanning the squares array
        for (index, &piece) in squares.iter().enumerate() {
            if !is_empty(piece) {
                let bb = square_to_bitboard(index as u8);
                let piece_type = piece_type(piece);
                let piece_color = piece_color(piece);
                
                match (piece_color, piece_type) {
                    (WHITE, PAWN) => self.white_pawns |= bb,
                    (WHITE, KNIGHT) => self.white_knights |= bb,
                    (WHITE, BISHOP) => self.white_bishops |= bb,
                    (WHITE, ROOK) => self.white_rooks |= bb,
                    (WHITE, QUEEN) => self.white_queens |= bb,
                    (WHITE, KING) => self.white_king |= bb,
                    (BLACK, PAWN) => self.black_pawns |= bb,
                    (BLACK, KNIGHT) => self.black_knights |= bb,
                    (BLACK, BISHOP) => self.black_bishops |= bb,
                    (BLACK, ROOK) => self.black_rooks |= bb,
                    (BLACK, QUEEN) => self.black_queens |= bb,
                    (BLACK, KING) => self.black_king |= bb,
                    _ => {}
                }
            }
        }
        
        self.update_aggregate_bitboards();
    }
    
    // Update bitboards when a single square changes
    pub fn update_square(&mut self, square: Square, piece: Piece) {
        let square_index = square_to_index(square);
        let bb = square_to_bitboard(square_index);
        
        // Clear this square from all bitboards first
        self.white_pawns &= !bb;
        self.white_knights &= !bb;
        self.white_bishops &= !bb;
        self.white_rooks &= !bb;
        self.white_queens &= !bb;
        self.white_king &= !bb;
        
        self.black_pawns &= !bb;
        self.black_knights &= !bb;
        self.black_bishops &= !bb;
        self.black_rooks &= !bb;
        self.black_queens &= !bb;
        self.black_king &= !bb;
        
        // Set the new piece if not empty
        if !is_empty(piece) {
            let piece_type = piece_type(piece);
            let piece_color = piece_color(piece);
            
            match (piece_color, piece_type) {
                (WHITE, PAWN) => self.white_pawns |= bb,
                (WHITE, KNIGHT) => self.white_knights |= bb,
                (WHITE, BISHOP) => self.white_bishops |= bb,
                (WHITE, ROOK) => self.white_rooks |= bb,
                (WHITE, QUEEN) => self.white_queens |= bb,
                (WHITE, KING) => self.white_king |= bb,
                (BLACK, PAWN) => self.black_pawns |= bb,
                (BLACK, KNIGHT) => self.black_knights |= bb,
                (BLACK, BISHOP) => self.black_bishops |= bb,
                (BLACK, ROOK) => self.black_rooks |= bb,
                (BLACK, QUEEN) => self.black_queens |= bb,
                (BLACK, KING) => self.black_king |= bb,
                _ => {}
            }
        }
        
        self.update_aggregate_bitboards();
    }
    
    // Update the aggregate bitboards (white_pieces, black_pieces, all_pieces)
    fn update_aggregate_bitboards(&mut self) {
        self.white_pieces = self.white_pawns | self.white_knights | self.white_bishops | 
                           self.white_rooks | self.white_queens | self.white_king;
        
        self.black_pieces = self.black_pawns | self.black_knights | self.black_bishops | 
                           self.black_rooks | self.black_queens | self.black_king;
        
        self.all_pieces = self.white_pieces | self.black_pieces;
    }
    
    // Get pieces of a specific color and type
    pub fn get_pieces(&self, color: u8, piece_type: u8) -> Bitboard {
        match (color, piece_type) {
            (WHITE, PAWN) => self.white_pawns,
            (WHITE, KNIGHT) => self.white_knights,
            (WHITE, BISHOP) => self.white_bishops,
            (WHITE, ROOK) => self.white_rooks,
            (WHITE, QUEEN) => self.white_queens,
            (WHITE, KING) => self.white_king,
            (BLACK, PAWN) => self.black_pawns,
            (BLACK, KNIGHT) => self.black_knights,
            (BLACK, BISHOP) => self.black_bishops,
            (BLACK, ROOK) => self.black_rooks,
            (BLACK, QUEEN) => self.black_queens,
            (BLACK, KING) => self.black_king,
            _ => EMPTY,
        }
    }
    
    // Get all pieces of a specific color
    pub fn get_all_pieces(&self, color: u8) -> Bitboard {
        match color {
            WHITE => self.white_pieces,
            BLACK => self.black_pieces,
            _ => EMPTY,
        }
    }
    
    // Count pieces efficiently using bit counting
    pub fn count_pieces(&self, color: u8, piece_type: u8) -> u32 {
        count_bits(self.get_pieces(color, piece_type))
    }
    
    // Find all squares containing pieces of a specific type and color
    pub fn find_pieces(&self, color: u8, piece_type: u8) -> Vec<Square> {
        let mut squares = Vec::new();
        let mut bitboard = self.get_pieces(color, piece_type);
        
        while let Some(square_index) = pop_lsb(&mut bitboard) {
            squares.push(index_to_square(square_index));
        }
        
        squares
    }
    
    // Check if a square is occupied
    pub fn is_occupied(&self, square: Square) -> bool {
        let square_index = square_to_index(square);
        get_bit(self.all_pieces, square_index)
    }
    
    // Check if a square is occupied by a specific color
    pub fn is_occupied_by(&self, square: Square, color: u8) -> bool {
        let square_index = square_to_index(square);
        let color_pieces = self.get_all_pieces(color);
        get_bit(color_pieces, square_index)
    }
}

use std::sync::Once;

// Static storage for knight attack masks
static mut KNIGHT_ATTACKS: [Bitboard; 64] = [0; 64];
static KNIGHT_INIT: Once = Once::new();

// Generate knight attack mask for a single square
fn generate_knight_attack_mask(square: u8) -> Bitboard {
    let file = square % 8;
    let rank = square / 8;
    let mut attacks = 0u64;
    
    let knight_offsets = [(-2, -1), (-2, 1), (-1, -2), (-1, 2), (1, -2), (1, 2), (2, -1), (2, 1)];
    
    for (df, dr) in knight_offsets {
        let new_file = file as i8 + df;
        let new_rank = rank as i8 + dr;
        
        if new_file >= 0 && new_file < 8 && new_rank >= 0 && new_rank < 8 {
            let target_square = (new_rank * 8 + new_file) as u8;
            attacks |= 1u64 << target_square;
        }
    }
    
    attacks
}

// Initialize all knight attack masks
pub fn initialize_knight_attacks() {
    unsafe {
        KNIGHT_INIT.call_once(|| {
            for square in 0..64 {
                KNIGHT_ATTACKS[square] = generate_knight_attack_mask(square as u8);
            }
        });
    }
}

// Get pre-computed knight attacks for a square
pub fn get_knight_attacks(square: u8) -> Bitboard {
    unsafe {
        KNIGHT_ATTACKS[square as usize]
    }
}

pub fn debug_knight_attacks() {
    println!("🔍 DEBUGGING KNIGHT ATTACK MASKS:");
    
    // Test knight on e5 (should attack d3, f3, c4, g4, c6, g6, d7, f7)
    let e5_square = Square::new(4, 4); // e5
    let knight_attacks = get_knight_attacks(e5_square.0);
    
    println!("Knight on e5 attacks mask: 0x{:016x}", knight_attacks);
    println!("Expected squares: d3(19), f3(21), c4(26), g4(30), c6(42), g6(46), d7(51), f7(53)");
    
    // Check each expected square
    let expected_squares = [19, 21, 26, 30, 42, 46, 51, 53];
    for square_idx in expected_squares {
        if (knight_attacks & (1u64 << square_idx)) != 0 {
            println!("✅ Square {} is attacked", square_idx);
        } else {
            println!("❌ Square {} NOT attacked - BUG!", square_idx);
        }
    }
}

pub fn initialize_engine() {
    println!("🔍 Initializing engine...");
    // Your existing initialization code here
    
    // Add debug call
    debug_knight_attacks();
    
    println!("✅ Engine initialization complete");
}


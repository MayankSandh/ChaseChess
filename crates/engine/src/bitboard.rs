use crate::types::*;
use std::sync::Once;

pub type Bitboard = u64;

// Bitboard constants
pub const BITBOARD_EMPTY: Bitboard = 0;
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

// Pre-generated knight attack masks
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


// Add this test function to your bitboard.rs
pub fn test_knight_mask_direct() -> Bitboard {
    // Generate mask for square 59 directly without using static array
    let square = 59u8;
    let file = square % 8;
    let rank = square / 8;
    let mut attacks = 0u64;
    
    let knight_offsets = [(-2, -1), (-2, 1), (-1, -2), (-1, 2), (1, -2), (1, 2), (2, -1), (2, 1)];
    
    println!("🐎 Direct generation for square {} ({}{})", square, (b'a' + file) as char, rank + 1);
    
    for (df, dr) in knight_offsets {
        let new_file = file as i8 + df;
        let new_rank = rank as i8 + dr;
        
        if new_file >= 0 && new_file < 8 && new_rank >= 0 && new_rank < 8 {
            let target_square = (new_rank * 8 + new_file) as u8;
            attacks |= 1u64 << target_square;
            println!("  ✅ Attack from {}{} (square {})", 
                     (b'a' + new_file as u8) as char, new_rank + 1, target_square);
        }
    }
    
    println!("  Final mask: 0b{:064b}", attacks);
    attacks
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
            white_pawns: BITBOARD_EMPTY,
            white_knights: BITBOARD_EMPTY,
            white_bishops: BITBOARD_EMPTY,
            white_rooks: BITBOARD_EMPTY,
            white_queens: BITBOARD_EMPTY,
            white_king: BITBOARD_EMPTY,
            
            black_pawns: BITBOARD_EMPTY,
            black_knights: BITBOARD_EMPTY,
            black_bishops: BITBOARD_EMPTY,
            black_rooks: BITBOARD_EMPTY,
            black_queens: BITBOARD_EMPTY,
            black_king: BITBOARD_EMPTY,
            
            white_pieces: BITBOARD_EMPTY,
            black_pieces: BITBOARD_EMPTY,
            all_pieces: BITBOARD_EMPTY,
        }
    }
    
    // Rebuild all bitboards from the squares array
    pub fn rebuild_from_squares(&mut self, squares: &[Piece; 64]) {
        // Clear all bitboards
        *self = BitboardManager::new();
        
        // Build bitboards by scanning the squares array
        for (index, &piece) in squares.iter().enumerate() {
            if !crate::types::is_empty(piece) {
                let bb = square_to_bitboard(index as u8);
                let piece_type = crate::types::piece_type(piece);
                let piece_color = crate::types::piece_color(piece);
                
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
        if !crate::types::is_empty(piece) {
            let piece_type = crate::types::piece_type(piece);
            let piece_color = crate::types::piece_color(piece);
            
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
    
    // Update the aggregate bitboards
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
            _ => BITBOARD_EMPTY,
        }
    }
    
    // Get all pieces of a specific color
    pub fn get_all_pieces(&self, color: u8) -> Bitboard {
        match color {
            WHITE => self.white_pieces,
            BLACK => self.black_pieces,
            _ => BITBOARD_EMPTY,
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

// Public initialization function for the entire engine
pub fn initialize_engine() {
    initialize_knight_attacks();
    initialize_king_attacks();
    // Add other initializations here later
}

// Helper function to generate expected knight mask (outside tests module)
pub fn generate_expected_knight_mask(square: u8) -> Bitboard {
    let file = square % 8;
    let rank = square / 8;
    let mut attacks = 0u64;
    
    // Knight move offsets: L-shaped moves
    let knight_offsets = [(-2, -1), (-2, 1), (-1, -2), (-1, 2), (1, -2), (1, 2), (2, -1), (2, 1)];
    
    for (df, dr) in knight_offsets {
        let new_file = file as i8 + df;
        let new_rank = rank as i8 + dr;
        
        // Check bounds
        if new_file >= 0 && new_file < 8 && new_rank >= 0 && new_rank < 8 {
            let target_square = (new_rank * 8 + new_file) as u8;
            attacks |= 1u64 << target_square;
        }
    }
    
    attacks
}

pub fn get_knight_attacks(square: u8) -> Bitboard {
    unsafe {
        KNIGHT_ATTACKS[square as usize]
    }
}


// Static storage for king attack masks  
static mut KING_ATTACKS: [Bitboard; 64] = [0; 64];

fn generate_king_attack_mask(square: u8) -> Bitboard {
    let file = square % 8;
    let rank = square / 8;
    let mut attacks = 0u64;
    
    // King moves in 8 directions (1 square each)
    let king_offsets = [(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)];
    
    for (df, dr) in king_offsets {
        let new_file = file as i8 + df;
        let new_rank = rank as i8 + dr;
        
        if new_file >= 0 && new_file < 8 && new_rank >= 0 && new_rank < 8 {
            let target_square = (new_rank * 8 + new_file) as u8;
            attacks |= 1u64 << target_square;
        }
    }
    
    attacks
}

pub fn initialize_king_attacks() {
    unsafe {
        for square in 0..64 {
            KING_ATTACKS[square] = generate_king_attack_mask(square as u8);
        }
    }
}

pub fn get_king_attacks(square: u8) -> Bitboard {
    unsafe {
        KING_ATTACKS[square as usize]
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knight_attack_masks() {
        // Initialize knight attacks
        initialize_knight_attacks();
        
        // Test knight on e4 (file=4, rank=3, so square index = 3*8+4 = 28)
        let e4_square = 28;
        let e4_attacks = get_knight_attacks(e4_square);
        
        println!("Testing knight on e4 (square {}):", e4_square);
        println!("Knight attack mask: 0b{:064b}", e4_attacks);
        
        // Knight on e4 should attack: c3, c5, d2, d6, f2, f6, g3, g5
        // Convert to square indices:
        // c3 = rank 2, file 2 = 2*8+2 = 18
        // c5 = rank 4, file 2 = 4*8+2 = 34
        // d2 = rank 1, file 3 = 1*8+3 = 11
        // d6 = rank 5, file 3 = 5*8+3 = 43
        // f2 = rank 1, file 5 = 1*8+5 = 13
        // f6 = rank 5, file 5 = 5*8+5 = 45
        // g3 = rank 2, file 6 = 2*8+6 = 22
        // g5 = rank 4, file 6 = 4*8+6 = 38
        let expected_squares = [18, 34, 11, 43, 13, 45, 22, 38];
        
        for &square in &expected_squares {
            assert!(get_bit(e4_attacks, square), 
                   "Knight on e4 should attack square {} ({}{})", 
                   square, 
                   (b'a' + (square % 8) as u8) as char,
                   (square / 8) + 1);
            println!("✅ Correctly attacks square {} ({}{})", 
                    square, 
                    (b'a' + (square % 8) as u8) as char,
                    (square / 8) + 1);
        }
        
        // Verify correct count
        let attack_count = count_bits(e4_attacks);
        assert_eq!(attack_count, 8, "Knight on e4 should have exactly 8 attack squares, got {}", attack_count);
        
        println!("✅ Knight mask test PASSED for e4 - {} attack squares", attack_count);
        
        // Test edge cases
        test_knight_corner_cases();
    }
    
    fn test_knight_corner_cases() {
        // Test knight on a1 (corner)
        let a1_attacks = get_knight_attacks(0); // a1 = 0
        let a1_count = count_bits(a1_attacks);
        println!("Knight on a1 has {} attack squares", a1_count);
        assert_eq!(a1_count, 2, "Knight on a1 should have 2 attack squares");
        
        // Test knight on h8 (opposite corner)
        let h8_attacks = get_knight_attacks(63); // h8 = 63
        let h8_count = count_bits(h8_attacks);
        println!("Knight on h8 has {} attack squares", h8_count);
        assert_eq!(h8_count, 2, "Knight on h8 should have 2 attack squares");
        
        println!("✅ Knight corner cases PASSED");
    }

    #[test]
    fn debug_knight_mask_issue() {
        println!("🔧 Testing knight mask generation vs static array access");
        
        // Test direct generation
        let direct_mask = test_knight_mask_direct();
        
        // Test static array access
        initialize_knight_attacks();
        let static_mask = get_knight_attacks(59);
        
        println!("Direct generation: 0b{:064b}", direct_mask);
        println!("Static array:      0b{:064b}", static_mask);
        println!("Direct mask count: {}", direct_mask.count_ones());
        println!("Static mask count: {}", static_mask.count_ones());
        
        if direct_mask != static_mask {
            println!("❌ MISMATCH: Static array doesn't match direct generation!");
        } else {
            println!("✅ Both methods produce identical results");
        }
        
        // Test if either mask includes square 53 (f7 attacking d8)
        let direct_includes_53 = get_bit(direct_mask, 53);
        let static_includes_53 = get_bit(static_mask, 53);
        
        println!("Direct includes square 53 (f7): {}", direct_includes_53);
        println!("Static includes square 53 (f7): {}", static_includes_53);
        
        assert_eq!(direct_mask, static_mask, "Static array should match direct generation");
    }

    #[test]
    fn test_all_knight_masks_comprehensive() {
        println!("🔧 Testing all 64 knight attack masks for correctness");
        
        // Initialize the static array
        initialize_knight_attacks();
        
        let mut total_errors = 0;
        let mut failed_squares = Vec::new();
        
        // Test every square on the board
        for square in 0..64 {
            let file = square % 8;
            let rank = square / 8;
            let square_name = format!("{}{}", (b'a' + file) as char, rank + 1);
            
            // Generate expected mask manually
            let expected_mask = generate_expected_knight_mask(square);
            
            // Get mask from static array
            let actual_mask = get_knight_attacks(square);
            
            // Compare
            if expected_mask != actual_mask {
                total_errors += 1;
                failed_squares.push(square);
                
                println!("❌ MISMATCH at square {} ({}):", square, square_name);
                println!("   Expected: 0b{:064b} (count: {})", expected_mask, expected_mask.count_ones());
                println!("   Actual:   0b{:064b} (count: {})", actual_mask, actual_mask.count_ones());
                
                // Show which attack squares differ
                let missing_attacks = expected_mask & !actual_mask; // In expected but not actual
                let extra_attacks = actual_mask & !expected_mask;   // In actual but not expected
                
                if missing_attacks != 0 {
                    println!("   Missing attacks: 0b{:064b}", missing_attacks);
                    for bit in 0..64 {
                        if (missing_attacks & (1u64 << bit)) != 0 {
                            let af = bit % 8;
                            let ar = bit / 8;
                            println!("     - Missing attack to {}{} (square {})", (b'a' + af) as char, ar + 1, bit);
                        }
                    }
                }
                
                if extra_attacks != 0 {
                    println!("   Extra attacks: 0b{:064b}", extra_attacks);
                    for bit in 0..64 {
                        if (extra_attacks & (1u64 << bit)) != 0 {
                            let af = bit % 8;
                            let ar = bit / 8;
                            println!("     - Extra attack to {}{} (square {})", (b'a' + af) as char, ar + 1, bit);
                        }
                    }
                }
                
                println!();
            } else {
                // Optionally print successful validations for a few squares
                if square == 0 || square == 28 || square == 63 || square % 10 == 0 {
                    println!("✅ Square {} ({}) - {} attacks", square, square_name, actual_mask.count_ones());
                }
            }
        }
        
        // Summary
        println!("\n📊 Test Summary:");
        println!("   Total squares tested: 64");
        println!("   Failed squares: {}", total_errors);
        println!("   Success rate: {:.1}%", (64 - total_errors) as f32 / 64.0 * 100.0);
        
        if total_errors > 0 {
            println!("   Failed squares: {:?}", failed_squares);
            panic!("❌ {} knight mask(s) failed validation!", total_errors);
        } else {
            println!("✅ All knight attack masks are correct!");
        }
    }
}

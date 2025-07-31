use crate::types::*;
use std::cell::RefCell;
// Declare submodules
pub mod moves;
pub mod validation;
pub mod state;
pub mod debug;


#[derive(Debug, Clone)]
pub struct Board {
    pub squares: [Piece; 64],
    pub current_turn: u8,
    pub move_history: Vec<GameMove>,
    pub game_status: GameStatus,
    pub half_move_clock: u16,
    pub full_move_number: u16,
    pub castling_rights: u8,
    pub en_passant_target: Option<Square>,
    pub en_passant_pawn: Option<Square>,
    pub ignore_square_for_threats: RefCell<Option<Square>>,
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
            en_passant_target: None,
            en_passant_pawn: None,
            ignore_square_for_threats: RefCell::new(None),
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

    // Basic board operations
    pub fn get_piece(&self, square: Square) -> Piece {
        // Check if this square should be ignored for threat detection
        if let Some(ignored) = *self.ignore_square_for_threats.borrow() {
            if square == ignored {
                return EMPTY;
            }
        }
        
        self.squares[square.0 as usize]
    }

    pub fn set_piece(&mut self, square: Square, piece: Piece) {
        self.squares[square.0 as usize] = piece;
    }

    // FEN parsing functionality
    pub fn from_fen(fen: &str) -> Result<Self, String> {
        let parts: Vec<&str> = fen.split_whitespace().collect();
        if parts.len() != 6 {
            return Err("Invalid FEN: must have 6 parts".to_string());
        }

        let mut board = Self {
            squares: [EMPTY; 64],
            current_turn: WHITE,
            move_history: Vec::new(),
            game_status: GameStatus::InProgress,
            half_move_clock: 0,
            full_move_number: 1,
            castling_rights: 0,
            en_passant_target: None,
            en_passant_pawn: None,
            ignore_square_for_threats: RefCell::new(None),            
        };

        // Parse piece placement (part 0)
        board.parse_piece_placement(parts[0])?;
        
        // Parse active color (part 1)
        board.current_turn = match parts[1] {
            "w" => WHITE,
            "b" => BLACK,
            _ => return Err("Invalid active color".to_string()),
        };
        
        // Parse castling rights (part 2)
        board.parse_castling_rights(parts[2])?;
        
        // Parse en passant (part 3)
        board.parse_en_passant(parts[3])?;
        
        // Parse halfmove clock (part 4)
        board.half_move_clock = parts[4].parse()
            .map_err(|_| "Invalid halfmove clock")?;
        
        // Parse fullmove number (part 5)
        board.full_move_number = parts[5].parse()
            .map_err(|_| "Invalid fullmove number")?;

        Ok(board)
    }

    fn parse_piece_placement(&mut self, placement: &str) -> Result<(), String> {
        let ranks: Vec<&str> = placement.split('/').collect();
        if ranks.len() != 8 {
            return Err("Invalid piece placement: must have 8 ranks".to_string());
        }

        for (rank_idx, rank_str) in ranks.iter().enumerate() {
            let rank = 7 - rank_idx; // FEN starts from rank 8, we start from rank 0
            let mut file = 0;

            for ch in rank_str.chars() {
                if ch.is_ascii_digit() {
                    let empty_squares = ch.to_digit(10).unwrap() as u8;
                    file += empty_squares;
                } else {
                    if file >= 8 {
                        return Err("Too many pieces in rank".to_string());
                    }

                    let piece = self.char_to_piece(ch)?;
                    self.set_piece(Square::new(file, rank as u8), piece);
                    file += 1;
                }
            }

            if file != 8 {
                return Err("Incomplete rank".to_string());
            }
        }

        Ok(())
    }

    fn char_to_piece(&self, ch: char) -> Result<Piece, String> {
        let piece_type = match ch.to_ascii_lowercase() {
            'p' => PAWN,
            'n' => KNIGHT,
            'b' => BISHOP,
            'r' => ROOK,
            'q' => QUEEN,
            'k' => KING,
            _ => return Err(format!("Unknown piece: {}", ch)),
        };

        let color = if ch.is_uppercase() { WHITE } else { BLACK };
        Ok(make_piece(piece_type, color))
    }

    fn parse_castling_rights(&mut self, castling_str: &str) -> Result<(), String> {
        if castling_str == "-" {
            self.castling_rights = 0;
            return Ok(());
        }

        for ch in castling_str.chars() {
            match ch {
                'K' => self.castling_rights |= WHITE_KINGSIDE,
                'Q' => self.castling_rights |= WHITE_QUEENSIDE,
                'k' => self.castling_rights |= BLACK_KINGSIDE,
                'q' => self.castling_rights |= BLACK_QUEENSIDE,
                _ => return Err(format!("Invalid castling right: {}", ch)),
            }
        }

        Ok(())
    }

    fn parse_en_passant(&mut self, en_passant_str: &str) -> Result<(), String> {
        if en_passant_str == "-" {
            self.en_passant_target = None;
            self.en_passant_pawn = None;
            return Ok(());
        }

        if en_passant_str.len() != 2 {
            return Err("Invalid en passant square".to_string());
        }

        let chars: Vec<char> = en_passant_str.chars().collect();
        let file = (chars[0] as u8).wrapping_sub(b'a');
        let rank = (chars[1] as u8).wrapping_sub(b'1');

        if file >= 8 || rank >= 8 {
            return Err("Invalid en passant square coordinates".to_string());
        }

        self.en_passant_target = Some(Square::new(file, rank));
        
        // Calculate the pawn square (the pawn that can be captured)
        let pawn_rank = if rank == 2 { 3 } else { 4 }; // En passant is on rank 3 or 6, pawn is on 4 or 5
        self.en_passant_pawn = Some(Square::new(file, pawn_rank));

        Ok(())
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

}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

// Helper function (outside the impl block)
pub fn square_to_algebraic(square: Square) -> String {
    let file = (b'a' + square.file()) as char;
    let rank = (b'1' + square.rank()) as char;
    format!("{}{}", file, rank)
}

// Helper function to display moves with promotion
pub fn move_to_algebraic(mv: Move) -> String {
    let from_str = square_to_algebraic(mv.from);
    let to_str = square_to_algebraic(mv.to);
    
    if let Some(promotion) = mv.promotion {
        let promotion_char = match promotion {
            QUEEN => 'q',
            ROOK => 'r', 
            BISHOP => 'b',
            KNIGHT => 'n',
            _ => '?',
        };
        format!("{}{}{}", from_str, to_str, promotion_char)
    } else {
        format!("{}{}", from_str, to_str)
    }
}


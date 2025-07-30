use crate::types::*;
use crate::perft::perft;

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
    pub en_passant_target: Option<Square>,  // Square "behind" the pawn that moved 2
    pub en_passant_pawn: Option<Square>,    // Square of the pawn that can be captured
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

    pub fn debug_game_state(&self) -> Vec<String> {
        let mut debug_info = Vec::new();
        
        debug_info.push(format!("Current turn: {}", 
                               if self.current_turn == WHITE { "White" } else { "Black" }));
        
        debug_info.push(format!("Castling rights: {:04b} ({})", 
                               self.castling_rights, self.castling_rights));
        
        if let Some(target) = self.en_passant_target {
            debug_info.push(format!("En passant target: {}", square_to_algebraic(target)));
        } else {
            debug_info.push("En passant target: None".to_string());
        }
        
        if let Some(pawn) = self.en_passant_pawn {
            debug_info.push(format!("En passant pawn: {}", square_to_algebraic(pawn)));
        } else {
            debug_info.push("En passant pawn: None".to_string());
        }
        
        debug_info.push(format!("Half-move clock: {}", self.half_move_clock));
        debug_info.push(format!("Full-move number: {}", self.full_move_number));
        
        debug_info
    }
    
    /// Test specific problematic sequence
    pub fn debug_e2e4_sequence(&mut self) -> Vec<String> {
        let mut debug_info = Vec::new();
        
        debug_info.push("=== Initial State ===".to_string());
        debug_info.extend(self.debug_game_state());
        
        // Make e2-e4
        let e2 = Square::new(4, 1);
        let e4 = Square::new(4, 3);
        let mv = Move::new(e2, e4);
        
        debug_info.push("\n=== After e2-e4 ===".to_string());
        if let Ok(_) = self.try_make_move(mv) {
            debug_info.extend(self.debug_game_state());
            
            // Count moves in this position
            let all_moves = self.get_all_legal_moves();
            debug_info.push(format!("Legal moves available: {}", all_moves.len()));
            
            // Check for suspicious moves
            let mut en_passant_moves = 0;
            let mut castling_moves = 0;
            
            for legal_mv in &all_moves {
                if self.is_en_passant_move(*legal_mv) {
                    en_passant_moves += 1;
                }
                if self.is_castling_move(*legal_mv).is_some() {
                    castling_moves += 1;
                }
            }
            
            debug_info.push(format!("En passant moves: {}", en_passant_moves));
            debug_info.push(format!("Castling moves: {}", castling_moves));
            
        } else {
            debug_info.push("Failed to make e2-e4 move!".to_string());
        }
        
        debug_info
    }

    pub fn debug_fen_loading() {
        println!("\n🔍 Testing FEN Loading:");
        
        // Test starting position FEN
        let starting_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let board1 = Board::from_fen(starting_fen).unwrap();
        let moves1 = board1.get_all_legal_moves().len();
        println!("Starting position from FEN: {} moves", moves1);
        
        // Test starting position from new()
        let board2 = Board::new();
        let moves2 = board2.get_all_legal_moves().len();
        println!("Starting position from new(): {} moves", moves2);
        
        // Test Kiwipete
        let kiwipete_fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
        let board3 = Board::from_fen(kiwipete_fen).unwrap();
        let moves3 = board3.get_all_legal_moves().len();
        println!("Kiwipete from FEN: {} moves (should be 48)", moves3);
        
        // Test after e2-e4 sequence
        let mut board4 = Board::new();
        let e2 = Square::new(4, 1);
        let e4 = Square::new(4, 3);
        let mv = Move::new(e2, e4);
        if board4.try_make_move(mv).is_ok() {
            let moves4 = board4.get_all_legal_moves().len();
            println!("After e2-e4: {} moves (should be ~20)", moves4);
            
            // Run a quick perft depth 2 to see the explosion
            let nodes = crate::perft::perft(&mut board4, 2);
            println!("Perft depth 2 after e2-e4: {} nodes (should be ~400-500)", nodes);
        }
    }
    
    /// Test undo functionality
    pub fn test_undo_functionality(&mut self) -> Vec<String> {
        let mut debug_info = Vec::new();
        
        // Capture initial state
        let initial_board = self.clone();
        let initial_moves = self.get_all_legal_moves().len();
        debug_info.push(format!("Initial moves: {}", initial_moves));
        
        // Test e2-e4 and undo
        let e2 = Square::new(4, 1);
        let e4 = Square::new(4, 3);
        let mv = Move::new(e2, e4);
        
        if let Ok(_game_move) = self.try_make_move(mv) {
            let after_move_moves = self.get_all_legal_moves().len();
            debug_info.push(format!("After e2-e4: {} moves", after_move_moves));
            
            // Undo the move
            if let Ok(_) = self.undo_move() {
                let after_undo_moves = self.get_all_legal_moves().len();
                debug_info.push(format!("After undo: {} moves", after_undo_moves));
                
                // Compare states
                if self.squares == initial_board.squares {
                    debug_info.push("✅ Board squares match after undo".to_string());
                } else {
                    debug_info.push("❌ Board squares DON'T match after undo".to_string());
                }
                
                if self.castling_rights == initial_board.castling_rights {
                    debug_info.push("✅ Castling rights match after undo".to_string());
                } else {
                    debug_info.push(format!("❌ Castling rights: {} vs {}", 
                                          self.castling_rights, initial_board.castling_rights));
                }
                
                if self.en_passant_target == initial_board.en_passant_target {
                    debug_info.push("✅ En passant target matches after undo".to_string());
                } else {
                    debug_info.push(format!("❌ En passant target: {:?} vs {:?}", 
                                          self.en_passant_target, initial_board.en_passant_target));
                }
                
                if initial_moves == after_undo_moves {
                    debug_info.push("✅ Move count matches after undo".to_string());
                } else {
                    debug_info.push(format!("❌ Move count: {} vs {}", after_undo_moves, initial_moves));
                }
                
            } else {
                debug_info.push("❌ Failed to undo move".to_string());
            }
        }
        
        debug_info
    }

    /// Debug which Black move after e2-e4 is causing the explosion
    pub fn debug_black_moves_after_e2e4(&mut self) -> Vec<String> {
        let mut debug_info = Vec::new();
        
        // Make e2-e4
        let e2 = Square::new(4, 1);
        let e4 = Square::new(4, 3);
        let mv = Move::new(e2, e4);
        
        if let Ok(_) = self.try_make_move(mv) {
            debug_info.push("After e2-e4, analyzing each Black move:".to_string());
            
            let black_moves = self.get_all_legal_moves();
            debug_info.push(format!("Black has {} legal moves", black_moves.len()));
            
            // Test each Black move
            for (i, black_move) in black_moves.iter().enumerate() {
                if let Ok(_) = self.try_make_move(*black_move) {
                    let white_moves = self.get_all_legal_moves();
                    let move_str = format!("{}{}", 
                                         square_to_algebraic(black_move.from), 
                                         square_to_algebraic(black_move.to));
                    
                    // Run perft depth 1 from this position
                    let nodes = crate::perft::perft(self, 1);
                    
                    debug_info.push(format!("{}. {}: {} white responses, {} total nodes", 
                                          i + 1, move_str, white_moves.len(), nodes));
                    
                    // Flag suspicious moves
                    if white_moves.len() > 30 || nodes > 30 {
                        debug_info.push(format!("  ⚠️  SUSPICIOUS: Too many moves/nodes!"));
                        
                        // Show some of the white moves
                        debug_info.push("  White moves:".to_string());
                        for (j, white_move) in white_moves.iter().take(5).enumerate() {
                            let white_move_str = format!("{}{}", 
                                                       square_to_algebraic(white_move.from), 
                                                       square_to_algebraic(white_move.to));
                            debug_info.push(format!("    {}: {}", j + 1, white_move_str));
                        }
                        if white_moves.len() > 5 {
                            debug_info.push(format!("    ... and {} more", white_moves.len() - 5));
                        }
                    }
                    
                    self.undo_move().expect("Failed to undo Black move");
                } else {
                    debug_info.push(format!("Failed to make Black move #{}", i + 1));
                }
            }
        }
        
        debug_info
    }

    /// Debug the specific d7-d5 issue
    pub fn debug_d7d5_issue(&mut self) -> Vec<String> {
        let mut debug_info = Vec::new();
        
        // Make e2-e4
        let e2 = Square::new(4, 1);
        let e4 = Square::new(4, 3);
        let mv1 = Move::new(e2, e4);
        
        if let Ok(_) = self.try_make_move(mv1) {
            // Make d7-d5 (the suspicious move)
            let d7 = Square::new(3, 6);
            let d5 = Square::new(3, 4);
            let mv2 = Move::new(d7, d5);
            
            if let Ok(_) = self.try_make_move(mv2) {
                debug_info.push("After 1.e2-e4 d7-d5:".to_string());
                debug_info.extend(self.debug_game_state());
                
                let white_moves = self.get_all_legal_moves();
                debug_info.push(format!("White has {} moves (should be ~20):", white_moves.len()));
                
                // Categorize the moves
                let mut pawn_moves = 0;
                let mut knight_moves = 0;
                let mut bishop_moves = 0;
                let mut rook_moves = 0;
                let mut queen_moves = 0;
                let mut king_moves = 0;
                let mut suspicious_moves = Vec::new();
                
                for mv in &white_moves {
                    let piece = self.get_piece(mv.from);
                    let piece_type_val = piece_type(piece);
                    let move_str = format!("{}{}", 
                                         square_to_algebraic(mv.from), 
                                         square_to_algebraic(mv.to));
                    
                    match piece_type_val {
                        1 => pawn_moves += 1,    // PAWN
                        2 => knight_moves += 1,  // KNIGHT
                        3 => bishop_moves += 1,  // BISHOP
                        4 => rook_moves += 1,    // ROOK
                        5 => queen_moves += 1,   // QUEEN
                        6 => king_moves += 1,    // KING
                        _ => suspicious_moves.push(move_str),
                    }
                }
                
                debug_info.push(format!("Pawns: {}, Knights: {}, Bishops: {}, Rooks: {}, Queens: {}, Kings: {}", 
                               pawn_moves, knight_moves, bishop_moves, rook_moves, queen_moves, king_moves));
                
                if !suspicious_moves.is_empty() {
                    debug_info.push(format!("Suspicious moves: {:?}", suspicious_moves));
                }
                
                // Check for excessive queen moves (common bug)
                if queen_moves > 8 {
                    debug_info.push(format!("⚠️ Queen has {} moves (suspicious!)", queen_moves));
                    debug_info.push("Queen moves:".to_string());
                    for mv in &white_moves {
                        let piece = self.get_piece(mv.from);
                        if piece_type(piece) == 5 { // QUEEN
                            let move_str = format!("{}{}", 
                                                 square_to_algebraic(mv.from), 
                                                 square_to_algebraic(mv.to));
                            debug_info.push(format!("  Queen: {}", move_str));
                        }
                    }
                }
                
                // Check for excessive bishop moves
                if bishop_moves > 6 {
                    debug_info.push(format!("⚠️ Bishops have {} moves (suspicious!)", bishop_moves));
                }
                
                self.undo_move().expect("Failed to undo d7-d5");
            }
            self.undo_move().expect("Failed to undo e2-e4");
        }
        
        debug_info
    }

    /// Show the specific illegal bishop and queen moves
    pub fn debug_illegal_sliding_moves(&mut self) -> Vec<String> {
        let mut debug_info = Vec::new();
        
        // Make e2-e4, then d7-d5
        let e2e4 = Move::new(Square::new(4, 1), Square::new(4, 3));
        let d7d5 = Move::new(Square::new(3, 6), Square::new(3, 4));
        
        if self.try_make_move(e2e4).is_ok() && self.try_make_move(d7d5).is_ok() {
            debug_info.push("🔍 Bishop moves:".to_string());
            
            // Check white bishops
            for rank in 0..8 {
                for file in 0..8 {
                    let square = Square::new(file, rank);
                    let piece = self.get_piece(square);
                    
                    if piece_type(piece) == BISHOP && piece_color(piece) == WHITE {
                        let moves = self.get_legal_moves(square);
                        debug_info.push(format!("  Bishop at {}: {} moves", 
                                               square_to_algebraic(square), moves.len()));
                        for mv in moves {
                            debug_info.push(format!("    -> {}", square_to_algebraic(mv)));
                        }
                    }
                }
            }
            
            debug_info.push("🔍 Queen moves:".to_string());
            
            // Check white queen
            for rank in 0..8 {
                for file in 0..8 {
                    let square = Square::new(file, rank);
                    let piece = self.get_piece(square);
                    
                    if piece_type(piece) == QUEEN && piece_color(piece) == WHITE {
                        let moves = self.get_legal_moves(square);
                        debug_info.push(format!("  Queen at {}: {} moves", 
                                               square_to_algebraic(square), moves.len()));
                        for mv in moves {
                            debug_info.push(format!("    -> {}", square_to_algebraic(mv)));
                        }
                    }
                }
            }
            
            self.undo_move().expect("Failed to undo");
            self.undo_move().expect("Failed to undo");
        }
        
        debug_info
    }

    /// Test the exact board state after e2-e4 d7-d5
    pub fn debug_board_state_after_moves(&mut self) -> Vec<String> {
        let mut debug_info = Vec::new();
        
        // Make the moves
        let e2e4 = Move::new(Square::new(4, 1), Square::new(4, 3));
        let d7d5 = Move::new(Square::new(3, 6), Square::new(3, 4));
        
        if self.try_make_move(e2e4).is_ok() && self.try_make_move(d7d5).is_ok() {
            
            // Check specific squares that should affect sliding pieces
            debug_info.push("🔍 Critical square states:".to_string());
            
            let critical_squares = [
                ("e2", Square::new(4, 1)),  // Should be EMPTY after pawn moved
                ("e4", Square::new(4, 3)),  // Should have WHITE pawn
                ("d5", Square::new(3, 4)),  // Should have BLACK pawn
                ("f1", Square::new(5, 0)),  // WHITE bishop
                ("d1", Square::new(3, 0)),  // WHITE queen
            ];
            
            for (name, square) in critical_squares {
                let piece = self.get_piece(square);
                let piece_type_val = if is_empty(piece) { 0 } else { piece_type(piece) };
                let piece_color_val = if is_empty(piece) { 0 } else { piece_color(piece) };
                
                debug_info.push(format!("  {}: piece={}, type={}, color={}, empty={}", 
                                    name, piece, piece_type_val, piece_color_val, is_empty(piece)));
            }
            
            // Test the specific illegal moves
            debug_info.push("\n🎯 Testing specific illegal bishop move f1-e2:".to_string());
            
            let f1 = Square::new(5, 0);  // f1 bishop
            let e2 = Square::new(4, 1);  // e2 target
            
            let bishop_piece = self.get_piece(f1);
            let e2_piece = self.get_piece(e2);
            let bishop_color = piece_color(bishop_piece);
            let e2_color = if is_empty(e2_piece) { 255 } else { piece_color(e2_piece) };
            
            debug_info.push(format!("  Bishop at f1: piece={}, color={}", bishop_piece, bishop_color));
            debug_info.push(format!("  Square e2: piece={}, color={}, empty={}", e2_piece, e2_color, is_empty(e2_piece)));
            debug_info.push(format!("  Colors equal? {}", bishop_color == e2_color));
            debug_info.push(format!("  Should block? {}", !is_empty(e2_piece) && piece_color(e2_piece) == bishop_color));
            
            // Test queen move d1-e2
            debug_info.push("\n🎯 Testing specific illegal queen move d1-e2:".to_string());
            
            let d1 = Square::new(3, 0);  // d1 queen
            let queen_piece = self.get_piece(d1);
            let queen_color = piece_color(queen_piece);
            
            debug_info.push(format!("  Queen at d1: piece={}, color={}", queen_piece, queen_color));
            debug_info.push(format!("  Same e2 analysis applies"));
            
            self.undo_move().expect("Failed to undo");
            self.undo_move().expect("Failed to undo");
        }
        
        debug_info
    }

    /// Format perft divide output like Stockfish for easy comparison
    pub fn debug_perft_divide_formatted(&self, depth: u32) -> Vec<String> {
        let mut formatted_output = Vec::new();
        let moves = self.get_all_legal_moves();
        let mut total_nodes = 0;
        
        for mv in moves {
            let mut temp_board = self.clone();
            if let Ok(_) = temp_board.try_make_move(mv) {
                let nodes = if depth > 1 {
                    crate::perft::perft(&mut temp_board, depth - 1)
                } else {
                    1
                };
                
                // Format as algebraic notation like Stockfish: "e2e4: 13164"
                let move_str = format!("{}{}", 
                    square_to_algebraic(mv.from), 
                    square_to_algebraic(mv.to));
                
                formatted_output.push(format!("{}: {}", move_str, nodes));
                total_nodes += nodes;
            }
        }
        
        // Sort moves alphabetically (like Stockfish does)
        formatted_output.sort();
        
        // Add total at the end
        formatted_output.push(format!("\nNodes searched: {}", total_nodes));
        
        formatted_output
    }    

    pub fn debug_move_deeper(&self, move_notation: &str, max_depth: u32) -> Vec<String> {
        let mut debug_info = Vec::new();
        
        // Parse the move notation (e.g., "g1f3")
        let from_file = (move_notation.chars().nth(0).unwrap() as u8) - b'a';
        let from_rank = (move_notation.chars().nth(1).unwrap() as u8) - b'1';
        let to_file = (move_notation.chars().nth(2).unwrap() as u8) - b'a';
        let to_rank = (move_notation.chars().nth(3).unwrap() as u8) - b'1';
        
        let from_square = Square::new(from_file, from_rank);
        let to_square = Square::new(to_file, to_rank);
        let target_move = Move::new(from_square, to_square);
        
        debug_info.push(format!("🔍 Deep analysis of move {}", move_notation));
        
        // Make the target move
        let mut temp_board = self.clone();
        if let Ok(_) = temp_board.try_make_move(target_move) {
            
            // Run perft divide at multiple depths
            for depth in 1..=max_depth {
                debug_info.push(format!("\n--- After {} at depth {} ---", move_notation, depth));
                
                let moves = temp_board.get_all_legal_moves();
                let mut total_nodes = 0;
                let mut move_results = Vec::new();
                
                for mv in moves {
                    let mut test_board = temp_board.clone();
                    if let Ok(_) = test_board.try_make_move(mv) {
                        let nodes = if depth > 1 {
                            crate::perft::perft(&mut test_board, depth - 1)
                        } else {
                            1
                        };
                        
                        let move_str = format!("{}{}", 
                            square_to_algebraic(mv.from), 
                            square_to_algebraic(mv.to));
                        
                        move_results.push((move_str, nodes));
                        total_nodes += nodes;
                    }
                }
                
                // Sort and display results
                move_results.sort();
                for (move_str, nodes) in move_results {
                    debug_info.push(format!("{}: {}", move_str, nodes));
                }
                debug_info.push(format!("Total nodes at depth {}: {}", depth, total_nodes));
            }
        }
        
        debug_info
    }

    /// Debug specific move sequence and show perft divide in Stockfish format
    pub fn debug_move_sequence_perft(&self, moves: &[&str], depth: u32) -> Vec<String> {
        let mut debug_info = Vec::new();
        let mut temp_board = self.clone();
        
        // Make the sequence of moves
        for move_notation in moves {
            let from_file = (move_notation.chars().nth(0).unwrap() as u8) - b'a';
            let from_rank = (move_notation.chars().nth(1).unwrap() as u8) - b'1';
            let to_file = (move_notation.chars().nth(2).unwrap() as u8) - b'a';
            let to_rank = (move_notation.chars().nth(3).unwrap() as u8) - b'1';
            
            let from_square = Square::new(from_file, from_rank);
            let to_square = Square::new(to_file, to_rank);
            let target_move = Move::new(from_square, to_square);
            
            if let Ok(_) = temp_board.try_make_move(target_move) {
                debug_info.push(format!("Made move: {}", move_notation));
            } else {
                debug_info.push(format!("Failed to make move: {}", move_notation));
                return debug_info;
            }
        }
        
        // Now run perft divide at specified depth
        let moves = temp_board.get_all_legal_moves();
        let mut move_results = Vec::new();
        let mut total_nodes = 0;
        
        for mv in moves {
            let mut test_board = temp_board.clone();
            if let Ok(_) = test_board.try_make_move(mv) {
                let nodes = if depth > 1 {
                    crate::perft::perft(&mut test_board, depth - 1)
                } else {
                    1
                };
                
                let move_str = format!("{}{}", 
                    square_to_algebraic(mv.from), 
                    square_to_algebraic(mv.to));
                
                move_results.push((move_str, nodes));
                total_nodes += nodes;
            }
        }
        
        // Sort moves alphabetically like Stockfish
        move_results.sort_by(|a, b| a.0.cmp(&b.0));
        
        // Format output exactly like Stockfish
        for (move_str, nodes) in move_results {
            debug_info.push(format!("{}: {}", move_str, nodes));
        }
        
        debug_info.push(format!("\nNodes searched: {}", total_nodes));
        
        debug_info
    }
    
    /// Debug the f3e5 position specifically
    pub fn debug_f3e5_position(&self) -> Vec<String> {
        let mut debug_info = Vec::new();
        let mut temp_board = self.clone();
        
        // Make the sequence: g1f3, d7d5, f3e5
        let moves = [
            ("g1f3", Square::new(6, 0), Square::new(5, 2)),
            ("d7d5", Square::new(3, 6), Square::new(3, 4)),
            ("f3e5", Square::new(5, 2), Square::new(4, 4)),
        ];
        
        for (notation, from, to) in moves {
            let mv = Move::new(from, to);
            if let Ok(_) = temp_board.try_make_move(mv) {
                debug_info.push(format!("Made move: {}", notation));
            } else {
                debug_info.push(format!("Failed to make move: {}", notation));
                return debug_info;
            }
        }
        
        // Show position state after 1.Nf3 d5 2.Ne5
        debug_info.push(format!("Position after 1.Nf3 d5 2.Ne5:"));
        debug_info.push(format!("Current turn: {}", 
                               if temp_board.current_turn == 0 { "Black" } else { "White" }));
        debug_info.push(format!("En passant target: {:?}", temp_board.en_passant_target));
        debug_info.push(format!("En passant pawn: {:?}", temp_board.en_passant_pawn));
        
        // Get all legal moves (should be 26, not 27)
        let moves = temp_board.get_all_legal_moves();
        debug_info.push(format!("Total legal moves: {} (should be 26)", moves.len()));
        
        // Show all moves
        for mv in &moves {
            let move_str = format!("{}{}", 
                square_to_algebraic(mv.from), 
                square_to_algebraic(mv.to));
            let piece = temp_board.get_piece(mv.from);
            let piece_name = match piece_type(piece) {
                1 => "Pawn",
                2 => "Knight", 
                3 => "Bishop",
                4 => "Rook",
                5 => "Queen",
                6 => "King",
                _ => "Unknown"
            };
            debug_info.push(format!("  {}: {}", move_str, piece_name));
        }
        
        debug_info
    }
    
    /// Test if king move validation is working correctly
    pub fn debug_king_safety_validation(&self) -> Vec<String> {
        let mut debug_info = Vec::new();
        let mut temp_board = self.clone();
        
        // Make the sequence: g1f3, d7d5, f3e5
        let moves = [
            ("g1f3", Square::new(6, 0), Square::new(5, 2)),
            ("d7d5", Square::new(3, 6), Square::new(3, 4)),
            ("f3e5", Square::new(5, 2), Square::new(4, 4)),
        ];
        
        for (notation, from, to) in moves {
            let mv = Move::new(from, to);
            temp_board.try_make_move(mv).unwrap();
        }
        
        // Test if d7 is under threat by the white knight on e5
        let d7_square = Square::new(3, 6);
        let e5_square = Square::new(4, 4);
        
        debug_info.push("Testing king safety validation:".to_string());
        debug_info.push(format!("White knight on e5: {}", !is_empty(temp_board.get_piece(e5_square))));
        debug_info.push(format!("Knight piece type: {}", piece_type(temp_board.get_piece(e5_square))));
        debug_info.push(format!("Knight color: {}", piece_color(temp_board.get_piece(e5_square))));
        
        // Test if the knight attacks d7
        let knight_attacks_d7 = temp_board.piece_attacks_square(e5_square, d7_square);
        debug_info.push(format!("Knight on e5 attacks d7: {}", knight_attacks_d7));
        
        // Test general threat detection
        let d7_under_threat = temp_board.is_under_threat(d7_square, 8); // WHITE = 8
        debug_info.push(format!("d7 square under threat by White: {}", d7_under_threat));
        
        // Test the specific illegal king move
        let king_square = Square::new(4, 7); // e8
        let illegal_move = Move::new(king_square, d7_square);
        let is_valid = temp_board.is_valid_move(illegal_move);
        debug_info.push(format!("King e8-d7 move considered valid: {} (should be false)", is_valid));
        
        debug_info
    }
    
    /// Debug exactly why the king move is considered valid
    pub fn debug_king_move_validation(&self) -> Vec<String> {
        let mut debug_info = Vec::new();
        let mut temp_board = self.clone();
        
        // Set up the position: g1f3, d7d5, f3e5
        let moves = [
            Move::new(Square::new(6, 0), Square::new(5, 2)), // g1f3
            Move::new(Square::new(3, 6), Square::new(3, 4)), // d7d5  
            Move::new(Square::new(5, 2), Square::new(4, 4)), // f3e5
        ];
        
        for mv in moves {
            temp_board.try_make_move(mv).unwrap();
        }
        
        // Test the specific illegal king move step by step
        let king_square = Square::new(4, 7); // e8
        let target_square = Square::new(3, 6); // d7
        let illegal_move = Move::new(king_square, target_square);
        
        debug_info.push("Step-by-step validation of e8-d7:".to_string());
        
        // 1. Basic move validation
        let from_piece = temp_board.get_piece(illegal_move.from);
        let to_piece = temp_board.get_piece(illegal_move.to);
        
        debug_info.push(format!("1. From piece: {} (type: {}, color: {})", 
                               from_piece, piece_type(from_piece), piece_color(from_piece)));
        debug_info.push(format!("2. To piece: {} (empty: {})", to_piece, is_empty(to_piece)));
        debug_info.push(format!("3. Is piece correct color: {}", 
                               is_piece_color(from_piece, temp_board.current_turn)));
        
        // 2. Check if move is in pseudo-legal moves
        let pseudo_legal_moves = temp_board.get_pseudo_legal_moves(king_square);
        let in_pseudo_legal = pseudo_legal_moves.contains(&target_square);
        debug_info.push(format!("4. In pseudo-legal moves: {}", in_pseudo_legal));
        
        // 3. Check if move is in legal moves  
        let legal_moves = temp_board.get_legal_moves(king_square);
        let in_legal_moves = legal_moves.contains(&target_square);
        debug_info.push(format!("5. In legal moves: {} (should be false)", in_legal_moves));
        
        // 4. Test king safety directly
        let would_be_in_check = temp_board.would_king_be_in_check_after_move(illegal_move);
        debug_info.push(format!("6. Would king be in check after move: {} (should be true)", would_be_in_check));
        
        // 5. Final validation result
        let is_valid = temp_board.is_valid_move(illegal_move);
        debug_info.push(format!("7. Final is_valid_move result: {} (should be false)", is_valid));
        
        debug_info
    }

    /// Test if king would be in check after a specific move
    fn would_king_be_in_check_after_move(&self, mv: Move) -> bool {
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

    pub fn debug_pawn_moves(&self) -> Vec<(Square, Vec<Square>)> {
        let mut pawn_moves = Vec::new();
        
        for rank in 0..8 {
            for file in 0..8 {
                let square = Square::new(file, rank);
                let piece = self.get_piece(square);
                
                if piece_type(piece) == PAWN && piece_color(piece) == self.current_turn {
                    let moves = self.get_legal_moves(square);
                    if !moves.is_empty() {
                        pawn_moves.push((square, moves));
                    }
                }
            }
        }
        
        pawn_moves
    }
    
    /// Debug move generation for a specific square
    pub fn debug_square_moves(&self, square: Square) -> Vec<String> {
        let piece = self.get_piece(square);
        if is_empty(piece) {
            return vec!["Empty square".to_string()];
        }
        
        let mut debug_info = Vec::new();
        debug_info.push(format!("Piece: {} (type: {}, color: {})", 
                               piece, piece_type(piece), piece_color(piece)));
        
        let pseudo_moves = self.get_pseudo_legal_moves(square);
        debug_info.push(format!("Pseudo-legal moves: {}", pseudo_moves.len()));
        
        let legal_moves = self.get_legal_moves(square);
        debug_info.push(format!("Legal moves: {}", legal_moves.len()));
        
        for mv in legal_moves {
            debug_info.push(format!("  -> {}", square_to_algebraic(mv)));
        }
        
        debug_info
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
        use crate::types::*;
        
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
        use crate::types::*;
        
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

    pub fn get_piece(&self, square: Square) -> Piece {
        self.squares[square.0 as usize]
    }
    
    pub fn set_piece(&mut self, square: Square, piece: Piece) {
        self.squares[square.0 as usize] = piece;
    }
    
    /// Debug method to analyze move generation
    pub fn debug_move_count_difference(&self, depth: u32) -> Vec<(String, u64, u64)> {
        let mut results = Vec::new();
        
        if depth == 0 {
            return results;
        }
        
        let moves = self.get_all_legal_moves();
        
        for mv in moves {
            let mut temp_board = self.clone();
            if let Ok(_) = temp_board.try_make_move(mv) {
                let nodes = if depth > 1 {
                    perft(&mut temp_board, depth - 1)
                } else {
                    1
                };
                
                // Create a readable move string
                let move_str = format!("{}{}", 
                    square_to_algebraic(mv.from), 
                    square_to_algebraic(mv.to));
                
                results.push((move_str, nodes, 1));
            }
        }
        
        results.sort_by(|a, b| b.1.cmp(&a.1));
        results
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
    
        // Check for special moves
        let is_castling = self.is_castling_move(mv).is_some();
        let is_en_passant = self.is_en_passant_move(mv);
    
        // Create game move record WITH STATE CAPTURE
        let mut game_move = if is_en_passant {
            let captured_pawn = self.get_piece(self.en_passant_pawn.unwrap());
            GameMove::with_capture_and_state(mv, captured_pawn, self)
        } else if is_empty(captured_piece) {
            GameMove::new_with_state(mv, self)
        } else {
            GameMove::with_capture_and_state(mv, captured_piece, self)
        };
    
        game_move.is_castling = is_castling;
        game_move.is_en_passant = is_en_passant;
    
        // Execute the move (rest of the method stays the same)
        if is_castling {
            let kingside = self.is_castling_move(mv).unwrap();
            self.execute_castling(piece_color(moving_piece), kingside);
        } else if is_en_passant {
            self.execute_en_passant(mv);
        } else {
            self.set_piece(mv.to, moving_piece);
            self.set_piece(mv.from, EMPTY);
            self.update_castling_rights(mv);
        }
    
        if !is_castling && !is_en_passant {
            self.setup_en_passant(mv);
        }
    
        self.move_history.push(game_move.clone());
        self.current_turn = opposite_color(self.current_turn);
    
        if piece_type(moving_piece) == PAWN || !is_empty(captured_piece) || is_en_passant {
            self.half_move_clock = 0;
        } else {
            self.half_move_clock += 1;
        }
    
        if self.current_turn == WHITE {
            self.full_move_number += 1;
        }
    
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
        
        // En passant capture
        if let Some(en_passant_target) = self.en_passant_target {
            // Check if we can capture en passant
            let target_file = en_passant_target.file() as i8;
            let target_rank = en_passant_target.rank() as i8;
            let our_file = file as i8;
            let our_rank = rank as i8;
            
            // Must be adjacent horizontally and on correct rank
            let file_diff = (target_file - our_file).abs();
            let rank_diff = target_rank - our_rank;
            
            if file_diff == 1 && rank_diff == direction {
                // Check if en passant is legal (doesn't leave king in check)
                let en_passant_move = Move::new(square, en_passant_target);
                if self.is_en_passant_legal(en_passant_move) {
                    moves.push(en_passant_target);
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
                // Not in check, but still need to validate king moves
                let piece = self.get_piece(square);
                if piece_type(piece) == KING {
                    // King moves need safety validation even when not in check
                    self.filter_king_moves_in_check(pseudo_moves, opponent_color)
                } else {
                    // Non-king pieces are legal when not in check
                    pseudo_moves
                }
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
    
    pub fn get_all_legal_moves(&self) -> Vec<Move> {
        let mut all_moves = Vec::new();
        
        for rank in 0..8 {
            for file in 0..8 {
                let square = Square::new(file, rank);
                let piece = self.get_piece(square);
                
                if !is_empty(piece) && is_piece_color(piece, self.current_turn) {
                    let piece_moves = self.get_legal_moves(square);
                    for target_square in piece_moves {
                        all_moves.push(Move::new(square, target_square));
                    }
                }
            }
        }
        
        all_moves
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
                    
                    // Regular capture
                    if !is_empty(target_piece) && piece_color(target_piece) != color {
                        moves.push(capture_square);
                    }
                    
                    // En passant capture (only if along pin line)
                    if let Some(en_passant_target) = self.en_passant_target {
                        if capture_square == en_passant_target {
                            let en_passant_move = Move::new(square, en_passant_target);
                            if self.is_en_passant_legal(en_passant_move) {
                                moves.push(en_passant_target);
                            }
                        }
                    }
                }
            }
        }
        
        moves
    }
    

    /// Check if a move is a double pawn push that enables en passant
    fn is_double_pawn_push(&self, mv: Move) -> bool {
        let moving_piece = self.get_piece(mv.to);
        
        
        // Must be a pawn
        if piece_type(moving_piece) != PAWN {
            return false;
        }
        
        let from_rank = mv.from.rank();
        let to_rank = mv.to.rank();
        let color = piece_color(moving_piece);
        
        // Must move exactly 2 squares
        let rank_diff = (to_rank as i8 - from_rank as i8).abs();
        
        if rank_diff != 2 {
            return false;
        }
        
        // Must be from starting position
        let starting_rank = if color == WHITE { 1 } else { 6 };
        
        if from_rank != starting_rank {
            return false;
        }
        
        // Must be moving in correct direction
        let expected_direction = if color == WHITE { 1 } else { -1 };
        let actual_direction = to_rank as i8 - from_rank as i8;
        
        
        let result = actual_direction == expected_direction * 2;
        
        result
    }
    
    /// Set up en passant target after a double pawn push
    fn setup_en_passant(&mut self, mv: Move) {
        
        if self.is_double_pawn_push(mv) {
            let moving_piece = self.get_piece(mv.to);
            let color = piece_color(moving_piece);
            
            // Calculate the square the pawn "jumped over"
            let target_rank = if color == WHITE {
                mv.from.rank() + 1 // Square between starting and ending position
            } else {
                mv.from.rank() - 1
            };
            
            self.en_passant_target = Some(Square::new(mv.from.file(), target_rank));
            self.en_passant_pawn = Some(mv.to); // The pawn that can be captured
            
        } else {
            // Clear en passant if not a double pawn push
            self.en_passant_target = None;
            self.en_passant_pawn = None;
        }
    }
    
    /// Check if a move is an en passant capture
    fn is_en_passant_move(&self, mv: Move) -> bool {
        // Must be a pawn move
        let moving_piece = self.get_piece(mv.from);
        if piece_type(moving_piece) != PAWN {
            return false;
        }
        
        // Must have an en passant target
        let target = match self.en_passant_target {
            Some(square) => square,
            None => return false,
        };
        
        // Must be moving to the en passant target square
        if mv.to != target {
            return false;
        }
        
        // Target square must be empty (pawn moves diagonally to empty square)
        if !is_empty(self.get_piece(mv.to)) {
            return false;
        }
        
        // Must be a diagonal move
        let file_diff = (mv.to.file() as i8 - mv.from.file() as i8).abs();
        let rank_diff = (mv.to.rank() as i8 - mv.from.rank() as i8).abs();
        
        file_diff == 1 && rank_diff == 1
    }
    
    /// Check if en passant move is legal (doesn't leave king in check)
    fn is_en_passant_legal(&self, mv: Move) -> bool {
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
        temp_board.set_piece(target_square, capturing_pawn);           // Move our pawn
        temp_board.set_piece(capturing_pawn_square, EMPTY);          // Clear original position
        temp_board.set_piece(captured_pawn_square, EMPTY);           // Remove captured pawn
        
        // Check if our king would be in check after this move
        !temp_board.is_under_threat(king_square, opponent_color)
    }
    
    /// Execute an en passant capture
    fn execute_en_passant(&mut self, mv: Move) {
        let capturing_pawn = self.get_piece(mv.from);
        let captured_pawn_square = self.en_passant_pawn.unwrap();
        
        // Move the capturing pawn
        self.set_piece(mv.to, capturing_pawn);
        self.set_piece(mv.from, EMPTY);
        
        // Remove the captured pawn
        self.set_piece(captured_pawn_square, EMPTY);
        
        // Clear en passant state
        self.en_passant_target = None;
        self.en_passant_pawn = None;
    }

    /// Undo the last move made
    pub fn undo_move(&mut self) -> Result<GameMove, String> {
        // Get the last move from history
        let last_move = match self.move_history.pop() {
            Some(mv) => mv,
            None => return Err("No moves to undo".to_string()),
        };

        // Restore the pieces on the board
        self.restore_pieces(&last_move);

        // Restore all board state
        self.castling_rights = last_move.previous_castling_rights;
        self.en_passant_target = last_move.previous_en_passant_target;
        self.en_passant_pawn = last_move.previous_en_passant_pawn;
        self.half_move_clock = last_move.previous_half_move_clock;
        self.full_move_number = last_move.previous_full_move_number;

        // Switch turn back
        self.current_turn = opposite_color(self.current_turn);

        Ok(last_move)
    }

    fn restore_pieces(&mut self, game_move: &GameMove) {
        let mv = game_move.mv;

        if game_move.is_castling {
            self.undo_castling(mv);
        } else if game_move.is_en_passant {
            self.undo_en_passant(game_move);
        } else {
            // Regular move - move piece back and restore captured piece
            let moving_piece = self.get_piece(mv.to);
            self.set_piece(mv.from, moving_piece);
            self.set_piece(mv.to, game_move.captured_piece);
        }
    }

    fn undo_castling(&mut self, mv: Move) {
        let _color = piece_color(self.get_piece(mv.to));
        let king_rank = mv.to.rank();

        // Determine if it was kingside or queenside castling
        let is_kingside = mv.to.file() == 6; // g-file

        if is_kingside {
            // Undo kingside castling
            let king = self.get_piece(Square::new(6, king_rank));
            let rook = self.get_piece(Square::new(5, king_rank));

            self.set_piece(Square::new(4, king_rank), king);  // King back to e-file
            self.set_piece(Square::new(7, king_rank), rook);  // Rook back to h-file
            self.set_piece(Square::new(6, king_rank), EMPTY);
            self.set_piece(Square::new(5, king_rank), EMPTY);
        } else {
            // Undo queenside castling
            let king = self.get_piece(Square::new(2, king_rank));
            let rook = self.get_piece(Square::new(3, king_rank));

            self.set_piece(Square::new(4, king_rank), king);  // King back to e-file
            self.set_piece(Square::new(0, king_rank), rook);  // Rook back to a-file
            self.set_piece(Square::new(2, king_rank), EMPTY);
            self.set_piece(Square::new(3, king_rank), EMPTY);
        }
    }

    fn undo_en_passant(&mut self, game_move: &GameMove) {
        let mv = game_move.mv;

        // Move our pawn back
        let our_pawn = self.get_piece(mv.to);
        self.set_piece(mv.from, our_pawn);
        self.set_piece(mv.to, EMPTY);

        // Restore the captured pawn
        let captured_pawn_square = if piece_color(our_pawn) == WHITE {
            Square::new(mv.to.file(), mv.to.rank() - 1)  // Black pawn was one rank below
        } else {
            Square::new(mv.to.file(), mv.to.rank() + 1)  // White pawn was one rank above
        };

        self.set_piece(captured_pawn_square, game_move.captured_piece);
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

// Helper function (outside the impl block)
fn square_to_algebraic(square: Square) -> String {
    let file = (b'a' + square.file()) as char;
    let rank = (b'1' + square.rank()) as char;
    format!("{}{}", file, rank)
}

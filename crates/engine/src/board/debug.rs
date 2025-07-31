use crate::types::*;
use super::{Board, square_to_algebraic};

impl Board {
    /// Debug game state information
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
                        debug_info.push("  ⚠️  SUSPICIOUS: Too many moves/nodes!".to_string());
                        
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
            debug_info.push("  Same e2 analysis applies".to_string());
            
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

    /// Debug specific move at deeper levels
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


    /// Debug the f3e5 position specifically
    pub fn debug_f3e5_position(&self) -> Vec<String> {
        let mut debug_info = Vec::new();
        let mut temp_board = self.clone();
        
        // Make the sequence: g1f3, d7d5, f3e5
        let _moves = [
            ("g1f3", Square::new(6, 0), Square::new(5, 2)),
            ("d7d5", Square::new(3, 6), Square::new(3, 4)),
            ("f3e5", Square::new(5, 2), Square::new(4, 4)),
        ];
        
        for (_notation, from, to) in _moves {
            let mv = Move::new(from, to);
            if let Ok(_) = temp_board.try_make_move(mv) {
                debug_info.push(format!("Made move: {}", _notation));
            } else {
                debug_info.push(format!("Failed to make move: {}", _notation));
                return debug_info;
            }
        }
        
        // Show position state after 1.Nf3 d5 2.Ne5
        debug_info.push("Position after 1.Nf3 d5 2.Ne5:".to_string());
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
        let _moves = [
            ("g1f3", Square::new(6, 0), Square::new(5, 2)),
            ("d7d5", Square::new(3, 6), Square::new(3, 4)),
            ("f3e5", Square::new(5, 2), Square::new(4, 4)),
        ];
        
        for (_notation, from, to) in _moves {
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


    /// Debug pawn moves specifically
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
                    crate::perft::perft(&mut temp_board, depth - 1)
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

    /// Debug Position 5 specifically - missing 3 moves at depth 1
    pub fn debug_position5_missing_moves(&self) -> Vec<String> {
        let mut debug_info = Vec::new();
        
        // Load Position 5 FEN
        let fen = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";
        let board = match Board::from_fen(fen) {
            Ok(b) => b,
            Err(e) => {
                debug_info.push(format!("Failed to load FEN: {}", e));
                return debug_info;
            }
        };
        
        debug_info.push("Position 5 Analysis (missing 3 moves):".to_string());
        debug_info.push(format!("FEN: {}", fen));
        
        let moves = board.get_all_legal_moves();
        debug_info.push(format!("Your engine: {} moves (should be 44)", moves.len()));
        
        // Show all moves with piece types for analysis
        let mut move_strings = Vec::new();
        for mv in moves {
            let piece = board.get_piece(mv.from);
            let piece_name = match piece_type(piece) {
                1 => "Pawn",
                2 => "Knight", 
                3 => "Bishop",
                4 => "Rook",
                5 => "Queen",
                6 => "King",
                _ => "Unknown"
            };
            let move_str = format!("{}{}", 
                square_to_algebraic(mv.from), 
                square_to_algebraic(mv.to));
            move_strings.push(format!("{}: {}", move_str, piece_name));
        }
        
        // Sort alphabetically for easier comparison with Stockfish
        move_strings.sort();
        for move_str in move_strings {
            debug_info.push(format!("  {}", move_str));
        }
        
        debug_info
    }

    // Add this function to your existing debug.rs impl block:
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
        
        // Sort moves alphabetically like Stockfish (THIS IS THE KEY CHANGE)
        move_results.sort_by(|a, b| a.0.cmp(&b.0));
        
        // Format output exactly like Stockfish - NO extra text, just moves and counts
        for (move_str, nodes) in move_results {
            debug_info.push(format!("{}: {}", move_str, nodes));
        }
        
        debug_info.push(format!("\nNodes searched: {}", total_nodes));
        
        debug_info
    }

    /// Debug a specific position after making moves - Stockfish format
    pub fn debug_position_stockfish_format(&self, setup_moves: &[&str], depth: u32) -> Vec<String> {
        let mut temp_board = self.clone();
        
        // Make setup moves
        for move_notation in setup_moves {
            let from_file = (move_notation.chars().nth(0).unwrap() as u8) - b'a';
            let from_rank = (move_notation.chars().nth(1).unwrap() as u8) - b'1';
            let to_file = (move_notation.chars().nth(2).unwrap() as u8) - b'a';
            let to_rank = (move_notation.chars().nth(3).unwrap() as u8) - b'1';
            
            let from_square = Square::new(from_file, from_rank);
            let to_square = Square::new(to_file, to_rank);
            let target_move = Move::new(from_square, to_square);
            
            temp_board.try_make_move(target_move).expect("Failed to make move");
        }
        
        // Generate perft divide output
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
                
                // FIX: Include promotion notation
                let move_str = if let Some(promotion) = mv.promotion {
                    let promotion_char = match promotion {
                        QUEEN => 'q',
                        ROOK => 'r', 
                        BISHOP => 'b',
                        KNIGHT => 'n',
                        _ => '?',
                    };
                    format!("{}{}{}", 
                        square_to_algebraic(mv.from), 
                        square_to_algebraic(mv.to),
                        promotion_char)
                } else {
                    format!("{}{}", 
                        square_to_algebraic(mv.from), 
                        square_to_algebraic(mv.to))
                };
                
                move_results.push((move_str, nodes));
                total_nodes += nodes;
            }
        }
        
        // Sort alphabetically for Stockfish comparison
        move_results.sort_by(|a, b| a.0.cmp(&b.0));
        
        let mut debug_info = Vec::new();
        for (move_str, nodes) in move_results {
            debug_info.push(format!("{}: {}", move_str, nodes));
        }
        debug_info.push(format!("\nNodes searched: {}", total_nodes));
        
        debug_info
    }


    /// Debug the h4g4 position specifically
    pub fn debug_h4g4_position(&self) -> Vec<String> {
        let mut debug_info = Vec::new();
        
        // Set up: Position 3 + a5a6 + h4g4
        let setup_moves = [
            ("a5a6", Square::new(0, 4), Square::new(0, 5)), // Ka5-a6
            ("h4g4", Square::new(7, 3), Square::new(6, 3)), // Kh4-g4
        ];
        
        // Start from Position 3
        let mut temp_board = Board::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap();
        
        for (notation, from, to) in setup_moves {
            let mv = Move::new(from, to);
            if let Ok(_) = temp_board.try_make_move(mv) {
                debug_info.push(format!("Made move: {}", notation));
            } else {
                debug_info.push(format!("Failed to make move: {}", notation));
                return debug_info;
            }
        }
        
        debug_info.push("Position after a5a6 h4g4:".to_string());
        debug_info.push(format!("Current turn: {}", 
                               if temp_board.current_turn == 0 { "Black" } else { "White" }));
        
        // Get all legal moves (should match Stockfish exactly)
        let moves = temp_board.get_all_legal_moves();
        debug_info.push(format!("Total legal moves: {} (should match Stockfish)", moves.len()));
        
        // Show all moves for analysis
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

    pub fn debug_threat_detection(&self) -> Vec<String> {
        let mut debug_info = Vec::new();
        
        // Set up the specific position: after a5a6 h4g4 b4f4
        let mut temp_board = Board::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap();
        temp_board.try_make_move(Move::new(Square::new(0, 4), Square::new(0, 5))).unwrap(); // a5a6
        temp_board.try_make_move(Move::new(Square::new(7, 3), Square::new(6, 3))).unwrap(); // h4g4
        temp_board.try_make_move(Move::new(Square::new(1, 3), Square::new(5, 3))).unwrap(); // b4f4
        
        let h4_square = Square::new(7, 3); // h4
        let f4_square = Square::new(5, 3); // f4 (where white rook is)
        
        debug_info.push("Testing threat detection:".to_string());
        debug_info.push(format!("White rook on f4: {}", piece_type(temp_board.get_piece(f4_square)) == ROOK));
        debug_info.push(format!("Rook attacks h4: {}", temp_board.piece_attacks_square(f4_square, h4_square)));
        debug_info.push(format!("h4 under threat by White: {}", temp_board.is_under_threat(h4_square, WHITE)));
        
        debug_info
    }

    /// Debug the exact board state after moves
    pub fn debug_board_state_f4_position(&self) -> Vec<String> {
        let mut debug_info = Vec::new();
        let mut temp_board = Board::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap();
        
        // Make the moves: a5a6, h4g4, b4f4
        let moves = [
            ("a5a6", Square::new(0, 4), Square::new(0, 5)), // Ka5-a6
            ("h4g4", Square::new(7, 3), Square::new(6, 3)), // Kh4-g4
            ("b4f4", Square::new(1, 3), Square::new(5, 3)), // Rb4-f4
        ];
        
        for (notation, from, to) in moves {
            let mv = Move::new(from, to);
            if let Ok(_) = temp_board.try_make_move(mv) {
                debug_info.push(format!("✅ Made move: {}", notation));
            } else {
                debug_info.push(format!("❌ Failed to make move: {}", notation));
                return debug_info;
            }
        }
        
        debug_info.push("\n🏁 Final board state:".to_string());
        
        // Check specific squares
        let critical_squares = [
            ("f4", Square::new(5, 3)), // Should have White rook
            ("g4", Square::new(6, 3)), // Should have Black king
            ("h4", Square::new(7, 3)), // Should be empty
            ("a6", Square::new(0, 5)), // Should have White king
        ];
        
        for (name, square) in critical_squares {
            let piece = temp_board.get_piece(square);
            let piece_type_val = if is_empty(piece) { 0 } else { piece_type(piece) };
            let piece_color_val = if is_empty(piece) { 0 } else { piece_color(piece) };
            
            debug_info.push(format!("  {}: piece={}, type={}, color={}, empty={}", 
                                name, piece, piece_type_val, piece_color_val, is_empty(piece)));
        }
        
        debug_info
    }
    
}

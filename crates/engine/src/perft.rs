use crate::{Board, Move, Square, square_to_algebraic};
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct PerftResult {
    pub nodes: u64,
    pub captures: u64,
    pub en_passant: u64,
    pub castles: u64,
    pub promotions: u64,
    pub checks: u64,
    pub checkmates: u64,
    pub time_ms: u128,
}

impl PerftResult {
    pub fn new() -> Self {
        Self {
            nodes: 0,
            captures: 0,
            en_passant: 0,
            castles: 0,
            promotions: 0,
            checks: 0,
            checkmates: 0,
            time_ms: 0,
        }
    }
    
    pub fn nodes_per_second(&self) -> u64 {
        if self.time_ms == 0 {
            return 0;
        }
        (self.nodes * 1000) / (self.time_ms as u64)
    }
}

#[derive(Debug)]
pub struct PerftTestCase {
    pub name: &'static str,
    pub fen: &'static str,
    pub expected_results: &'static [(u32, u64)], // (depth, expected_nodes)
}

// Standard perft test positions
pub const PERFT_POSITIONS: &[PerftTestCase] = &[
    PerftTestCase {
        name: "Starting Position",
        fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        expected_results: &[
            (1, 20),
            (2, 400),
            (3, 8_902),
            (4, 197_281),
            (5, 4_865_609),
            (6, 119_060_324),
        ],
    },
    PerftTestCase {
        name: "Kiwipete",
        fen: "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        expected_results: &[
            (1, 48),
            (2, 2_039),
            (3, 97_862),
            (4, 4_085_603),
            (5, 193_690_690),
        ],
    },
    PerftTestCase {
        name: "Position 3",
        fen: "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
        expected_results: &[
            (1, 14),
            (2, 191),
            (3, 2_812),
            (4, 43_238),
            (5, 674_624),
            (6, 11_030_083),
        ],
    },
    PerftTestCase {
        name: "Position 4",
        fen: "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
        expected_results: &[
            (1, 6),
            (2, 264),
            (3, 9_467),
            (4, 422_333),
            (5, 15_833_292),
        ],
    },
    PerftTestCase {
        name: "Position 5",
        fen: "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
        expected_results: &[
            (1, 44),
            (2, 1_486),
            (3, 62_379),
            (4, 2_103_487),
            (5, 89_941_194),
        ],
    },
    PerftTestCase {
        name: "Position 6",
        fen: "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
        expected_results: &[
            (1, 46),
            (2, 2_079),
            (3, 89_890),
            (4, 3_894_594),
            (5, 164_075_551),
        ],
    },
];

/// Main perft function - counts all legal moves to a given depth
pub fn perft(board: &mut Board, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }
    
    let mut nodes = 0;
    let moves = board.get_all_legal_moves();
    
    for mv in moves {
        if let Ok(_) = board.try_make_move(mv) {
            nodes += perft(board, depth - 1);
            board.undo_move().expect("Failed to undo move");
        }
    }
    
    nodes
}

/// Detailed perft that tracks different move types
pub fn perft_detailed(board: &mut Board, depth: u32) -> PerftResult {
    let start_time = Instant::now();
    let mut result = PerftResult::new();
    
    if depth == 0 {
        result.nodes = 1;
        result.time_ms = start_time.elapsed().as_millis();
        return result;
    }
    
    let moves = board.get_all_legal_moves();
    
    for mv in moves {
        if let Ok(game_move) = board.try_make_move(mv) {
            let sub_result = perft_detailed(board, depth - 1);
            
            result.nodes += sub_result.nodes;
            
            // Count move types at depth 1
            if depth == 1 {
                if game_move.captured_piece != 0 {
                    result.captures += 1;
                }
                if game_move.is_en_passant {
                    result.en_passant += 1;
                }
                if game_move.is_castling {
                    result.castles += 1;
                }
                // TODO: Add promotion counting when implemented
                // TODO: Add check/checkmate counting when implemented
            } else {
                result.captures += sub_result.captures;
                result.en_passant += sub_result.en_passant;
                result.castles += sub_result.castles;
                result.promotions += sub_result.promotions;
                result.checks += sub_result.checks;
                result.checkmates += sub_result.checkmates;
            }
            
            board.undo_move().expect("Failed to undo move");
        }
    }
    
    result.time_ms = start_time.elapsed().as_millis();
    result
}

/// Divide perft - shows per-move breakdown for debugging
pub fn perft_divide(board: &mut Board, depth: u32) -> Vec<(Move, u64)> {
    let mut results = Vec::new();
    let moves = board.get_all_legal_moves();
    
    for mv in moves {
        if let Ok(_) = board.try_make_move(mv) {
            let nodes = if depth > 1 {
                perft(board, depth - 1)
            } else {
                1
            };
            results.push((mv, nodes));
            board.undo_move().expect("Failed to undo move");
        }
    }
    
    results.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by node count descending
    results
}

/// Run a single perft test
pub fn run_perft_test(board: &mut Board, depth: u32, expected: u64) -> bool {
    println!("Running perft depth {} (expected: {})", depth, expected);
    
    let start_time = Instant::now();
    let nodes = perft(board, depth);
    let elapsed = start_time.elapsed();
    
    let success = nodes == expected;
    let status = if success { "✅ PASS" } else { "❌ FAIL" };
    
    println!("{} - Depth {}: {} nodes in {:.3}s ({:.0} nodes/sec)", 
             status, depth, nodes, elapsed.as_secs_f64(), 
             nodes as f64 / elapsed.as_secs_f64());
    
    if !success {
        println!("Expected: {}, Got: {}", expected, nodes);
    }
    
    success
}

/// Run all perft tests for a position
pub fn run_position_tests(test_case: &PerftTestCase, max_depth: Option<u32>) -> bool {
    println!("\n🏁 Testing: {}", test_case.name);
    println!("FEN: {}", test_case.fen);
    
    let mut board = Board::from_fen(test_case.fen).expect("Invalid FEN");
    let mut all_passed = true;
    
    for &(depth, expected) in test_case.expected_results {
        if let Some(max) = max_depth {
            if depth > max {
                break;
            }
        }
        
        let passed = run_perft_test(&mut board, depth, expected);
        all_passed &= passed;
        
        if !passed {
            println!("🔍 Running divide to debug:");
            let divide_results = perft_divide(&mut board, depth);
            for (mv, nodes) in divide_results.iter().take(10) {
                println!("  {:?}: {}", mv, nodes);
            }
            break; // Stop on first failure for debugging
        }
    }
    
    all_passed
}

/// Run all standard perft tests
pub fn run_all_tests(max_depth: Option<u32>) {
    println!("🚀 Starting Perft Tests");
    println!("Max depth: {:?}", max_depth.unwrap_or(99));
    
    let mut passed = 0;
    let mut total = 0;
    
    for test_case in PERFT_POSITIONS {
        total += 1;
        if run_position_tests(test_case, max_depth) {
            passed += 1;
        }
    }
    
    println!("\n📊 Results: {}/{} positions passed", passed, total);
    
    if passed == total {
        println!("🎉 All tests passed! Your move generation is correct.");
    } else {
        println!("❌ Some tests failed. Check move generation implementation.");
    }
}

/// Debug perft differences
pub fn debug_perft_starting_position() {
    let board = Board::new();
    
    println!("\n🔍 Debugging starting position depth 4 moves:");
    let debug_results = board.debug_move_count_difference(4);
    
    let total_nodes: u64 = debug_results.iter().map(|(_, nodes, _)| nodes).sum();
    println!("Total nodes calculated: {}", total_nodes);
    println!("Expected: 197281");
    println!("Difference: {}", total_nodes as i64 - 197281);
    
    println!("\nTop 10 moves by node count:");
    for (i, (move_str, nodes, _)) in debug_results.iter().take(10).enumerate() {
        println!("{}. {}: {} nodes", i + 1, move_str, nodes);
    }
    
    // Compare with known good values for some key moves
    println!("\n🎯 Checking specific moves:");
    for (move_str, nodes, _) in &debug_results {
        match move_str.as_str() {
            "e2e4" => println!("e2-e4: {} (should be ~9744)", nodes),
            "d2d4" => println!("d2-d4: {} (should be ~9748)", nodes),
            "g1f3" => println!("Ng1-f3: {} (should be ~8885)", nodes),
            "b1c3" => println!("Nb1-c3: {} (should be ~9755)", nodes),
            _ => {}
        }
    }
}

/// Debug specific issues in starting position
pub fn debug_starting_position_issues() {
    let board = Board::new();
    
    println!("\n🔍 Debugging Starting Position Issues:");
    
    // Debug pawn moves specifically
    println!("\n📍 White Pawn Moves:");
    let pawn_moves = board.debug_pawn_moves();
    for (square, moves) in pawn_moves {
        let square_name = square_to_algebraic(square);
        println!("  {}: {} moves", square_name, moves.len());
        for mv in moves {
            println!("    -> {}", square_to_algebraic(mv));
        }
    }
    
    // Debug specific problematic squares
    println!("\n🎯 Debugging e2 pawn:");
    let e2_square = Square::new(4, 1); // e2
    let e2_debug = board.debug_square_moves(e2_square);
    for info in e2_debug {
        println!("  {}", info);
    }
    
    println!("\n🎯 Debugging d2 pawn:");
    let d2_square = Square::new(3, 1); // d2
    let d2_debug = board.debug_square_moves(d2_square);
    for info in d2_debug {
        println!("  {}", info);
    }
}

/// Debug the specific e2-e4 issue
pub fn debug_e2e4_issue() {
    let mut board = Board::new();
    
    println!("\n🔍 Debugging e2-e4 sequence:");
    let debug_info = board.debug_e2e4_sequence();
    for info in debug_info {
        println!("{}", info);
    }
    
    // Test a few moves deep
    let mut board = Board::new();
    let e2 = Square::new(4, 1);
    let e4 = Square::new(4, 3);
    let mv = Move::new(e2, e4);
    
    if board.try_make_move(mv).is_ok() {
        println!("\n🎯 After e2-e4, running perft depth 3:");
        let nodes = perft(&mut board, 3);
        println!("Nodes: {} (this contributes to the e2-e4 total)", nodes);
        
        // Expected value for comparison
        println!("Expected: ~650-700 nodes for this position");
    }
}

/// Debug FEN loading issues
pub fn debug_fen_loading() {
    Board::debug_fen_loading();
}

/// Test undo functionality
pub fn test_undo_functionality() {
    let mut board = Board::new();
    
    println!("\n🔧 Testing Undo Functionality:");
    let test_results = board.test_undo_functionality();
    for result in test_results {
        println!("{}", result);
    }
}

/// Debug which Black move is causing the explosion
pub fn debug_black_explosion() {
    let mut board = Board::new();
    
    println!("\n🔍 Debugging Black Move Explosion:");
    let debug_info = board.debug_black_moves_after_e2e4();
    for info in debug_info {
        println!("{}", info);
    }
}

/// Debug the d7-d5 illegal move issue
pub fn debug_d7d5_illegal_moves() {
    let mut board = Board::new();
    
    println!("\n🎯 Debugging d7-d5 Illegal Moves:");
    let debug_info = board.debug_d7d5_issue();
    for info in debug_info {
        println!("{}", info);
    }
}

/// Debug illegal sliding piece moves
pub fn debug_illegal_sliding_moves() {
    let mut board = Board::new();
    
    println!("\n🔍 Debugging Illegal Sliding Moves:");
    let debug_info = board.debug_illegal_sliding_moves();
    for info in debug_info {
        println!("{}", info);
    }
}

/// Debug the exact board state issue
pub fn debug_board_state_issue() {
    let mut board = Board::new();
    
    println!("\n🔍 Debugging Exact Board State Issue:");
    let debug_info = board.debug_board_state_after_moves();
    for info in debug_info {
        println!("{}", info);
    }
}

/// Print perft divide in Stockfish format for easy comparison
pub fn print_perft_divide_formatted() {
    let board = Board::new();
    
    println!("Your Engine Perft Divide 4:");
    let formatted_output = board.debug_perft_divide_formatted(4);
    for line in formatted_output {
        println!("{}", line);
    }
}

/// Debug specific problematic moves at deeper levels
pub fn debug_problematic_moves() {
    let board = Board::new();
    
    // Test the most problematic moves
    let problematic_moves = ["g1f3", "e2e4", "e2e3"];
    
    for move_notation in problematic_moves {
        println!("\n{}", "=".repeat(50));
        let debug_info = board.debug_move_deeper(move_notation, 3);
        for info in debug_info {
            println!("{}", info);
        }
    }
}

/// Debug the g1f3 d7d5 sequence exactly like Stockfish
pub fn debug_g1f3_d7d5_sequence() {
    let board = Board::new();
    
    println!("Your Engine Output for position after g1f3 d7d5:");
    let debug_info = board.debug_move_sequence_perft(&["g1f3", "d7d5"], 2);
    for info in debug_info {
        println!("{}", info);
    }
}

/// Debug the specific f3e5 bug
pub fn debug_f3e5_bug() {
    let board = Board::new();
    
    println!("\n🔍 Debugging f3e5 position (27 vs 26 nodes):");
    let debug_info = board.debug_f3e5_position();
    for info in debug_info {
        println!("{}", info);
    }
}

/// Debug the king safety validation bug
pub fn debug_king_safety_bug() {
    let board = Board::new();
    
    println!("\n🔍 Debugging King Safety Validation:");
    let debug_info = board.debug_king_safety_validation();
    for info in debug_info {
        println!("{}", info);
    }
}

/// Debug the specific king move validation logic
pub fn debug_king_move_validation_detailed() {
    let board = Board::new();
    
    println!("\n🔍 Detailed King Move Validation Debug:");
    let debug_info = board.debug_king_move_validation();
    for info in debug_info {
        println!("{}", info);
    }
}

/// Debug position 3 with a5a6 move in Stockfish format
pub fn debug_position3_a5a6_stockfish() {
    let board = Board::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap();
    
    println!("Your Engine Output for position after Position 3 + a5a6:");
    let debug_info = board.debug_position_stockfish_format(&["a5a6"], 3);
    for info in debug_info {
        println!("{}", info);
    }
}

/// Debug the h4g4 position specifically
pub fn debug_h4g4_position() {
    let board = Board::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap();
    
    println!("Your Engine Output for position after a5a6 h4g4:");
    let debug_info = board.debug_position_stockfish_format(&["a5a6", "h4g4"], 2);
    for info in debug_info {
        println!("{}", info);
    }
}

/// Debug Position 5 missing moves
pub fn debug_position5_missing_moves() {
    let board = Board::new();
    
    println!("\n🔍 Debugging Position 5 Missing Moves:");
    let debug_info = board.debug_position5_missing_moves();
    for info in debug_info {
        println!("{}", info);
    }
}

/// Debug Position 4 analysis
pub fn debug_position4_analysis() {
    let board = Board::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1").unwrap();
    
    println!("Your Engine Output for Position 4:");
    let debug_info = board.debug_position_stockfish_format(&[], 2);
    for info in debug_info {
        println!("{}", info);
    }
}

/// Debug Kiwipete analysis
pub fn debug_kiwipete_analysis() {
    let board = Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1").unwrap();
    
    println!("Your Engine Output for Kiwipete:");
    let debug_info = board.debug_position_stockfish_format(&[], 3);
    for info in debug_info {
        println!("{}", info);
    }
}

/// Debug the specific b4f4 move that has +1 extra node
pub fn debug_b4f4_discrepancy() {
    let board = Board::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap();
    
    println!("Debugging b4f4 move discrepancy:");
    println!("Your Engine vs Stockfish: b4f4 should have 3 nodes, you have 4");
    
    // Drill down into b4f4 specifically
    let debug_info = board.debug_position_stockfish_format(&["a5a6", "h4g4", "b4f4"], 1);
    for info in debug_info {
        println!("{}", info);
    }
}

/// Debug Position 5 missing moves (most important - only depth 1)
pub fn debug_position5_targeted() {
    let board = Board::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8").unwrap();
    
    println!("Position 5 Analysis - Missing 3 moves:");
    println!("Your Engine: 41 moves, Stockfish: 44 moves");
    
    let debug_info = board.debug_position_stockfish_format(&[], 1);
    for info in debug_info {
        println!("{}", info);
    }
}

/// Debug Position 4 missing moves 
pub fn debug_position4_targeted() {
    let board = Board::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1").unwrap();
    
    println!("Position 4 Analysis - Missing 36 moves:");
    println!("Your Engine: 228 moves, Stockfish: 264 moves");
    
    let debug_info = board.debug_position_stockfish_format(&[], 2);
    for info in debug_info {
        println!("{}", info);
    }
}

/// Target the exact position causing +1 node in Position 3
pub fn debug_position3_exact_bug() {
    println!("=== EXACT BUG LOCATION ===");
    println!("Position: 8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1 moves a5a6 h4g4 b4f4");
    println!("Your Engine generates 4 moves, Stockfish generates 3 moves");
    println!("This +1 extra move is the source of your +30 node discrepancy at depth 4");
    
    debug_b4f4_discrepancy();
}

/// Debug threat detection for the h4 square issue
pub fn debug_threat_detection() {
    let board = Board::new();
    
    println!("\n🔍 Debugging Threat Detection for h4 Square:");
    let debug_info = board.debug_threat_detection();
    for info in debug_info {
        println!("{}", info);
    }
}

/// Debug board state verification
pub fn debug_board_state_verification() {
    let board = Board::new();
    
    println!("🔍 Board State Verification:");
    let debug_info = board.debug_board_state_f4_position();
    for info in debug_info {
        println!("{}", info);
    }
}

/// Test promotion move undo functionality
pub fn test_promotion_undo_cycles() {
    let mut board = Board::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8").unwrap();
    
    println!("Testing promotion undo cycles...");
    
    // Get all moves
    let moves = board.get_all_legal_moves();
    let initial_move_count = moves.len();
    println!("Initial moves: {}", initial_move_count);
    
    // Test promotion moves specifically
    for mv in moves {
        if mv.is_promotion() {
            println!("Testing promotion move: {}{} -> {:?}", 
                     square_to_algebraic(mv.from), 
                     square_to_algebraic(mv.to), 
                     mv.promotion);
            
            // Make the move
            if let Ok(_game_move) = board.try_make_move(mv) {
                let _after_move_count = board.get_all_legal_moves().len();
                
                // Undo the move
                if let Ok(_) = board.undo_move() {
                    let after_undo_count = board.get_all_legal_moves().len();
                    
                    if initial_move_count != after_undo_count {
                        println!("❌ UNDO BUG: {} != {} for promotion move", 
                                initial_move_count, after_undo_count);
                    } else {
                        println!("✅ Undo working for this promotion");
                    }
                } else {
                    println!("❌ Failed to undo promotion move");
                }
            }
        }
    }
}


/// Test only Position 4 to verify the fix
pub fn run_perft_position4_only() {
    let board = Board::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1").unwrap();
    
    println!("🏁 Testing Position 4 ONLY:");
    
    // Test depth 3 where the issue was
    let start_time = std::time::Instant::now();
    let mut test_board = board.clone();
    let nodes = crate::perft::perft(&mut test_board, 3);
    let duration = start_time.elapsed();
    
    println!("Depth 3: {} nodes in {:.3}s", nodes, duration.as_secs_f64());
    
    if nodes == 9467 {
        println!("✅ POSITION 4 FIXED! Node count matches Stockfish.");
    } else if nodes == 9631 {
        println!("❌ POSITION 4 STILL BROKEN: Still generating +164 extra nodes.");
        println!("The fix wasn't applied correctly or there's another issue.");
    } else {
        println!("❓ UNEXPECTED RESULT: Got {} nodes (expected 9467)", nodes);
    }
}



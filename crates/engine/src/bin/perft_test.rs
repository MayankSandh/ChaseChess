use engine::perft::*;
use engine::Board;

fn main() {
    println!("🎯 DEBUGGING POSITION 4: c4c5 e8e7 h6f5 KNIGHT MOVE (+46 EXTRA NODES)");
    println!("Expected: 5, Your engine: 51 (+46 extra)");
    println!("{}", "=".repeat(60));
    
    // Debug the specific move sequence: c4c5 e8e7 h6f5 at depth 1
    let board = Board::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1").unwrap();
    
    println!("Debugging c4c5 e8e7 h6f5 move sequence specifically:");
    let debug_info = board.debug_position_stockfish_format(&["c4c5", "e8e7", "h6f5"], 1);
    for info in debug_info {
        println!("{}", info);
    }
    
    println!("\n📋 STOCKFISH COMPARISON:");
    println!("position fen r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1 moves c4c5 e8e7 h6f5");
    println!("go perft 1");
    
    println!("\n🎯 EXPECTED TOTAL: 5 nodes");
    println!("Your engine will likely show 51 (+46 extra)");
    
    println!("\n🤔 ANALYSIS: This is the PIN VALIDATION bug!");
    println!("After h6f5 (knight move), your engine generates 46 illegal moves");
    println!("that should be filtered out because they leave the king in check.");
    println!("The h6 knight is PINNED and cannot legally move to f5!");
}

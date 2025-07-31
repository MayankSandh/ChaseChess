use engine::perft::*;
use engine::Board;

// Add this to your perft_test.rs main function:
fn main() {
    // First run the quick test above, then:
    
    println!("\n🎯 FULL POSITION 3 PERFT TEST");
    println!("{}", "=".repeat(60));
    
    let mut board = Board::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap();
    
    let start_time = std::time::Instant::now();
    let nodes = engine::perft::perft(&mut board, 5);
    let duration = start_time.elapsed();
    
    println!("Position 3 Depth 5 Result:");
    println!("Your engine: {} nodes", nodes);
    println!("Expected:    674624 nodes");
    println!("Time: {:.3}s", duration.as_secs_f64());
    
    if nodes == 674624 {
        println!("🎉 POSITION 3 COMPLETELY FIXED!");
        println!("✅ En passant during check resolution is working perfectly");
    } else if nodes == 674543 {
        println!("❌ STILL BROKEN: You still have the -81 node deficit");
        println!("❌ The fix wasn't applied correctly");
    } else {
        println!("❓ UNEXPECTED: Got {} nodes", nodes);
        println!("❓ Check if there are other issues");
    }
}


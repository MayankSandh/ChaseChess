use engine::perft::*;
use engine::{Board, Move, Square};

fn main() {
    println!("🎯 DEBUGGING e1c1 KING MOVE - 1 Extra Node");
    println!("Your engine: 43 nodes, Stockfish: 42 nodes (+1 extra)");
    println!("{}", "=".repeat(60));
    
    // Set up position after a2a4 e8c8 e1c1
    let mut board = Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1").unwrap();
    let a2a4_move = Move::new(Square::new(0, 1), Square::new(0, 3)); // a2 to a4
    board.try_make_move(a2a4_move).expect("a2a4 should be legal");
    let e8c8_move = Move::new(Square::new(4, 7), Square::new(2, 7)); // e8 to c8 (castling)
    board.try_make_move(e8c8_move).expect("e8c8 should be legal");
    let e1c1_move = Move::new(Square::new(4, 0), Square::new(2, 0)); // e1 to c1
    board.try_make_move(e1c1_move).expect("e1c1 should be legal");
    
    println!("Testing position after a2a4 e8c8 e1c1 at depth 1:");
    let debug_info = board.debug_position_stockfish_format(&[], 1);
    for info in debug_info {
        println!("{}", info);
    }
    
    println!("\n📋 STOCKFISH COMPARISON COMMAND:");
    println!("position fen r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1 moves a2a4 e8c8 e1c1");
    println!("go perft 1");
    
    println!("\n🎯 EXPECTED RESULT:");
    println!("Expected: 42 moves, Your engine shows: 43 moves");
    println!("Find which 1 move your engine allows but Stockfish doesn't");
}

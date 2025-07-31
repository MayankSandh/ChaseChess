use engine::Board;
use ai::SearchEngine;

fn main() {
    println!("🤖 Testing Basic AI Implementation");
    println!("{}", "=".repeat(50));
    
    // Test position: starting position
    let mut board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    let mut search_engine = SearchEngine::new();
    
    println!("Initial position (White to move):");
    let result = search_engine.search(&mut board, 4);
    
    match result.best_move {
        Some(mv) => {
            println!("✅ Best move found: ({},{}) -> ({},{})", 
                mv.from.file(), mv.from.rank(), 
                mv.to.file(), mv.to.rank());
            println!("📊 Evaluation: {}", result.evaluation);
            println!("🔍 Nodes searched: {}", result.nodes_searched);
            println!("📏 Search depth: {}", result.depth);
        }
        None => {
            println!("❌ No move found!");
        }
    }
    
    // Test with a simple tactical position
    println!("\n🎯 Testing tactical position:");
    let mut tactical_board = Board::from_fen("r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/3P1N2/PPP2PPP/RNBQK2R w KQkq - 4 4").unwrap();
    
    let tactical_result = search_engine.search(&mut tactical_board, 4);
    
    match tactical_result.best_move {
        Some(mv) => {
            println!("✅ Tactical move: ({},{}) -> ({},{})", 
                mv.from.file(), mv.from.rank(), 
                mv.to.file(), mv.to.rank());
            println!("📊 Evaluation: {}", tactical_result.evaluation);
            println!("🔍 Nodes searched: {}", tactical_result.nodes_searched);
        }
        None => {
            println!("❌ No tactical move found!");
        }
    }
}

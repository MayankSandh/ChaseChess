use engine::{Board, Move, Square};

fn main() {
    println!("🎯 DEBUGGING find_king and find_checking_pieces METHODS");
    println!("Testing the exact failing move sequence: c4c5 e8e7 h6f5");
    println!("{}", "=".repeat(60));
    
    let mut board = Board::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1").unwrap();
    
    println!("🔍 INITIAL POSITION (WHITE to move):");
    let white_king = board.find_king(8).unwrap(); // WHITE = 8
    let black_king = board.find_king(16).unwrap(); // BLACK = 16
    println!("WHITE king at {:?}, BLACK king at {:?}", white_king, black_king);
    
    println!("\n🔍 AFTER MOVE c4c5 (BLACK to move):");
    let move1 = Move::new(Square::new(2, 3), Square::new(2, 4)); // c4c5
    if board.try_make_move(move1).is_ok() {
        let black_king = board.find_king(16).unwrap(); // BLACK king
        let white_checks_black = board.find_checking_pieces(black_king, 16); // WHITE pieces checking BLACK king
        println!("BLACK king at {:?}, {} WHITE pieces checking it", black_king, white_checks_black.len());
        
        println!("\n🔍 AFTER MOVE e8e7 (WHITE to move):");
        let move2 = Move::new(Square::new(4, 7), Square::new(4, 6)); // e8e7
        println!("🔍 DEBUG: Attempting move from e8({}) to e7({})", 
                 Square::new(4, 7).0, Square::new(4, 6).0);
        
        match board.try_make_move(move2) {
            Ok(_) => {
                println!("✅ Move e8e7 successful");
                
                println!("\n🔍 TESTING h6f5 KNIGHT MOVE:");
                let move3 = Move::new(Square::new(7, 5), Square::new(5, 4)); // h6f5
                if board.try_make_move(move3).is_ok() {
                    let black_king = board.find_king(16).unwrap();
                    let white_checks_black = board.find_checking_pieces(black_king, 16);
                    
                    if white_checks_black.is_empty() {
                        println!("❌ BUG: BLACK king not in check after h6f5 - this explains +46 extra nodes!");
                    } else {
                        println!("✅ BLACK king correctly in check after h6f5");
                    }
                } else {
                    println!("✅ h6f5 correctly rejected as illegal");
                }
            }
            Err(e) => {
                println!("❌ ERROR: Move e8e7 failed: {}", e);
                println!("This suggests coordinate conversion or move validation bug");
            }
        }
    }
}

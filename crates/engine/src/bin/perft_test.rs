use engine::perft::*;
use engine::Board;

// Add this to your perft_test.rs main function:
fn main() {
    engine::bitboard::initialize_engine();
    run_all_tests(Some(6))
}


use engine::perft::{debug_e2e4_issue, run_all_tests, debug_perft_starting_position, debug_starting_position_issues};

fn main() {
    println!("Perft Test Suite for Chess Engine");
    run_all_tests(Some(4));
}


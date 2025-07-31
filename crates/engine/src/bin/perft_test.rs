use engine::perft::*;
use engine::Board;

fn main() {
    run_position_tests(    &PerftTestCase {
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
    }, Some(5));

}

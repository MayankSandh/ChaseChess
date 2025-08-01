mod app;

use app::ChessApp;

fn main() -> eframe::Result<()> {

    engine::bitboard::initialize_engine();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 800.0])
            .with_title("Chess Engine"),
        ..Default::default()
    };
    
    eframe::run_native(
        "Chess Engine",
        options,
        Box::new(|_cc| Ok(Box::new(ChessApp::new()))), // Added Ok() wrapper
    )
}

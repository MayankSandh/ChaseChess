use egui::{Color32, Rect, Sense, Vec2}; 
use engine::{Board, Move, Square, piece_type, piece_color, is_empty}; // Removed unused is_white, is_black
use engine::{PAWN, KNIGHT, BISHOP, ROOK, QUEEN, KING, WHITE, BLACK};

#[derive(Default)]
pub struct ChessApp {
    board: Board,
    selected_square: Option<Square>,
    legal_moves: Vec<Square>,
}

impl ChessApp {
    pub fn new() -> Self {
        Self {
            board: Board::new(),
            selected_square: None,
            legal_moves: Vec::new(),
        }
    }
}

impl eframe::App for ChessApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Chess Engine");
            
            let available_size = ui.available_size();
            let board_size = available_size.x.min(available_size.y) - 20.0;
            let square_size = board_size / 8.0;
            
            let board_rect = Rect::from_min_size(
                ui.cursor().min,
                Vec2::splat(board_size),
            );
            
            let response = ui.allocate_rect(board_rect, Sense::click());
            
            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    let relative_pos = pos - board_rect.min;
                    if let Some(clicked_square) = Square::from_coords(
                        relative_pos.x,
                        relative_pos.y,
                        square_size,
                    ) {
                        self.handle_square_click(clicked_square);
                    }
                }
            }
            
            // Draw the board
            self.draw_board(ui, board_rect, square_size);
        });
    }
}

impl ChessApp {
    fn handle_square_click(&mut self, clicked_square: Square) {
        if let Some(selected) = self.selected_square {
            if selected == clicked_square {
                // Deselect
                self.selected_square = None;
                self.legal_moves.clear();
            } else if self.legal_moves.contains(&clicked_square) {
                // Make move
                let mv = Move::new(selected, clicked_square);
                if self.board.try_make_move(mv).is_ok() {
                    self.selected_square = None;
                    self.legal_moves.clear();
                } // If invalid, do nothing (can add error handling here later)
            } else if !is_empty(self.board.get_piece(clicked_square)) {
                // Select new piece
                self.selected_square = Some(clicked_square);
                self.legal_moves = self.board.get_legal_moves(clicked_square);
            } else {
                // Click on empty square, deselect
                self.selected_square = None;
                self.legal_moves.clear();
            }
        } else if !is_empty(self.board.get_piece(clicked_square)) {
            // Select piece
            self.selected_square = Some(clicked_square);
            self.legal_moves = self.board.get_legal_moves(clicked_square);
        }
    }
    
    fn draw_board(&self, ui: &mut egui::Ui, board_rect: Rect, square_size: f32) {
        let painter = ui.painter();
        
        // Draw squares
        for rank in 0..8 {
            for file in 0..8 {
                let square = Square::new(file, rank);
                let is_light = (file + rank) % 2 == 0;
                
                let square_rect = Rect::from_min_size(
                    board_rect.min + Vec2::new(file as f32 * square_size, (7 - rank) as f32 * square_size),
                    Vec2::splat(square_size),
                );
                
                // Base square color
                let base_color = if is_light {
                    Color32::from_rgb(240, 217, 181)
                } else {
                    Color32::from_rgb(181, 136, 99)
                };
                
                // Highlight selected square
                let square_color = if Some(square) == self.selected_square {
                    Color32::from_rgb(255, 255, 0) // Yellow highlight
                } else {
                    base_color
                };
                
                painter.rect_filled(square_rect, 0.0, square_color);
                
                // Draw legal move indicators
                if self.legal_moves.contains(&square) {
                    let center = square_rect.center();
                    
                    if !is_empty(self.board.get_piece(square)) {
                        // Capture square - draw donut
                        let outer_radius = square_size * 0.4;
                        let inner_radius = square_size * 0.25;
                        painter.circle_filled(center, outer_radius, Color32::from_rgba_premultiplied(128, 128, 128, 179)); // 70% opacity
                        painter.circle_filled(center, inner_radius, square_color);
                    } else {
                        // Empty square - draw dot
                        let radius = square_size * 0.15;
                        painter.circle_filled(center, radius, Color32::from_rgba_premultiplied(128, 128, 128, 179)); // 70% opacity
                    }
                }
                
                // Draw piece
                let piece = self.board.get_piece(square);
                if !is_empty(piece) {
                    self.draw_piece(painter, piece, square_rect);
                }
            }
        }
        
        // Draw board border
        painter.rect_stroke(board_rect, 0.0, egui::Stroke::new(2.0, Color32::BLACK));
    }
    
    fn draw_piece(&self, painter: &egui::Painter, piece: u8, square_rect: Rect) {
        let center = square_rect.center();
        let size = square_rect.size() * 0.8;
        
        let piece_char = match (piece_type(piece), piece_color(piece)) {
            (KING, WHITE) => "♔",
            (QUEEN, WHITE) => "♕",
            (ROOK, WHITE) => "♖",
            (BISHOP, WHITE) => "♗",
            (KNIGHT, WHITE) => "♘",
            (PAWN, WHITE) => "♙",
            (KING, BLACK) => "♚",
            (QUEEN, BLACK) => "♛",
            (ROOK, BLACK) => "♜",
            (BISHOP, BLACK) => "♝",
            (KNIGHT, BLACK) => "♞",
            (PAWN, BLACK) => "♟",
            _ => "?", // Should never happen
        };
        
        painter.text(
            center,
            egui::Align2::CENTER_CENTER,
            piece_char,
            egui::FontId::proportional(size.x),
            Color32::BLACK,
        );
    }
}

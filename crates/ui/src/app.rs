use egui::{Color32, Rect, Sense, Vec2}; 
use engine::{Board, Move, Square, piece_type, piece_color, is_empty}; // Removed unused is_white, is_black
use engine::{PAWN, KNIGHT, BISHOP, ROOK, QUEEN, KING, WHITE, BLACK};
use ai::SearchEngine;
use std::time::Instant;

#[derive(Default)]
pub struct ChessApp {
    board: Board,
    selected_square: Option<Square>,
    legal_moves: Vec<Square>,
    ai_engine: SearchEngine,
    ai_enabled: bool,
    is_ai_thinking: bool,
    ai_move_scheduled: Option<Instant>,  // Changed from ai_needs_to_move
    last_ai_move: Option<Move>,
    game_over: bool,
}

impl ChessApp {

    pub fn new() -> Self {
        Self {
            board: Board::new(),
            selected_square: None,
            legal_moves: Vec::new(),
            ai_engine: SearchEngine::new(),
            ai_enabled: true,
            is_ai_thinking: false,
            ai_move_scheduled: None,  // Changed from ai_needs_to_move: false
            last_ai_move: None,
            game_over: false,
        }
    }


    fn is_ai_last_move_square(&self, square: Square) -> bool {
        if let Some(last_move) = self.last_ai_move {
            square == last_move.from || square == last_move.to
        } else {
            false
        }
    }    
}

impl eframe::App for ChessApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Chess Engine - Human vs AI");
            
            // Status display
            ui.horizontal(|ui| {
                let current_player = if self.board.current_turn == WHITE { "White" } else { "Black" };
                let status = if self.game_over {
                    "Game Over".to_string()
                } else {
                    format!("{}'s turn", current_player)
                };
                
                ui.label(format!("Status: {}", status));
                
                if !self.game_over && self.board.is_in_check() {
                    ui.colored_label(Color32::RED, "CHECK!");
                }
                
                // Add some spacing
                ui.add_space(20.0);
                
                // AI status beside the New Game button
                if self.is_ai_thinking {
                    ui.spinner();
                    ui.label("AI is calculating...");
                } else if self.ai_move_scheduled.is_some() {
                    ui.spinner();
                    ui.label("AI will move shortly...");
                }
                
                // Push New Game button to the right
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("New Game").clicked() {
                        self.board = Board::new();
                        self.selected_square = None;
                        self.legal_moves.clear();
                        self.last_ai_move = None;
                        self.game_over = false;
                        self.is_ai_thinking = false;
                        self.ai_move_scheduled = None;
                    }
                });
            });
            
            let available_size = ui.available_size();
            let board_size = (available_size.x.min(available_size.y) - 80.0).max(400.0);
            let square_size = board_size / 8.0;

            let board_rect = Rect::from_min_size(
                ui.cursor().min,
                Vec2::splat(board_size),
            );

            let response = ui.allocate_rect(board_rect, Sense::click());

            // Handle clicks
            if response.clicked() && !self.is_ai_thinking && self.ai_move_scheduled.is_none() &&
               (self.board.current_turn == WHITE || !self.ai_enabled) {
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
        
        // Handle AI move timing outside the panel
        if let Some(scheduled_time) = self.ai_move_scheduled {
            let elapsed = scheduled_time.elapsed();
            if elapsed.as_millis() >= 500 { // 500ms delay
                println!("TIMING: AI delay complete, triggering move");
                self.ai_move_scheduled = None;
                if !self.game_over && !self.is_ai_thinking {
                    self.trigger_ai_move();
                }
            } else {
                // Use egui's proper timing system to schedule the next check
                let remaining = 500 - elapsed.as_millis() as u64;
                ctx.request_repaint_after(std::time::Duration::from_millis(remaining.min(50)));
            }
        }
    }
}




impl ChessApp {

    fn handle_square_click(&mut self, clicked_square: Square) {
        // Don't allow moves if game is over or AI is processing
        if self.game_over || self.is_ai_thinking || self.ai_move_scheduled.is_some() {
            return;
        }
        
        // Only allow human moves on White's turn
        if self.board.current_turn == BLACK && self.ai_enabled {
            return;
        }
        
        if let Some(selected) = self.selected_square {
            if selected == clicked_square {
                self.selected_square = None;
                self.legal_moves.clear();
            } else if self.legal_moves.contains(&clicked_square) {
                let mv = Move::new(selected, clicked_square);
                if self.board.try_make_move(mv).is_ok() {
                    println!("USER: Move applied successfully");
                    self.selected_square = None;
                    self.legal_moves.clear();
                    
                    // Schedule AI move with proper timing
                    if self.board.current_turn == BLACK && self.ai_enabled {
                        self.ai_move_scheduled = Some(Instant::now());
                        println!("USER: AI move scheduled for 0.5 seconds from now");
                    }
                    
                    self.check_game_over();
                }
            } else if !is_empty(self.board.get_piece(clicked_square)) && 
                     piece_color(self.board.get_piece(clicked_square)) == self.board.current_turn {
                self.selected_square = Some(clicked_square);
                self.legal_moves = self.board.get_legal_moves(clicked_square);
            } else {
                self.selected_square = None;
                self.legal_moves.clear();
            }
        } else if !is_empty(self.board.get_piece(clicked_square)) && 
                 piece_color(self.board.get_piece(clicked_square)) == self.board.current_turn {
            self.selected_square = Some(clicked_square);
            self.legal_moves = self.board.get_legal_moves(clicked_square);
        }
    } 
    
    fn trigger_ai_move(&mut self) {
        if self.game_over {
            println!("AI: Move cancelled - game is over");
            return;
        }
        
        println!("AI: Triggering move for BLACK");
        println!("AI: Board current_turn = {}", 
            if self.board.current_turn == 0 { "BLACK" } else { "WHITE" });
        
        self.is_ai_thinking = true;
        
        println!("AI: Starting search with depth 3");
        let result = self.ai_engine.search(&mut self.board, 4);
        println!("AI: Search returned");
        
        if let Some(ai_move) = result.best_move {
            println!("AI: Applying best move: ({},{}) to ({},{})", 
                ai_move.from.file(), ai_move.from.rank(),
                ai_move.to.file(), ai_move.to.rank());
            
            if self.board.try_make_move(ai_move).is_ok() {
                self.last_ai_move = Some(ai_move);
                println!("AI: Move applied successfully - Eval: {} | Nodes: {}", 
                    result.evaluation, result.nodes_searched);
            } else {
                println!("AI: ERROR - Failed to apply AI move");
            }
        } else {
            println!("AI: No valid move found");
        }
        
        self.is_ai_thinking = false;
        self.check_game_over();
        println!("AI: Move sequence complete");
    }
    
    
    fn check_game_over(&mut self) {
        let legal_moves = self.board.get_all_legal_moves();
        if legal_moves.is_empty() {
            self.game_over = true;
            if self.board.is_in_check() {
                println!("Checkmate! {} wins!", 
                    if self.board.current_turn == WHITE { "Black" } else { "White" });
            } else {
                println!("Stalemate! It's a draw.");
            }
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

                // Determine square color with highlights
                let square_color = if Some(square) == self.selected_square {
                    Color32::from_rgb(255, 255, 0) // Yellow highlight for selected
                } else if self.is_ai_last_move_square(square) {
                    // ✅ NEW: Highlight AI's last move in blue
                    if is_light {
                        Color32::from_rgb(173, 216, 230) // Light blue
                    } else {
                        Color32::from_rgb(100, 149, 237) // Darker blue
                    }
                } else {
                    base_color
                };

                painter.rect_filled(square_rect, 0.0, square_color);

                // Draw legal move indicators (same as before)
                if self.legal_moves.contains(&square) {
                    let center = square_rect.center();
                    if !is_empty(self.board.get_piece(square)) {
                        // Capture square - draw donut
                        let outer_radius = square_size * 0.4;
                        let inner_radius = square_size * 0.25;
                        painter.circle_filled(center, outer_radius, Color32::from_rgba_premultiplied(128, 128, 128, 179));
                        painter.circle_filled(center, inner_radius, square_color);
                    } else {
                        // Empty square - draw dot
                        let radius = square_size * 0.15;
                        painter.circle_filled(center, radius, Color32::from_rgba_premultiplied(128, 128, 128, 179));
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

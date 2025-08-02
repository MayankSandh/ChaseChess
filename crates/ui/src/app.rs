use egui::{Color32, Rect, Sense, Vec2}; 
use engine::{Board, Move, Square, piece_type, piece_color, is_empty}; // Removed unused is_white, is_black
use engine::{PAWN, KNIGHT, BISHOP, ROOK, QUEEN, KING, WHITE, BLACK};
use ai::SearchEngine;
use std::time::Instant;

// Add these lines after your imports
const FILES: [char; 8] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
const RANKS: [char; 8] = ['1', '2', '3', '4', '5', '6', '7', '8'];


#[derive(Default)]
pub struct ChessApp {
    board: Board,
    selected_square: Option<Square>,
    legal_moves: Vec<Square>,
    ai_engine: SearchEngine,
    ai_enabled: bool,
    is_ai_thinking: bool,
    ai_move_scheduled: Option<Instant>,  
    last_ai_move: Option<Move>,
    game_over: bool,
    promotion_pending: Option<PendingPromotion>,
    show_promotion_dialog: bool,
    move_history: Vec<Move>,     
    redo_history: Vec<Move>,    
}

#[derive(Clone, Debug)]
struct PendingPromotion {
    from_square: Square,
    to_square: Square,
    player_color: u8,
}

impl ChessApp {

    pub fn new() -> Self {
        engine::bitboard::initialize_engine();
        Self {
            board: Board::new(),
            selected_square: None,
            legal_moves: Vec::new(),
            ai_engine: SearchEngine::new(),
            ai_enabled: true,
            is_ai_thinking: false,
            ai_move_scheduled: None,  
            last_ai_move: None,
            game_over: false,
            promotion_pending: None,
            show_promotion_dialog: false,
            move_history: Vec::new(),
            redo_history: Vec::new(),
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
                        self.promotion_pending = None;
                        self.show_promotion_dialog = false;
                        self.move_history.clear();
                        self.redo_history.clear();
                    }

                    // ADD: Redo button
                    if ui.add_enabled(self.can_redo(), egui::Button::new("Redo")).clicked() {
                        self.redo_move();
                    }
                    
                    // ADD: Undo button  
                    if ui.add_enabled(self.can_undo(), egui::Button::new("Undo")).clicked() {
                        self.undo_move();
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
            if elapsed.as_millis() >= 10 { // 500ms delay
                self.ai_move_scheduled = None;
                if !self.game_over && !self.is_ai_thinking {
                    self.trigger_ai_move();
                }
            } else {
                // Use egui's proper timing system to schedule the next check
                let remaining = 10 - elapsed.as_millis() as u64;
                ctx.request_repaint_after(std::time::Duration::from_millis(remaining.min(10)));
            }
        }
        self.show_promotion_dialog(ctx);
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
                let piece = self.board.get_piece(selected);
                let piece_type_val = piece_type(piece);
                let piece_color_val = piece_color(piece);
                
                // Check if this is a pawn promotion move
                if piece_type_val == PAWN {
                    let promotion_rank = if piece_color_val == WHITE { 7 } else { 0 };
                    
                    if clicked_square.rank() == promotion_rank {
                        // This is a promotion move - show dialog instead of executing immediately
                        self.promotion_pending = Some(PendingPromotion {
                            from_square: selected,
                            to_square: clicked_square,
                            player_color: piece_color_val,
                        });
                        self.show_promotion_dialog = true;
                        self.selected_square = None;
                        self.legal_moves.clear();
                        return; // Don't execute the move yet
                    }
                }
                let mv = Move::new(selected, clicked_square);
                if self.board.try_make_move(mv).is_ok() {

                    self.move_history.push(mv);
                    self.redo_history.clear();

                    self.selected_square = None;
                    self.legal_moves.clear();
                    
                    // Schedule AI move with proper timing
                    if self.board.current_turn == BLACK && self.ai_enabled {
                        self.ai_move_scheduled = Some(Instant::now());
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
            return;
        }
        
        
        self.is_ai_thinking = true;
        
        let result = self.ai_engine.search(&mut self.board, 4);
        
        if let Some(ai_move) = result.best_move {
            
            if self.board.try_make_move(ai_move).is_ok() {
                self.move_history.push(ai_move);
                self.redo_history.clear();

                self.last_ai_move = Some(ai_move);
            } 
        } 
        
        self.is_ai_thinking = false;
        self.check_game_over();
    }
    
    
    fn check_game_over(&mut self) {
        let legal_moves = self.board.get_all_legal_moves();
        if legal_moves.is_empty() {
            self.game_over = true;
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

        // Draw coordinate labels  
        let label_font = egui::FontId::new(16.0, egui::FontFamily::Monospace);
        let label_color = Color32::DARK_GRAY;


        // Draw file labels (a-h) at the bottom
        for (file, &file_char) in FILES.iter().enumerate() {
            let x = board_rect.min.x + (file as f32 * square_size) + (square_size / 2.0);
            let y = board_rect.max.y + 8.0;
            
            painter.text(
                egui::Pos2::new(x, y),
                egui::Align2::CENTER_TOP,
                file_char.to_string(),
                label_font.clone(),
                label_color,
            );
        }

        // Draw rank labels (1-8) at the right edge  
        for (rank_index, &rank_char) in RANKS.iter().enumerate() {
            let x = board_rect.max.x + 8.0;
            let y = board_rect.min.y + ((7 - rank_index) as f32 * square_size) + (square_size / 2.0);
            
            painter.text(
                egui::Pos2::new(x, y),
                egui::Align2::LEFT_CENTER,
                rank_char.to_string(),
                label_font.clone(),
                label_color,
            );
        }

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

    fn show_promotion_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_promotion_dialog {
            return;
        }
        
        let Some(pending) = &self.promotion_pending else {
            return;
        };
        
        // EXTRACT the values we need BEFORE entering the closure
        let from_square = pending.from_square;
        let to_square = pending.to_square;
        let player_color = pending.player_color;
        
        // Create modal dialog
        egui::Window::new("Choose Promotion")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);
                    
                    ui.heading("Choose a piece to promote to:");
                    ui.add_space(20.0);
                    
                    // Choose piece symbols based on player color
                    let (queen_symbol, rook_symbol, bishop_symbol, knight_symbol) = 
                        if player_color == WHITE {
                            ("♕", "♖", "♗", "♘")
                        } else {
                            ("♛", "♜", "♝", "♞")
                        };
                    
                    // Create promotion piece buttons
                    ui.horizontal(|ui| {
                        ui.add_space(10.0);
                        
                        // Queen button
                        if ui.add_sized([80.0, 80.0], 
                            egui::Button::new(format!("{}\nQueen", queen_symbol))).clicked() {
                            self.execute_promotion_move(from_square, to_square, QUEEN);
                        }
                        
                        ui.add_space(5.0);
                        
                        // Rook button
                        if ui.add_sized([80.0, 80.0], 
                            egui::Button::new(format!("{}\nRook", rook_symbol))).clicked() {
                            self.execute_promotion_move(from_square, to_square, ROOK);
                        }
                        
                        ui.add_space(5.0);
                        
                        // Bishop button
                        if ui.add_sized([80.0, 80.0], 
                            egui::Button::new(format!("{}\nBishop", bishop_symbol))).clicked() {
                            self.execute_promotion_move(from_square, to_square, BISHOP);
                        }
                        
                        ui.add_space(5.0);
                        
                        // Knight button
                        if ui.add_sized([80.0, 80.0], 
                            egui::Button::new(format!("{}\nKnight", knight_symbol))).clicked() {
                            self.execute_promotion_move(from_square, to_square, KNIGHT);
                        }
                        
                        ui.add_space(10.0);
                    });
                    
                    ui.add_space(15.0);
                    
                    // Cancel button (optional)
                    if ui.button("Cancel").clicked() {
                        self.promotion_pending = None;
                        self.show_promotion_dialog = false;
                    }
                    
                    ui.add_space(10.0);
                });
            });
    }
    

    fn execute_promotion_move(&mut self, from: Square, to: Square, promotion_piece: u8) {
        let promotion_move = Move::new_promotion(from, to, promotion_piece);
        
        if self.board.try_make_move(promotion_move).is_ok() {

            self.move_history.push(promotion_move);
            self.redo_history.clear();
            // Schedule AI move if it's now AI's turn
            if self.board.current_turn == BLACK && self.ai_enabled {
                self.ai_move_scheduled = Some(Instant::now());
            }
            self.check_game_over();
        }
        
        // Clear promotion state
        self.promotion_pending = None;
        self.show_promotion_dialog = false;
    }
    
    fn can_undo(&self) -> bool {
        !self.move_history.is_empty() && !self.is_ai_thinking && self.ai_move_scheduled.is_none()
    }
    
    fn can_redo(&self) -> bool {
        !self.redo_history.is_empty() && !self.is_ai_thinking && self.ai_move_scheduled.is_none()
    }
    
    fn undo_move(&mut self) {
        if let Some(last_move) = self.move_history.pop() {
            // Use your existing undo function
            if self.board.undo_move().is_ok() {
                // Move the undone move to redo stack
                self.redo_history.push(last_move);
                
                // Clear UI state
                self.selected_square = None;
                self.legal_moves.clear();
                self.last_ai_move = None;
                self.game_over = false;
            } else {
                // If undo failed, restore the move to history
                self.move_history.push(last_move);
            }
        }
    }
    
    fn redo_move(&mut self) {
        if let Some(redo_move) = self.redo_history.pop() {
            if self.board.try_make_move(redo_move).is_ok() {
                // Move back to move history
                self.move_history.push(redo_move);
                
                // Clear UI state
                self.selected_square = None;
                self.legal_moves.clear();
                self.game_over = false;
            } else {
                // If redo failed, restore the move to redo stack
                self.redo_history.push(redo_move);
            }
        }
    }
    
}

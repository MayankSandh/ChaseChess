use std::fs::{self, File};
use std::io::Write;
use std::time::Instant;
use chrono::{Local};
use crate::types::*;

pub struct ChessLogger {
    pub debug_enabled: bool,
    log_buffer: String,
    game_start_time: Instant,
    move_number: u16,
}

impl ChessLogger {
    pub fn new(debug_enabled: bool) -> Self {
        let mut logger = Self {
            debug_enabled,
            log_buffer: String::with_capacity(10000), // Pre-allocate for performance
            game_start_time: Instant::now(),
            move_number: 1,
        };
        
        if debug_enabled {
            logger.log_game_start();
        }
        
        logger
    }
    
    fn log_game_start(&mut self) {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        self.append_log(&format!("=== CHESS ENGINE DEBUG LOG ==="));
        self.append_log(&format!("Game started at: {}", timestamp));
        self.append_log(&format!("Debug logging enabled: {}", self.debug_enabled));
        self.append_log(&format!(""));
    }
    
    fn append_log(&mut self, message: &str) {
        match self.debug_enabled {
            true => {
                let elapsed = self.game_start_time.elapsed();
                self.log_buffer.push_str(&format!("[{:06.3}s] {}\n", elapsed.as_secs_f64(), message));
            }
            false => {} // No overhead when disabled
        }
    }
    
    // Human move logging
    pub fn log_human_move(&mut self, from: Square, to: Square, piece_type: u8, captured: bool) {
        match self.debug_enabled {
            true => {
                let move_notation = format!("{}{}{}",
                    square_to_notation(from),
                    if captured { "x" } else { "-" },
                    square_to_notation(to)
                );
                
                self.append_log(&format!("Move {}: HUMAN played {} ({})",
                    self.move_number,
                    move_notation,
                    piece_type_to_string(piece_type)
                ));
                self.move_number += 1;
            }
            false => {}
        }
    }
    
    // AI thinking phase
    pub fn log_ai_thinking_start(&mut self, position_fen: &str, game_phase: &str) {
        match self.debug_enabled {
            true => {
                self.append_log(&format!("Move {}: AI THINKING started", self.move_number));
                self.append_log(&format!("  Position: {}", position_fen));
                self.append_log(&format!("  Game Phase: {}", game_phase));
            }
            false => {}
        }
    }
    
    // AI move result
    pub fn log_ai_move_result(&mut self, from: Square, to: Square, piece_type: u8, 
                             captured: bool, thinking_time: f64, evaluation: i32, depth: u32) {
        match self.debug_enabled {
            true => {
                let move_notation = format!("{}{}{}",
                    square_to_notation(from),
                    if captured { "x" } else { "-" },
                    square_to_notation(to)
                );
                
                self.append_log(&format!("Move {}: AI played {} ({})",
                    self.move_number,
                    move_notation,
                    piece_type_to_string(piece_type)
                ));
                self.append_log(&format!("  Thinking time: {:.3}s", thinking_time));
                self.append_log(&format!("  Evaluation: {} centipawns", evaluation));
                self.append_log(&format!("  Search depth: {}", depth));
                self.append_log(&format!(""));
                self.move_number += 1;
            }
            false => {}
        }
    }
    
    // Alpha-beta search logging
    pub fn log_alpha_beta_update(&mut self, depth: u32, alpha: i32, beta: i32, 
                                 best_move: Option<(Square, Square)>, evaluation: i32) {
        match self.debug_enabled {
            true => {
                match best_move {
                    Some((from, to)) => {
                        self.append_log(&format!("    [D{}] α={}, β={}, Best: {}->{}, Eval: {}",
                            depth, alpha, beta,
                            square_to_notation(from), square_to_notation(to),
                            evaluation
                        ));
                    }
                    None => {
                        self.append_log(&format!("    [D{}] α={}, β={}, Eval: {}",
                            depth, alpha, beta, evaluation
                        ));
                    }
                }
            }
            false => {}
        }
    }
    
    // Position evaluation details
    pub fn log_position_evaluation(&mut self, material_score: i32, positional_score: i32, 
                                  total_score: i32, game_phase_value: f32) {
        match self.debug_enabled {
            true => {
                self.append_log(&format!("  Evaluation breakdown:"));
                self.append_log(&format!("    Material: {} cp", material_score));
                self.append_log(&format!("    Positional: {} cp", positional_score));
                self.append_log(&format!("    Total: {} cp", total_score));
                self.append_log(&format!("    Game Phase: {:.2}", game_phase_value));
            }
            false => {}
        }
    }
    
    // Game ending
    pub fn log_game_end(&mut self, result: &str, reason: &str) {
        match self.debug_enabled {
            true => {
                let total_time = self.game_start_time.elapsed();
                self.append_log(&format!(""));
                self.append_log(&format!("=== GAME ENDED ==="));
                self.append_log(&format!("Result: {}", result));
                self.append_log(&format!("Reason: {}", reason));
                self.append_log(&format!("Total moves: {}", self.move_number - 1));
                self.append_log(&format!("Total game time: {:.1}s", total_time.as_secs_f64()));
                self.append_log(&format!("=== END LOG ==="));
            }
            false => {}
        }
    }
    
    // Save log to file
    pub fn save_log_to_file(&self) -> Result<String, std::io::Error> {
        match self.debug_enabled {
            true => {
                // Create logs directory
                fs::create_dir_all("logs")?;
                
                // Generate filename with MM/DD/YYYY format
                let now = Local::now();
                let filename = format!("chess_log_{}.txt", 
                    now.format("%m-%d-%Y_%H-%M-%S"));
                let filepath = format!("logs/{}", filename);
                
                // Write log buffer to file
                let mut file = File::create(&filepath)?;
                file.write_all(self.log_buffer.as_bytes())?;
                file.flush()?;
                
                Ok(filepath)
            }
            false => Ok("Logging disabled".to_string())
        }
    }
    
    // Clear log buffer for new game
    pub fn reset_for_new_game(&mut self) {
        match self.debug_enabled {
            true => {
                self.log_buffer.clear();
                self.game_start_time = Instant::now();
                self.move_number = 1;
                self.log_game_start();
            }
            false => {}
        }
    }
}

// Helper functions
fn square_to_notation(square: Square) -> String {
    let file = (b'a' + square.file()) as char;
    let rank = (b'1' + square.rank()) as char;
    format!("{}{}", file, rank)
}

fn piece_type_to_string(piece_type: u8) -> &'static str {
    match piece_type {
        PAWN => "Pawn",
        KNIGHT => "Knight", 
        BISHOP => "Bishop",
        ROOK => "Rook",
        QUEEN => "Queen",
        KING => "King",
        _ => "Unknown"
    }
}

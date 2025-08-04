use std::fs::{self, File};
use std::io::Write;
use std::time::Instant;
use crate::{Move, piece_type, piece_color, is_empty};
use crate::{PAWN, KNIGHT, BISHOP, ROOK, QUEEN, KING, WHITE, BLACK};

#[derive(Debug)]
pub struct ChessLogger {
    pub log_buffer: String,
    pub advanced_logging: bool,
    game_start_time: Instant,
    move_count: u32,
    current_search_depth: u32,
    indent_level: usize,
    last_game_phase: Option<u8>,
    pub in_evaluation: bool,
}

impl ChessLogger {
    pub fn new() -> Self {
        let mut logger = Self {
            log_buffer: String::with_capacity(2 * 1024 * 1024), // 2MB buffer
            advanced_logging: false,
            game_start_time: Instant::now(),
            move_count: 0,
            current_search_depth: 0,
            indent_level: 0,
            last_game_phase: None,
            in_evaluation: false,
        };
        
        logger.log("🎯 === Chess Engine Game Log Started ===");
        logger.log(&format!("📅 Date: {}", chrono::Local::now().format("%m/%d/%Y %H:%M:%S")));
        logger
    }

    pub fn should_log_advanced(&self) -> bool {
        self.advanced_logging
    }

    pub fn enable_advanced_logging(&mut self) {
        self.advanced_logging = true;
        self.log("🔬 Advanced logging enabled - Deep engine analysis active");
    }

    pub fn disable_advanced_logging(&mut self) {
        self.advanced_logging = false;
        self.log("📊 Advanced logging disabled - Basic mode active");
    }

    // pub fn log(&mut self, message: &str) {
    //     let timestamp = self.game_start_time.elapsed().as_millis();
    //     self.log_buffer.push_str(&format!("[{:>6}ms] {}\n", timestamp, message));
    // }

    // pub fn log_with_indent(&mut self, message: &str) {
    //     let indent = "  ".repeat(self.indent_level);
    //     let timestamp = self.game_start_time.elapsed().as_millis();
    //     self.log_buffer.push_str(&format!("[{:>6}ms] {}{}\n", timestamp, indent, message));
    // }

    pub fn log(&mut self, message: &str) {
        // ❌ Current (with timestamp):
        // let timestamp = self.game_start_time.elapsed().as_millis();
        // self.log_buffer.push_str(&format!("[{:>6}ms] {}\n", timestamp, message));
        
        // ✅ Fix (without timestamp):
        self.log_buffer.push_str(&format!("{}\n", message));
    }

    pub fn log_with_indent(&mut self, message: &str) {
        let indent = "  ".repeat(self.indent_level);
        // ❌ Current (with timestamp):
        // let timestamp = self.game_start_time.elapsed().as_millis();
        // self.log_buffer.push_str(&format!("[{:>6}ms] {}{}\n", timestamp, indent, message));
        
        // ✅ Fix (without timestamp):
        self.log_buffer.push_str(&format!("{}{}\n", indent, message));
    }

    pub fn increase_indent(&mut self) { self.indent_level += 1; }
    pub fn decrease_indent(&mut self) { 
        if self.indent_level > 0 { self.indent_level -= 1; }
    }

    // 🎯 MOVE LOGGING
    pub fn log_human_move(&mut self, mv: Move, time_ms: u64) {
        self.move_count += 1;
        self.log(&format!(
            "{}. {} (Human move - {}ms)", 
            self.move_count, 
            move_to_string(mv), 
            time_ms
        ));
    }

    pub fn log_ai_move(&mut self, mv: Move, time_ms: u64, eval: i32) {
        self.move_count += 1;
        self.log(&format!(
            "{}. {} (AI move - {}ms) Eval: {} {}", 
            self.move_count, 
            move_to_string(mv), 
            time_ms,
            eval,
            if eval > 0 { "📈" } else { "📉" }
        ));
    }

    pub fn log_undo(&mut self, mv: Move) {
        self.log(&format!("↩️ UNDO: {}", move_to_string(mv)));
    }

    pub fn log_redo(&mut self, mv: Move) {
        self.log(&format!("↪️ REDO: {}", move_to_string(mv)));
    }

    // 🎯 ALPHA-BETA TRACKING
    pub fn log_search_start(&mut self, depth: u32, move_count: usize) {
        if self.should_log_advanced() {
            self.current_search_depth = depth;
            self.log_with_indent(&format!(
                "🔍 Search depth {} | Analyzing {} moves", depth, move_count
            ));
            self.increase_indent();
        }
    }

    pub fn log_alpha_change(&mut self, old_alpha: i32, new_alpha: i32, mv: Move) {
        if self.should_log_advanced() {
            self.log_with_indent(&format!(
                "📈 ALPHA IMPROVED! {} → {} (+{}) after move {} 🎯",
                old_alpha, new_alpha, new_alpha - old_alpha, move_to_string(mv)
            ));
        }
    }

    pub fn log_beta_cutoff(&mut self, beta: i32, score: i32, mv: Move) {
        if self.should_log_advanced() {
            self.log_with_indent(&format!(
                "✂️ BETA CUTOFF! Score {} ≥ Beta {} - Pruning after move {} 🚫",
                score, beta, move_to_string(mv)
            ));
        }
    }

    pub fn log_move_analysis(&mut self, mv: Move, move_num: usize, total_moves: usize, score: i32) {
        if self.should_log_advanced() {
            self.log_with_indent(&format!(
                "🔄 Move {}/{}: {} → Score: {} {}",
                move_num, total_moves, move_to_string(mv), score,
                if score > 0 { "📊" } else { "📉" }
            ));
        }
    }

    pub fn log_search_complete(&mut self, best_move: Option<Move>, best_score: i32, nodes: u64) {
        if self.should_log_advanced() {
            self.decrease_indent();
            match best_move {
                Some(mv) => self.log_with_indent(&format!(
                    "✅ Search complete: {} (Score: {}) | Nodes: {} 🎯",
                    move_to_string(mv), best_score, nodes
                )),
                None => self.log_with_indent("❌ No legal moves found"),
            }
        }
    }

    // 🎯 TRANSPOSITION TABLE
    pub fn log_tt_hit(&mut self, cached_depth: i32, current_depth: i32, cached_score: i32, best_move: Option<Move>) {
        if self.should_log_advanced() {
            self.log_with_indent(&format!(
                "⚡ TT HIT! Cached depth: {} (current: {}) | Score: {} {} | Move: {}",
                cached_depth, current_depth, cached_score,
                if cached_depth >= current_depth { "✅" } else { "❌" },
                best_move.map(move_to_string).unwrap_or("None".to_string())
            ));
        }
    }

    // 🎯 GAME PHASE TRANSITIONS
    pub fn check_and_log_phase_transition(&mut self, current_phase: u8, trigger: &str) {
        if self.should_log_advanced() {
            if let Some(old_phase) = self.last_game_phase {
                if old_phase != current_phase {
                    self.log_game_phase_transition(old_phase, current_phase, trigger);
                }
            }
            self.last_game_phase = Some(current_phase);
        }
    }

    fn log_game_phase_transition(&mut self, old_phase: u8, new_phase: u8, trigger: &str) {
        let old_pct = (old_phase as f32 / 255.0 * 100.0) as u8;
        let new_pct = (new_phase as f32 / 255.0 * 100.0) as u8;
        
        self.log("🔄 GAME PHASE TRANSITION!");
        self.log(&format!("├─ Previous: {}/255 ({}% endgame)", old_phase, old_pct));
        self.log(&format!("├─ Current: {}/255 ({}% endgame)", new_phase, new_pct));
        self.log(&format!("├─ Trigger: {}", trigger));
        self.log("└─ PST Tables: Switching evaluation priorities 👑");
    }

    pub fn log_game_aborted(&mut self, reason: &str) {
        self.log(&format!("🛑 Game finished abruptly - {}", reason));
    }

    pub fn save_to_file(&mut self, reason: &str) -> Result<String, String> {
        // Create logs directory
        if let Err(e) = fs::create_dir_all("logs") {
            return Err(format!("Failed to create logs directory: {}", e));
        }

        // Generate filename with current date/time
        let now = chrono::Local::now();
        let filename = format!("logs/{}.txt", now.format("%m_%d_%Y_%H_%M_%S"));

        // Add final log entry
        self.log(&format!("💾 Game ended: {} - Saving log", reason));
        
        // Write to file
        match File::create(&filename) {
            Ok(mut file) => {
                if let Err(e) = file.write_all(self.log_buffer.as_bytes()) {
                    return Err(format!("Failed to write log file: {}", e));
                }
                Ok(filename)
            }
            Err(e) => Err(format!("Failed to create log file: {}", e)),
        }
    }

    // 🎯 EVALUATION BREAKDOWN - Safe versions with recursion guard
    pub fn log_evaluation_breakdown_safe(&mut self, 
        material_white: i32, material_black: i32,
        pst_total: i32, game_phase: u8, 
        total_eval: i32
    ) {
        if self.should_log_advanced() && !self.in_evaluation {
            self.in_evaluation = true;
            
            self.log_with_indent("🔬 DEEP EVALUATION BREAKDOWN:");
            self.increase_indent();
            
            self.log_with_indent(&format!("📊 Game Phase: {}/255 ({}% endgame)", 
                game_phase, (game_phase as f32 / 255.0 * 100.0) as u8));
            
            self.log_with_indent(&format!("💎 Material: White={}, Black={} ({:+})", 
                material_white, material_black, material_white - material_black));
            
            self.log_with_indent(&format!("📍 PST Total: {:+}", pst_total));
            
            self.log_with_indent(&format!("🏆 Final Eval: {:+}", total_eval));
            
            self.decrease_indent();
            
            self.in_evaluation = false;
        }
    }

    pub fn log_raw_pst_breakdown_safe(&mut self, board: &crate::Board) {
        if self.should_log_advanced() && !self.in_evaluation {
            self.in_evaluation = true;
            
            // Simplified logging to avoid deep recursion
            self.log_with_indent("🔬 RAW PST BREAKDOWN:");
            self.increase_indent();
            
            // White pieces
            self.log_with_indent("├─ White Pieces:");
            self.increase_indent();
            for rank in 0..8 {
                for file in 0..8 {
                    let square = crate::Square::new(file, rank);
                    let piece = board.get_piece(square);
                    if !is_empty(piece) && piece_color(piece) == WHITE {
                        let piece_char = match piece_type(piece) {
                            KING => "♔", QUEEN => "♕", ROOK => "♖", 
                            BISHOP => "♗", KNIGHT => "♘", PAWN => "♙",
                            _ => "?",
                        };
                        let square_name = format!("{}{}", (b'a' + file) as char, rank + 1);
                        self.log_with_indent(&format!("├─ {} {}: Square[{}]", 
                            piece_char, square_name, square.0));
                    }
                }
            }
            self.decrease_indent();
            
            // Black pieces  
            self.log_with_indent("└─ Black Pieces:");
            self.increase_indent();
            for rank in 0..8 {
                for file in 0..8 {
                    let square = crate::Square::new(file, rank);
                    let piece = board.get_piece(square);
                    if !is_empty(piece) && piece_color(piece) == BLACK {
                        let piece_char = match piece_type(piece) {
                            KING => "♚", QUEEN => "♛", ROOK => "♜", 
                            BISHOP => "♝", KNIGHT => "♞", PAWN => "♟",
                            _ => "?",
                        };
                        let square_name = format!("{}{}", (b'a' + file) as char, rank + 1);
                        self.log_with_indent(&format!("├─ {} {}: Square[{}]", 
                            piece_char, square_name, square.0));
                    }
                }
            }
            self.decrease_indent();
            self.decrease_indent();
            
            self.in_evaluation = false;
        }
    }

    // Original methods kept for backward compatibility
    pub fn log_evaluation_breakdown(&mut self, 
        material_white: i32, material_black: i32,
        pst_total: i32, game_phase: u8, 
        total_eval: i32
    ) {
        self.log_evaluation_breakdown_safe(material_white, material_black, pst_total, game_phase, total_eval);
    }

    pub fn log_raw_pst_breakdown(&mut self, board: &crate::Board) {
        self.log_raw_pst_breakdown_safe(board);
    }

    pub fn log_endgame_pattern(&mut self, pattern: &str, details: &str) {
        if self.should_log_advanced() {
            self.log_with_indent(&format!("🏁 Endgame Pattern: {} detected", pattern));
            if !details.is_empty() {
                self.log_with_indent(&format!("├─ {}", details));
            }
        }
    }

    // 🎯 ROOT SEARCH LOGGING
    pub fn log_search_root_start(&mut self, depth: u32, moves: &[Move]) {
        if self.should_log_advanced() {
            self.log(&format!("🔍 === ROOT SEARCH DEPTH {} ===", depth));
            self.log(&format!("📋 Legal moves ({}):", moves.len()));
            self.increase_indent();
            for (i, &mv) in moves.iter().enumerate() {
                self.log_with_indent(&format!("{:2}. {}", i + 1, move_to_string(mv)));
            }
            self.decrease_indent();
        }
    }

    pub fn log_move_ordering_result(&mut self, ordered_moves: &[Move], board: &crate::Board) {
        if self.should_log_advanced() {
            self.log("🎯 Move ordering result:");
            self.increase_indent();
            for (i, &mv) in ordered_moves.iter().enumerate() {
                let from_piece = board.get_piece(mv.from);
                let to_piece = board.get_piece(mv.to);
                let capture = if !is_empty(to_piece) { " [CAPTURE]" } else { "" };
                self.log_with_indent(&format!("{:2}. {}{}", i + 1, move_to_string(mv), capture));
            }
            self.decrease_indent();
        }
    }

    pub fn log_root_move_start(&mut self, mv: Move, move_num: usize, total_moves: usize, alpha: i32, beta: i32) {
        if self.should_log_advanced() {
            self.log(&format!("🔄 === ANALYZING ROOT MOVE {}/{}: {} ===", move_num, total_moves, move_to_string(mv)));
            self.log(&format!("   Alpha: {}, Beta: {}", alpha, beta));
            self.increase_indent();
        }
    }

    pub fn log_root_move_result(&mut self, mv: Move, score: i32, alpha: i32, beta: i32) {
        if self.should_log_advanced() {
            self.log(&format!("🔄 === ANALYSIS DONE! {} ===", move_to_string(mv)));
            let status = if score > alpha { "📈 IMPROVED" } else if score >= beta { "✂️ CUTOFF" } else { "📊 NORMAL" };
            self.log_with_indent(&format!("✅ {} → Score: {} | {}", move_to_string(mv), score, status));
            self.decrease_indent();
        }
    }

    pub fn log_root_alpha_change(&mut self, old_alpha: i32, new_alpha: i32, mv: Move) {
        if self.should_log_advanced() {
            self.log(&format!("🎯 NEW BEST MOVE: {} | Alpha: {} → {} (+{})", 
                move_to_string(mv), old_alpha, new_alpha, new_alpha - old_alpha));
        }
    }

    // 🎯 ALPHA-BETA NODE LOGGING
    pub fn log_alphabeta_node_enter(&mut self, depth: i32, alpha: i32, beta: i32, nodes: u64) {
        if self.should_log_advanced() && depth >= 2 { // Only log deeper nodes to avoid spam
            self.log_with_indent(&format!("├─ Node depth {} | α={}, β={} | Nodes: {}", depth, alpha, beta, nodes));
            self.increase_indent();
        }
    }

    pub fn log_alphabeta_move_start(&mut self, mv: Move, move_num: usize, total_moves: usize, depth: i32, alpha: i32, beta: i32) {
        if self.should_log_advanced() && depth >= 2 {
            self.log_with_indent(&format!("├─ Move {}/{}: {} (depth {})", move_num, total_moves, move_to_string(mv), depth));
        }
    }

    pub fn log_alphabeta_move_result(&mut self, mv: Move, score: i32, alpha: i32, beta: i32) {
        if self.should_log_advanced() {
            let comparison = if score > alpha { ">" } else if score == alpha { "=" } else { "<" };
            self.log_with_indent(&format!("   └─ {} → {} {} α({})", move_to_string(mv), score, comparison, alpha));
        }
    }


    // 🎯 QUIESCENCE LOGGING
    pub fn log_quiescence_start(&mut self, alpha: i32, beta: i32) {
        if self.should_log_advanced() {
            self.log_with_indent(&format!("🔍 Quiescence search | α={}, β={}", alpha, beta));
            self.increase_indent();
        }
    }

    pub fn log_quiescence_result(&mut self, eval: i32) {
        if self.should_log_advanced() {
            self.log_with_indent(&format!("✅ Quiescence result: {}", eval));
            self.decrease_indent();
        }
    }

    pub fn log_terminal_node(&mut self, eval: i32, in_check: bool) {
        if self.should_log_advanced() {
            let reason = if in_check { "CHECKMATE" } else { "STALEMATE" };
            self.log_with_indent(&format!("🏁 Terminal node: {} ({})", eval, reason));
        }
    }

    
}

fn move_to_string(mv: Move) -> String {
    let files = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
    let from_file = files[mv.from.file() as usize];
    let from_rank = mv.from.rank() + 1;
    let to_file = files[mv.to.file() as usize];
    let to_rank = mv.to.rank() + 1;
    
    if mv.is_promotion() {
        let promotion_char = match mv.promotion.unwrap() {
            QUEEN => 'Q',
            ROOK => 'R', 
            BISHOP => 'B',
            KNIGHT => 'N',
            _ => '?',
        };
        format!("{}{}-{}{}={}", from_file, from_rank, to_file, to_rank, promotion_char)
    } else {
        format!("{}{}-{}{}", from_file, from_rank, to_file, to_rank)
    }
}

impl Default for ChessLogger {
    fn default() -> Self {
        Self::new()
    }
}

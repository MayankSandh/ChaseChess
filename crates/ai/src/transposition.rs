use engine::{Board, Move, Square, types::*};
use std::collections::HashMap;

/// Type of transposition table entry
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NodeType {
    Exact,      // Exact score (PV node)
    LowerBound, // Beta cutoff (score >= beta)
    UpperBound, // Alpha cutoff (score <= alpha)
}

/// Transposition table entry
#[derive(Debug, Clone)]
pub struct TTEntry {
    pub zobrist_key: u64,
    pub depth: i32,
    pub score: i32,
    pub best_move: Option<Move>,
    pub node_type: NodeType,
    pub age: u8, // For replacement strategy
}

/// Zobrist hash keys for position hashing
pub struct ZobristKeys {
    pieces: [[[u64; 8]; 8]; 12], // [piece_type][file][rank]
    side_to_move: u64,
    castling_rights: [u64; 16],
    en_passant: [u64; 8], // by file
}

impl ZobristKeys {
    pub fn new() -> Self {
        
        let mut keys = ZobristKeys {
            pieces: [[[0; 8]; 8]; 12],
            side_to_move: 0,
            castling_rights: [0; 16],
            en_passant: [0; 8],
        };
        
        let mut counter = 1u64;
        
        // Generate piece keys
        for piece in 0..12 {
            for file in 0..8 {
                for rank in 0..8 {
                    keys.pieces[piece][file][rank] = counter;
                    counter = counter.wrapping_mul(1103515245).wrapping_add(12345);
                }
            }
        }
        
        // Generate other keys
        keys.side_to_move = counter;
        counter = counter.wrapping_mul(1103515245).wrapping_add(12345);
        
        for i in 0..16 {
            keys.castling_rights[i] = counter;
            counter = counter.wrapping_mul(1103515245).wrapping_add(12345);
        }
        
        for i in 0..8 {
            keys.en_passant[i] = counter;
            counter = counter.wrapping_mul(1103515245).wrapping_add(12345);
        }
        
        keys
    }
    
    pub fn hash_position(&self, board: &Board) -> u64 {
        let mut hash = 0u64;
        
        // Hash pieces
        for rank in 0..8 {
            for file in 0..8 {
                let square = Square::new(file, rank);
                let piece = board.get_piece(square);
                
                if !is_empty(piece) {
                    let piece_type = piece_type(piece);
                    let piece_color = piece_color(piece);
                    let piece_index = (piece_type - 1) as usize + if piece_color == WHITE { 0 } else { 6 };
                    hash ^= self.pieces[piece_index][file as usize][rank as usize];
                }
            }
        }
        
        // Hash side to move
        if board.current_turn == BLACK {
            hash ^= self.side_to_move;
        }
        
        // Hash castling rights
        hash ^= self.castling_rights[board.castling_rights as usize & 15];
        
        // Hash en passant
        if let Some(en_passant_square) = board.en_passant_target {
            hash ^= self.en_passant[en_passant_square.file() as usize];
        }
        
        hash
    }
}

/// Transposition Table
pub struct TranspositionTable {
    table: HashMap<u64, TTEntry>,
    zobrist: ZobristKeys,
    age: u8,
    max_size: usize,
}

impl TranspositionTable {
    pub fn new(size_mb: usize) -> Self {
        let entries_per_mb = 1024 * 1024 / std::mem::size_of::<TTEntry>();
        let max_size = size_mb * entries_per_mb;
        
        Self {
            table: HashMap::with_capacity(max_size),
            zobrist: ZobristKeys::new(),
            age: 0,
            max_size,
        }
    }
    
    pub fn get_hash(&self, board: &Board) -> u64 {
        self.zobrist.hash_position(board)
    }
    
    pub fn probe(&self, hash: u64, depth: i32, alpha: i32, beta: i32) -> Option<(i32, Option<Move>)> {
        if let Some(entry) = self.table.get(&hash) {
            if entry.depth >= depth {
                match entry.node_type {
                    NodeType::Exact => return Some((entry.score, entry.best_move)),
                    NodeType::LowerBound if entry.score >= beta => return Some((entry.score, entry.best_move)),
                    NodeType::UpperBound if entry.score <= alpha => return Some((entry.score, entry.best_move)),
                    _ => {}
                }
            }
            // Return best move even if depth is insufficient
            return Some((entry.score, entry.best_move));
        }
        None
    }
    
    pub fn store(&mut self, hash: u64, depth: i32, score: i32, best_move: Option<Move>, node_type: NodeType) {
        // Replacement strategy: always replace if table not full, or replace older/shallower entries
        let should_replace = if let Some(existing) = self.table.get(&hash) {
            depth >= existing.depth || existing.age < self.age
        } else {
            true
        };
        
        if should_replace {
            // Clear old entries if table is getting too large
            if self.table.len() >= self.max_size {
                self.clear_old_entries();
            }
            
            let entry = TTEntry {
                zobrist_key: hash,
                depth,
                score,
                best_move,
                node_type,
                age: self.age,
            };
            
            self.table.insert(hash, entry);
        }
    }
    
    pub fn new_search(&mut self) {
        self.age = self.age.wrapping_add(1);
    }
    
    fn clear_old_entries(&mut self) {
        let old_age = self.age.wrapping_sub(2);
        self.table.retain(|_, entry| entry.age > old_age);
    }
    
    pub fn clear(&mut self) {
        self.table.clear();
    }
    
    pub fn size(&self) -> usize {
        self.table.len()
    }
}

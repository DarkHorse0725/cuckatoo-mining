//! Bitmap-based trimming implementation matching C++ reference miner
//! 
//! This implements the exact same trimming algorithm as the C++ version:
//! - Uses bitmaps to track edge survival
//! - Generates edges only after trimming
//! - Implements the 4-step trimming process

use crate::{Node, Edge, Result};
use crate::hashing::SipHash;

/// Bitmap-based trimmer matching C++ implementation
pub struct BitmapTrimmer {
    edge_bits: u32,
    number_of_edges: u64,
    node_mask: u64,
    edges_bitmap: Vec<u64>,
    nodes_bitmap: Vec<u64>,
}

impl BitmapTrimmer {
    /// Create a new bitmap trimmer
    pub fn new(edge_bits: u32) -> Self {
        let number_of_edges = 1 << edge_bits;
        let node_mask = number_of_edges - 1;
        
        // Calculate bitmap sizes (64 bits per u64)
        let edges_bitmap_size = ((number_of_edges + 63) / 64) as usize;
        let nodes_bitmap_size = ((number_of_edges + 63) / 64) as usize;
        
        Self {
            edge_bits,
            number_of_edges,
            node_mask,
            edges_bitmap: vec![0; edges_bitmap_size],
            nodes_bitmap: vec![0; nodes_bitmap_size],
        }
    }
    
    /// Perform lean trimming matching C++ implementation
    /// 
    /// This implements the exact same algorithm as the C++ lean trimming:
    /// 1. Clear nodes bitmap
    /// 2. Step one: Generate all possible edge indices in edges bitmap
    /// 3. Step two: Trim edges based on node pairs
    /// 4. Repeat steps 3-4 for multiple rounds
    pub fn trim_edges(&mut self, siphash: &SipHash, trimming_rounds: u32) -> Result<Vec<Edge>> {
        // Step 1: Generate all possible edge indices in edges bitmap
        self.generate_edges_bitmap(siphash)?;
        
        // Perform trimming rounds
        for round in 0..trimming_rounds {
            if round == 0 {
                // First round: steps 1 and 2
                self.trim_edges_step_one(siphash)?;
                self.trim_edges_step_two(siphash)?;
            } else {
                // Subsequent rounds: steps 3 and 4
                self.trim_edges_step_three(siphash)?;
                self.trim_edges_step_four(siphash)?;
            }
        }
        
        // Generate final edges from surviving bits in edges bitmap
        self.generate_final_edges(siphash)
    }
    
    /// Step 1: Generate all possible edge indices in edges bitmap
    /// This matches C++ trimEdgesStepOne
    fn generate_edges_bitmap(&mut self, _siphash: &SipHash) -> Result<()> {
        // Set all bits in edges bitmap (all edges are initially present)
        for i in 0..self.edges_bitmap.len() {
            self.edges_bitmap[i] = u64::MAX;
        }
        
        // Clear any excess bits beyond number_of_edges
        let excess_bits = (self.edges_bitmap.len() * 64) as u64 - self.number_of_edges;
        if excess_bits > 0 {
            let last_index = self.edges_bitmap.len() - 1;
            let mask = (1u64 << (64 - excess_bits)) - 1;
            self.edges_bitmap[last_index] &= mask;
        }
        
        // Debug: Print initial edges bitmap state
        println!("DEBUG: Initial edges bitmap has {} bits set", 
                 self.edges_bitmap.iter().map(|&x| x.count_ones()).sum::<u32>());
        println!("DEBUG: Number of edges: {}", self.number_of_edges);
        
        Ok(())
    }
    
    /// Step 1: Clear nodes bitmap and generate nodes for all edges
    /// This matches C++ trimEdgesStepOne
    fn trim_edges_step_one(&mut self, siphash: &SipHash) -> Result<()> {
        // Clear nodes bitmap
        self.nodes_bitmap.fill(0);
        
        // Go through all edges in the edges bitmap
        for (bitmap_index, &bitmap_unit) in self.edges_bitmap.iter().enumerate() {
            if bitmap_unit == 0 {
                continue;
            }
            
            // Go through all set bits in the unit
            let mut unit = bitmap_unit;
            let mut bit_index = 0;
            while unit != 0 {
                let bit_pos = unit.trailing_zeros() as u8;
                let edge_index = (bitmap_index * 64 + bit_index * 64 + bit_pos as usize) as u64;
                
                if edge_index < self.number_of_edges {
                    // Get edge's first node using SipHash
                    let node = self.siphash24(siphash, edge_index * 2);
                    
                    // Enable node in nodes bitmap
                    Self::set_bit_in_bitmap(&mut self.nodes_bitmap, node.value());
                }
                
                // Clear the bit and continue
                unit &= unit - 1;
                bit_index += 1;
            }
        }
        
        // Debug: Print nodes bitmap state after step one
        println!("DEBUG: After step one, nodes bitmap has {} bits set", 
                 self.nodes_bitmap.iter().map(|&x| x.count_ones()).sum::<u32>());
        
        Ok(())
    }
    
    /// Step 2: Trim edges based on node pairs
    /// This matches C++ trimEdgesStepTwo
    fn trim_edges_step_two(&mut self, siphash: &SipHash) -> Result<()> {
        // Go through all edges in the edges bitmap
        for bitmap_index in 0..self.edges_bitmap.len() {
            if self.edges_bitmap[bitmap_index] == 0 {
                continue;
            }
            
            let mut new_unit = 0u64;
            let mut bit_index = 0;
            let mut unit = self.edges_bitmap[bitmap_index];
            
            // Go through all set bits in the unit
            while unit != 0 {
                let bit_pos = unit.trailing_zeros() as u8;
                let edge_index = (bitmap_index * 64 + bit_index * 64 + bit_pos as usize) as u64;
                
                if edge_index < self.number_of_edges {
                    // Get edge's first node using SipHash
                    let node = self.siphash24(siphash, edge_index * 2);
                    
                    // Check if node has a pair in the nodes bitmap
                    if Self::is_bit_set_in_bitmap(&self.nodes_bitmap, node.value() ^ 1) {
                        // Enable edge
                        new_unit |= 1u64 << bit_pos;
                    }
                }
                
                // Clear the bit and continue
                unit &= unit - 1;
                bit_index += 1;
            }
            
            self.edges_bitmap[bitmap_index] = new_unit;
        }
        
        Ok(())
    }
    
    /// Step 3: Clear nodes bitmap and generate nodes for surviving edges
    /// This matches C++ trimEdgesStepThree
    fn trim_edges_step_three(&mut self, siphash: &SipHash) -> Result<()> {
        // Clear nodes bitmap
        self.nodes_bitmap.fill(0);
        
        // Go through all surviving edges in the edges bitmap
        for (bitmap_index, &bitmap_unit) in self.edges_bitmap.iter().enumerate() {
            if bitmap_unit == 0 {
                continue;
            }
            
            // Go through all set bits in the unit
            let mut unit = bitmap_unit;
            let mut bit_index = 0;
            while unit != 0 {
                let bit_pos = unit.trailing_zeros() as u8;
                let edge_index = (bitmap_index * 64 + bit_index * 64 + bit_pos as usize) as u64;
                
                if edge_index < self.number_of_edges {
                    // Get edge's second node using SipHash
                    let node = self.siphash24(siphash, edge_index * 2 + 1);
                    
                    // Enable node in nodes bitmap
                    Self::set_bit_in_bitmap(&mut self.nodes_bitmap, node.value());
                }
                
                // Clear the bit and continue
                unit &= unit - 1;
                bit_index += 1;
            }
        }
        
        Ok(())
    }
    
    /// Step 4: Trim edges based on node pairs (second partition)
    /// This matches C++ trimEdgesStepFour
    fn trim_edges_step_four(&mut self, siphash: &SipHash) -> Result<()> {
        // Go through all edges in the edges bitmap
        for bitmap_index in 0..self.edges_bitmap.len() {
            if self.edges_bitmap[bitmap_index] == 0 {
                continue;
            }
            
            let mut new_unit = 0u64;
            let mut bit_index = 0;
            let mut unit = self.edges_bitmap[bitmap_index];
            
            // Go through all set bits in the unit
            while unit != 0 {
                let bit_pos = unit.trailing_zeros() as u8;
                let edge_index = (bitmap_index * 64 + bit_index * 64 + bit_pos as usize) as u64;
                
                if edge_index < self.number_of_edges {
                    // Get edge's second node using SipHash
                    let node = self.siphash24(siphash, edge_index * 2 + 1);
                    
                    // Check if node has a pair in the nodes bitmap
                    if Self::is_bit_set_in_bitmap(&self.nodes_bitmap, node.value() ^ 1) {
                        // Enable edge
                        new_unit |= 1u64 << bit_pos;
                    }
                }
                
                // Clear the bit and continue
                unit &= unit - 1;
                bit_index += 1;
            }
            
            self.edges_bitmap[bitmap_index] = new_unit;
        }
        
        Ok(())
    }
    
    /// Generate final edges from surviving bits in edges bitmap
    /// This matches C++ edge generation after trimming
    fn generate_final_edges(&self, siphash: &SipHash) -> Result<Vec<Edge>> {
        let mut edges = Vec::new();
        
        // Go through all surviving edges in the edges bitmap
        for (bitmap_index, &bitmap_unit) in self.edges_bitmap.iter().enumerate() {
            if bitmap_unit == 0 {
                continue;
            }
            
            // Go through all set bits in the unit
            let mut unit = bitmap_unit;
            let mut bit_index = 0;
            while unit != 0 {
                let bit_pos = unit.trailing_zeros() as u8;
                let edge_index = (bitmap_index * 64 + bit_index * 64 + bit_pos as usize) as u64;
                
                if edge_index < self.number_of_edges {
                    // Generate edge's nodes using SipHash
                    let u = self.siphash24(siphash, edge_index * 2);
                    let v = self.siphash24(siphash, edge_index * 2 + 1);
                    
                    // Create edge (preserve order like C++)
                    let edge = Edge::new(u, v);
                    edges.push(edge);
                }
                
                // Clear the bit and continue
                unit &= unit - 1;
                bit_index += 1;
            }
        }
        
        Ok(edges)
    }
    
    /// SipHash-2-4 implementation matching C++ version
    fn siphash24(&self, siphash: &SipHash, nonce: u64) -> Node {
        // Use the same SipHash implementation as the main hashing module
        let key = siphash.get_key();
        let node_value = if self.edge_bits == 32 {
            self.siphash24_internal(key, nonce)
        } else {
            self.siphash24_internal(key, nonce) & self.node_mask
        };
        
        Node::new(node_value)
    }
    
    /// Internal SipHash-2-4 implementation
    fn siphash24_internal(&self, key: [u64; 4], nonce: u64) -> u64 {
        // Initialize states with keys
        let mut states = key;
        
        // Perform hash on states (exactly like C++ implementation)
        states[3] ^= nonce;
        self.sip_round(&mut states);
        self.sip_round(&mut states);
        states[0] ^= nonce;
        states[2] ^= 255;
        self.sip_round(&mut states);
        self.sip_round(&mut states);
        self.sip_round(&mut states);
        self.sip_round(&mut states);
        
        // Get node from states
        states[0] ^ states[1] ^ states[2] ^ states[3]
    }
    
    /// SipRound implementation matching C++ version
    fn sip_round(&self, states: &mut [u64; 4]) {
        // Perform SipRound on states (exactly like C++ implementation)
        states[0] = states[0].wrapping_add(states[1]);
        states[1] = states[1].rotate_left(13);
        states[1] ^= states[0];
        states[0] = states[0].rotate_left(32);
        states[2] = states[2].wrapping_add(states[3]);
        states[3] = states[3].rotate_left(16);
        states[3] ^= states[2];
        states[0] = states[0].wrapping_add(states[3]);
        states[3] = states[3].rotate_left(21);
        states[3] ^= states[0];
        states[2] = states[2].wrapping_add(states[1]);
        states[1] = states[1].rotate_left(17);
        states[1] ^= states[2];
        states[2] = states[2].rotate_left(32);
    }
    
    /// Set bit in bitmap
    fn set_bit_in_bitmap(bitmap: &mut [u64], index: u64) {
        let word_index = (index / 64) as usize;
        let bit_index = (index % 64) as u8;
        if word_index < bitmap.len() {
            bitmap[word_index] |= 1u64 << bit_index;
        }
    }
    
    /// Check if bit is set in bitmap
    fn is_bit_set_in_bitmap(bitmap: &[u64], index: u64) -> bool {
        let word_index = (index / 64) as usize;
        let bit_index = (index % 64) as u8;
        if word_index < bitmap.len() {
            (bitmap[word_index] & (1u64 << bit_index)) != 0
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Header;

    #[test]
    fn test_bitmap_trimmer_basic() {
        let header = Header::new(&[0u8; 238]);
        let siphash = SipHash::new_from_header(&header, 0);
        let mut trimmer = BitmapTrimmer::new(10);
        
        // Test basic trimming
        let result = trimmer.trim_edges(&siphash, 1);
        assert!(result.is_ok());
        
        let edges = result.unwrap();
        assert!(!edges.is_empty());
        assert!(edges.len() < 1024); // Should be trimmed down
    }
    
    #[test]
    fn test_bitmap_operations() {
        let _trimmer = BitmapTrimmer::new(10);
        let mut bitmap = vec![0u64; 2];
        
        // Test setting and checking bits
        BitmapTrimmer::set_bit_in_bitmap(&mut bitmap, 0);
        assert!(BitmapTrimmer::is_bit_set_in_bitmap(&bitmap, 0));
        assert!(!BitmapTrimmer::is_bit_set_in_bitmap(&bitmap, 1));
        
        BitmapTrimmer::set_bit_in_bitmap(&mut bitmap, 65);
        assert!(BitmapTrimmer::is_bit_set_in_bitmap(&bitmap, 65));
    }
}

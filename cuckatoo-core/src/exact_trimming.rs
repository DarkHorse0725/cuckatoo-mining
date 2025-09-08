//! Exact bitmap trimming implementation matching C++ OpenCL reference
//! 
//! This implements the exact same trimming algorithm as the C++ OpenCL version,
//! including the 4-step process and exact bit manipulation.

use crate::{Edge, Result, ExactSipHash};

/// Exact bitmap trimmer matching C++ OpenCL implementation
pub struct ExactTrimmer {
    /// Edge bits
    _edge_bits: u32,
    /// Number of edges (2^edge_bits)
    number_of_edges: u32,
    /// Node mask (2^edge_bits - 1)
    _node_mask: u32,
    /// Edges bitmap (using 64-bit words like C++)
    edges_bitmap: Vec<u64>,
    /// Nodes bitmap (using 32-bit words like C++ OpenCL)
    nodes_bitmap: Vec<u32>,
}

impl ExactTrimmer {
    /// Create new exact trimmer
    pub fn new(edge_bits: u32) -> Self {
        let number_of_edges = 1 << edge_bits;
        let node_mask = number_of_edges - 1;
        
        // Calculate bitmap sizes
        // Edges bitmap: 64 bits per u64 word
        let edges_bitmap_size = ((number_of_edges + 63) / 64) as usize;
        // Nodes bitmap: 32 bits per u32 word (like C++ OpenCL)
        let nodes_bitmap_size = ((number_of_edges + 31) / 32) as usize;
        
        Self {
            _edge_bits: edge_bits,
            number_of_edges,
            _node_mask: node_mask,
            edges_bitmap: vec![0; edges_bitmap_size],
            nodes_bitmap: vec![0; nodes_bitmap_size],
        }
    }
    
    /// Perform exact trimming matching C++ implementation
    pub fn trim_edges(&mut self, siphash: &ExactSipHash, trimming_rounds: u32) -> Result<Vec<Edge>> {
        // Initialize edges bitmap with all edges present
        self.initialize_edges_bitmap();
        
        // Perform trimming rounds (exactly like C++ comment lines 3-11)
        for round in 0..trimming_rounds {
            if round == 0 {
                // Trimming round 1: clear nodes bitmap, step one, step two
                self.clear_nodes_bitmap();
                self.trim_edges_step_one(siphash)?;
                self.trim_edges_step_two(siphash)?;
            } else {
                // Trimming round 2+: clear nodes bitmap, step three, step four
                self.clear_nodes_bitmap();
                self.trim_edges_step_three(siphash)?;
                self.trim_edges_step_four(siphash)?;
            }
        }
        
        // Generate final edges from surviving bits
        self.generate_final_edges(siphash)
    }
    
    /// Initialize edges bitmap with all edges present
    fn initialize_edges_bitmap(&mut self) {
        // Set all bits in edges bitmap
        for i in 0..self.edges_bitmap.len() {
            self.edges_bitmap[i] = u64::MAX;
        }
        
        // Clear any excess bits beyond number_of_edges
        let excess_bits = (self.edges_bitmap.len() * 64) as u32 - self.number_of_edges;
        if excess_bits > 0 {
            let last_index = self.edges_bitmap.len() - 1;
            let mask = (1u64 << (64 - excess_bits)) - 1;
            self.edges_bitmap[last_index] &= mask;
        }
    }
    
    /// Clear nodes bitmap
    fn clear_nodes_bitmap(&mut self) {
        self.nodes_bitmap.fill(0);
    }
    
    /// Trim edges step one (exactly matching C++ OpenCL trimEdgesStepOne)
    fn trim_edges_step_one(&mut self, siphash: &ExactSipHash) -> Result<()> {
        // Go through all edges (like C++ work items)
        for edge_index in 0..self.number_of_edges {
            // Get edge's node using SipHash (exactly like C++ line 103)
            let node = siphash.hash_nonce((edge_index as u64) * 2);
            
            // Enable node in nodes bitmap (exactly like C++ line 106)
            self.set_bit_in_nodes_bitmap(node.value() as u32);
        }
        
        Ok(())
    }
    
    /// Trim edges step two (exactly matching C++ OpenCL trimEdgesStepTwo)
    fn trim_edges_step_two(&mut self, siphash: &ExactSipHash) -> Result<()> {
        // Go through all edges bitmap words (like C++ work groups)
        for word_index in 0..self.edges_bitmap.len() {
            let mut new_edges = 0u64;
            let word = self.edges_bitmap[word_index];
            
            // Go through all bits in the word (like C++ work items)
            for bit_index in 0..64 {
                if (word & (1u64 << bit_index)) != 0 {
                    let edge_index = (word_index * 64 + bit_index) as u32;
                    
                    if edge_index < self.number_of_edges {
                        // Get edge's node using SipHash (exactly like C++ line 129)
                        let node = siphash.hash_nonce((edge_index as u64) * 2);
                        
                        // Check if node has a pair in the nodes bitmap (exactly like C++ line 132)
                        if self.is_bit_set_in_nodes_bitmap((node.value() as u32) ^ 1) {
                            // Enable edge (exactly like C++ line 135)
                            new_edges |= 1u64 << bit_index;
                        }
                    }
                }
            }
            
            self.edges_bitmap[word_index] = new_edges;
        }
        
        Ok(())
    }
    
    /// Trim edges step three (exactly matching C++ OpenCL trimEdgesStepThree)
    fn trim_edges_step_three(&mut self, siphash: &ExactSipHash) -> Result<()> {
        // Go through all edges bitmap words
        for word_index in 0..self.edges_bitmap.len() {
            let word = self.edges_bitmap[word_index];
            
            // Go through all enabled edges in the word
            for bit_index in 0..64 {
                if (word & (1u64 << bit_index)) != 0 {
                    let edge_index = (word_index * 64 + bit_index) as u32;
                    
                    if edge_index < self.number_of_edges {
                        // Get edge's node using SipHash (exactly like C++ line 162)
                        // Note: C++ uses nodesInSecondPartition = 1 for step three
                        let node = siphash.hash_nonce(((edge_index as u64) * 2) | 1);
                        
                        // Enable node in nodes bitmap (exactly like C++ line 165)
                        self.set_bit_in_nodes_bitmap(node.value() as u32);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Trim edges step four (exactly matching C++ OpenCL trimEdgesStepFour)
    fn trim_edges_step_four(&mut self, siphash: &ExactSipHash) -> Result<()> {
        // Go through all edges bitmap words
        for word_index in 0..self.edges_bitmap.len() {
            let mut new_edges = self.edges_bitmap[word_index];
            let word = self.edges_bitmap[word_index];
            
            // Go through all enabled edges in the word
            for bit_index in 0..64 {
                if (word & (1u64 << bit_index)) != 0 {
                    let edge_index = (word_index * 64 + bit_index) as u32;
                    
                    if edge_index < self.number_of_edges {
                        // Get edge's node using SipHash (exactly like C++ line 189)
                        // Note: C++ uses nodesInSecondPartition = 1 for step four
                        let node = siphash.hash_nonce(((edge_index as u64) * 2) | 1);
                        
                        // Check if node doesn't have a pair in the nodes bitmap (exactly like C++ line 192)
                        if !self.is_bit_set_in_nodes_bitmap((node.value() as u32) ^ 1) {
                            // Disable edge (exactly like C++ line 195)
                            new_edges ^= 1u64 << bit_index;
                        }
                    }
                }
            }
            
            self.edges_bitmap[word_index] = new_edges;
        }
        
        Ok(())
    }
    
    /// Generate final edges from surviving bits
    fn generate_final_edges(&self, siphash: &ExactSipHash) -> Result<Vec<Edge>> {
        let mut edges = Vec::new();
        
        // Go through all surviving edges in the edges bitmap
        for word_index in 0..self.edges_bitmap.len() {
            let word = self.edges_bitmap[word_index];
            
            // Go through all enabled edges in the word
            for bit_index in 0..64 {
                if (word & (1u64 << bit_index)) != 0 {
                    let edge_index = (word_index * 64 + bit_index) as u32;
                    
                    if edge_index < self.number_of_edges {
                        // Generate edge's nodes using SipHash (exactly like C++ edge generation)
                        let u = siphash.hash_nonce((edge_index as u64) * 2);
                        let v = siphash.hash_nonce((edge_index as u64) * 2 + 1);
                        
                        // Create edge (preserve order like C++)
                        let edge = Edge::new(u, v);
                        edges.push(edge);
                    }
                }
            }
        }
        
        Ok(edges)
    }
    
    /// Set bit in nodes bitmap (exactly matching C++ OpenCL setBitInBitmap)
    fn set_bit_in_nodes_bitmap(&mut self, index: u32) {
        let word_index = (index / 32) as usize;
        let bit_index = (index % 32) as u8;
        if word_index < self.nodes_bitmap.len() {
            self.nodes_bitmap[word_index] |= 1u32 << bit_index;
        }
    }
    
    /// Check if bit is set in nodes bitmap (exactly matching C++ OpenCL isBitSetInBitmap)
    fn is_bit_set_in_nodes_bitmap(&self, index: u32) -> bool {
        let word_index = (index / 32) as usize;
        let bit_index = (index % 32) as u8;
        if word_index < self.nodes_bitmap.len() {
            (self.nodes_bitmap[word_index] & (1u32 << bit_index)) != 0
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_trimmer_basic() {
        let keys = [0x1234567890abcdef, 0xfedcba0987654321, 0x1111222233334444, 0x5555666677778888];
        let siphash = ExactSipHash::new(keys, 8);
        let mut trimmer = ExactTrimmer::new(8);
        
        // Test basic trimming
        let result = trimmer.trim_edges(&siphash, 1);
        assert!(result.is_ok());
        
        let edges = result.unwrap();
        assert!(!edges.is_empty());
        assert!(edges.len() < 256); // Should be trimmed down from 256
    }
    
    #[test]
    fn test_bitmap_operations() {
        let mut trimmer = ExactTrimmer::new(8);
        
        // Test setting and checking bits
        trimmer.set_bit_in_nodes_bitmap(0);
        assert!(trimmer.is_bit_set_in_nodes_bitmap(0));
        assert!(!trimmer.is_bit_set_in_nodes_bitmap(1));
        
        trimmer.set_bit_in_nodes_bitmap(65);
        assert!(trimmer.is_bit_set_in_nodes_bitmap(65));
    }
}

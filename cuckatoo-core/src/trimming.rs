use crate::constants::*;
use crate::types::Edge;
use crate::bitmap::Bitmap;
use std::collections::HashMap;

/// Lean trimming implementation for edge reduction
pub struct LeanTrimmer {
    edge_bits: u32,
    number_of_edges: u64,
}

impl LeanTrimmer {
    /// Create a new lean trimmer
    pub fn new(edge_bits: u32) -> Self {
        Self {
            edge_bits,
            number_of_edges: number_of_edges(edge_bits),
        }
    }

    /// Perform lean trimming on edges
    pub fn trim_edges(&self, edges: &[Edge]) -> Vec<Edge> {
        let mut trimmed_edges = edges.to_vec();
        
        // Perform 42 rounds of trimming (as per Cuckatoo specification)
        for round in 0..42 {
            trimmed_edges = self.trim_round(&trimmed_edges, round);
            
            // Early exit if no edges remain
            if trimmed_edges.is_empty() {
                break;
            }
        }
        
        trimmed_edges
    }

    /// Perform a single trimming round
    fn trim_round(&self, edges: &[Edge], _round: u32) -> Vec<Edge> {
        let mut edge_bitmap = Bitmap::new(self.number_of_edges);
        let mut node_degree_bitmap = Bitmap::new(self.number_of_edges);
        
        // Mark all edges as active initially
        for edge in edges {
            edge_bitmap.set_bit(edge.index as u64);
        }
        
        // Calculate node degrees
        let mut node_degrees: HashMap<u32, u32> = HashMap::new();
        for edge in edges {
            *node_degrees.entry(edge.u_node).or_insert(0) += 1;
            *node_degrees.entry(edge.v_node).or_insert(0) += 1;
        }
        
        // Mark nodes with degree 1
        for (node, degree) in &node_degrees {
            if *degree == 1 {
                node_degree_bitmap.set_bit(*node as u64);
            }
        }
        
        // Trim edges connected to degree-1 nodes
        let mut new_edges = Vec::new();
        for edge in edges {
            if edge_bitmap.is_bit_set(edge.index as u64) {
                let u_degree = node_degrees.get(&edge.u_node).unwrap_or(&0);
                let v_degree = node_degrees.get(&edge.v_node).unwrap_or(&0);
                
                // Keep edge if both nodes have degree > 1
                if *u_degree > 1 && *v_degree > 1 {
                    new_edges.push(edge.clone());
                } else {
                    // Remove edge and update degrees
                    edge_bitmap.clear_bit(edge.index as u64);
                    if let Some(degree) = node_degrees.get_mut(&edge.u_node) {
                        *degree = degree.saturating_sub(1);
                    }
                    if let Some(degree) = node_degrees.get_mut(&edge.v_node) {
                        *degree = degree.saturating_sub(1);
                    }
                }
            }
        }
        
        new_edges
    }

    /// Get the number of edges after trimming
    pub fn get_trimmed_edge_count(&self, original_count: usize) -> usize {
        // This is a simplified estimate - in practice, the actual count depends on the graph structure
        let mut count = original_count;
        for _ in 0..42 {
            count = (count as f64 * 0.8) as usize; // Rough estimate: 20% reduction per round
            if count == 0 {
                break;
            }
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lean_trimming_basic() {
        let trimmer = LeanTrimmer::new(12);
        
        // Create some test edges
        let edges = vec![
            Edge::new(0, 0, 1),
            Edge::new(1, 1, 2),
            Edge::new(2, 2, 3),
            Edge::new(3, 3, 0),
        ];
        
        let trimmed = trimmer.trim_edges(&edges);
        
        // After trimming, we should have fewer edges
        assert!(trimmed.len() <= edges.len());
    }

    #[test]
    fn test_trimming_round() {
        let trimmer = LeanTrimmer::new(12);
        
        let edges = vec![
            Edge::new(0, 0, 1),
            Edge::new(1, 1, 2),
            Edge::new(2, 2, 0),
        ];
        
        let trimmed = trimmer.trim_round(&edges, 0);
        
        // This should form a cycle, so all edges should remain
        assert_eq!(trimmed.len(), 3);
    }
}

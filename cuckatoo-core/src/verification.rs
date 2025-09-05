//! Cycle verification for Cuckatoo
//! 
//! This implements the 42-cycle verification algorithm to check
//! if a set of edges contains a valid Cuckatoo cycle.

use crate::{Edge, Node, Result, PerformanceMetrics};
use std::collections::{HashMap, HashSet};
use std::time::Instant;

/// Cycle verifier for Cuckatoo
/// 
/// Implements the 42-cycle verification algorithm used in the
/// C++ reference miner.
pub struct CycleVerifier {
    /// Performance metrics
    metrics: PerformanceMetrics,
}

impl CycleVerifier {
    /// Create a new cycle verifier
    pub fn new() -> Self {
        Self {
            metrics: PerformanceMetrics::new(),
        }
    }
    
    /// Find a 42-cycle in the given edges
    /// 
    /// This is the main method used by the CLI
    pub fn find_42_cycle(&mut self, edges: &[Edge]) -> Result<Option<Vec<Node>>> {
        self.verify_cycle(edges)
    }
    
    /// Verify if edges contain a 42-cycle
    /// 
    /// This implements the same algorithm as the C++ reference miner:
    /// 1. Build adjacency list from edges
    /// 2. Use DFS to find cycles of length 42
    /// 3. Return the first valid cycle found
    pub fn verify_cycle(&mut self, edges: &[Edge]) -> Result<Option<Vec<Node>>> {
        let start_time = Instant::now();
        
        if edges.len() < 42 {
            // Not enough edges for a 42-cycle
            return Ok(None);
        }
        
        // Build adjacency list
        let adjacency = self.build_adjacency_list(edges);
        
        // Try to find a 42-cycle starting from each node
        for &start_node in adjacency.keys() {
            if let Some(cycle) = self.find_cycle_from_node(start_node, &adjacency, 42) {
                let searching_time = start_time.elapsed().as_secs_f64();
                self.metrics.searching_time = searching_time;
                self.metrics.solutions_found = 1;
                
                println!("42-cycle found in {:.6}s", searching_time);
                println!("Cycle: {:?}", cycle);
                
                return Ok(Some(cycle));
            }
        }
        
        let searching_time = start_time.elapsed().as_secs_f64();
        self.metrics.searching_time = searching_time;
        self.metrics.solutions_found = 0;
        
        println!("No 42-cycle found in {:.6}s", searching_time);
        
        Ok(None)
    }
    
    /// Build adjacency list from edges
    fn build_adjacency_list(&self, edges: &[Edge]) -> HashMap<Node, Vec<Node>> {
        let mut adjacency: HashMap<Node, Vec<Node>> = HashMap::new();
        
        for edge in edges {
            adjacency.entry(edge.u).or_insert_with(Vec::new).push(edge.v);
            adjacency.entry(edge.v).or_insert_with(Vec::new).push(edge.u);
        }
        
        adjacency
    }
    
    /// Find a cycle of specified length starting from a node
    fn find_cycle_from_node(
        &self,
        start_node: Node,
        adjacency: &HashMap<Node, Vec<Node>>,
        cycle_length: usize,
    ) -> Option<Vec<Node>> {
        let mut visited = HashSet::new();
        let mut path = Vec::new();
        
        self.dfs_cycle(
            start_node,
            start_node,
            adjacency,
            &mut visited,
            &mut path,
            cycle_length,
        )
    }
    
    /// DFS to find cycles
    fn dfs_cycle(
        &self,
        current: Node,
        start: Node,
        adjacency: &HashMap<Node, Vec<Node>>,
        visited: &mut HashSet<Node>,
        path: &mut Vec<Node>,
        target_length: usize,
    ) -> Option<Vec<Node>> {
        // Add current node to path
        path.push(current);
        
        // Check if we have a cycle of the right length
        if path.len() == target_length {
            if let Some(last_neighbor) = adjacency.get(&current) {
                if last_neighbor.contains(&start) {
                    // Found a cycle!
                    return Some(path.clone());
                }
            }
            // Path too long, backtrack
            path.pop();
            return None;
        }
        
        // Mark current node as visited
        visited.insert(current);
        
        // Try all neighbors
        if let Some(neighbors) = adjacency.get(&current) {
            for &neighbor in neighbors {
                if !visited.contains(&neighbor) {
                    if let Some(cycle) = self.dfs_cycle(
                        neighbor,
                        start,
                        adjacency,
                        visited,
                        path,
                        target_length,
                    ) {
                        return Some(cycle);
                    }
                }
            }
        }
        
        // Backtrack
        visited.remove(&current);
        path.pop();
        None
    }
    
    /// Verify a specific cycle is valid
    pub fn verify_specific_cycle(&self, cycle: &[Node], edges: &[Edge]) -> bool {
        if cycle.len() < 3 {
            return false;
        }
        
        // Check that consecutive nodes in the cycle are connected by edges
        for i in 0..cycle.len() {
            let current = cycle[i];
            let next = cycle[(i + 1) % cycle.len()];
            
            // Create edge in both possible orders and check if either exists
            let edge1 = Edge::new(current, next);
            let edge2 = Edge::new(next, current);
            
            if !edges.contains(&edge1) && !edges.contains(&edge2) {
                return false;
            }
        }
        
        true
    }
    
    /// Get performance metrics
    pub fn metrics(&self) -> &PerformanceMetrics {
        &self.metrics
    }
    
    /// Reset performance metrics
    pub fn reset_metrics(&mut self) {
        self.metrics = PerformanceMetrics::new();
    }
}

impl Default for CycleVerifier {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper struct for cycle finding with better performance
pub struct OptimizedCycleVerifier {
    /// Performance metrics
    metrics: PerformanceMetrics,
}

impl OptimizedCycleVerifier {
    /// Create a new optimized cycle verifier
    pub fn new() -> Self {
        Self {
            metrics: PerformanceMetrics::new(),
        }
    }
    
    /// Find all cycles of specified length
    pub fn find_all_cycles(&mut self, edges: &[Edge], cycle_length: usize) -> Result<Vec<Vec<Node>>> {
        let start_time = Instant::now();
        
        if edges.len() < cycle_length {
            return Ok(vec![]);
        }
        
        let adjacency = self.build_adjacency_list(edges);
        let mut all_cycles = Vec::new();
        
        // Try to find cycles starting from each node
        for &start_node in adjacency.keys() {
            if let Some(cycles) = self.find_cycles_from_node(start_node, &adjacency, cycle_length) {
                all_cycles.extend(cycles);
            }
        }
        
        let searching_time = start_time.elapsed().as_secs_f64();
        self.metrics.searching_time = searching_time;
        self.metrics.solutions_found = all_cycles.len() as u64;
        
                println!("Found {} cycles of length {} in {:.6}s", 
                    all_cycles.len(), cycle_length, searching_time);
        
        Ok(all_cycles)
    }
    
    /// Build adjacency list from edges
    fn build_adjacency_list(&self, edges: &[Edge]) -> HashMap<Node, Vec<Node>> {
        let mut adjacency: HashMap<Node, Vec<Node>> = HashMap::new();
        
        for edge in edges {
            adjacency.entry(edge.u).or_insert_with(Vec::new).push(edge.v);
            adjacency.entry(edge.v).or_insert_with(Vec::new).push(edge.u);
        }
        
        adjacency
    }
    
    /// Find cycles starting from a specific node
    fn find_cycles_from_node(
        &self,
        start_node: Node,
        adjacency: &HashMap<Node, Vec<Node>>,
        cycle_length: usize,
    ) -> Option<Vec<Vec<Node>>> {
        let mut visited = HashSet::new();
        let mut path = Vec::new();
        let mut cycles = Vec::new();
        
        self.dfs_all_cycles(
            start_node,
            start_node,
            adjacency,
            &mut visited,
            &mut path,
            cycle_length,
            &mut cycles,
        );
        
        if cycles.is_empty() {
            None
        } else {
            Some(cycles)
        }
    }
    
    /// DFS to find all cycles
    fn dfs_all_cycles(
        &self,
        current: Node,
        start: Node,
        adjacency: &HashMap<Node, Vec<Node>>,
        visited: &mut HashSet<Node>,
        path: &mut Vec<Node>,
        target_length: usize,
        cycles: &mut Vec<Vec<Node>>,
    ) {
        path.push(current);
        
        if path.len() == target_length {
            if let Some(neighbors) = adjacency.get(&current) {
                if neighbors.contains(&start) {
                    // Found a cycle!
                    cycles.push(path.clone());
                }
            }
            path.pop();
            return;
        }
        
        visited.insert(current);
        
        if let Some(neighbors) = adjacency.get(&current) {
            for &neighbor in neighbors {
                if !visited.contains(&neighbor) {
                    self.dfs_all_cycles(
                        neighbor,
                        start,
                        adjacency,
                        visited,
                        path,
                        target_length,
                        cycles,
                    );
                }
            }
        }
        
        visited.remove(&current);
        path.pop();
    }
    
    /// Get performance metrics
    pub fn metrics(&self) -> &PerformanceMetrics {
        &self.metrics
    }
}

impl Default for OptimizedCycleVerifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cycle_verifier_creation() {
        let verifier = CycleVerifier::new();
        assert_eq!(verifier.metrics().solutions_found, 0);
    }
    
    #[test]
    fn test_adjacency_list_building() {
        let edges = vec![
            Edge::new(Node::new(0), Node::new(1)),
            Edge::new(Node::new(1), Node::new(2)),
            Edge::new(Node::new(2), Node::new(0)),
        ];
        
        let verifier = CycleVerifier::new();
        let adjacency = verifier.build_adjacency_list(&edges);
        
        assert_eq!(adjacency.len(), 3);
        assert_eq!(adjacency.get(&Node::new(0)).unwrap().len(), 2);
        assert_eq!(adjacency.get(&Node::new(1)).unwrap().len(), 2);
        assert_eq!(adjacency.get(&Node::new(2)).unwrap().len(), 2);
    }
    
    #[test]
    fn test_simple_cycle_verification() {
        let mut verifier = CycleVerifier::new();
        
        // Create a simple 3-cycle
        let edges = vec![
            Edge::new(Node::new(0), Node::new(1)),
            Edge::new(Node::new(1), Node::new(2)),
            Edge::new(Node::new(2), Node::new(0)),
        ];
        
        // This should find a 3-cycle, not a 42-cycle
        let result = verifier.verify_cycle(&edges);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none()); // No 42-cycle
    }
    
    #[test]
    fn test_cycle_verification_not_enough_edges() {
        let mut verifier = CycleVerifier::new();
        
        let edges = vec![
            Edge::new(Node::new(0), Node::new(1)),
            Edge::new(Node::new(1), Node::new(2)),
        ];
        
        let result = verifier.verify_cycle(&edges);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }
    
    #[test]
    fn test_specific_cycle_verification() {
        let verifier = CycleVerifier::new();
        
        let edges = vec![
            Edge::new(Node::new(0), Node::new(1)),
            Edge::new(Node::new(1), Node::new(2)),
            Edge::new(Node::new(2), Node::new(0)),
        ];
        
        let cycle = vec![
            Node::new(0),
            Node::new(1),
            Node::new(2),
        ];
        
        // This should verify a 3-cycle
        assert!(verifier.verify_specific_cycle(&cycle, &edges));
        
        // Invalid cycle
        let invalid_cycle = vec![Node::new(0), Node::new(1)];
        assert!(!verifier.verify_specific_cycle(&invalid_cycle, &edges));
    }
    
    #[test]
    fn test_optimized_cycle_verifier() {
        let mut verifier = OptimizedCycleVerifier::new();
        
        let edges = vec![
            Edge::new(Node::new(0), Node::new(1)),
            Edge::new(Node::new(1), Node::new(2)),
            Edge::new(Node::new(2), Node::new(0)),
        ];
        
        let result = verifier.find_all_cycles(&edges, 3);
        assert!(result.is_ok());
        
        let cycles = result.unwrap();
        assert!(cycles.len() >= 1); // At least one 3-cycle (may find duplicates with different starting points)
        
        let cycle = &cycles[0];
        assert_eq!(cycle.len(), 3);
    }
}

/// Synthetic test fixtures for cycle verification
/// 
/// These provide known graphs with cycles for testing the cycle checker
/// on small EDGE_BITS values (12-16) as required by milestone 1.
pub mod test_fixtures {
    use super::*;
    
    /// Create a synthetic graph with a known 42-cycle
    /// 
    /// This creates a simple cycle graph for testing purposes.
    /// The graph is designed to always contain a 42-cycle.
    pub fn create_synthetic_42_cycle_graph() -> Vec<Edge> {
        let mut edges = Vec::with_capacity(42);
        
        // Create a simple cycle: 0->1->2->...->41->0
        for i in 0..42 {
            let u = Node::new(i);
            let v = Node::new((i + 1) % 42);
            edges.push(Edge::new(u, v));
        }
        
        edges
    }
    
    /// Create a synthetic graph with multiple small cycles
    /// 
    /// This creates a graph with several small cycles that can be used
    /// to test cycle detection without requiring a full 42-cycle.
    pub fn create_synthetic_small_cycles_graph() -> Vec<Edge> {
        let mut edges = Vec::new();
        
        // Create several 3-cycles
        for i in 0..10 {
            let base = i * 3;
            edges.push(Edge::new(Node::new(base), Node::new(base + 1)));
            edges.push(Edge::new(Node::new(base + 1), Node::new(base + 2)));
            edges.push(Edge::new(Node::new(base + 2), Node::new(base)));
        }
        
        edges
    }
    
    /// Create a synthetic graph with no cycles (tree structure)
    /// 
    /// This creates a tree graph that contains no cycles, useful for
    /// testing that the cycle detector correctly identifies when no
    /// cycles exist.
    pub fn create_synthetic_tree_graph() -> Vec<Edge> {
        let mut edges = Vec::new();
        
        // Create a binary tree structure
        for i in 0..20 {
            let left_child = 2 * i + 1;
            let right_child = 2 * i + 2;
            
            if left_child < 40 {
                edges.push(Edge::new(Node::new(i), Node::new(left_child)));
            }
            if right_child < 40 {
                edges.push(Edge::new(Node::new(i), Node::new(right_child)));
            }
        }
        
        edges
    }
    
    /// Create a synthetic graph with a specific cycle length
    /// 
    /// This creates a graph with a cycle of the specified length,
    /// useful for testing cycle detection with different cycle sizes.
    pub fn create_synthetic_cycle_graph(cycle_length: usize) -> Vec<Edge> {
        if cycle_length < 3 {
            return vec![];
        }
        
        let mut edges = Vec::with_capacity(cycle_length);
        
        // Create a cycle of the specified length
        for i in 0..cycle_length {
            let u = Node::new(i as u64);
            let v = Node::new(((i + 1) % cycle_length) as u64);
            edges.push(Edge::new(u, v));
        }
        
        edges
    }
}

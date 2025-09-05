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
    pub fn find_42_cycle(&mut self, edges: &[Edge]) -> Result<Option<Vec<Edge>>> {
        self.verify_cycle(edges)
    }
    
    /// Verify if edges contain a 42-cycle
    /// 
    /// This implements the same algorithm as the C++ reference miner:
    /// 1. Use DFS to find cycles of length 42 (sequence of incident edges)
    /// 2. Return the first valid cycle found
    pub fn verify_cycle(&mut self, edges: &[Edge]) -> Result<Option<Vec<Edge>>> {
        let start_time = Instant::now();
        
        if edges.len() < 42 {
            // Not enough edges for a 42-cycle
            return Ok(None);
        }
        
        // Try to find a 42-cycle starting from each edge
        for (i, &start_edge) in edges.iter().enumerate() {
            if let Some(cycle) = self.find_cycle_from_edge(start_edge, edges, 42, i) {
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
    #[allow(dead_code)]
    fn build_adjacency_list(&self, edges: &[Edge]) -> HashMap<Node, Vec<Node>> {
        let mut adjacency: HashMap<Node, Vec<Node>> = HashMap::new();
        
        for edge in edges {
            adjacency.entry(edge.u).or_insert_with(Vec::new).push(edge.v);
            adjacency.entry(edge.v).or_insert_with(Vec::new).push(edge.u);
        }
        
        adjacency
    }
    
    /// Find a cycle of specified length starting from a node
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
    
    /// Find a cycle of specified length starting from an edge
    fn find_cycle_from_edge(
        &self,
        start_edge: Edge,
        edges: &[Edge],
        target_length: usize,
        start_index: usize,
    ) -> Option<Vec<Edge>> {
        let mut used_edges = HashSet::new();
        let mut path = Vec::new();
        
        // Start the cycle from the start edge
        self.dfs_edge_cycle(start_edge, start_edge, edges, &mut used_edges, &mut path, target_length, start_index)
    }
    
    /// DFS helper for edge-based cycle finding
    /// Tracks the current endpoint to ensure proper connectivity
    fn dfs_edge_cycle(
        &self,
        current_edge: Edge,
        start_edge: Edge,
        edges: &[Edge],
        used_edges: &mut HashSet<usize>,
        path: &mut Vec<Edge>,
        target_length: usize,
        start_index: usize,
    ) -> Option<Vec<Edge>> {
        if path.len() == target_length {
            // Check if we can return to the start edge
            let last_edge = path[path.len() - 1];
            
            // Find which endpoint of the last edge should connect to the start edge
            let connecting_endpoint = if path.len() == 1 {
                // This shouldn't happen, but handle it
                last_edge.v
            } else {
                // Find which endpoint of last_edge connects to the previous edge
                let prev_edge = path[path.len() - 2];
                if prev_edge.u == last_edge.u || prev_edge.v == last_edge.u {
                    last_edge.v  // Connect from v endpoint
                } else {
                    last_edge.u  // Connect from u endpoint
                }
            };
            
            // Check if start_edge connects to the connecting_endpoint
            if start_edge.u == connecting_endpoint || start_edge.v == connecting_endpoint {
                return Some(path.clone());
            }
            return None;
        }
        
        if path.len() > target_length {
            return None;
        }
        
        // Find the index of current edge
        let current_index = edges.iter().position(|&e| e == current_edge)?;
        used_edges.insert(current_index);
        path.push(current_edge);
        
        // Find the endpoint that connects to the next edge
        let connecting_endpoint = if path.len() == 1 {
            // For the first edge, we can connect from either endpoint
            current_edge.v
        } else {
            // For subsequent edges, find which endpoint of current_edge connects to the previous edge
            let prev_edge = path[path.len() - 2];
            if prev_edge.u == current_edge.u || prev_edge.v == current_edge.u {
                current_edge.v  // Connect from v endpoint
            } else {
                current_edge.u  // Connect from u endpoint
            }
        };
        
        // Find edges that connect to the connecting_endpoint
        for (i, &edge) in edges.iter().enumerate() {
            if !used_edges.contains(&i) {
                // Check if this edge connects to our connecting_endpoint
                if edge.u == connecting_endpoint || edge.v == connecting_endpoint {
                    if let Some(result) = self.dfs_edge_cycle(edge, start_edge, edges, used_edges, path, target_length, start_index) {
                        return Some(result);
                    }
                }
            }
        }
        
        used_edges.remove(&current_index);
        path.pop();
        None
    }
    
    /// Check if two edges are incident (share an endpoint)
    #[allow(dead_code)]
    fn edges_are_incident(&self, edge1: Edge, edge2: Edge) -> bool {
        edge1.u == edge2.u || edge1.u == edge2.v || edge1.v == edge2.u || edge1.v == edge2.v
    }
    
    /// Check if two edges are properly connected (share exactly one endpoint)
    /// This ensures that consecutive edges in a cycle form a proper path
    fn edges_are_properly_connected(&self, edge1: Edge, edge2: Edge) -> bool {
        // Two edges are properly connected if they share exactly one endpoint
        let shares_u_u = edge1.u == edge2.u;
        let shares_u_v = edge1.u == edge2.v;
        let shares_v_u = edge1.v == edge2.u;
        let shares_v_v = edge1.v == edge2.v;
        
        // Count how many endpoints are shared
        let shared_count = (shares_u_u as u8) + (shares_u_v as u8) + (shares_v_u as u8) + (shares_v_v as u8);
        
        // Must share exactly one endpoint
        shared_count == 1
    }
    
    /// Verify a specific cycle is valid
    /// In Cuckatoo, a cycle is a sequence of edges where consecutive edges share an endpoint
    pub fn verify_specific_cycle(&self, cycle_edges: &[Edge], all_edges: &[Edge]) -> bool {
        if cycle_edges.len() < 3 {
            return false;
        }
        
        // Check that all cycle edges exist in the edge set
        for edge in cycle_edges {
            if !all_edges.contains(edge) {
                return false;
            }
        }
        
        // Check that consecutive edges are properly connected
        for i in 0..cycle_edges.len() {
            let current_edge = cycle_edges[i];
            let next_edge = cycle_edges[(i + 1) % cycle_edges.len()];
            
            if !self.edges_are_properly_connected(current_edge, next_edge) {
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
    #[allow(dead_code)]
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
        
        // Valid 3-cycle using edges
        let cycle = vec![
            Edge::new(Node::new(0), Node::new(1)),
            Edge::new(Node::new(1), Node::new(2)),
            Edge::new(Node::new(2), Node::new(0)),
        ];
        
        // This should verify a 3-cycle
        assert!(verifier.verify_specific_cycle(&cycle, &edges));
        
        // Invalid cycle (only 2 edges, not connected)
        let invalid_cycle = vec![
            Edge::new(Node::new(0), Node::new(1)),
            Edge::new(Node::new(2), Node::new(3)),
        ];
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


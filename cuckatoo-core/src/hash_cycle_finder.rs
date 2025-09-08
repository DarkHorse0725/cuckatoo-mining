//! Hash table-based cycle finder matching C++ reference miner exactly
//! 
//! This implements the exact same cycle finding algorithm as the C++ version,
//! including the hash table-based node connection tracking and the two-partition
//! search approach.

use crate::{Node, Edge, Result, SOLUTION_SIZE, EDGE_NUMBER_OF_COMPONENTS};
use std::collections::HashMap;

/// Node connection link matching C++ CuckatooNodeConnectionsLink exactly
#[derive(Clone, Debug)]
pub struct NodeConnectionLink {
    pub previous_link: Option<Box<NodeConnectionLink>>,
    pub node: Node,
    pub edge_index: u32,
}

/// Hash cycle finder matching C++ getCuckatooSolution algorithm exactly
pub struct HashCycleFinder {
    // Thread-local global variables matching C++ exactly
    u_newest_connections: HashMap<Node, NodeConnectionLink>,
    v_newest_connections: HashMap<Node, NodeConnectionLink>,
    u_visited_pairs: HashMap<u64, u32>,
    v_visited_pairs: HashMap<u64, u32>,
    root_node: Node,
}

impl HashCycleFinder {
    pub fn new() -> Self {
        Self {
            u_newest_connections: HashMap::new(),
            v_newest_connections: HashMap::new(),
            u_visited_pairs: HashMap::new(),
            v_visited_pairs: HashMap::new(),
            root_node: Node::new(0),
        }
    }
    
    /// Initialize thread-local global variables (matching C++ initializeCuckatooThreadLocalGlobalVariables)
    pub fn initialize_thread_local_global_variables(&mut self) -> bool {
        // Reset thread local global variables
        self.u_newest_connections.clear();
        self.v_newest_connections.clear();
        self.u_visited_pairs.clear();
        self.v_visited_pairs.clear();
        self.root_node = Node::new(0);
        
        true
    }

    /// Get cuckatoo solution (matching C++ getCuckatooSolution exactly)
    pub fn get_cuckatoo_solution(&mut self, solution: &mut [u32; SOLUTION_SIZE], 
                                node_connections: &mut [NodeConnectionLink], 
                                edges: &[u32], 
                                number_of_edges: u64) -> bool {
        
        // Go through all edges (matching C++ loop exactly)
        let mut node_connections_index = 0;
        let mut edges_index = 0;
        
        while node_connections_index < (number_of_edges * 2) as usize {
            // Get edge's index and nodes (matching C++ exactly)
            let index = &edges[edges_index];
            let node = Node::new(edges[edges_index + 1] as u64);
            self.root_node = Node::new(edges[edges_index + 2] as u64);
            
            // Replace newest node connection for the node on the first partition and add node connection to list
            let previous_u = self.u_newest_connections.get(&node).cloned();
            let new_u_link = NodeConnectionLink {
                previous_link: previous_u.map(|link| Box::new(link)),
                node,
                edge_index: *index,
            };
            node_connections[node_connections_index] = new_u_link.clone();
            self.u_newest_connections.insert(node, new_u_link);
            
            // Replace newest node connection for the node on the second partition and add node connection to list
            let previous_v = self.v_newest_connections.get(&self.root_node).cloned();
            let new_v_link = NodeConnectionLink {
                previous_link: previous_v.map(|link| Box::new(link)),
                node: self.root_node,
                edge_index: *index,
            };
            node_connections[node_connections_index + 1] = new_v_link.clone();
            self.v_newest_connections.insert(self.root_node, new_v_link);
            
            // Check if both nodes have a pair
            if self.u_newest_connections.contains_key(&Node::new(node.value() ^ 1)) &&
               self.v_newest_connections.contains_key(&Node::new(self.root_node.value() ^ 1)) {
                
                // Reset visited nodes
                self.u_visited_pairs.clear();
                self.v_visited_pairs.clear();
                
                // Go through all nodes in the cycle (matching C++ complex loop exactly)
                let mut cycle_size = 1u8;
                let mut current_node = node;
                let mut current_index = *index;
                
                loop {
                    // Set that node pair has been visited
                    self.u_visited_pairs.insert(current_node.value() >> 1, current_index);
                    
                    // Check if node's pair has more than one connection
                    if let Some(node_connection) = self.u_newest_connections.get(&Node::new(current_node.value() ^ 1)) {
                        if node_connection.previous_link.is_some() {
                            // Collect all connections first to avoid borrowing issues
                            let mut connections = Vec::new();
                            let mut current_link = Some(node_connection);
                            while let Some(link) = current_link {
                                connections.push((link.node, link.edge_index));
                                current_link = link.previous_link.as_ref().map(|boxed| boxed.as_ref());
                            }
                            
                            // Go through all of the node's pair's connections
                            for (connected_node, connected_edge_index) in connections {
                                // Check if the connected node's pair wasn't already visited
                                let connected_node_pair_index = (connected_node.value() + 1) >> 1; // (nodeConnection + 1)->node >> 1
                                if !self.v_visited_pairs.contains_key(&connected_node_pair_index) {
                                    
                                    // Check if cycle is complete
                                    if (connected_node.value() ^ 1) == self.root_node.value() {
                                        
                                        // Check if cycle is a solution
                                        if cycle_size == (SOLUTION_SIZE - 1) as u8 {
                                            
                                            // Get solution from visited nodes
                                            self.get_solution_from_visited_nodes(solution, connected_edge_index);
                                            
                                            // Sort solution in ascending order
                                            solution.sort();
                                            
                                            return true;
                                        }
                                    }
                                    
                                    // Otherwise check if cycle could be as solution
                                    else if cycle_size != (SOLUTION_SIZE - 1) as u8 {
                                        
                                        // Check if the connected node has a pair
                                        if self.v_newest_connections.contains_key(&Node::new(connected_node.value() ^ 1)) {
                                            
                                            // Check if solution was found at the connected node's pair
                                            if self.search_node_connections_second_partition(cycle_size + 1, (connected_node.value() ^ 1) as u32, connected_edge_index) {
                                                
                                                // Get solution from visited nodes
                                                self.get_solution_from_visited_nodes(solution, 0);
                                                
                                                // Sort solution in ascending order
                                                solution.sort();
                                                
                                                return true;
                                            }
                                        }
                                    }
                                }
                            }
                            
                            // Break
                            break;
                        }
                        
                        // Go to node's pair opposite end and get its edge index
                        current_index = node_connection.edge_index;
                        current_node = node_connection.node;
                        
                        // Check if node pair was already visited
                        if self.v_visited_pairs.contains_key(&(current_node.value() >> 1)) {
                            break;
                        }
                        
                        // Check if cycle is complete
                        if (current_node.value() ^ 1) == self.root_node.value() {
                            
                            // Check if cycle is a solution
                            if cycle_size == (SOLUTION_SIZE - 1) as u8 {
                                
                                // Get solution from visited nodes
                                self.get_solution_from_visited_nodes(solution, current_index);
                                
                                // Sort solution in ascending order
                                solution.sort();
                                
                                return true;
                            }
                            
                            // Break
                            break;
                        }
                        
                        // Check if cycle isn't a solution
                        if cycle_size == (SOLUTION_SIZE - 1) as u8 {
                            break;
                        }
                        
                        // Check if node doesn't have a pair
                        if !self.v_newest_connections.contains_key(&Node::new(current_node.value() ^ 1)) {
                            break;
                        }
                        
                        // Set that node pair has been visited
                        self.v_visited_pairs.insert(current_node.value() >> 1, current_index);
                        
                        // Check if node's pair has more than one connection
                        if let Some(node_connection) = self.v_newest_connections.get(&Node::new(current_node.value() ^ 1)) {
                        if node_connection.previous_link.is_some() {
                            // Collect all connections first to avoid borrowing issues
                            let mut connections = Vec::new();
                            let mut current_link = Some(node_connection);
                            while let Some(link) = current_link {
                                connections.push((link.node, link.edge_index));
                                current_link = link.previous_link.as_ref().map(|boxed| boxed.as_ref());
                            }
                            
                            // Go through all of the node's pair's connections
                            for (connected_node, connected_edge_index) in connections {
                                // Check if the connected node has a pair
                                if self.u_newest_connections.contains_key(&Node::new(connected_node.value() ^ 1)) {
                                    
                                    // Check if the connected node's pair wasn't already visited
                                    if !self.u_visited_pairs.contains_key(&(connected_node.value() >> 1)) {
                                        
                                        // Check if solution was found at the connected node's pair
                                        if self.search_node_connections_first_partition(cycle_size + 2, (connected_node.value() ^ 1) as u32, connected_edge_index) {
                                            
                                            // Get solution from visited nodes
                                            self.get_solution_from_visited_nodes(solution, 0);
                                            
                                            // Sort solution in ascending order
                    solution.sort();
                    
                                            return true;
                                        }
                                    }
                                }
                            }
                                
                                // Break
                                break;
                            }
                            
                            // Go to node's pair opposite end and get its edge index
                            current_index = node_connection.edge_index;
                            current_node = node_connection.node;
                            
                            // Check if node pair was already visited
                            if self.u_visited_pairs.contains_key(&(current_node.value() >> 1)) {
                                break;
                            }
                            
                            // Check if node doesn't have a pair
                            if !self.u_newest_connections.contains_key(&Node::new(current_node.value() ^ 1)) {
                                break;
                            }
                            
                            cycle_size += 2;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
            
            // Update indices for next iteration
            node_connections_index += 2;
            edges_index += EDGE_NUMBER_OF_COMPONENTS as usize;
        }
        
        false
    }

    /// Search node connections for cuckatoo solution first partition (matching C++ exactly)
    fn search_node_connections_first_partition(&mut self, cycle_size: u8, node: u32, index: u32) -> bool {
        // Set that node pair has been visited
        let visited_node_pair_index = node >> 1;
        self.u_visited_pairs.insert(visited_node_pair_index as u64, index);
        
        // Go through all of the node's connections
        if let Some(node_connection) = self.u_newest_connections.get(&Node::new(node as u64)) {
            // Collect all connections first to avoid borrowing issues
        let mut connections = Vec::new();
            let mut current_link = Some(node_connection);
            while let Some(link) = current_link {
                connections.push((link.node, link.edge_index));
                current_link = link.previous_link.as_ref().map(|boxed| boxed.as_ref());
        }
        
        for (connected_node, connected_edge_index) in connections {
            // Check if the connected node's pair wasn't already visited
                let connected_node_pair_index = (connected_node.value() + 1) >> 1; // (nodeConnection + 1)->node >> 1
            if !self.v_visited_pairs.contains_key(&connected_node_pair_index) {
                
                // Check if cycle is complete
                if (connected_node.value() ^ 1) == self.root_node.value() {
                        
                    // Check if cycle is a solution
                        if cycle_size == (SOLUTION_SIZE - 1) as u8 {
                            
                        // Set that the connected node's pair has been visited
                        self.v_visited_pairs.insert(connected_node_pair_index, connected_edge_index);
                            
                        return true;
                    }
                }
                    
                    // Otherwise check if cycle could be as solution
                    else if cycle_size != (SOLUTION_SIZE - 1) as u8 {
                        
                    // Check if the connected node has a pair
                    if self.v_newest_connections.contains_key(&Node::new(connected_node.value() ^ 1)) {
                            
                        // Check if solution was found at the connected node's pair
                            if self.search_node_connections_second_partition(cycle_size + 1, (connected_node.value() ^ 1) as u32, connected_edge_index) {
                            return true;
                            }
                        }
                    }
                }
            }
        }
        
        // Set that node pair hasn't been visited
        self.u_visited_pairs.remove(&(visited_node_pair_index as u64));
        
        false
    }
    
    /// Search node connections for cuckatoo solution second partition (matching C++ exactly)
    fn search_node_connections_second_partition(&mut self, cycle_size: u8, node: u32, index: u32) -> bool {
        // Set that node pair has been visited
        let visited_node_pair_index = node >> 1;
        self.v_visited_pairs.insert(visited_node_pair_index as u64, index);
        
        // Go through all of the node's connections
        if let Some(node_connection) = self.v_newest_connections.get(&Node::new(node as u64)) {
            // Collect all connections first to avoid borrowing issues
            let mut connections = Vec::new();
            let mut current_link = Some(node_connection);
            while let Some(link) = current_link {
                connections.push((link.node, link.edge_index));
                current_link = link.previous_link.as_ref().map(|boxed| boxed.as_ref());
            }
            
            for (connected_node, connected_edge_index) in connections {
                // Check if the connected node has a pair
                if self.u_newest_connections.contains_key(&Node::new(connected_node.value() ^ 1)) {
                    
                    // Check if the connected node's pair wasn't already visited
                    if !self.u_visited_pairs.contains_key(&(connected_node.value() >> 1)) {
                        
                        // Check if solution was found at the connected node's pair
                        if self.search_node_connections_first_partition(cycle_size + 1, (connected_node.value() ^ 1) as u32, connected_edge_index) {
                            return true;
                        }
                    }
                }
            }
        }
        
        // Set that node pair hasn't been visited
        self.v_visited_pairs.remove(&(visited_node_pair_index as u64));
        
        false
    }
    
    /// Get solution from visited nodes (matching C++ getValues)
    fn get_solution_from_visited_nodes(&self, solution: &mut [u32; SOLUTION_SIZE], last_edge_index: u32) {
        let mut i = 0;
        
        // Get values from U visited pairs
        for &edge_index in self.u_visited_pairs.values() {
            if i < SOLUTION_SIZE / 2 {
                solution[i] = edge_index;
                i += 1;
            }
        }
        
        // Get values from V visited pairs
        for &edge_index in self.v_visited_pairs.values() {
            if i < SOLUTION_SIZE - 1 {
                solution[i] = edge_index;
                i += 1;
            }
        }
        
        // Add the last edge index
        if i < SOLUTION_SIZE {
            solution[i] = last_edge_index;
        }
    }

    /// Find cycle using the C++ algorithm (wrapper for getCuckatooSolution)
    pub fn find_cycle(&mut self, edges: &[Edge]) -> Result<Option<Vec<usize>>> {
        // Initialize thread-local global variables
        self.initialize_thread_local_global_variables();
        
        // Convert edges to C++ format [edge_index, node_u, node_v]
        let mut cpp_edges = Vec::new();
        for (i, edge) in edges.iter().enumerate() {
            cpp_edges.push(i as u32); // edge_index
            cpp_edges.push(edge.u.value() as u32); // node_u
            cpp_edges.push(edge.v.value() as u32); // node_v
        }
        
        // Create node connections array
        let mut node_connections = vec![
            NodeConnectionLink {
                previous_link: None,
                node: Node::new(0),
                edge_index: 0,
            };
            edges.len() * 2
        ];
        
        // Call the C++ algorithm
        let mut solution = [0u32; SOLUTION_SIZE];
        if self.get_cuckatoo_solution(&mut solution, &mut node_connections, &cpp_edges, edges.len() as u64) {
            // Convert solution indices to Vec<usize>
            let solution_indices: Vec<usize> = solution.iter().map(|&idx| idx as usize).collect();
            Ok(Some(solution_indices))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_cycle_finder_basic() {
        let mut finder = HashCycleFinder::new();
        assert!(finder.initialize_thread_local_global_variables());
        
        // Test with empty edges
        let edges = vec![];
        let result = finder.find_cycle(&edges);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }
}
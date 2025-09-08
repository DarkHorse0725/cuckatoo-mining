//! Exact C++ getCuckatooSolution algorithm implementation
//! 
//! This implements the exact same cycle finding algorithm as the C++ version,
//! including the exact data structures, loop structure, and logic flow.

use crate::{SOLUTION_SIZE, EDGE_NUMBER_OF_COMPONENTS};
use std::collections::HashMap;

/// Node connection link matching C++ CuckatooNodeConnectionsLink exactly
#[derive(Clone, Debug)]
pub struct CuckatooNodeConnectionsLink {
    pub previous_node_connection_link: Option<Box<CuckatooNodeConnectionsLink>>,
    pub node: u32,
    pub edge_index: u32,
}

/// Hash table matching C++ HashTable template
pub struct HashTable {
    data: HashMap<u32, CuckatooNodeConnectionsLink>,
}

impl HashTable {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
    
    pub fn clear(&mut self) {
        self.data.clear();
    }
    
    pub fn contains(&self, key: u32) -> bool {
        self.data.contains_key(&key)
    }
    
    pub fn get(&self, key: u32) -> Option<&CuckatooNodeConnectionsLink> {
        self.data.get(&key)
    }
    
    pub fn replace(&mut self, key: u32, new_link: &CuckatooNodeConnectionsLink) -> Option<CuckatooNodeConnectionsLink> {
        self.data.insert(key, new_link.clone())
    }
}

/// Visited node pairs hash table
pub struct VisitedNodePairs {
    data: HashMap<u64, u32>,
}

impl VisitedNodePairs {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
    
    pub fn clear(&mut self) {
        self.data.clear();
    }
    
    pub fn contains(&self, key: u64) -> bool {
        self.data.contains_key(&key)
    }
    
    pub fn set_unique(&mut self, key: u64, value: u32) {
        self.data.insert(key, value);
    }
    
    pub fn get_values(&self, solution: &mut [u32]) {
        let mut i = 0;
        for &value in self.data.values() {
            if i < solution.len() {
                solution[i] = value;
                i += 1;
            }
        }
    }
}

/// Exact C++ getCuckatooSolution implementation
pub struct CppCycleFinder {
    // Thread-local global variables matching C++ exactly
    cuckatoo_u_newest_node_connections: HashTable,
    cuckatoo_v_newest_node_connections: HashTable,
    cuckatoo_u_visited_node_pairs: VisitedNodePairs,
    cuckatoo_v_visited_node_pairs: VisitedNodePairs,
    cuckatoo_root_node: u32,
}

impl CppCycleFinder {
    pub fn new() -> Self {
        Self {
            cuckatoo_u_newest_node_connections: HashTable::new(),
            cuckatoo_v_newest_node_connections: HashTable::new(),
            cuckatoo_u_visited_node_pairs: VisitedNodePairs::new(),
            cuckatoo_v_visited_node_pairs: VisitedNodePairs::new(),
            cuckatoo_root_node: 0,
        }
    }
    
    /// Initialize thread-local global variables (matching C++ initializeCuckatooThreadLocalGlobalVariables)
    pub fn initialize_cuckatoo_thread_local_global_variables(&mut self) -> bool {
        // Reset thread local global variables
        self.cuckatoo_u_newest_node_connections.clear();
        self.cuckatoo_v_newest_node_connections.clear();
        self.cuckatoo_u_visited_node_pairs.clear();
        self.cuckatoo_v_visited_node_pairs.clear();
        self.cuckatoo_root_node = 0;
        
        true
    }
    
    /// Get cuckatoo solution (matching C++ getCuckatooSolution exactly)
    pub fn get_cuckatoo_solution(&mut self, solution: &mut [u32; SOLUTION_SIZE], 
                                node_connections: &mut [CuckatooNodeConnectionsLink], 
                                edges: &[u32], 
                                number_of_edges: u64) -> bool {
        
        // Go through all edges (matching C++ loop exactly)
        let mut node_connections_index = 0;
        let mut edges_index = 0;
        
        while node_connections_index < (number_of_edges * 2) as usize {
            // Get edge's index and nodes (matching C++ exactly)
            let index = &edges[edges_index];
            let node = edges[edges_index + 1];
            self.cuckatoo_root_node = edges[edges_index + 2];
            
            // Replace newest node connection for the node on the first partition and add node connection to list
            let previous_u = self.cuckatoo_u_newest_node_connections.get(node).cloned();
            let new_u_link = CuckatooNodeConnectionsLink {
                previous_node_connection_link: previous_u.map(|link| Box::new(link)),
                node,
                edge_index: *index,
            };
            node_connections[node_connections_index] = new_u_link.clone();
            self.cuckatoo_u_newest_node_connections.replace(node, &new_u_link);
            
            // Replace newest node connection for the node on the second partition and add node connection to list
            let previous_v = self.cuckatoo_v_newest_node_connections.get(self.cuckatoo_root_node).cloned();
            let new_v_link = CuckatooNodeConnectionsLink {
                previous_node_connection_link: previous_v.map(|link| Box::new(link)),
                node: self.cuckatoo_root_node,
                edge_index: *index,
            };
            node_connections[node_connections_index + 1] = new_v_link.clone();
            self.cuckatoo_v_newest_node_connections.replace(self.cuckatoo_root_node, &new_v_link);
            
            // Check if both nodes have a pair
            if self.cuckatoo_u_newest_node_connections.contains(node ^ 1) && 
               self.cuckatoo_v_newest_node_connections.contains(self.cuckatoo_root_node ^ 1) {
                
                // Reset visited nodes
                self.cuckatoo_u_visited_node_pairs.clear();
                self.cuckatoo_v_visited_node_pairs.clear();
                
                // Go through all nodes in the cycle (matching C++ complex loop exactly)
                let mut cycle_size = 1u8;
                let mut current_node = node;
                let mut current_index = *index;
                
                loop {
                    // Set that node pair has been visited
                    self.cuckatoo_u_visited_node_pairs.set_unique((current_node >> 1) as u64, current_index);
                    
                    // Check if node's pair has more than one connection
                    if let Some(node_connection) = self.cuckatoo_u_newest_node_connections.get(current_node ^ 1) {
                        if node_connection.previous_node_connection_link.is_some() {
                            // Go through all of the node's pair's connections
                            let mut current_connection = node_connection;
                            loop {
                                // Check if the connected node's pair wasn't already visited
                                if !self.cuckatoo_v_visited_node_pairs.contains(((current_connection.node + 1) >> 1) as u64) {
                                    // Check if cycle is complete
                                    if ((current_connection.node + 1) ^ 1) == self.cuckatoo_root_node {
                                        // Check if cycle is a solution
                                        if cycle_size == SOLUTION_SIZE as u8 - 1 {
                                            // Get solution from visited nodes
                                            self.cuckatoo_u_visited_node_pairs.get_values(&mut solution[0..SOLUTION_SIZE/2]);
                                            self.cuckatoo_v_visited_node_pairs.get_values(&mut solution[SOLUTION_SIZE/2..SOLUTION_SIZE-1]);
                                            solution[SOLUTION_SIZE - 1] = current_connection.edge_index + 1;
                                            
                                            // Sort solution in ascending order
                                            solution.sort();
                                            
                                            return true;
                                        }
                                    }
                                    // Otherwise check if cycle could be as solution
                                    else if cycle_size != SOLUTION_SIZE as u8 - 1 {
                                        // Check if the connected node has a pair
                                        let has_pair = self.cuckatoo_v_newest_node_connections.contains((current_connection.node + 1) ^ 1);
                                        if has_pair {
                                            // Check if solution was found at the connected node's pair
                                            let next_node = (current_connection.node + 1) ^ 1;
                                            let next_index = current_connection.edge_index + 1;
                                            if self.search_node_connections_for_cuckatoo_solution_second_partition(
                                                cycle_size + 1,
                                                next_node,
                                                &next_index
                                            ) {
                                                // Get solution from visited nodes
                                                self.cuckatoo_u_visited_node_pairs.get_values(&mut solution[0..SOLUTION_SIZE/2]);
                                                self.cuckatoo_v_visited_node_pairs.get_values(&mut solution[SOLUTION_SIZE/2..SOLUTION_SIZE]);
                                                
                                                // Sort solution in ascending order
                                                solution.sort();
                                                
                                                return true;
                                            }
                                        }
                                    }
                                }
                                
                                // Move to previous connection
                                if let Some(ref prev) = current_connection.previous_node_connection_link {
                                    current_connection = prev;
                                } else {
                                    break;
                                }
                            }
                            
                            // Break
                            break;
                        }
                        
                        // Go to node's pair opposite end and get its edge index
                        current_index = node_connection.edge_index + 1;
                        current_node = node_connection.node + 1;
                        
                        // Check if node pair was already visited
                        if self.cuckatoo_v_visited_node_pairs.contains((current_node >> 1) as u64) {
                            break;
                        }
                        
                        // Check if cycle is complete
                        if (current_node ^ 1) == self.cuckatoo_root_node {
                            // Check if cycle is a solution
                            if cycle_size == SOLUTION_SIZE as u8 - 1 {
                                // Get solution from visited nodes
                                self.cuckatoo_u_visited_node_pairs.get_values(&mut solution[0..SOLUTION_SIZE/2]);
                                self.cuckatoo_v_visited_node_pairs.get_values(&mut solution[SOLUTION_SIZE/2..SOLUTION_SIZE-1]);
                                solution[SOLUTION_SIZE - 1] = current_index;
                                
                                // Sort solution in ascending order
                                solution.sort();
                                
                                return true;
                            }
                            
                            // Break
                            break;
                        }
                        
                        // Check if cycle isn't a solution
                        if cycle_size == SOLUTION_SIZE as u8 - 1 {
                            break;
                        }
                        
                        // Check if node doesn't have a pair
                        if !self.cuckatoo_v_newest_node_connections.contains(current_node ^ 1) {
                            break;
                        }
                        
                        // Set that node pair has been visited
                        self.cuckatoo_v_visited_node_pairs.set_unique((current_node >> 1) as u64, current_index);
                        
                        // Check if node's pair has more than one connection
                        if let Some(node_connection) = self.cuckatoo_v_newest_node_connections.get(current_node ^ 1) {
                            if node_connection.previous_node_connection_link.is_some() {
                                // Go through all of the node's pair's connections
                                let mut current_connection = node_connection;
                                loop {
                                    // Check if the connected node has a pair
                                    let has_pair = self.cuckatoo_u_newest_node_connections.contains((current_connection.node - 1) ^ 1);
                                    if has_pair {
                                        // Check if the connected node's pair wasn't already visited
                                        if !self.cuckatoo_u_visited_node_pairs.contains(((current_connection.node - 1) >> 1) as u64) {
                                            // Check if solution was found at the connected node's pair
                                            let next_node = (current_connection.node - 1) ^ 1;
                                            let next_index = current_connection.edge_index - 1;
                                            if self.search_node_connections_for_cuckatoo_solution_first_partition(
                                                cycle_size + 2, 
                                                next_node, 
                                                &next_index
                                            ) {
                                                // Get solution from visited nodes
                                                self.cuckatoo_u_visited_node_pairs.get_values(&mut solution[0..SOLUTION_SIZE/2]);
                                                self.cuckatoo_v_visited_node_pairs.get_values(&mut solution[SOLUTION_SIZE/2..SOLUTION_SIZE]);
                                                
                                                // Sort solution in ascending order
                                                solution.sort();
                                                
                                                return true;
                                            }
                                        }
                                    }
                                    
                                    // Move to previous connection
                                    if let Some(ref prev) = current_connection.previous_node_connection_link {
                                        current_connection = prev;
                                    } else {
                                        break;
                                    }
                                }
                                
                                // Break
                                break;
                            }
                            
                        // Go to node's pair opposite end and get its edge index
                        current_index = node_connection.edge_index - 1;
                        current_node = node_connection.node - 1;
                            
                            // Check if node pair was already visited
                            if self.cuckatoo_u_visited_node_pairs.contains((current_node >> 1) as u64) {
                                break;
                            }
                            
                            // Check if node doesn't have a pair
                            if !self.cuckatoo_u_newest_node_connections.contains(current_node ^ 1) {
                                break;
                            }
                        } else {
                            break;
                        }
                        
                        cycle_size += 2;
                    } else {
                        break;
                    }
                }
            }
            
            // Update indices (matching C++ exactly)
            node_connections_index += 2;
            edges_index += EDGE_NUMBER_OF_COMPONENTS;
        }
        
        false
    }
    
    /// Search node connections for cuckatoo solution first partition (matching C++ exactly)
    fn search_node_connections_for_cuckatoo_solution_first_partition(&mut self, cycle_size: u8, node: u32, index: &u32) -> bool {
        // Set that node pair has been visited
        self.cuckatoo_u_visited_node_pairs.set_unique((node >> 1) as u64, *index);
        
        // Go through all of the node's connections
        if let Some(node_connection) = self.cuckatoo_u_newest_node_connections.get(node) {
            let mut current_connection = node_connection;
            loop {
                // Check if the connected node's pair wasn't already visited
                if !self.cuckatoo_v_visited_node_pairs.contains(((current_connection.node + 1) >> 1) as u64) {
                    // Check if cycle is complete
                    if ((current_connection.node + 1) ^ 1) == self.cuckatoo_root_node {
                        // Check if cycle is a solution
                        if cycle_size == SOLUTION_SIZE as u8 - 1 {
                            // Set that the connected node's pair has been visited
                            self.cuckatoo_v_visited_node_pairs.set_unique(((current_connection.node + 1) >> 1) as u64, current_connection.edge_index + 1);
                            
                            return true;
                        }
                    }
                    // Otherwise check if cycle could be as solution
                    else if cycle_size != SOLUTION_SIZE as u8 - 1 {
                        // Check if the connected node has a pair
                        let has_pair = self.cuckatoo_v_newest_node_connections.contains((current_connection.node + 1) ^ 1);
                        if has_pair {
                            // Check if solution was found at the connected node's pair
                            let next_node = (current_connection.node + 1) ^ 1;
                            let next_index = current_connection.edge_index + 1;
                            if self.search_node_connections_for_cuckatoo_solution_second_partition(
                                cycle_size + 1,
                                next_node,
                                &next_index
                            ) {
                                return true;
                            }
                        }
                    }
                }
                
                // Move to previous connection
                if let Some(ref prev) = current_connection.previous_node_connection_link {
                    current_connection = prev;
                } else {
                    break;
                }
            }
        }
        
        // Set that node pair hasn't been visited (remove from visited)
        self.cuckatoo_u_visited_node_pairs.data.remove(&((node >> 1) as u64));
        
        false
    }
    
    /// Search node connections for cuckatoo solution second partition (matching C++ exactly)
    fn search_node_connections_for_cuckatoo_solution_second_partition(&mut self, cycle_size: u8, node: u32, index: &u32) -> bool {
        // Set that node pair has been visited
        self.cuckatoo_v_visited_node_pairs.set_unique((node >> 1) as u64, *index);
        
        // Go through all of the node's connections
        if let Some(node_connection) = self.cuckatoo_v_newest_node_connections.get(node) {
            let mut current_connection = node_connection;
            loop {
                // Check if the connected node's pair wasn't already visited
                if !self.cuckatoo_u_visited_node_pairs.contains(((current_connection.node - 1) >> 1) as u64) {
                    // Check if cycle is complete
                    if ((current_connection.node - 1) ^ 1) == self.cuckatoo_root_node {
                        // Check if cycle is a solution
                        if cycle_size == SOLUTION_SIZE as u8 - 1 {
                            // Set that the connected node's pair has been visited
                            self.cuckatoo_u_visited_node_pairs.set_unique(((current_connection.node - 1) >> 1) as u64, current_connection.edge_index - 1);
                            
                            return true;
                        }
                    }
                    // Otherwise check if cycle could be as solution
                    else if cycle_size != SOLUTION_SIZE as u8 - 1 {
                        // Check if the connected node has a pair
                        let has_pair = self.cuckatoo_u_newest_node_connections.contains((current_connection.node - 1) ^ 1);
                        if has_pair {
                            // Check if solution was found at the connected node's pair
                            let next_node = (current_connection.node - 1) ^ 1;
                            let next_index = current_connection.edge_index - 1;
                            if self.search_node_connections_for_cuckatoo_solution_first_partition(
                                cycle_size + 1, 
                                next_node, 
                                &next_index
                            ) {
                                return true;
                            }
                        }
                    }
                }
                
                // Move to previous connection
                if let Some(ref prev) = current_connection.previous_node_connection_link {
                    current_connection = prev;
                } else {
                    break;
                }
            }
        }
        
        // Set that node pair hasn't been visited (remove from visited)
        self.cuckatoo_v_visited_node_pairs.data.remove(&((node >> 1) as u64));
        
        false
    }
}

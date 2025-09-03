use crate::constants::get_cycle_length;

/// Edge in the Cuckatoo graph
#[derive(Debug, Clone, PartialEq)]
pub struct Edge {
    pub index: u32,
    pub u_node: u32,
    pub v_node: u32,
}

impl Edge {
    pub fn new(index: u32, u_node: u32, v_node: u32) -> Self {
        Self {
            index,
            u_node,
            v_node,
        }
    }
}

/// Node connection link for building adjacency lists
#[derive(Debug, Clone, PartialEq)]
pub struct NodeConnectionLink {
    pub node: u32,
    pub edge_index: u32,
}

impl NodeConnectionLink {
    pub fn new(node: u32, edge_index: u32) -> Self {
        Self { node, edge_index }
    }
}

/// Solution containing cycle edges
#[derive(Debug, Clone, PartialEq)]
pub struct Solution {
    pub edges: Vec<u32>,
}

impl Solution {
    pub fn new(edges: [u32; 42]) -> Self {
        Self {
            edges: edges.to_vec(),
        }
    }

    /// Create a solution with configurable cycle length
    pub fn with_cycle_length(edges: Vec<u32>) -> Self {
        let cycle_length = get_cycle_length();
        if edges.len() != cycle_length {
            panic!("Solution must have exactly {} edges, got {}", cycle_length, edges.len());
        }
        Self { edges }
    }

    /// Sort the edges for consistent representation
    pub fn sort(&mut self) {
        self.edges.sort();
    }

    /// Get the cycle length of this solution
    pub fn cycle_length(&self) -> usize {
        self.edges.len()
    }
}

/// Mining configuration
#[derive(Debug, Clone)]
pub struct MinerConfig {
    pub edge_bits: u32,
    pub mode: MiningMode,
    pub tuning: bool,
}

impl MinerConfig {
    pub fn new(edge_bits: u32, mode: MiningMode, tuning: bool) -> Self {
        Self {
            edge_bits,
            mode,
            tuning,
        }
    }
}

/// Mining mode
#[derive(Debug, Clone, PartialEq)]
pub enum MiningMode {
    Lean,
    Mean,
    Slean,
}

impl std::str::FromStr for MiningMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "lean" => Ok(MiningMode::Lean),
            "mean" => Ok(MiningMode::Mean),
            "slean" => Ok(MiningMode::Slean),
            _ => Err(format!("Unknown mining mode: {}", s)),
        }
    }
}

impl std::fmt::Display for MiningMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MiningMode::Lean => write!(f, "lean"),
            MiningMode::Mean => write!(f, "mean"),
            MiningMode::Slean => write!(f, "slean"),
        }
    }
}

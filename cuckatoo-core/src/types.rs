use crate::constants::*;

/// Represents an edge in the Cuckatoo graph
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

/// Represents a node connection link for cycle detection
#[derive(Debug, Clone)]
pub struct NodeConnectionLink {
    pub previous_link: Option<Box<NodeConnectionLink>>,
    pub node: u32,
    pub edge_index: u32,
}

impl NodeConnectionLink {
    pub fn new(node: u32, edge_index: u32) -> Self {
        Self {
            previous_link: None,
            node,
            edge_index,
        }
    }
}

/// Represents a solution found by the cycle verifier
#[derive(Debug, Clone)]
pub struct Solution {
    pub edges: [u32; SOLUTION_SIZE],
}

impl Solution {
    pub fn new(edges: [u32; SOLUTION_SIZE]) -> Self {
        Self { edges }
    }

    /// Sort the solution edges in ascending order
    pub fn sort(&mut self) {
        self.edges.sort();
    }
}

/// Configuration for the miner
#[derive(Debug, Clone)]
pub struct MinerConfig {
    pub edge_bits: u32,
    pub mode: MiningMode,
    pub tuning: bool,
}

/// Mining modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

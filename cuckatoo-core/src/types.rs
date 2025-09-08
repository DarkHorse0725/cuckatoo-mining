//! Core data types for Cuckatoo mining

use std::fmt;

// Constants matching C++ implementation
/// Solution size (42-cycle)
pub const SOLUTION_SIZE: usize = 42;

/// Edge number of components (C++ uses 3: [edge_index, node_u, node_v])
pub const EDGE_NUMBER_OF_COMPONENTS: usize = 3;

/// Edge in the Cuckatoo graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Edge {
    /// First node of the edge
    pub u: Node,
    /// Second node of the edge
    pub v: Node,
}

impl Edge {
    /// Create a new edge
    pub fn new(u: Node, v: Node) -> Self {
        Self { u, v }
    }
    
    /// Get the other node given one node
    pub fn other(&self, node: Node) -> Option<Node> {
        if self.u == node {
            Some(self.v)
        } else if self.v == node {
            Some(self.u)
        } else {
            None
        }
    }
    
    /// Check if this edge contains the given node
    pub fn contains(&self, node: Node) -> bool {
        self.u == node || self.v == node
    }
}

impl fmt::Display for Edge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.u, self.v)
    }
}

/// Node in the Cuckatoo graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Node(pub u64);

impl std::ops::BitXor<u64> for Node {
    type Output = Node;
    
    fn bitxor(self, rhs: u64) -> Self::Output {
        Node(self.0 ^ rhs)
    }
}

impl std::ops::BitXor<u64> for &Node {
    type Output = Node;
    
    fn bitxor(self, rhs: u64) -> Self::Output {
        Node(self.0 ^ rhs)
    }
}

impl Node {
    /// Create a new node
    pub fn new(value: u64) -> Self {
        Self(value)
    }
    
    /// Get the node value
    pub fn value(&self) -> u64 {
        self.0
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Header for mining (input to edge generation)
#[derive(Debug, Clone)]
pub struct Header {
    /// Header bytes
    pub bytes: Vec<u8>,
    /// Nonce for mining
    pub nonce: u64,
}

impl Header {
    /// Create a new header from bytes
    pub fn new(bytes: &[u8]) -> Self {
        Self { 
            bytes: bytes.to_vec(), 
            nonce: 0 
        }
    }
    
    /// Create a new header with bytes and nonce
    pub fn new_with_nonce(bytes: &[u8], nonce: u64) -> Self {
        Self { 
            bytes: bytes.to_vec(), 
            nonce 
        }
    }
    
    /// Get header bytes
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    
    /// Get header bytes as slice (alias for bytes)
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
    
    /// Get nonce
    pub fn nonce(&self) -> u64 {
        self.nonce
    }
}

/// Configuration for Cuckatoo mining
#[derive(Debug, Clone)]
pub struct Config {
    /// Number of edge bits (determines graph size)
    pub edge_bits: u32,
    /// Number of trimming rounds
    pub trimming_rounds: u32,
    /// Trimming mode
    pub mode: TrimmingMode,
    /// Whether to run in tuning mode (offline)
    pub tuning: bool,
}

impl Config {
    /// Create a new configuration
    pub fn new(edge_bits: u32) -> Self {
        Self {
            edge_bits,
            trimming_rounds: 90, // Default from C++ Makefile
            mode: TrimmingMode::Lean,
            tuning: false,
        }
    }
    
    /// Create a new configuration with C++ Makefile defaults
    pub fn new_cuckatoo31() -> Self {
        Self {
            edge_bits: 31, // From C++ Makefile: EDGE_BITS = 31
            trimming_rounds: 90, // From C++ Makefile: TRIMMING_ROUNDS = 90
            mode: TrimmingMode::Lean,
            tuning: false,
        }
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> Result<(), crate::CuckatooError> {
        if self.edge_bits < 10 || self.edge_bits > 32 {
            return Err(crate::CuckatooError::InvalidEdgeBits(self.edge_bits));
        }
        Ok(())
    }
    
    /// Calculate the number of edges based on edge bits
    pub fn edge_count(&self) -> u64 {
        1 << self.edge_bits
    }
    
    /// Calculate the number of nodes based on edge bits
    pub fn node_count(&self) -> u64 {
        1 << (self.edge_bits - 1)
    }
}

/// Trimming mode for edge trimming
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrimmingMode {
    /// Lean trimming (most memory efficient)
    Lean,
    /// Mean trimming (fastest)
    Mean,
    /// Slean trimming (balanced)
    Slean,
}

impl fmt::Display for TrimmingMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TrimmingMode::Lean => write!(f, "lean"),
            TrimmingMode::Mean => write!(f, "mean"),
            TrimmingMode::Slean => write!(f, "slean"),
        }
    }
}

impl std::str::FromStr for TrimmingMode {
    type Err = crate::CuckatooError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "lean" => Ok(TrimmingMode::Lean),
            "mean" => Ok(TrimmingMode::Mean),
            "slean" => Ok(TrimmingMode::Slean),
            _ => Err(crate::CuckatooError::InternalError(
                format!("Unknown trimming mode: {}", s)
            )),
        }
    }
}

/// Performance metrics for mining operations
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Time spent searching (CPU)
    pub searching_time: f64,
    /// Time spent trimming (GPU/CPU)
    pub trimming_time: f64,
    /// Total graphs processed
    pub graphs_processed: u64,
    /// Solutions found
    pub solutions_found: u64,
    /// Mining rate (graphs per second)
    pub mining_rate: f64,
    /// Nodes processed (for compatibility)
    pub nodes_processed: u64,
}

impl PerformanceMetrics {
    /// Create new performance metrics
    pub fn new() -> Self {
        Self {
            searching_time: 0.0,
            trimming_time: 0.0,
            graphs_processed: 0,
            solutions_found: 0,
            mining_rate: 0.0,
            nodes_processed: 0,
        }
    }
    
    /// Calculate total time
    pub fn total_time(&self) -> f64 {
        self.searching_time + self.trimming_time
    }
    
    /// Calculate efficiency ratio
    pub fn efficiency_ratio(&self) -> f64 {
        if self.trimming_time > 0.0 {
            self.searching_time / self.trimming_time
        } else {
            0.0
        }
    }
}

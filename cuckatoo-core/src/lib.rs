//! Cuckatoo Core - Core algorithms and data types for Cuckatoo mining
//! 
//! This crate provides the foundational algorithms for Cuckatoo cycle finding:
//! - Header to edge generation using SipHash-2-4
//! - Lean edge trimming with bitmap-based approach
//! - Cycle verification for 42-cycles
//! - Performance timing and benchmarking

pub mod types;
pub mod hashing;
pub mod blake2b;
pub mod trimming;
pub mod bitmap_trimming;
pub mod hash_cycle_finder;
// pub mod cpp_cycle_finder; // Temporarily disabled due to complex borrowing issues
pub mod exact_siphash;
pub mod exact_trimming;
pub mod verification;
pub mod timing;

pub use types::*;
pub use hashing::*;
pub use blake2b::*;
pub use trimming::*;
pub use bitmap_trimming::*;
pub use hash_cycle_finder::*;
pub use exact_siphash::*;
pub use exact_trimming::*;
pub use verification::*;
pub use timing::*;

/// Result type for Cuckatoo operations
pub type Result<T> = std::result::Result<T, CuckatooError>;

/// Main error type for Cuckatoo operations
#[derive(Debug)]
pub enum CuckatooError {
    InvalidEdgeBits(u32),
    HashingError(String),
    TrimmingError(String),
    VerificationError(String),
    MemoryError(String),
    InternalError(String),
}

impl std::fmt::Display for CuckatooError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CuckatooError::InvalidEdgeBits(bits) => write!(f, "Invalid edge bits: {}", bits),
            CuckatooError::HashingError(msg) => write!(f, "Hashing failed: {}", msg),
            CuckatooError::TrimmingError(msg) => write!(f, "Trimming failed: {}", msg),
            CuckatooError::VerificationError(msg) => write!(f, "Verification failed: {}", msg),
            CuckatooError::MemoryError(msg) => write!(f, "Memory allocation failed: {}", msg),
            CuckatooError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for CuckatooError {}


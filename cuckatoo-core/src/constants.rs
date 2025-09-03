use std::env;

/// Default cycle length for Cuckatoo (configurable at runtime)
pub const DEFAULT_CYCLE_LENGTH: usize = 42;

/// Get the cycle length from environment variable or use default
pub fn get_cycle_length() -> usize {
    env::var("CYCLE_LENGTH")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_CYCLE_LENGTH)
}

/// Solution size (configurable)
pub fn solution_size() -> usize {
    get_cycle_length()
}

/// Minimum edge bits (expanded range)
pub const MIN_EDGE_BITS: u32 = 4;

/// Maximum edge bits (expanded range)
pub const MAX_EDGE_BITS: u32 = 63;

/// Bitmap unit width in bits
pub const BITMAP_UNIT_WIDTH: usize = 64;

/// Number of bits in a byte
pub const BITS_IN_A_BYTE: usize = 8;

/// Edge number of components
pub const EDGE_NUMBER_OF_COMPONENTS: usize = 2;

/// SipHash keys size
pub const SIPHASH_KEYS_SIZE: usize = 16;

/// SipHash round rotation constants
pub const SIP_ROUND_ROTATION: [u32; 4] = [13, 16, 17, 21];

/// Calculate number of edges based on edge bits
pub fn number_of_edges(edge_bits: u32) -> u64 {
    1u64 << edge_bits
}

/// Calculate node mask based on edge bits
pub fn node_mask(edge_bits: u32) -> u32 {
    (1u32 << edge_bits) - 1
}

/// Calculate edges bitmap size based on edge bits
pub fn edges_bitmap_size(edge_bits: u32) -> usize {
    let edges_count = number_of_edges(edge_bits);
    ((edges_count + (BITMAP_UNIT_WIDTH as u64 - 1)) / BITMAP_UNIT_WIDTH as u64) as usize
}

/// Validate edge bits range
pub fn validate_edge_bits(edge_bits: u32) -> Result<(), String> {
    if edge_bits < MIN_EDGE_BITS || edge_bits > MAX_EDGE_BITS {
        Err(format!(
            "Edge bits must be between {} and {}, got {}",
            MIN_EDGE_BITS, MAX_EDGE_BITS, edge_bits
        ))
    } else {
        Ok(())
    }
}

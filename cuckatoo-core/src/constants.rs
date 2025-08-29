// Solution size for Cuckatoo cycle
pub const SOLUTION_SIZE: usize = 42;

// Edge bits range validation
pub const MIN_EDGE_BITS: u32 = 10;
pub const MAX_EDGE_BITS: u32 = 32;

// Bitmap unit width (64 bits)
pub const BITMAP_UNIT_WIDTH: usize = 64;

// Bits in a byte
pub const BITS_IN_A_BYTE: usize = 8;

// Edge number of components (index, u_node, v_node)
pub const EDGE_NUMBER_OF_COMPONENTS: usize = 3;

// SipHash constants
pub const SIPHASH_KEYS_SIZE: usize = 4;
pub const SIP_ROUND_ROTATION: u32 = 21;

/// Calculate the number of edges based on edge bits
pub fn number_of_edges(edge_bits: u32) -> u64 {
    1u64 << edge_bits
}

/// Calculate the node mask based on edge bits
pub fn node_mask(edge_bits: u32) -> u64 {
    number_of_edges(edge_bits) - 1
}

/// Calculate the edges bitmap size based on edge bits
pub fn edges_bitmap_size(edge_bits: u32) -> u64 {
    number_of_edges(edge_bits) / BITMAP_UNIT_WIDTH as u64
}

/// Validate edge bits
pub fn validate_edge_bits(edge_bits: u32) -> bool {
    edge_bits >= MIN_EDGE_BITS && edge_bits <= MAX_EDGE_BITS
}

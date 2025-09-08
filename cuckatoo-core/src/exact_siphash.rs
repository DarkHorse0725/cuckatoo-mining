//! Exact SipHash-2-4 implementation matching C++ reference miner
//! 
//! This implements the exact same SipHash algorithm as the C++ version,
//! including all the specific constants and operations.

use crate::Node;

/// Exact SipHash-2-4 implementation matching C++ version
pub struct ExactSipHash {
    /// SipHash keys (4 u64 values)
    keys: [u64; 4],
    /// Edge bits for node mask calculation
    edge_bits: u32,
}

impl ExactSipHash {
    /// Create new SipHash with keys
    pub fn new(keys: [u64; 4], edge_bits: u32) -> Self {
        Self { keys, edge_bits }
    }
    
    /// Get the SipHash keys
    pub fn get_keys(&self) -> [u64; 4] {
        self.keys
    }
    
    /// Hash a single nonce to get a node (exact C++ implementation)
    pub fn hash_nonce(&self, nonce: u64) -> Node {
        // Initialize states with keys (exactly like C++)
        let mut states = self.keys;
        
        // Perform hash on states (exactly like C++ siphash.h lines 42-50)
        states[3] ^= nonce;
        self.sip_round(&mut states);
        self.sip_round(&mut states);
        states[0] ^= nonce;
        states[2] ^= 255;
        self.sip_round(&mut states);
        self.sip_round(&mut states);
        self.sip_round(&mut states);
        self.sip_round(&mut states);
        
        // Get node from states (exactly like C++ siphash.h lines 52-63)
        let node_value = if self.edge_bits == 32 {
            // For EDGE_BITS == 32, no mask applied
            states[0] ^ states[1] ^ states[2] ^ states[3]
        } else {
            // For other edge bits, apply NODE_MASK
            let node_mask = (1u64 << self.edge_bits) - 1;
            (states[0] ^ states[1] ^ states[2] ^ states[3]) & node_mask
        };
        
        Node::new(node_value)
    }
    
    /// SipRound implementation (exactly matching C++ siphash.h lines 67-84)
    fn sip_round(&self, states: &mut [u64; 4]) {
        // Perform SipRound on states (exactly like C++)
        states[0] = states[0].wrapping_add(states[1]);
        states[2] = states[2].wrapping_add(states[3]);
        
        // Rotate states[1] left by 13 bits
        states[1] = (states[1] << 13) | (states[1] >> (64 - 13));
        
        // Rotate states[3] left by 16 bits  
        states[3] = (states[3] << 16) | (states[3] >> (64 - 16));
        
        states[1] ^= states[0];
        states[3] ^= states[2];
        
        // Rotate states[0] left by 32 bits
        states[0] = (states[0] << 32) | (states[0] >> (64 - 32));
        
        states[2] = states[2].wrapping_add(states[1]);
        states[0] = states[0].wrapping_add(states[3]);
        
        // Rotate states[1] left by 17 bits
        states[1] = (states[1] << 17) | (states[1] >> (64 - 17));
        
        // Rotate states[3] left by SIP_ROUND_ROTATION (21) bits
        states[3] = (states[3] << 21) | (states[3] >> (64 - 21));
        
        states[1] ^= states[2];
        states[3] ^= states[0];
        
        // Rotate states[2] left by 32 bits
        states[2] = (states[2] << 32) | (states[2] >> (64 - 32));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_siphash_basic() {
        let keys = [0x1234567890abcdef, 0xfedcba0987654321, 0x1111222233334444, 0x5555666677778888];
        let siphash = ExactSipHash::new(keys, 10);
        
        let node = siphash.hash_nonce(0);
        assert!(node.value() < (1u64 << 10)); // Should be within node mask
    }
    
    #[test]
    fn test_exact_siphash_consistency() {
        let keys = [0x1234567890abcdef, 0xfedcba0987654321, 0x1111222233334444, 0x5555666677778888];
        let siphash = ExactSipHash::new(keys, 10);
        
        let node1 = siphash.hash_nonce(42);
        let node2 = siphash.hash_nonce(42);
        
        assert_eq!(node1, node2); // Same input should produce same output
    }
    
    #[test]
    fn test_exact_siphash_different_nonces() {
        let keys = [0x1234567890abcdef, 0xfedcba0987654321, 0x1111222233334444, 0x5555666677778888];
        let siphash = ExactSipHash::new(keys, 10);
        
        let node1 = siphash.hash_nonce(0);
        let node2 = siphash.hash_nonce(1);
        
        assert_ne!(node1, node2); // Different inputs should produce different outputs
    }
}

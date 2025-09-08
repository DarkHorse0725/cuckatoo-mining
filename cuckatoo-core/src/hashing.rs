//! SipHash-2-4 implementation for Cuckatoo edge generation
//! Based on the C++ reference miner implementation

use crate::{Edge, Header, Node, Result, CuckatooError};
use crate::blake2b::blake2b;

/// SipHash-2-4 implementation for Cuckatoo
/// 
/// This implements the exact same hashing algorithm used in the C++ reference miner
/// to generate edges from headers and nonces.
pub struct SipHash {
    /// SipHash key (256-bit for Cuckatoo) - generated from Blake2b
    key: [u64; 4],
}

impl SipHash {
    /// Create a new SipHash instance with keys generated from header and nonce
    /// This matches the C++ implementation: blake2b(sipHashKeys, jobHeader, jobNonce)
    pub fn new_from_header(header: &Header, nonce: u64) -> Self {
        // Generate SipHash keys using Blake2b, exactly like C++ implementation
        let key = blake2b(header.as_bytes(), nonce);
        Self { key }
    }
    
    /// Create a new SipHash instance with custom key (for testing)
    pub fn with_key(key: [u64; 4]) -> Self {
        Self { key }
    }
    
    /// Get the SipHash key
    pub fn get_key(&self) -> [u64; 4] {
        self.key
    }
    
    /// Hash a header and nonce to generate edges
    /// 
    /// This generates 2^edge_bits edges using SipHash-2-4
    /// as specified in the Cuckatoo algorithm.
    pub fn hash_header(&self, _header: &Header, edge_bits: u32) -> Result<Vec<Edge>> {
        if edge_bits < 10 || edge_bits > 32 {
            return Err(CuckatooError::InvalidEdgeBits(edge_bits));
        }
        
        let edge_count = 1 << edge_bits;
        let node_mask = edge_count - 1;
        
        let mut edges = Vec::with_capacity(edge_count as usize);
        
        // Generate edges exactly like C++ implementation
        for edge_index in 0..edge_count {
            // Generate nodes using SipHash-2-4 with nonces (edge_index * 2) and (edge_index * 2 + 1)
            let nonce1 = edge_index * 2;
            let nonce2 = edge_index * 2 + 1;
            
            let u = self.siphash24(nonce1, edge_bits, node_mask);
            let v = self.siphash24(nonce2, edge_bits, node_mask);
            
            // Create edge connecting U and V partitions (preserve order like C++)
            let edge = Edge::new(u, v);
            edges.push(edge);
        }
        
        Ok(edges)
    }
    
    /// SipHash-2-4 implementation matching the C++ version exactly
    /// 
    /// This implements the same algorithm as the C++ sipHash24 function
    fn siphash24(&self, nonce: u64, edge_bits: u32, node_mask: u64) -> Node {
        // Initialize states with keys (like C++: states[i] += keys[i])
        let mut states = self.key;
        
        // Perform hash on states (exactly like C++ implementation)
        states[3] ^= nonce;
        self.sip_round(&mut states);
        self.sip_round(&mut states);
        states[0] ^= nonce;
        states[2] ^= 255;
        self.sip_round(&mut states);
        self.sip_round(&mut states);
        self.sip_round(&mut states);
        self.sip_round(&mut states);
        
        // Get node from states (like C++: *nodes = (states[0] ^ states[1] ^ states[2] ^ states[3]) & NODE_MASK)
        let node_value = if edge_bits == 32 {
            states[0] ^ states[1] ^ states[2] ^ states[3]
        } else {
            (states[0] ^ states[1] ^ states[2] ^ states[3]) & node_mask
        };
        
        Node::new(node_value)
    }
    
    /// SipRound implementation matching the C++ version exactly
    /// 
    /// This implements the same algorithm as the C++ sipRound function
    fn sip_round(&self, states: &mut [u64; 4]) {
        // Perform SipRound on states (exactly like C++ implementation)
        // C++: states[0] += states[1];
        states[0] = states[0].wrapping_add(states[1]);
        
        // C++: states[2] += states[3];
        states[2] = states[2].wrapping_add(states[3]);
        
        // C++: states[1] = (states[1] << 13) | (states[1] >> (64 - 13));
        states[1] = states[1].rotate_left(13);
        
        // C++: states[3] = (states[3] << 16) | (states[3] >> (64 - 16));
        states[3] = states[3].rotate_left(16);
        
        // C++: states[1] ^= states[0];
        states[1] ^= states[0];
        
        // C++: states[3] ^= states[2];
        states[3] ^= states[2];
        
        // C++: states[0] = (states[0] << 32) | (states[0] >> (64 - 32));
        states[0] = states[0].rotate_left(32);
        
        // C++: states[2] += states[1];
        states[2] = states[2].wrapping_add(states[1]);
        
        // C++: states[0] += states[3];
        states[0] = states[0].wrapping_add(states[3]);
        
        // C++: states[1] = (states[1] << 17) | (states[1] >> (64 - 17));
        states[1] = states[1].rotate_left(17);
        
        // C++: states[3] = (states[3] << SIP_ROUND_ROTATION) | (states[3] >> (64 - SIP_ROUND_ROTATION));
        // SIP_ROUND_ROTATION = 21
        states[3] = states[3].rotate_left(21);
        
        // C++: states[1] ^= states[2];
        states[1] ^= states[2];
        
        // C++: states[3] ^= states[0];
        states[3] ^= states[0];
        
        // C++: states[2] = (states[2] << 32) | (states[2] >> (64 - 32));
        states[2] = states[2].rotate_left(32);
    }
}

impl Default for SipHash {
    fn default() -> Self {
        // Default key for testing (should not be used in production)
        Self {
            key: [
                0x736f6d6570736575, 0x646f72616e646f6d,
                0x6c7967656e657261, 0x7465646279746573
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_siphash_basic() {
        let header = Header::new(b"test header");
        let siphash = SipHash::new_from_header(&header, 12345);
        
        // Test that we can generate edges
        let edges = siphash.hash_header(&header, 10).unwrap();
        assert_eq!(edges.len(), 1024); // 2^10
        
        // Test that edges have valid nodes
        for edge in &edges {
            assert!(edge.u.value() < 1024);
            assert!(edge.v.value() < 1024);
        }
    }
    
    #[test]
    fn test_siphash_consistency() {
        let header = Header::new(b"test header");
        let siphash1 = SipHash::new_from_header(&header, 12345);
        let siphash2 = SipHash::new_from_header(&header, 12345);
        
        // Same header and nonce should produce same keys
        assert_eq!(siphash1.get_key(), siphash2.get_key());
        
        // Same keys should produce same edges
        let edges1 = siphash1.hash_header(&header, 10).unwrap();
        let edges2 = siphash2.hash_header(&header, 10).unwrap();
        assert_eq!(edges1, edges2);
    }
    
    #[test]
    fn test_siphash_different_nonces() {
        let header = Header::new(b"test header");
        let siphash1 = SipHash::new_from_header(&header, 12345);
        let siphash2 = SipHash::new_from_header(&header, 12346);
        
        // Different nonces should produce different keys
        assert_ne!(siphash1.get_key(), siphash2.get_key());
        
        // Different keys should produce different edges
        let edges1 = siphash1.hash_header(&header, 10).unwrap();
        let edges2 = siphash2.hash_header(&header, 10).unwrap();
        assert_ne!(edges1, edges2);
    }
}
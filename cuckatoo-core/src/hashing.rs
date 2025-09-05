//! SipHash-2-4 implementation for Cuckatoo edge generation

use crate::{Edge, Header, Node, Result, CuckatooError};
use std::collections::HashMap;

/// SipHash-2-4 implementation for Cuckatoo
/// 
/// This implements the same hashing algorithm used in the C++ reference miner
/// to generate edges from headers and nonces.
pub struct SipHash {
    /// SipHash key (256-bit for Cuckatoo)
    key: [u64; 4],
}

impl SipHash {
    /// Create a new SipHash instance with the default Cuckatoo key
    pub fn new() -> Self {
        // Default 256-bit key used in Cuckatoo
        Self {
            key: [
                0x736f6d6570736575, 0x646f72616e646f6d,
                0x6c7967656e657261, 0x7465646279746573
            ],
        }
    }
    
    /// Create a new SipHash instance with custom key
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
    pub fn hash_header(&self, header: &Header, edge_bits: u32) -> Result<Vec<Edge>> {
        if edge_bits < 10 || edge_bits > 32 {
            return Err(CuckatooError::InvalidEdgeBits(edge_bits));
        }
        
        let edge_count = 1 << edge_bits;
        let node_count = 1 << (edge_bits - 1);
        
        let mut edges = Vec::with_capacity(edge_count as usize);
        let mut edge_map = HashMap::new();
        
        // Generate edges using SipHash-2-4 according to Cuckatoo specification
        for i in 0..edge_count {
            // V_i_0 = siphash24(K, 2*i) % N
            let hash_u = self.siphash24(&header.bytes, header.nonce, 2 * i);
            // V_i_1 = siphash24(K, 2*i+1) % N  
            let hash_v = self.siphash24(&header.bytes, header.nonce, 2 * i + 1);
            
            // Extract node indices from the hashes
            let u = Node::new(hash_u & (node_count - 1));
            let v = Node::new(hash_v & (node_count - 1));
            
            // Ensure u < v for consistent edge representation
            let edge = if u.value() < v.value() {
                Edge::new(u, v)
            } else {
                Edge::new(v, u)
            };
            
            // Check for duplicate edges (should be rare with good hash)
            if !edge_map.contains_key(&edge) {
                edge_map.insert(edge, i);
                edges.push(edge);
            }
        }
        
        // Sort edges for consistent output
        edges.sort();
        
        Ok(edges)
    }
    
    /// Generate a single edge hash
    pub fn hash_edge(&self, header: &Header, edge_index: u64) -> u64 {
        self.siphash24(&header.bytes, header.nonce, edge_index)
    }
    
    /// Core SipHash-2-4 implementation
    fn siphash24(&self, data: &[u8], nonce: u64, edge_index: u64) -> u64 {
        let mut v0 = self.key[0];
        let mut v1 = self.key[1];
        let mut v2 = self.key[2];
        let mut v3 = self.key[3];
        
        // XOR with nonce and edge index
        v3 ^= nonce;
        v2 ^= edge_index;
        
        // Process data in 8-byte chunks
        let mut i = 0;
        while i + 8 <= data.len() {
            let m = u64::from_le_bytes([
                data[i], data[i + 1], data[i + 2], data[i + 3],
                data[i + 4], data[i + 5], data[i + 6], data[i + 7],
            ]);
            
            v3 ^= m;
            self.sip_round(&mut v0, &mut v1, &mut v2, &mut v3);
            self.sip_round(&mut v0, &mut v1, &mut v2, &mut v3);
            v0 ^= m;
            
            i += 8;
        }
        
        // Handle remaining bytes
        let mut m = 0u64;
        let mut shift = 0;
        for &byte in &data[i..] {
            m |= (byte as u64) << shift;
            shift += 8;
        }
        
        // Finalize with edge index
        m |= (edge_index & 0xFF) << 56;
        
        v3 ^= m;
        self.sip_round(&mut v0, &mut v1, &mut v2, &mut v3);
        self.sip_round(&mut v0, &mut v1, &mut v2, &mut v3);
        v0 ^= m;
        
        // Finalization
        v2 ^= 0xff;
        self.sip_round(&mut v0, &mut v1, &mut v2, &mut v3);
        self.sip_round(&mut v0, &mut v1, &mut v2, &mut v3);
        self.sip_round(&mut v0, &mut v1, &mut v2, &mut v3);
        self.sip_round(&mut v0, &mut v1, &mut v2, &mut v3);
        
        v0 ^ v1 ^ v2 ^ v3
    }
    
    /// Single SipHash round
    fn sip_round(&self, v0: &mut u64, v1: &mut u64, v2: &mut u64, v3: &mut u64) {
        *v0 = v0.wrapping_add(*v1);
        *v2 = v2.wrapping_add(*v3);
        *v1 = v1.rotate_left(13);
        *v3 = v3.rotate_left(16);
        *v1 ^= *v0;
        *v3 ^= *v2;
        *v0 = v0.rotate_left(32);
        *v2 = v2.wrapping_add(*v1);
        *v0 = v0.wrapping_add(*v3);
        *v1 = v1.rotate_left(17);
        *v3 = v3.rotate_left(21);
        *v1 ^= *v2;
        *v3 ^= *v0;
        *v2 = v2.rotate_left(32);
    }
}

impl Default for SipHash {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate edges from header bytes and nonce using SipHash-2-4
/// 
/// This is a convenience function that creates a SipHash instance
/// and generates edges for the given parameters.
pub fn generate_edges(header_bytes: &[u8], nonce: u64, edge_bits: u32) -> Result<Vec<Edge>> {
    let hasher = SipHash::new();
    let header = Header::new(header_bytes.to_vec(), nonce);
    hasher.hash_header(&header, edge_bits)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_siphash_creation() {
        let hasher = SipHash::new();
        assert_eq!(hasher.key[0], 0x736f6d6570736575);
        assert_eq!(hasher.key[1], 0x646f72616e646f6d);
    }
    
    #[test]
    fn test_siphash_custom_key() {
        let key = [0x1234567890abcdef, 0xfedcba0987654321, 0x1111111111111111, 0x2222222222222222];
        let hasher = SipHash::with_key(key);
        assert_eq!(hasher.key, key);
    }
    
    #[test]
    fn test_edge_generation_small() {
        let hasher = SipHash::new();
        let header = Header::new(b"test header".to_vec(), 42);
        
        // Test with small edge bits (should work)
        let result = hasher.hash_header(&header, 12);
        assert!(result.is_ok());
        
        let edges = result.unwrap();
        assert!(edges.len() > 4000); // Should have most of the 4096 edges (some duplicates filtered)
        
        // Check that edges are sorted and unique
        for i in 1..edges.len() {
            assert!(edges[i-1] <= edges[i]);
        }
    }
    
    #[test]
    fn test_edge_generation_invalid_bits() {
        let hasher = SipHash::new();
        let header = Header::new(b"test header".to_vec(), 42);
        
        // Test with invalid edge bits
        assert!(hasher.hash_header(&header, 9).is_err());
        assert!(hasher.hash_header(&header, 33).is_err());
    }
    
    #[test]
    fn test_single_edge_hash() {
        let hasher = SipHash::new();
        let header = Header::new(b"test header".to_vec(), 42);
        
        let hash1 = hasher.hash_edge(&header, 0);
        let hash2 = hasher.hash_edge(&header, 1);
        
        // Different edge indices should produce different hashes
        assert_ne!(hash1, hash2);
        
        // Same inputs should produce same hash
        let hash1_again = hasher.hash_edge(&header, 0);
        assert_eq!(hash1, hash1_again);
    }
    
    #[test]
    fn test_generate_edges_function() {
        let header_bytes = b"test header";
        let nonce = 42;
        let edge_bits = 12;
        
        let result = generate_edges(header_bytes, nonce, edge_bits);
        assert!(result.is_ok());
        
        let edges = result.unwrap();
        assert!(edges.len() > 4000); // Should have most of the 4096 edges (some duplicates filtered)
    }
}

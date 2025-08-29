use crate::constants::*;
use crate::types::Edge;

/// SipHash-2-4 implementation for generating edges from nonces
pub struct SipHash {
    keys: [u64; SIPHASH_KEYS_SIZE],
}

impl SipHash {
    /// Create a new SipHash instance with the given keys
    pub fn new(keys: [u64; SIPHASH_KEYS_SIZE]) -> Self {
        Self { keys }
    }

    /// Generate nodes from nonces using SipHash-2-4
    pub fn hash_nodes(&self, nonces: &[u64], edge_bits: u32) -> Vec<u64> {
        let mut nodes = Vec::with_capacity(nonces.len());
        
        for &nonce in nonces {
            let node = self.siphash24_single(nonce, edge_bits);
            nodes.push(node);
        }
        
        nodes
    }

    /// Generate a single node from a nonce
    pub fn siphash24_single(&self, nonce: u64, edge_bits: u32) -> u64 {
        let mut states = self.keys;
        
        // XOR nonce into state[3]
        states[3] ^= nonce;
        
        // Perform SipRounds
        self.sip_round(&mut states);
        self.sip_round(&mut states);
        
        // XOR nonce into state[0]
        states[0] ^= nonce;
        
        // XOR 255 into state[2]
        states[2] ^= 255;
        
        // Perform 4 more SipRounds
        self.sip_round(&mut states);
        self.sip_round(&mut states);
        self.sip_round(&mut states);
        self.sip_round(&mut states);
        
        // Combine states
        let result = states[0] ^ states[1] ^ states[2] ^ states[3];
        
        // Apply node mask if not 32-bit
        if edge_bits != 32 {
            result & node_mask(edge_bits)
        } else {
            result
        }
    }

    /// Perform a single SipRound
    fn sip_round(&self, states: &mut [u64; SIPHASH_KEYS_SIZE]) {
        states[0] = states[0].wrapping_add(states[1]);
        states[2] = states[2].wrapping_add(states[3]);
        states[1] = states[1].rotate_left(13);
        states[3] = states[3].rotate_left(16);
        states[1] ^= states[0];
        states[3] ^= states[2];
        states[0] = states[0].rotate_left(32);
        states[2] = states[2].wrapping_add(states[1]);
        states[0] = states[0].wrapping_add(states[3]);
        states[1] = states[1].rotate_left(17);
        states[3] = states[3].rotate_left(SIP_ROUND_ROTATION);
        states[1] ^= states[2];
        states[3] ^= states[0];
        states[2] = states[2].rotate_left(32);
    }

    /// Generate edges from a header and nonces
    pub fn generate_edges(&self, header: &[u8], nonces: &[u64], edge_bits: u32) -> Vec<Edge> {
        let mut edges = Vec::with_capacity(nonces.len());
        
        for (i, &nonce) in nonces.iter().enumerate() {
            // Create a combined input for hashing
            let mut input = Vec::new();
            input.extend_from_slice(header);
            input.extend_from_slice(&nonce.to_le_bytes());
            
            // Generate u_node and v_node
            let u_node = self.siphash24_single(nonce, edge_bits);
            let v_node = self.siphash24_single(nonce.wrapping_add(1), edge_bits);
            
            edges.push(Edge::new(i as u32, u_node as u32, v_node as u32));
        }
        
        edges
    }
}

impl Default for SipHash {
    fn default() -> Self {
        // Default keys (should be replaced with actual keys from header)
        Self::new([0x0706050403020100, 0x0f0e0d0c0b0a0908, 0x1716151413121110, 0x1f1e1d1c1b1a1918])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_siphash_basic() {
        let keys = [0x0706050403020100, 0x0f0e0d0c0b0a0908, 0x1716151413121110, 0x1f1e1d1c1b1a1918];
        let siphash = SipHash::new(keys);
        
        let nonce = 0x123456789abcdef0;
        let node = siphash.siphash24_single(nonce, 16);
        
        // Basic test - should produce a valid node within the range
        assert!(node < (1u64 << 16));
    }

    #[test]
    fn test_edge_generation() {
        let siphash = SipHash::default();
        let header = b"test header";
        let nonces = vec![0, 1, 2, 3];
        
        let edges = siphash.generate_edges(header, &nonces, 12);
        
        assert_eq!(edges.len(), 4);
        for (i, edge) in edges.iter().enumerate() {
            assert_eq!(edge.index, i as u32);
            assert!(edge.u_node < (1u32 << 12));
            assert!(edge.v_node < (1u32 << 12));
        }
    }
}

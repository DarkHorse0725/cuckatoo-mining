use crate::constants::*;

/// A bitmap for efficient bit operations
pub struct Bitmap {
    buffer: Vec<u64>,
    size: u64,
}

impl Bitmap {
    /// Create a new bitmap with the specified size
    pub fn new(size: u64) -> Self {
        let buffer_size = (size + BITMAP_UNIT_WIDTH as u64 - 1) / BITMAP_UNIT_WIDTH as u64;
        Self {
            buffer: vec![0; buffer_size as usize],
            size,
        }
    }

    /// Set a bit at the specified index
    pub fn set_bit(&mut self, index: u64) {
        if index < self.size {
            let word_index = (index / BITMAP_UNIT_WIDTH as u64) as usize;
            let bit_index = (index % BITMAP_UNIT_WIDTH as u64) as u32;
            self.buffer[word_index] |= 1u64 << bit_index;
        }
    }

    /// Clear a bit at the specified index
    pub fn clear_bit(&mut self, index: u64) {
        if index < self.size {
            let word_index = (index / BITMAP_UNIT_WIDTH as u64) as usize;
            let bit_index = (index % BITMAP_UNIT_WIDTH as u64) as u32;
            self.buffer[word_index] &= !(1u64 << bit_index);
        }
    }

    /// Check if a bit is set at the specified index
    pub fn is_bit_set(&self, index: u64) -> bool {
        if index < self.size {
            let word_index = (index / BITMAP_UNIT_WIDTH as u64) as usize;
            let bit_index = (index % BITMAP_UNIT_WIDTH as u64) as u32;
            (self.buffer[word_index] & (1u64 << bit_index)) != 0
        } else {
            false
        }
    }

    /// Set all bits
    pub fn set_all_bits(&mut self) {
        for word in &mut self.buffer {
            *word = u64::MAX;
        }
    }

    /// Clear all bits
    pub fn clear_all_bits(&mut self) {
        for word in &mut self.buffer {
            *word = 0;
        }
    }

    /// Get the underlying buffer
    pub fn buffer(&self) -> &[u64] {
        &self.buffer
    }

    /// Get the mutable underlying buffer
    pub fn buffer_mut(&mut self) -> &mut [u64] {
        &mut self.buffer
    }

    /// Get the size of the bitmap
    pub fn size(&self) -> u64 {
        self.size
    }

    /// Count the number of set bits
    pub fn count_set_bits(&self) -> u64 {
        self.buffer.iter().map(|word| word.count_ones() as u64).sum()
    }
}

impl Default for Bitmap {
    fn default() -> Self {
        Self::new(0)
    }
}

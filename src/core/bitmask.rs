use std::cmp::min;

#[derive(Debug)]
pub struct BitMask {
    pub bits: Vec<u64>,
}

impl BitMask {
    fn to_index(&self, index: usize) -> (usize, usize) {
        (index >> 6, index & 63)
    }

    pub fn new(size: usize) -> Self {
        Self {
            bits: vec![0u64; size],
        }
    }

    pub fn from_vec(bits: Vec<u64>) -> Self {
        Self { bits }
    }

    pub fn from_slice(bits: &[u64]) -> Self {
        Self {
            bits: bits.to_vec(),
        }
    }

    pub fn as_slice(&self) -> &[u64] {
        &self.bits
    }

    fn assure_capacity(&mut self, index: usize) {
        if index >= self.bits.len() {
            self.bits.resize(index + 1, 0);
        }
    }

    pub fn size(&self) -> usize {
        self.bits.len()
    }

    #[inline(always)]
    pub fn is_set(&self, index: usize) -> bool {
        let (word_index, bit_index) = self.to_index(index);

        if word_index >= self.bits.len() {
            return false;
        }

        (self.bits[word_index] & (1u64 << bit_index)) != 0
    }
    #[inline(always)]
    pub fn set_bit(&mut self, index: usize) {
        let (word_index, bit_index) = self.to_index(index);
        self.assure_capacity(word_index);

        self.bits[word_index] |= 1u64 << bit_index;
    }
    #[inline(always)]
    pub fn skip(&self, pc: usize) -> Option<usize> {
        let word_index = (pc + 1) >> 6;
        let bit_index = (pc + 1) & 63;

        if word_index >= self.size() {
            return None;
        }
        let word = self.bits[word_index];
        let mask = word >> (bit_index);
        if mask != 0 {
            return Some(min(mask.trailing_zeros() as usize, 24));
        }
        if word_index + 1 >= self.size() {
            return None;
        }
        let this_word_remaining_zeros: usize = 64 - bit_index;
        let next_word = self.bits[word_index + 1];

        let next_word_trailing_zeros = next_word.trailing_zeros() as usize;
        return Some(min(
            this_word_remaining_zeros + next_word_trailing_zeros,
            24,
        ));
    }
}

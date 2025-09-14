use crate::consts::TERMINATION_INSTRUCTIONS;
use std::cmp::min;

#[inline(always)]
pub fn skip(bitmask: &[u64], pc: usize) -> Option<usize> {
    let word_index = (pc + 1) >> 6;
    let bit_index = (pc + 1) & 63;

    if word_index >= bitmask.len() {
        return None;
    }
    let word = bitmask[word_index];
    let mask = word >> (bit_index);
    if mask != 0 {
        return Some(min(mask.trailing_zeros() as usize, 24));
    }

    if word_index + 1 >= bitmask.len() {
        return None;
    }
    let this_word_remaining_zeros: usize = 64 - bit_index;
    let next_word = bitmask[word_index + 1];

    let next_word_trailing_zeros = next_word.trailing_zeros() as usize;
    return Some(min(this_word_remaining_zeros + next_word_trailing_zeros, 24));
}

pub fn build_start_set(program: &[u8], bitmask: &[u64]) -> Vec<u64> {
    // Determine how many u64 words we need to cover the program
    let num_words = (program.len() + 63) / 64;
    let mut start_set = vec![0u64; num_words];
    start_set[0] |= 1;

    let mut pos = 0;
    while pos < program.len() {
        // Check if current position is a set bit in the bitmask
        let word_index = pos >> 6;
        let bit_index = pos & 63;
        if (bitmask[word_index] & (1u64 << bit_index)) != 0 {
            let opcode = program[pos];
            if TERMINATION_INSTRUCTIONS[opcode as usize] {
                // Mark the next instruction after termination as jumpable
                if let Some(next_pos) = skip(bitmask, pos) {
                    let jump_pos = pos + 1 + next_pos;
                    let jump_word = jump_pos >> 6;
                    let jump_bit = jump_pos & 63;
                    if jump_word >= start_set.len() {
                        start_set.resize(jump_word + 1, 0);
                    }
                    start_set[jump_word] |= 1u64 << jump_bit;
                }
            }
        }

        if let Some(skip_count) = skip(bitmask, pos) {
            pos += 1 + skip_count;
        } else {
            break;
        }
    }

    start_set
}


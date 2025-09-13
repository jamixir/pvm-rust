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
    // Mask out bits before pc in the first word
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

        // Move to next set bit
        if let Some(skip_count) = skip(bitmask, pos) {
            pos += 1 + skip_count;
        } else {
            break;
        }
    }

    start_set
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decoder::{pack_bitmask_le, add_trap_bit};

    // Helper function to check if bit is set in start_set
    fn is_bit_set(start_set: &[u64], pos: usize) -> bool {
        let word_index = pos / 64;
        let bit_index = pos % 64;
        if word_index >= start_set.len() {
            return false;
        }
        (start_set[word_index] & (1u64 << bit_index)) != 0
    }

    #[test]
    fn test_skip_basic_patterns() {

        let bitmask = pack_bitmask_le(&[0b11]); //
        let skip_distance = skip(&bitmask, 0);
        assert_eq!(skip_distance, Some(0)); 


        let bitmask = pack_bitmask_le(&[0b1001]); 
        assert_eq!(skip(&bitmask, 0), Some(2));

        let bitmask = pack_bitmask_le(&[0b1,0b1 ]);  
        assert_eq!(skip(&bitmask, 0), Some(7)); 
    }

    #[test]
    fn test_skip_complex_instructions() {
        // Complex pattern: bit pattern 10010100 = 148
        // Bits 2, 4, 7 are set (positions 2, 4, 7 in little-endian)
        let bitmask = pack_bitmask_le(&[0b0101_0001, 0b10]); // 0b10010100
        
        // From position 0, next instruction is at position 2 (skip 1)
        assert_eq!(skip(&bitmask, 0), Some(3));
        
        // From position 2, next instruction is at position 4 (skip 1) 
        assert_eq!(skip(&bitmask, 4), Some(1));
        
        // From position 4, next instruction is at position 7 (skip 2)
        assert_eq!(skip(&bitmask, 6), Some(2));
        
        // From position 7, no more instructions
        assert_eq!(skip(&bitmask, 9), None);
    }

    #[test]
    fn test_skip_boundary_conditions() {
        // At end of bitmask - bit pattern: 10000000 = 128
        let bitmask = pack_bitmask_le(&[0b1]); // 0b10000000, only bit 7 set
        assert_eq!(skip(&bitmask, 7), None); // At end, no next instruction

        // Beyond bitmask length
        let bitmask = pack_bitmask_le(&[0b1]); // 0b10000000
        assert_eq!(skip(&bitmask, 8), None); // Beyond length

        // Empty bitmask
        let bitmask = pack_bitmask_le(&[]);
        assert_eq!(skip(&bitmask, 0), None);
    }



    #[test]
    fn test_build_start_set_simple_program() {
        // Program: [trap, fallthrough, store_imm_u8, imm1, imm2]
        let program = vec![0x00, 0x01, 0x3E, 0x01, 0x05];
        // Bitmask with instructions at positions 0, 1, 2 (bits 0, 1, 2 set = 0b00000111 = 7)
        let bitmask = pack_bitmask_le(&[7]); // 0b00000111
        
        let starts = build_start_set(&program, &bitmask);
        
        // Should have start at position 0 (always)
        assert!(is_bit_set(&starts, 0));
        
        // After trap (0x00), should mark next instruction as start
        assert!(is_bit_set(&starts, 1));
        
        // After fallthrough (0x01), should mark next instruction as start  
        assert!(is_bit_set(&starts, 2));
    }

    #[test]
    fn test_build_start_set_with_jumps_and_branches() {
        // Program: [jump, imm1, imm2, branch_eq, imm1, imm2]
        let program = vec![0x28, 0x01, 0x05, 0xAA, 0x01, 0x0A]; // 0x28=40=jump, 0xAA=170=branch_eq
        // Instructions at positions 0 and 3 (bits 0 and 3 set = 0b00001001 = 9)
        let mut bitmask = pack_bitmask_le(&[9]); // 0b00001001
        add_trap_bit(&mut bitmask, program.len());
        
        let starts = build_start_set(&program, &bitmask);
        
        // Should have start at position 0 (always)
        assert!(is_bit_set(&starts, 0));
        
        // After jump, should mark next instruction as start
        assert!(is_bit_set(&starts, 3));
        
        // After branch_eq, should mark next instruction as start
        assert!(is_bit_set(&starts, 6));
    }


    #[test]
    fn test_build_start_set_no_termination_instructions() {
        // Program with only non-termination instruction: store_imm_u8
        let program = vec![0x3E, 0x01, 0x05];
        let bitmask = pack_bitmask_le(&[1]); // Only bit 0 set = 0b1
        
        let starts = build_start_set(&program, &bitmask);
        
        // Should only have start at position 0
        assert!(is_bit_set(&starts, 0));
        
        // Should not have starts at other positions
        assert!(!is_bit_set(&starts, 1));
        assert!(!is_bit_set(&starts, 2));
    }

    #[test]
    fn test_build_start_set_multiple_termination_instructions() {
        // Program: [fallthrough, trap, jump, imm1, imm2]
        let program = vec![0x01, 0x00, 0x28, 0x01, 0x05]; // fallthrough, trap, jump
        let bitmask = pack_bitmask_le(&[7]); // Bits 0, 1, 2 set = 0b00000111 = 7
        
        let starts = build_start_set(&program, &bitmask);
        
        // Should have start at position 0 (always)
        assert!(is_bit_set(&starts, 0));
        
        // After fallthrough at pos 0, should mark pos 1 as start
        assert!(is_bit_set(&starts, 1));
        
        // After trap at pos 1, should mark pos 2 as start  
        assert!(is_bit_set(&starts, 2));

    }
}

use crate::start_set::StartSet;
use crate::vm::{InstructionResult, Vm};

impl Vm {
    pub fn next_counter(&self, exit_reason: &InstructionResult, prev_counter: usize) -> usize {
        match exit_reason {
            InstructionResult::Continue => {
                prev_counter + 1 + self.context.bitmask.skip(prev_counter).unwrap_or(0)
            }
            InstructionResult::Ecall { .. } => {
                prev_counter + 1 + self.context.bitmask.skip(prev_counter).unwrap_or(0)
            }
            InstructionResult::Jump(destination) => *destination,
            _ => prev_counter,
        }
    }
}

#[inline(always)]
pub fn decode_and_extend(program: &[u8], start_index: usize, count: usize) -> u64 {
    let mut x: u64 = 0;
    for i in 0..count {
        x |= (program[start_index + i] as u64) << (8 * i);
    }
    extend_to_64bit(x, count)
}

/// Extend value to 64-bit
#[inline(always)]
pub fn extend_to_64bit(x: u64, n: usize) -> u64 {
    let shift = 64u32.wrapping_sub((n * 8) as u32);
    ((x << shift) as i64 >> shift) as u64
}

pub fn decode_bytes(program: &[u8], start_index: usize, count: usize) -> u64 {
    let mut buf = [0u8; 8];
    unsafe {
        let src = program.as_ptr().add(start_index);
        std::ptr::copy_nonoverlapping(src, buf.as_mut_ptr(), count.min(8));
    }
    u64::from_le_bytes(buf)
}

pub fn unsigned_to_signed(x: u64, num_bytes: usize) -> i64 {
    let threshold = 1u64 << (8 * num_bytes - 1);
    if x < threshold {
        x as i64
    } else {
        x as i64 - (1i64 << (8 * num_bytes))
    }
}

pub fn branch(destination: usize, discriminator: bool, start_set: &StartSet) -> InstructionResult {
    match (discriminator, start_set.includes(destination)) {
        (true, true) => InstructionResult::Jump(destination),
        (true, false) => InstructionResult::Panic,
        (false, _) => InstructionResult::Continue,
    }
}

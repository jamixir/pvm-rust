pub mod execute_instruction;
pub mod single_step;
pub mod types;
pub mod utils;

pub use types::*;
use crate::memory::{Memory, MemoryRef};
use std::sync::Arc;

pub struct Vm {
    pub context: Arc<VmContext>,
    pub memory_ref: MemoryRef,
    pub memory: Option<Memory>,
    pub state: VmState,
}

impl Vm {
    pub fn new(
        context: Arc<VmContext>,
        state: VmState,
        memory_ref: MemoryRef,
        memory: Option<Memory>,
    ) -> Self {
        Self {
            context,
            memory_ref,
            memory,
            state,
        }
    }

    pub fn execute(&mut self) -> ExecutionResult {
        loop {
            let exit_reason = self.single_step_run();

            match exit_reason {
                StepResult::Continue if self.state.has_gas() => {
                    continue;
                }
                _ => {
                    return ExecutionResult::from(exit_reason);
                }
            }
        }
    }

    #[inline(always)]
    fn decode_and_extend(&self, start_index: usize, count: usize) -> u64 {
        if start_index + count as usize > self.context.program.len() {
            return 0; // Panic case
        }

        let bytes = &self.context.program[start_index..start_index + count as usize];
        let mut value = 0u64;

        // Decode little-endian
        for (i, &byte) in bytes.iter().enumerate() {
            value |= (byte as u64) << (i * 8);
        }

        // Sign extend to 64-bit
        self.extend_to_64bit(value, count)
    }

    /// Sign extend value to 64-bit
    fn extend_to_64bit(&self, x: u64, n: usize) -> u64 {
        if n == 0 {
            return x;
        }

        let shift = 8 * n - 1;
        if shift < 64 {
            let sign_bit = (x >> shift) & 1;
            let mask = if sign_bit != 0 { !0u64 } else { 0u64 };
            x | (mask << (8 * n))
        } else {
            x
        }
    }
}

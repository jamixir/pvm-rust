use crate::vm::types::SingleStepExit;
use crate::{utils::skip, vm::Vm};

impl Vm {
    pub fn skip(&self, pc: usize) -> Option<usize> {
        skip(&self.context.bitmask, pc)
    }

    pub fn next_counter(&self, exit_reason: &SingleStepExit, prev_counter: usize) -> usize {
        match exit_reason {
            SingleStepExit::Continue => {
                if self.state.pc != prev_counter {
                    self.state.pc
                } else {
                    prev_counter + 1 + self.skip(prev_counter).unwrap_or(0)
                }
            }
            SingleStepExit::Ecall { .. } => prev_counter + 1 + self.skip(prev_counter).unwrap_or(0),
            _ => prev_counter,
        }
    }
}

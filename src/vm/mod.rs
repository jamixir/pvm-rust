pub mod dispatchers;
pub mod execute_instruction;
pub mod instructions;
pub mod single_step;
pub mod types;
pub mod utils;

use crate::memory::{Memory, MemoryRef};
pub use dispatchers::*;
use std::sync::Arc;
pub use types::*;

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
}

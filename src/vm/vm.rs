use crate::{
    core::Memory,
    vm::{ExecutionResult, StepResult, VmContext, VmState},
};
use std::sync::Arc;

pub struct Vm {
    pub context: Arc<VmContext>,
    pub memory: Option<Memory>,
    pub state: VmState,
}

impl Vm {
    pub fn new(context: Arc<VmContext>, state: VmState, memory: Option<Memory>) -> Self {
        Self {
            context,
            memory,
            state,
        }
    }

    pub fn test_instance(context: Arc<VmContext>, state: VmState, memory: Option<Memory>) -> Self {
        Self::new(context, state, memory)
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

    pub fn single_step(&mut self) -> StepResult {
        self.single_step_run()
    }

    pub fn get_state(&self) -> &VmState {
        &self.state
    }

    pub fn get_state_mut(&mut self) -> &mut VmState {
        &mut self.state
    }

    pub fn get_memory(&self) -> Option<&Memory> {
        self.memory.as_ref()
    }

    pub fn get_memory_mut(&mut self) -> Option<&mut Memory> {
        self.memory.as_mut()
    }

    pub fn take_memory(&mut self) -> Option<Memory> {
        self.memory.take()
    }
}

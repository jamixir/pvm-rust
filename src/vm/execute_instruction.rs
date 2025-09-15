use crate::vm::types::StepResult;
use crate::utils::skip;
use crate::vm::{Vm, VmState};

impl Vm {
    pub fn execute_instruction(&self, opcode: u8) -> (StepResult, VmState) {
        match opcode {
            0x0A => self.execute_ecalli(),

            0x00 => (StepResult::Halt, self.state),

            // Unknown opcode - panic
            _ => (StepResult::Panic, self.state),
        }
    }

    fn execute_ecalli(&self) -> (StepResult, VmState) {
        // Decode immediate value
        let l_x = skip(&self.context.bitmask, self.state.pc)
            .unwrap_or(0)
            .min(4);
        let value = self.decode_and_extend(self.state.pc + 1, l_x);

        // For ecalli, we don't modify the PC - it will be handled by next_counter
        (
            StepResult::Ecall {
                call_id: value as u32,
            },
            self.state,
        )
    }
}

use crate::vm::types::SingleStepExit;
use crate::utils::skip;
use crate::vm::{Vm, VmState};

impl Vm {
    pub fn execute_instruction(&self, opcode: u8) -> (SingleStepExit, VmState) {
        match opcode {
            0x0A => self.execute_ecalli(),

            0x00 => (SingleStepExit::Halt, self.state),

            // Unknown opcode - panic
            _ => (SingleStepExit::Panic, self.state),
        }
    }

    fn execute_ecalli(&self) -> (SingleStepExit, VmState) {
        // Decode immediate value
        let l_x = skip(&self.context.bitmask, self.state.pc)
            .unwrap_or(0)
            .min(4);
        let value = self.decode_and_extend(self.state.pc + 1, l_x);

        // For ecalli, we don't modify the PC - it will be handled by next_counter
        (
            SingleStepExit::Ecall {
                call_id: value as u32,
            },
            self.state,
        )
    }
}

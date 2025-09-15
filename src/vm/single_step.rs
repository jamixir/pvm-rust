use crate::consts::GAS_COST;
use crate::vm::types::SingleStepExit;
use crate::vm::Vm;

impl Vm {
    pub fn single_step_run(&mut self) -> SingleStepExit {
        if self.state.pc >= self.context.program.len() {
            return SingleStepExit::Panic;
        }

        let opcode = self.context.program[self.state.pc];
        let prev_counter = self.state.pc;

        let (exit_reason, mut state_) = self.execute_instruction(opcode);

        state_.consume_gas(GAS_COST);

        self.state = state_;

        let next_counter = self.next_counter(&exit_reason, prev_counter);
        //  For halt/panic/fault,next_counter is the same as prev_counter
        self.state.pc = next_counter;

        exit_reason
    }
}

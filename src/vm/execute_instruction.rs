use crate::vm::{get_category, InstructionCategory};
use crate::vm::{InstructionResult, Vm};

impl Vm {
    #[inline(always)]
    pub fn execute_instruction(&mut self, opcode: u8) -> InstructionResult {
        match get_category(opcode) {
            InstructionCategory::NoArgs => self.dispatch_no_args(opcode),
            InstructionCategory::OneImmediate => self.dispatch_one_immediate(opcode),
            InstructionCategory::TwoImmediates => self.dispatch_two_immediates(opcode),
            InstructionCategory::RegImmediate => self.dispatch_reg_immediate(opcode),
            InstructionCategory::RegImmediateOffset => self.dispatch_reg_immediate_offset(opcode),

            InstructionCategory::Unknown => InstructionResult::Panic,

            _ => unimplemented!(),
        }
    }
}

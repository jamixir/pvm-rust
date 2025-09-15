use crate::vm::instructions::instruction_category::{get_category, InstructionCategory};
use crate::vm::{dispatch_no_args, dispatch_one_immediate, InstructionResult, Vm};

impl Vm {
    #[inline(always)]
    pub fn execute_instruction(&mut self, opcode: u8) -> InstructionResult {
        let category = get_category(opcode);

        match category {
            InstructionCategory::NoArgs => dispatch_no_args(opcode, self),

            InstructionCategory::OneImmediate => dispatch_one_immediate(opcode, self),

            InstructionCategory::Unknown => InstructionResult::Panic,

            _ => unimplemented!(),
        }
    }
}

use crate::vm::instructions::opcodes::*;
use crate::vm::{InstructionResult, Vm};

#[inline(always)]
pub fn dispatch_no_args(opcode: u8, _vm: &mut Vm) -> InstructionResult {
    match opcode {
        TRAP => InstructionResult::Panic,
        FALLTHROUGH => InstructionResult::Continue,
        _ => InstructionResult::Panic,
    }
}

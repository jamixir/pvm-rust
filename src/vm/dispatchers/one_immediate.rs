use crate::vm::instructions::opcodes::*;
use crate::vm::utils::{branch, decode_bytes, unsigned_to_signed};
use crate::vm::InstructionResult;
use crate::vm::{utils::decode_and_extend, Vm, VmContext, VmState};

#[inline(always)]
pub fn dispatch_one_immediate(
    opcode: u8,
    context: &VmContext,
    state: &mut VmState,
) -> InstructionResult {
    let VmContext {
        program,
        bitmask,
        start_set,
        ..
    } = context;

    match opcode {
        ECALLI => {
            let l_x = bitmask.skip(state.pc).min(4);
            let value = decode_and_extend(program.as_slice(), state.pc + 1, l_x as usize);
            InstructionResult::Ecall { call_id: value }
        }
        JUMP => {
            // Section (A.5.5) v0.7.0 - JUMP instruction
            // Formula (A.23) v0.7.0
            let l_x = bitmask.skip(state.pc).min(4);
            let shift_signed =
                unsigned_to_signed(decode_bytes(program.as_slice(), state.pc + 1, l_x), l_x);
            let v_x = (state.pc as i64 + shift_signed) as usize;
            branch(v_x, true, start_set)
        }
        _ => {
            // Unknown opcode in one-immediate category - panic
            InstructionResult::Panic
        }
    }
}

impl Vm {
    #[inline(always)]
    pub fn dispatch_one_immediate(&mut self, opcode: u8) -> InstructionResult {
        dispatch_one_immediate(opcode, &self.context, &mut self.state)
    }
}

use crate::vm::instructions::opcodes::*;
use crate::vm::utils::{decode_and_extend, decode_bytes, extend_to_64bit};
use crate::vm::{InstructionResult, Vm, VmContext};

const U8_MASK: u64 = 0xFF;
const U16_MASK: u64 = 0xFFFF;
const U32_MASK: u64 = 0xFFFF_FFFF;

#[inline(always)]
pub fn dispatch_reg_immediate(opcode: u8, vm: &mut Vm) -> InstructionResult {
    let (register_index, addr) = get_reg_immediate_params(&vm.context, vm.state.pc);

    match opcode {
        LOAD_IMM_64 => {
            let program = &vm.context.program;
            let pc = vm.state.pc;
            let register_index = (program[pc + 1] & 0x0F).min(12) as usize;
            let value = decode_bytes(program, pc + 2, 8);

            vm.state.registers.set(register_index, value);
            InstructionResult::Continue
        }
        JUMP_IND => {
            let register_value = vm.state.registers.get(register_index);
            let target = ((register_value + addr) & U32_MASK) as usize;
            vm.branch(target, true)
        }
        LOAD_IMM => {
            vm.state.registers.set(register_index, addr);
            InstructionResult::Continue
        }
        LOAD_U8 => match vm.read_memory(addr as usize, 1) {
            Ok(data) => {
                let value = data[0] as u64;
                vm.state.registers.set(register_index, value);
                InstructionResult::Continue
            }
            Err(crate::core::MemoryError::Fault { page_addr }) => {
                InstructionResult::Fault { page: page_addr }
            }
            Err(crate::core::MemoryError::Panic) => InstructionResult::Panic,
        },
        LOAD_I8 => match vm.read_memory(addr as usize, 1) {
            Ok(data) => {
                let value = data[0] as u64;
                let signed_value = extend_to_64bit(value, 1);
                vm.state.registers.set(register_index, signed_value);
                InstructionResult::Continue
            }
            Err(crate::core::MemoryError::Fault { page_addr }) => {
                InstructionResult::Fault { page: page_addr }
            }
            Err(crate::core::MemoryError::Panic) => InstructionResult::Panic,
        },
        LOAD_U16 => match vm.read_memory(addr as usize, 2) {
            Ok(data) => {
                let value = u16::from_le_bytes([data[0], data[1]]) as u64;
                vm.state.registers.set(register_index, value);
                InstructionResult::Continue
            }
            Err(crate::core::MemoryError::Fault { page_addr }) => {
                InstructionResult::Fault { page: page_addr }
            }
            Err(crate::core::MemoryError::Panic) => InstructionResult::Panic,
        },
        LOAD_I16 => match vm.read_memory(addr as usize, 2) {
            Ok(data) => {
                let value = u16::from_le_bytes([data[0], data[1]]) as u64;
                let signed_value = extend_to_64bit(value, 2);
                vm.state.registers.set(register_index, signed_value);
                InstructionResult::Continue
            }
            Err(crate::core::MemoryError::Fault { page_addr }) => {
                InstructionResult::Fault { page: page_addr }
            }
            Err(crate::core::MemoryError::Panic) => InstructionResult::Panic,
        },
        LOAD_U32 => match vm.read_memory(addr as usize, 4) {
            Ok(data) => {
                let value = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as u64;
                vm.state.registers.set(register_index, value);
                InstructionResult::Continue
            }
            Err(crate::core::MemoryError::Fault { page_addr }) => {
                InstructionResult::Fault { page: page_addr }
            }
            Err(crate::core::MemoryError::Panic) => InstructionResult::Panic,
        },
        LOAD_I32 => match vm.read_memory(addr as usize, 4) {
            Ok(data) => {
                let value = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as u64;
                let signed_value = extend_to_64bit(value, 4);
                vm.state.registers.set(register_index, signed_value);
                InstructionResult::Continue
            }
            Err(crate::core::MemoryError::Fault { page_addr }) => {
                InstructionResult::Fault { page: page_addr }
            }
            Err(crate::core::MemoryError::Panic) => InstructionResult::Panic,
        },
        LOAD_U64 => match vm.read_memory(addr as usize, 8) {
            Ok(data) => {
                let value = u64::from_le_bytes([
                    data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
                ]);
                vm.state.registers.set(register_index, value);
                InstructionResult::Continue
            }
            Err(crate::core::MemoryError::Fault { page_addr }) => {
                InstructionResult::Fault { page: page_addr }
            }
            Err(crate::core::MemoryError::Panic) => InstructionResult::Panic,
        },
        STORE_U8 => {
            let value = vm.state.registers.get(register_index);
            let stored_value = (value & U8_MASK) as u8;
            let data = &[stored_value];
            vm.write_memory(addr as usize, data)
        }
        STORE_U16 => {
            let value = vm.state.registers.get(register_index);
            let stored_value = (value & U16_MASK) as u16;
            let data = &stored_value.to_le_bytes()[..2];
            vm.write_memory(addr as usize, data)
        }
        STORE_U32 => {
            let value = vm.state.registers.get(register_index);
            let stored_value = (value & U32_MASK) as u32;
            let data = &stored_value.to_le_bytes()[..4];
            vm.write_memory(addr as usize, data)
        }
        STORE_U64 => {
            let value = vm.state.registers.get(register_index);
            let data = &value.to_le_bytes()[..8];
            vm.write_memory(addr as usize, data)
        }
        _ => InstructionResult::Panic,
    }
}

#[inline(always)]
fn get_reg_immediate_params(context: &VmContext, pc: usize) -> (usize, u64) {
    let program = &context.program;
    let bitmask = &context.bitmask;

    let register_index = (program[pc + 1] & 0x0F).min(12) as usize;
    let l_x = (bitmask.skip(pc) - 1).min(4) as usize;
    let addr = decode_and_extend(program, pc + 2, l_x);

    (register_index, addr)
}

impl Vm {
    #[inline(always)]
    pub fn dispatch_reg_immediate(&mut self, opcode: u8) -> InstructionResult {
        dispatch_reg_immediate(opcode, self)
    }
}

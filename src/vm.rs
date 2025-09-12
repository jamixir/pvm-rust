use crate::memory::{Memory, MemoryRef};

/// Result of executing a single instruction
#[derive(Debug)]
pub enum ExecutionResult {
    Continue,
    HostCall(u32),
    Halt,
    Error(&'static str),
}
pub struct Vm {
    pub registers: [u64; 13],
    pub program: Vec<u8>,
    pub bitmask: Vec<u64>,
    pub jump_table: Vec<u64>,
    pub pc: usize,
    pub gas: u64,
}

impl Vm {
    pub fn new(program: Vec<u8>, bitmask: Vec<u64>, jump_table: Vec<u64>, registers: [u64; 13], pc: usize, gas: u64) -> Self {
        Self {
            registers,
            program,
            bitmask,
            jump_table,
            pc,
            gas,
        }
    }
}

/// Execute the next instruction in the VM
pub fn execute_next_instruction(vm: &mut Vm, _memory_ref: MemoryRef) -> ExecutionResult {
    // Check if we have gas remaining
    if vm.gas == 0 {
        return ExecutionResult::Error("Out of gas");
    }

    // Check PC bounds
    if vm.pc >= vm.program.len() {
        return ExecutionResult::Halt;
    }

    // Consume gas for instruction execution
    vm.gas = vm.gas.saturating_sub(1);

    // Read instruction - for simplicity, assuming 4-byte instructions
    if vm.pc + 4 > vm.program.len() {
        return ExecutionResult::Error("Invalid instruction: PC out of bounds");
    }

    let instruction = u32::from_le_bytes([
        vm.program[vm.pc],
        vm.program[vm.pc + 1],
        vm.program[vm.pc + 2],
        vm.program[vm.pc + 3],
    ]);

    // Move PC forward by default (some instructions may override this)
    vm.pc += 4;

    // Decode instruction - simplified instruction format
    // For this example, let's assume:
    // - opcode in bits 0-6 (lower 7 bits)
    // - register/immediate data in remaining bits
    let opcode = instruction & 0x7F;

    match opcode {
        // HALT instruction
        0x00 => ExecutionResult::Halt,

        // ECALLI instruction - external call immediate
        // Format: ecalli host_id (host_id in bits 7-31)
        0x01 => {
            let host_id = (instruction >> 7) & 0xFFFFFF; // 24-bit host_id
            ExecutionResult::HostCall(host_id)
        }

        // ADD instruction (example) - add register values
        // Format: add rd, rs1, rs2 (simplified encoding)
        0x02 => {
            let rd = ((instruction >> 7) & 0x1F) as usize; // 5 bits for rd
            let rs1 = ((instruction >> 12) & 0x1F) as usize; // 5 bits for rs1
            let rs2 = ((instruction >> 17) & 0x1F) as usize; // 5 bits for rs2

            if rd < 13 && rs1 < 13 && rs2 < 13 {
                vm.registers[rd] = vm.registers[rs1].wrapping_add(vm.registers[rs2]);
                ExecutionResult::Continue
            } else {
                ExecutionResult::Error("Invalid register in ADD instruction")
            }
        }

        // LOAD immediate instruction - load immediate value to register
        // Format: li rd, imm (simplified)
        0x03 => {
            let rd = ((instruction >> 7) & 0x1F) as usize;
            let imm = (instruction >> 12) & 0xFFFFF; // 20-bit immediate

            if rd < 13 {
                vm.registers[rd] = imm as u64;
                ExecutionResult::Continue
            } else {
                ExecutionResult::Error("Invalid register in LI instruction")
            }
        }

        // Jump instruction - unconditional jump
        0x04 => {
            let offset = ((instruction >> 7) as i32) << 7 >> 7; // Sign extend 25-bit offset
            let new_pc = (vm.pc as i32 - 4).wrapping_add(offset * 4) as usize;

            if new_pc < vm.program.len() {
                vm.pc = new_pc;
                ExecutionResult::Continue
            } else {
                ExecutionResult::Error("Jump target out of bounds")
            }
        }

        // NOP instruction
        0x05 => ExecutionResult::Continue,

        // Unknown opcode
        _ => ExecutionResult::Error("Unknown instruction opcode"),
    }
}

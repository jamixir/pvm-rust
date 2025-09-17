use crate::core::{BitMask, Memory, Program, Registers, StartSet};
use crate::vm::{Vm, VmContext, VmState};
use std::sync::Arc;

pub struct VmTestBuilder {
    program: Option<Program>,
    bitmask: Option<BitMask>,
    registers: Option<Registers>,
    pc: usize,
    gas: u64,
    jump_table: Option<Vec<usize>>,
    memory_pages: Option<usize>,
    memory: Option<Memory>,
}

impl Default for VmTestBuilder {
    fn default() -> Self {
        Self {
            program: None,
            bitmask: None,
            registers: None,
            pc: 0,
            gas: 1000,
            jump_table: None,
            memory_pages: None,
            memory: None,
        }
    }
}

impl VmTestBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_program(mut self, program_bytes: Vec<u8>) -> Self {
        self.program = Some(Program::from_vec(program_bytes));
        self
    }

    pub fn with_bitmask_bytes(mut self, bitmask_bytes: &[u8], program_len: usize) -> Self {
        self.bitmask = Some(BitMask::from_bytes(bitmask_bytes, program_len));
        self
    }

    pub fn with_registers(mut self, registers: Registers) -> Self {
        self.registers = Some(registers);
        self
    }

    pub fn with_pc(mut self, pc: usize) -> Self {
        self.pc = pc;
        self
    }

    pub fn with_gas(mut self, gas: u64) -> Self {
        self.gas = gas;
        self
    }

    pub fn with_jump_table(mut self, jump_table: Vec<usize>) -> Self {
        self.jump_table = Some(jump_table);
        self
    }

    pub fn with_memory_pages(mut self, pages: usize) -> Self {
        self.memory_pages = Some(pages);
        self
    }

    pub fn build(self) -> Vm {
        let program = self.program.expect("Program is required");
        let program_len = program.len();

        let bitmask = self
            .bitmask
            .unwrap_or_else(|| BitMask::from_bytes(&[0b1], program_len));

        let start_set = StartSet::build(&program, &bitmask);

        let context = Arc::new(VmContext {
            program,
            bitmask,
            jump_table: self.jump_table.unwrap_or_default(),
            start_set,
        });

        let registers = self.registers.unwrap_or_else(Registers::new);
        let pc = self.pc;
        let gas = self.gas;
        let state = VmState::new(registers, pc, gas);

        let memory = self
            .memory
            .or_else(|| self.memory_pages.map(Memory::test_memory));

        Vm::new_for_test(context, state, memory)
    }

    pub fn simple(program_bytes: Vec<u8>, bitmask_bytes: &[u8]) -> Self {
        let program_len = program_bytes.len();
        Self::new()
            .with_program(program_bytes)
            .with_bitmask_bytes(bitmask_bytes, program_len)
            .with_memory_pages(33)
    }

    pub fn with_regs(program_bytes: Vec<u8>, bitmask_bytes: &[u8], registers: Registers) -> Self {
        let program_len = program_bytes.len();
        Self::new()
            .with_program(program_bytes)
            .with_bitmask_bytes(bitmask_bytes, program_len)
            .with_registers(registers)
            .with_memory_pages(33)
    }
}

pub const TEST_ADDR: u32 = 0x10E00;

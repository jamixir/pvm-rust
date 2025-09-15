use crate::registers::Registers;
use rustler::NifStruct;
#[derive(Debug, Clone, PartialEq)]
pub enum StepResult
 {
    Continue,
    Halt,
    Panic,
    Fault { page: usize },
    Ecall { call_id: u32 },
}

#[derive(Debug, Clone, NifStruct, Copy)]
#[module = "VmState"]
pub struct VmState {
    pub registers: Registers,
    pub pc: usize,
    pub gas: u64,
    pub overspent: u64,
}

impl VmState {
    pub fn new(registers: Registers, pc: usize, gas: u64) -> Self {
        Self {
            registers,
            pc,
            gas,
            overspent: 0,
        }
    }

    pub fn consume_gas(&mut self, amount: u64) {
        if amount > self.gas {
            let diff = amount - self.gas;
            self.gas = 0;
            self.overspent += diff;
        } else {
            self.gas -= amount;
        }
    }

    pub fn has_gas(&self) -> bool {
        self.overspent == 0
    }

    pub fn used_gas(&self, original_gas: u64) -> u64 {
        original_gas - self.gas + self.overspent
    }
}

#[derive(Debug)]
pub struct VmContext {
    pub program: Vec<u8>,
    pub bitmask: Vec<u64>,
    pub jump_table: Vec<u64>,
    pub start_set: Vec<u64>,
}

#[derive(Debug)]
pub enum ExecutionResult {
    Halt,
    Panic,
    OutOfGas,
    Fault { page: usize },
    HostCall { call_id: u32 },
}


impl From<StepResult> for ExecutionResult {
    fn from(exit_reason: StepResult) -> Self {
        match exit_reason {
            StepResult::Continue => ExecutionResult::OutOfGas,
            StepResult::Halt => ExecutionResult::Halt,
            StepResult::Panic => ExecutionResult::Panic,
            StepResult::Fault { page } => ExecutionResult::Fault { page },
            StepResult::Ecall { call_id } => ExecutionResult::HostCall { call_id },
        }
    }
}

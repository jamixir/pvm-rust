use crate::registers::Registers;
use rustler::NifStruct;
#[derive(Debug, Clone, PartialEq)]
pub enum SingleStepExit {
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
pub enum VmResult {
    Halt { state: VmState },
    Panic { state: VmState },
    OutOfGas { state: VmState },
    Fault { page: usize, state: VmState },
    HostCall { call_id: u32, state: VmState },
}

impl VmResult {
    pub fn state(&self) -> &VmState {
        match self {
            VmResult::Halt { state } => state,
            VmResult::Panic { state } => state,
            VmResult::OutOfGas { state } => state,
            VmResult::Fault { state, .. } => state,
            VmResult::HostCall { state, .. } => state,
        }
    }
}

impl From<(SingleStepExit, VmState)> for VmResult {
    fn from((exit_reason, state): (SingleStepExit, VmState)) -> Self {
        match exit_reason {
            SingleStepExit::Continue => VmResult::OutOfGas { state },
            SingleStepExit::Halt => VmResult::Halt {
                state: VmState { gas: 0, ..state },
            },
            SingleStepExit::Panic => VmResult::Panic {
                state: VmState { gas: 0, ..state },
            },
            SingleStepExit::Fault { page } => VmResult::Fault { page, state },
            SingleStepExit::Ecall { call_id } => VmResult::HostCall { call_id, state },
        }
    }
}

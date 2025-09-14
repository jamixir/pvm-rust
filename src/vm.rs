use crate::memory::{Memory, MemoryRef};
use rustler::NifStruct;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq)]
pub enum VmExitReason {
    Continue,
    Panic,
    Halt,
    OutOfGas,
    Fault { page: usize },
    Ecall { call_id: u32 },
}

#[derive(Debug, Clone, NifStruct)]
#[module = "VmState"]
pub struct VmState {
    pub registers: Vec<u64>, 
    pub pc: usize,
    pub gas: u64,
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
    Waiting { call_id: u32, state: VmState },
}

impl VmResult {
    pub fn state(&self) -> &VmState {
        match self {
            VmResult::Halt { state } => state,
            VmResult::Panic { state } => state,
            VmResult::OutOfGas { state } => state,
            VmResult::Fault { state, .. } => state,
            VmResult::Waiting { state, .. } => state,
        }
    }
}

pub struct Vm {
    pub context: Arc<VmContext>,
    pub memory_ref: MemoryRef,
    pub memory: Option<Memory>,
    pub state: VmState,
}

impl Vm {
    pub fn new(
        context: Arc<VmContext>,
        state: VmState,
        memory_ref: MemoryRef,
        memory: Option<Memory>,
    ) -> Self {
        Self {
            context,
            memory_ref,
            memory,
            state,
        }
    }


    pub fn execute(&mut self) -> VmResult {
        match self.vm_execute() {
            Ok(VmExitReason::Panic) => VmResult::Panic {
                state: self.state.clone(),
            },
            Ok(VmExitReason::Halt) => VmResult::Halt {
                state: self.state.clone(),
            },
            Ok(VmExitReason::OutOfGas) => VmResult::OutOfGas {
                state: self.state.clone(),
            },

            Err(VmExitReason::Fault { page }) => VmResult::Fault {
                page,
                state: self.state.clone(),
            },

            Err(VmExitReason::Ecall { call_id }) => {
                // Just return waiting - let the NIF layer handle message passing
                VmResult::Waiting {
                    call_id,
                    state: self.state.clone(),
                }
            }

            _ => unreachable!("Invalid VM execution result"),
        }
    }

    fn vm_execute(&mut self) -> Result<VmExitReason, VmExitReason> {
        loop {
            let exit_reason = self.single_step_run()?;

            match exit_reason {
                VmExitReason::Continue if self.state.gas > 0 => continue,

                VmExitReason::Continue => {
                    return Ok(VmExitReason::OutOfGas);
                }

                VmExitReason::Panic | VmExitReason::Halt => {
                    self.state.pc = 0;
                    return Ok(exit_reason);
                }

                other => {
                    return Err(other);
                }
            }
        }
    }

    fn single_step_run(&mut self) -> Result<VmExitReason, VmExitReason> {
        // This is where single instruction execution would happen
        // For now, implementing a basic version that matches the existing execute_next_instruction

        // Check if we have gas remaining
        if self.state.gas == 0 {
            return Ok(VmExitReason::OutOfGas);
        }

        // Check PC bounds
        if self.state.pc >= self.context.program.len() {
            return Ok(VmExitReason::Halt);
        }

        // Consume gas for instruction execution
        self.state.gas = self.state.gas.saturating_sub(1);

        // Read instruction - for simplicity, assuming 4-byte instructions
        if self.state.pc + 4 > self.context.program.len() {
            return Ok(VmExitReason::Panic);
        }

        let instruction = u32::from_le_bytes([
            self.context.program[self.state.pc],
            self.context.program[self.state.pc + 1],
            self.context.program[self.state.pc + 2],
            self.context.program[self.state.pc + 3],
        ]);

        // Move PC forward by default (some instructions may override this)
        self.state.pc += 4;

        // Decode instruction - simplified instruction format
        let opcode = instruction & 0x7F;

        match opcode {
            // HALT instruction
            0x00 => Ok(VmExitReason::Halt),

            // ECALLI instruction - external call immediate
            0x01 => {
                let call_id = (instruction >> 7) & 0xFFFFFF; // 24-bit call_id
                Err(VmExitReason::Ecall { call_id })
            }

            // ADD instruction (example)
            0x02 => {
                let rd = ((instruction >> 7) & 0x1F) as usize;
                let rs1 = ((instruction >> 12) & 0x1F) as usize;
                let rs2 = ((instruction >> 17) & 0x1F) as usize;

                if rd < 13 && rs1 < 13 && rs2 < 13 {
                    self.state.registers[rd] =
                        self.state.registers[rs1].wrapping_add(self.state.registers[rs2]);
                    Ok(VmExitReason::Continue)
                } else {
                    Ok(VmExitReason::Panic)
                }
            }

            // LOAD immediate instruction
            0x03 => {
                let rd = ((instruction >> 7) & 0x1F) as usize;
                let imm = (instruction >> 12) & 0xFFFFF;

                if rd < 13 {
                    self.state.registers[rd] = imm as u64;
                    Ok(VmExitReason::Continue)
                } else {
                    Ok(VmExitReason::Panic)
                }
            }

            // Jump instruction
            0x04 => {
                let offset = ((instruction >> 7) as i32) << 7 >> 7;
                let new_pc = (self.state.pc as i32 - 4).wrapping_add(offset * 4) as usize;

                if new_pc < self.context.program.len() {
                    self.state.pc = new_pc;
                    Ok(VmExitReason::Continue)
                } else {
                    Ok(VmExitReason::Panic)
                }
            }

            // NOP instruction
            0x05 => Ok(VmExitReason::Continue),

            // Unknown opcode
            _ => Ok(VmExitReason::Panic),
        }
    }
}

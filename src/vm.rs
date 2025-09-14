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

#[derive(Debug, Clone, PartialEq)]
pub enum ExitReason {
    Panic,
    Halt,
    OutOfGas,
    Fault { page: usize },
}

pub type ExecutionResult<T> = Result<T, VmExitReason>;

/// VM state that can be passed around and updated
#[derive(Debug, Clone, NifStruct)]
#[module = "VmState"]
pub struct VmState {
    pub registers: Vec<u64>, // Must be Vec for NIF serialization
    pub pc: usize,
    pub gas: u64,
}

/// Static VM context - contains data that doesn't change during execution
#[derive(Debug)]
pub struct VmContext {
    pub program: Vec<u8>,  // Owned data for safe storage across NIF calls
    pub bitmask: Vec<u64>, // These are small, can move
    pub jump_table: Vec<u64>,
    pub start_set: Vec<u64>,
}

/// Result from VM execution - used for pause/resume functionality
#[derive(Debug)]
pub enum VmResult {
    Halted {
        final_state: VmState,
    },
    Panic {
        final_state: VmState,
    },
    OutOfGas {
        final_state: VmState,
    },
    Fault {
        page: usize,
        final_state: VmState,
    },
    Waiting {
        call_id: u32,
        current_state: VmState,
    },
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

    /// Main execution function - runs until completion or ecalli instruction
    /// Returns VmResult indicating the reason for stopping
    pub fn execute(&mut self) -> VmResult {
        match self.vm_execute() {
            Ok(VmExitReason::Panic) => VmResult::Panic {
                final_state: self.state.clone(),
            },
            Ok(VmExitReason::Halt) => VmResult::Halted {
                final_state: self.state.clone(),
            },
            Ok(VmExitReason::OutOfGas) => VmResult::OutOfGas {
                final_state: self.state.clone(),
            },

            Err(VmExitReason::Fault { page }) => VmResult::Fault {
                page,
                final_state: self.state.clone(),
            },

            Err(VmExitReason::Ecall { call_id }) => {
                // Just return waiting - let the NIF layer handle message passing
                VmResult::Waiting {
                    call_id,
                    current_state: self.state.clone(),
                }
            }

            _ => unreachable!("Invalid VM execution result"),
        }
    }

    fn vm_execute(&mut self) -> ExecutionResult<VmExitReason> {
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

    fn single_step_run(&mut self) -> ExecutionResult<VmExitReason> {
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

#![cfg_attr(not(feature = "std"), no_std)]

pub mod core;
pub mod encoding;
pub mod execution;
pub mod init;
pub mod vm;

pub use core::{
    BitMask, Memory, MemoryBuilder, MemoryError, Permission, Program, Registers, StartSet,
};

pub use vm::{ExecutionResult, InstructionResult, StepResult, Vm, VmContext, VmState};

pub use encoding::{deblob, encode_program, Deblob, DecodeError};

pub use init::{construct_memory, initialize_program, parse_program_segments, ProgramSegments};

pub use execution::{execute_disassembled, execute_linked, ExecutionOutput};

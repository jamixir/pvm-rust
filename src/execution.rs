use crate::{
    core::{Memory, Registers},
    deblob,
    init::initialize_program,
    vm::{ExecutionResult, Vm, VmContext, VmState},
    StartSet,
};
use std::sync::Arc;
use std::env;
use crate::vm::tracer::Tracer;

#[derive(Debug)]
pub struct ExecutionOutput {
    pub result: ExecutionResult,
    pub final_state: VmState,
    pub final_memory: Option<Memory>,
}

pub fn execute_linked(
    bytecode: &[u8],
    args: &[u8],
    pc: usize,
    gas: u64,
) -> Option<ExecutionOutput> {
    let (code, registers, memory) = initialize_program(bytecode, args)?;

    let deblob_result = match deblob(&code) {
        Ok(result) => result,
        Err(_) => return None,
    };
    let start_set = StartSet::build(&deblob_result.program, &deblob_result.bitmask);

    let tracer = if env::var("PVM_TRACE").map(|v| v == "true").unwrap_or(false) {
        Some(Tracer::new())
    } else {
        None
    };

    let context = Arc::new(VmContext {
        program: deblob_result.program,
        bitmask: deblob_result.bitmask,
        jump_table: deblob_result.jump_table,
        start_set,
        tracer,
    });

    let state = VmState::new(registers, pc, gas);
    let mut vm = Vm::new(context, state, Some(memory));

    let result = vm.execute();
    let final_state = vm.get_state().clone();
    let final_memory = vm.take_memory();

    Some(ExecutionOutput {
        result,
        final_state,
        final_memory,
    })
}

pub fn execute_disassembled(
    code: &[u8],
    custom_registers: Option<Registers>,
    custom_memory: Option<Memory>,
    pc: usize,
    gas: u64,
) -> ExecutionOutput {
    let deblob_result = deblob(code).expect("Failed to decode program");
    let start_set = StartSet::build(&deblob_result.program, &deblob_result.bitmask);

    let tracer = if env::var("PVM_TRACE").map(|v| v == "true").unwrap_or(false) {
        Some(Tracer::new())
    } else {
        None
    };

    let context = Arc::new(VmContext {
        program: deblob_result.program,
        bitmask: deblob_result.bitmask,
        jump_table: deblob_result.jump_table,
        start_set,
        tracer,
    });

    let registers = custom_registers.unwrap_or_else(|| Registers::default());

    let memory = custom_memory.or_else(|| Some(Memory::default()));

    let state = VmState::new(registers, pc, gas);
    let mut vm = Vm::new(context, state, memory);

    let result = vm.execute();
    let final_state = vm.get_state().clone();
    let final_memory = vm.take_memory();

    ExecutionOutput {
        result,
        final_state,
        final_memory,
    }
}

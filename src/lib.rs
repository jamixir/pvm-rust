pub mod atoms;
pub mod consts;
pub mod decoder;
pub mod encoder;
pub mod errors;
pub mod init_program;
pub mod memory;
pub mod types;
pub mod utils;
pub mod vm;

use rustler::{Binary, Decoder, Encoder, Env, NifResult, OwnedEnv, Term};
use std::sync::{Arc, OnceLock};

use crate::{
    decoder::{deblob, Deblob},
    errors::{to_rustler_error, ToNifResult},
    init_program::initialize_program,
    memory::{get_owned, put_owned, MemoryRef, MemoryResource},
    types::host_result::{ExecuteResult, HostOutput},
    utils::build_start_set,
    vm::{Vm, VmContext, VmResult, VmState},
};

static VM_CONTEXT: OnceLock<Arc<VmContext>> = OnceLock::new();

rustler::init!("Elixir.Pvm.Native", load = load);
fn load(env: Env, _info: Term) -> bool {
    env.register::<MemoryResource>().is_ok()
}

fn execute_vm<'a>(env: Env<'a>, mut vm: Vm) -> NifResult<ExecuteResult> {
    let result = vm.execute();
    match result {
        VmResult::Waiting { call_id, state } => {
            // Memory dance: move memory from Option to ref before pausing
            if let Some(memory) = vm.memory.take() {
                let _ = put_owned(&vm.memory_ref, memory).to_nif()?;
            }

            // Send message to Elixir with VmState + memory_ref
            let mut owned_env = OwnedEnv::new();
            let memory_ref = vm.memory_ref.clone();
            let used_gas = vm.state.gas.saturating_sub(state.gas);

            if let Err(_) = owned_env.send_and_clear(&env.pid(), move |env| {
                (atoms::ecall(), call_id, state, memory_ref).encode(env)
            }) {
                return Err(to_rustler_error!(atoms::send_failed()));
            }

            Ok(ExecuteResult {
                used_gas,
                output: HostOutput::Atom(atoms::waiting()),
            })
        }
        _ => {
            let used_gas = vm.state.gas.saturating_sub(result.state().gas);
            let output = match result {
                VmResult::Halt { state } => {
                    let start = state.registers[7] as usize;
                    let len = state.registers[8] as usize;
                    match vm
                        .memory
                        .as_ref()
                        .expect("Memory not available")
                        .read(start, len)
                    {
                        Ok(bytes) => HostOutput::Bytes(bytes.to_vec()),
                        Err(_) => HostOutput::Bytes(vec![]),
                    }
                }
                VmResult::Panic { .. } => HostOutput::Atom(atoms::panic()),
                VmResult::OutOfGas { .. } => HostOutput::Atom(atoms::out_of_gas()),
                VmResult::Fault { .. } => HostOutput::Atom(atoms::panic()),
                VmResult::Waiting { .. } => HostOutput::Atom(atoms::waiting()),
            };

            Ok(ExecuteResult { used_gas, output })
        }
    }
}

#[rustler::nif(schedule = "DirtyCpu")]
fn execute<'a>(
    env: Env<'a>,
    program_term: Term<'a>,
    pc_term: Term<'a>,
    gas_term: Term<'a>,
    args_term: Term<'a>,
    memory_ref_term: Term<'a>,
) -> NifResult<ExecuteResult> {
    let linked_program: Binary<'a> = Binary::decode(program_term)?;
    let pc: usize = usize::decode(pc_term)?;
    let gas: u64 = u64::decode(gas_term)?;
    let args: Binary<'a> = Binary::decode(args_term)?;
    let memory_ref: MemoryRef = MemoryRef::decode(memory_ref_term)?;

    let (code, registers, memory) = match initialize_program(&linked_program, &args) {
        None => {
            return Ok(ExecuteResult {
                used_gas: 0,
                output: HostOutput::Atom(atoms::panic()),
            });
        }
        Some(v) => v,
    };

    let Deblob {
        program,
        bitmask,
        jump_table,
    } = deblob(code);
    let start_set = build_start_set(&program, &bitmask);

    let context = Arc::new(VmContext {
        program,
        bitmask,
        jump_table,
        start_set,
    });
    // Store context globally for resume NIF to access
    let _ = VM_CONTEXT.set(context.clone());

    let state = VmState {
        registers: registers.to_vec(),
        pc,
        gas,
    };

    let vm = Vm::new(context, state, memory_ref.clone(), Some(memory));

    execute_vm(env, vm)
}

#[rustler::nif(schedule = "DirtyCpu")]
fn resume<'a>(
    env: Env<'a>,
    new_state_term: Term<'a>,
    memory_ref_term: Term<'a>,
) -> NifResult<ExecuteResult> {
    let new_state: VmState = VmState::decode(new_state_term)?;
    let memory_ref: MemoryRef = MemoryRef::decode(memory_ref_term)?;

    // Get stored context from global storage
    let context = VM_CONTEXT
        .get()
        .ok_or(to_rustler_error!(atoms::no_vm_context()))?
        .clone();

    let memory = get_owned(&memory_ref).to_nif()?;

    let vm = Vm::new(context, new_state, memory_ref, memory);

    execute_vm(env, vm)
}

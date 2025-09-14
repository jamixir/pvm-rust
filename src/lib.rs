use std::sync::{Arc, OnceLock};

use rustler::{Binary, Decoder, Encoder, Env, NifResult, OwnedEnv, Term};

use crate::decoder::{deblob, Deblob};
use crate::init_program::initialize_program;
use crate::memory::{memory_put, memory_take, MemoryRef, MemoryResource};
use crate::utils::build_start_set;
use crate::vm::{Vm, VmContext, VmResult, VmState};
mod consts;
mod decoder;
mod encoder;
mod init_program;
mod memory;
mod utils;
mod vm;

static VM_CONTEXT: OnceLock<Arc<VmContext>> = OnceLock::new();

rustler::init!("Elixir.Pvm.Native", load = load);
fn load(env: Env, _info: Term) -> bool {
    env.register::<MemoryResource>().is_ok()
}

fn execute_vm<'a>(env: Env<'a>, mut vm: Vm) -> NifResult<Term<'a>> {
    match vm.execute() {
        VmResult::Halted { final_state } => Ok(("halted".to_string(), final_state).encode(env)),
        VmResult::Panic { final_state } => Ok(("panic".to_string(), final_state).encode(env)),
        VmResult::OutOfGas { final_state } => {
            Ok(("out_of_gas".to_string(), final_state).encode(env))
        }
        VmResult::Fault { page, final_state } => {
            Ok(("fault".to_string(), page, final_state).encode(env))
        }
        VmResult::Waiting {
            call_id,
            current_state,
        } => {
            // Memory dance: move memory from Option to ref before pausing
            if let Some(memory) = vm.memory.take() {
                let _ = memory_put(&vm.memory_ref, memory)?;
            }

            // Send message to Elixir with VmState + memory_ref
            let mut owned_env = OwnedEnv::new();
            let pid = env.pid();
            let memory_ref = vm.memory_ref.clone();

            if let Err(_) = owned_env.send_and_clear(&pid, move |env| {
                ("ecall".to_string(), call_id, current_state, memory_ref).encode(env)
            }) {
                return Err(rustler::Error::Atom("send_failed"));
            }

            // Return waiting status - Elixir will call resume with updated state
            Ok(("waiting".to_string()).encode(env))
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
) -> NifResult<Term<'a>> {
    let linked_program: Binary<'a> = Binary::decode(program_term)?;
    let pc: usize = usize::decode(pc_term)?;
    let gas: u64 = u64::decode(gas_term)?;
    let args: Binary<'a> = Binary::decode(args_term)?;
    let memory_ref: MemoryRef = MemoryRef::decode(memory_ref_term)?;

    let (code, registers, memory) = match initialize_program(&linked_program, &args) {
        None => {
            // Return {0, :panic, context} equivalent - for now just return {0, :panic}
            return Ok((0u64, "panic").encode(env));
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
) -> NifResult<Term<'a>> {
    let new_state: VmState = VmState::decode(new_state_term)?;
    let memory_ref: MemoryRef = MemoryRef::decode(memory_ref_term)?;

    // Get stored context from global storage
    let context = VM_CONTEXT
        .get()
        .ok_or(rustler::Error::Atom("no_vm_context"))?
        .clone();

    let memory = memory_take(&memory_ref)?;

    let vm = Vm::new(context, new_state, memory_ref, memory);

    execute_vm(env, vm)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Helper function to convert hex string to bytes
    fn hex_to_bytes(hex: &str) -> Vec<u8> {
        (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect()
    }

    /// Helper function to create test arguments
    fn create_test_args() -> Vec<u8> {
        vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]
    }

    fn test_memory_regions(memory: &mut crate::memory::Memory, program_bytes: &[u8], args: &[u8]) {
        use crate::consts::{LAYOUT_BUFFER_SIZE, MAX_ADDR, MIN_ADDR, PAGE_SIZE};
        use crate::memory::MemoryError;

        //  Zero length read works everywhere
        assert!(memory.read(100, 0).is_ok());
        assert_eq!(memory.read(0100, 0).unwrap().len(), 0);

        assert_eq!(
            memory.read(100, 10),
            Err(MemoryError::Panic),
            "Access below MIN_ADDR should fail"
        );

        // Program text region - should be readable
        let prog_start = MIN_ADDR;
        let Ok(prog_data) = memory.read(prog_start, 100) else {
            panic!("Should be able to read program text at MIN_ADDR");
        };
        assert_eq!(
            prog_data.len(),
            100,
            "Should read exactly 100 bytes from program text"
        );

        // Verify we're actually reading the program text data
        // The program text should match the first part of our parsed program
        // (after the header which is 11 + o_size + w_size + 4 bytes)
        let header_size = 11; // Basic header
        let o_size =
            u32::from_le_bytes([program_bytes[0], program_bytes[1], program_bytes[2], 0]) as usize;
        let program_text_start = header_size;

        let expected_program_text = &program_bytes[program_text_start..program_text_start + 100];
        assert_eq!(
            prog_data, expected_program_text,
            "Program text in memory should match original data"
        );

        //  Data region - should be readable and writable
        let data_start = 2 * MIN_ADDR + o_size.div_ceil(MIN_ADDR) * MIN_ADDR;
        let Ok(data_read) = memory.read(data_start, 50) else {
            panic!("Should be able to read from data region");
        };
        assert_eq!(
            data_read.len(),
            50,
            "Should read exactly 50 bytes from data region"
        );

        let Ok(_data_write) = memory.write(data_start, &data_read.to_vec()) else {
            panic!("Should be able to write to data region");
        };

        //  Args region - should be readable
        let args_start = MAX_ADDR - MIN_ADDR - LAYOUT_BUFFER_SIZE;
        let Ok(args_data) = memory.read(args_start, args.len()) else {
            panic!("Should be able to read args region");
        };
        assert_eq!(args_data, args, "Args data should match what we put in");

        //  Stack region - should be readable and writable
        let stack_start = MAX_ADDR
            - 2 * MIN_ADDR
            - LAYOUT_BUFFER_SIZE
            - 8192usize.div_ceil(PAGE_SIZE) * PAGE_SIZE;
        let Ok(stack_data) = memory.read(stack_start, 100) else {
            panic!("Should be able to read from stack region");
        };
        assert_eq!(
            stack_data.len(),
            100,
            "Should read exactly 100 bytes from stack"
        );

        assert_eq!(memory.write(stack_start, &stack_data.to_vec()), Ok(()));

        // Test gap between program and data regions
        let gap1_start = MIN_ADDR + o_size.div_ceil(PAGE_SIZE) * PAGE_SIZE + PAGE_SIZE;
        assert_eq!(
            memory.read(gap1_start, 10),
            Err(MemoryError::Fault {
                page_addr: gap1_start
            })
        );
    }

    #[test]
    fn test_initialize_program_with_real_service_code() {
        let hex_content =
            fs::read_to_string("service_code_0.hex").expect("Failed to read service_code_0.hex");

        // Remove any whitespace/newlines and convert to bytes
        let hex_clean = hex_content.trim().replace('\n', "").replace(' ', "");
        let program_bytes = hex_to_bytes(&hex_clean);

        // Create test arguments
        let args = create_test_args();

        // Test the REAL initialize_program function with byte slices
        let result = initialize_program(&program_bytes, &args);
        assert!(
            result.is_some(),
            "initialize_program should succeed with valid service code"
        );

        let (code, registers, mut memory) = result.unwrap();

        // Verify registers are set correctly according to the function
        assert_eq!(
            registers[0], 0xFFFF_0000,
            "Register 0 should be set to 0xFFFF_0000"
        );
        assert_eq!(
            registers[1], 0xFEFE_0000,
            "Register 1 should be set to 0xFEFE_0000"
        );
        assert_eq!(
            registers[7], 0xFEFF_0000,
            "Register 7 should be set to 0xFEFF_0000"
        );
        assert_eq!(
            registers[8],
            args.len() as u64,
            "Register 8 should contain args length"
        );

        // Verify other registers are zero
        for i in 2..=6 {
            assert_eq!(registers[i], 0, "Register {} should be zero", i);
        }
        for i in 9..=12 {
            assert_eq!(registers[i], 0, "Register {} should be zero", i);
        }

        assert!(!code.is_empty(), "Code segment should not be empty");
        assert_eq!(memory.current_heap_pointer, 208896);
        assert_eq!(memory.max_heap_pointer, 4278050816);

        let deblob_result = deblob(code);

        // Verify deblob results
        assert!(
            !deblob_result.program.is_empty(),
            "Deblobed program should not be empty"
        );
        assert!(
            !deblob_result.bitmask.is_empty(),
            "Bitmask should not be empty"
        );

        // Verify bitmask structure
        let total_bits = deblob_result.bitmask.len() * 64;
        assert!(
            total_bits >= deblob_result.program.len(),
            "Bitmask should have enough bits for program length"
        );

        // Verify trap bit is set (should be at program length position)
        let trap_pos = deblob_result.program.len();
        let word_index = trap_pos / 64;
        let bit_index = trap_pos % 64;

        if word_index < deblob_result.bitmask.len() {
            let trap_bit = (deblob_result.bitmask[word_index] >> bit_index) & 1;
            assert_eq!(trap_bit, 1, "Trap bit should be set at program end");
        }

        test_memory_regions(&mut memory, &program_bytes, &args);
    }
}

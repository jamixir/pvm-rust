use rustler::{Binary, Decoder, Encoder, Env, NifResult, Term};

use crate::decoder::deblob;
use crate::init_program::initialize_program;
use crate::memory::{MemoryRef, MemoryResource};
use crate::utils::build_start_set;
use crate::vm::Vm;
mod consts;
mod decoder;
mod encoder;
mod init_program;
mod memory;
mod utils;
mod vm;

rustler::init!("Elixir.Pvm.Native", load = load);
fn load(env: Env, _info: Term) -> bool {
    env.register::<MemoryResource>().is_ok()
}

#[rustler::nif]
fn execute<'a>(
    env: Env<'a>,
    program_term: Term<'a>,
    pc_term: Term<'a>,
    gas_term: Term<'a>,
    args_term: Term<'a>,
    memory_ref_term: Term<'a>,
) -> NifResult<Term<'a>> {

    let program: Binary<'a> = Binary::decode(program_term)?;
    let pc: usize = usize::decode(pc_term)?;
    let gas: u64 = u64::decode(gas_term)?;
    let args: Binary<'a> = Binary::decode(args_term)?;
    let memory_ref: MemoryRef = MemoryRef::decode(memory_ref_term)?;
    // Convert Binary to slice and try to initialize the program
    let program_slice: &[u8] = program.as_slice();
    let args_slice: &[u8] = args.as_slice();
    let (code, registers, memory) = match initialize_program(program_slice, args_slice) {
        None => {
            // Return {0, :panic, context} equivalent - for now just return {0, :panic}
            return Ok((0u64, "panic").encode(env));
        }
        Some(v) => v,
    };

    let deblob = deblob(code);
    let start_set = build_start_set(deblob.program, &deblob.bitmask);
    let _vm = Vm::new(
        code,
        deblob.bitmask,
        deblob.jump_table,
        start_set,
        registers,
        pc,
        gas,
        memory_ref,
        Some(memory),
    );

    return Ok(registers.encode(env));
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

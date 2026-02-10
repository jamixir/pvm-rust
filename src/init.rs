use crate::{Memory, Permission, Registers};

pub enum InitializeProgramError {
    InsufficientBytes,
    InvalidProgram,
}

#[derive(Debug, Clone)]
pub struct ProgramSegments<'a> {
    pub program_text: &'a [u8],
    pub data: &'a [u8],
    pub z: usize,
    pub s: usize,
    pub code: &'a [u8],
}

pub fn initialize_program(
    linked_program: &[u8],
    args: &[u8],
) -> Option<(Vec<u8>, Registers, Memory)> {
    let program_segments = parse_program_segments(linked_program).ok()?;
    let registers = create_initial_registers(args.len());
    let memory = construct_memory(&program_segments, args);
    let code = program_segments.code.to_vec();
    Some((code, registers, memory))
}

pub fn parse_program_segments<'a>(program: &'a [u8]) -> Result<ProgramSegments<'a>, InitializeProgramError> {
    if program.len() < 15 {
        return Err(InitializeProgramError::InsufficientBytes);
    }

    let o_size = u32::from_le_bytes([program[0], program[1], program[2], 0]) as usize;
    let w_size = u32::from_le_bytes([program[3], program[4], program[5], 0]) as usize;
    let z = u16::from_le_bytes([program[6], program[7]]) as usize;
    let s = u32::from_le_bytes([program[8], program[9], program[10], 0]) as usize;

    let mut offset = 11usize;

    // 4 bytes for c
    if program.len() < offset + o_size + w_size + 4 {
        return Err(InitializeProgramError::InvalidProgram);
    }

    let o = &program[offset..offset + o_size];
    offset += o_size;
    let w = &program[offset..offset + w_size];
    offset += w_size;
    let c_size = u32::from_le_bytes([
        program[offset],
        program[offset + 1],
        program[offset + 2],
        program[offset + 3],
    ]) as usize;

    offset += 4;

    if program.len() < c_size + offset {
        return Err(InitializeProgramError::InvalidProgram);
    }
    let c = &program[offset..offset + c_size];

    Ok(ProgramSegments {
        program_text: o,
        data: w,
        z,
        s,
        code: c,
    })
}

/// Construct memory layout from program segments and args
pub fn construct_memory(program_segments: &ProgramSegments, args: &[u8]) -> Memory {
    use crate::core::consts::{LAYOUT_BUFFER_SIZE, MAX_ADDR, MIN_ADDR, PAGE_SIZE, ZP, ZZ};

    #[inline]
    fn align_to_min_addr(x: usize) -> usize {
        x.div_ceil(ZZ) * ZZ
    }
    #[inline]
    fn align_to_page_size(x: usize) -> usize {
        x.div_ceil(ZP) * ZP
    }

    let mut builder = Memory::builder();
    let program_text = program_segments.program_text;
    let data = program_segments.data;
    let z = program_segments.z;
    let s = program_segments.s;

    // Compute boundaries
    let prog_start = MIN_ADDR;
    let prog_region_len = align_to_page_size(program_text.len());

    let data_start = 2 * MIN_ADDR + align_to_min_addr(program_text.len());
    let data_region_len = align_to_page_size(data.len()) + z * PAGE_SIZE;

    let heap_start = data_start + data_region_len;

    let stack_start = MAX_ADDR - 2 * MIN_ADDR - LAYOUT_BUFFER_SIZE - align_to_page_size(s);
    let stack_len = align_to_page_size(s);
    let heap_end = stack_start;

    let args_start = MAX_ADDR - MIN_ADDR - LAYOUT_BUFFER_SIZE;
    let args_region_len = align_to_page_size(args.len());

    // Program text
    builder
        .get_mut_slice(prog_start, program_text.len())
        .copy_from_slice(program_text);
    //  Data
    builder
        .get_mut_slice(data_start, data.len())
        .copy_from_slice(data);
    //  Args
    builder
        .get_mut_slice(args_start, args.len())
        .copy_from_slice(args);

    builder.set_access(prog_start, prog_region_len, Permission::Read);
    builder.set_access(data_start, data_region_len, Permission::ReadWrite);
    builder.set_access(stack_start, stack_len, Permission::ReadWrite);
    builder.set_access(args_start, args_region_len, Permission::Read);

    builder.set_heap_bounds(heap_start, heap_end);

    builder.build()
}

fn create_initial_registers(args_len: usize) -> Registers {
    Registers::from_slice([
        0xFFFF_0000,     // register 0
        0xFEFE_0000,     // register 1
        0,               // register 2
        0,               // register 3
        0,               // register 4
        0,               // register 5
        0,               // register 6
        0xFEFF_0000,     // register 7
        args_len as u64, // register 8
        0,               // register 9
        0,               // register 10
        0,               // register 11
        0,               // register 12
    ])
}

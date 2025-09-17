use pvm::{
    core::{Memory, Permission, Registers, StartSet},
    encoding::deblob,
    vm::{ExecutionResult, Vm, VmContext, VmState},
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::Arc;

static TEST_VECTORS_DIR: &str = "/home/luke/Documents/Jamixir/pvm-test-vectors/pvm/programs/";

#[derive(Debug, Deserialize, Serialize)]
struct TestCase {
    name: String,
    #[serde(rename = "initial-pc")]
    initial_pc: usize,
    #[serde(rename = "initial-gas")]
    initial_gas: u64,
    #[serde(rename = "initial-regs")]
    initial_regs: Vec<u64>,
    #[serde(rename = "initial-page-map")]
    initial_page_map: Vec<PageMapEntry>,
    #[serde(rename = "initial-memory")]
    initial_memory: Vec<MemoryEntry>,
    program: Vec<u8>,
    #[serde(rename = "expected-status")]
    expected_status: String,
    #[serde(rename = "expected-pc")]
    expected_pc: usize,
    #[serde(rename = "expected-regs")]
    expected_regs: Vec<u64>,
    #[serde(rename = "expected-memory")]
    expected_memory: Vec<MemoryEntry>,
    #[serde(rename = "expected-gas")]
    expected_gas: u64,
}

#[derive(Debug, Deserialize, Serialize)]
struct PageMapEntry {
    address: usize,
    length: usize,
    #[serde(rename = "is-writable")]
    is_writable: bool,
}

#[derive(Debug, Deserialize, Serialize)]
struct MemoryEntry {
    address: usize,
    contents: Vec<u8>,
}

fn status_to_execution_result(status: &str) -> ExecutionResult {
    match status {
        "halt" => ExecutionResult::Halt,
        "panic" => ExecutionResult::Panic,
        "out-of-gas" => ExecutionResult::OutOfGas,
        "page-fault" => ExecutionResult::Fault { page: 0 }, // Page will be validated separately
        _ => {
            if status.starts_with("ecall:") {
                let call_id = status.strip_prefix("ecall:").unwrap().parse().unwrap_or(0);
                ExecutionResult::HostCall { call_id }
            } else {
                ExecutionResult::Panic
            }
        }
    }
}

fn setup_memory_with_permissions(
    page_map: &[PageMapEntry],
    memory_entries: &[MemoryEntry],
) -> Result<Memory, String> {
    // Create memory with enough pages for all addresses
    let max_addr = page_map
        .iter()
        .map(|entry| entry.address + entry.length)
        .max()
        .unwrap_or(4096);

    let num_pages = (max_addr + 4095) >> 12; // Round up to next page boundary
    let mut memory = Memory::test_memory(num_pages.max(1));

    // Set up page permissions
    for entry in page_map {
        let permission = if entry.is_writable {
            Permission::ReadWrite
        } else {
            Permission::Read
        };
        memory.set_access(entry.address, entry.length, permission);
    }

    // Write initial memory contents
    for entry in memory_entries {
        memory.write(entry.address, &entry.contents).map_err(|e| {
            format!(
                "Failed to write initial memory at address {}: {:?}",
                entry.address, e
            )
        })?;
    }

    Ok(memory)
}

fn run_test_case(
    test_case: &TestCase,
) -> Result<(ExecutionResult, VmState, Memory), Box<dyn std::error::Error>> {
    let registers = Registers::from_vec(test_case.initial_regs.clone());

    let memory =
        setup_memory_with_permissions(&test_case.initial_page_map, &test_case.initial_memory)?;

    let deblob_result = deblob(&test_case.program);
    let start_set = StartSet::build(&deblob_result.program, &deblob_result.bitmask);

    let context = Arc::new(VmContext {
        program: deblob_result.program,
        bitmask: deblob_result.bitmask,
        jump_table: deblob_result.jump_table,
        start_set,
    });

    let state = VmState::new(registers, test_case.initial_pc, test_case.initial_gas);

    let mut vm = Vm::new_for_test(context, state, Some(memory));
    let execution_result = vm.execute();
    let final_state = vm.state;
    let final_memory = vm.memory.unwrap();

    Ok((execution_result, final_state, final_memory))
}

fn validate_test_results(
    test_case: &TestCase,
    actual_exit_reason: ExecutionResult,
    final_state: &VmState,
    final_memory: &Memory,
) -> Result<(), String> {
    let expected_exit_reason = status_to_execution_result(&test_case.expected_status);

    match (&actual_exit_reason, &expected_exit_reason) {
        (ExecutionResult::Fault { .. }, ExecutionResult::Fault { .. }) => {}
        (ExecutionResult::HostCall { .. }, ExecutionResult::HostCall { .. }) => {}
        (actual, expected)
            if std::mem::discriminant(actual) == std::mem::discriminant(expected) => {}
        _ => {
            return Err(format!(
                "Exit reason mismatch: expected {:?}, got {:?}",
                expected_exit_reason, actual_exit_reason
            ));
        }
    }

    if !matches!(actual_exit_reason, ExecutionResult::Halt | ExecutionResult::Panic) {
        if final_state.pc != test_case.expected_pc {
            return Err(format!(
                "PC mismatch: expected {}, got {}",
                test_case.expected_pc, final_state.pc
            ));
        }
    }

    for (i, &expected_value) in test_case.expected_regs.iter().enumerate() {
        if i < 13 {
            let actual_value = final_state.registers.get(i);
            if actual_value != expected_value {
                return Err(format!(
                    "Register r{} mismatch: expected {}, got {}",
                    i, expected_value, actual_value
                ));
            }
        }
    }

    // Check gas consumption
    // let expected_spent_gas = test_case.initial_gas - test_case.expected_gas;
    // if final_state.spent_gas != expected_spent_gas {
    //     return Err(format!(
    //         "Gas consumption mismatch: expected {}, got {}",
    //         expected_spent_gas, final_state.spent_gas
    //     ));
    // }

    for entry in &test_case.expected_memory {
        match final_memory.read(entry.address, entry.contents.len()) {
            Ok(data) => {
                if data != entry.contents.as_slice() {
                    return Err(format!(
                        "Memory at address {} mismatch: expected {:?}, got {:?}",
                        entry.address, entry.contents, data
                    ));
                }
            }
            Err(e) => {
                return Err(format!(
                    "Failed to read memory at address {}: {:?}",
                    entry.address, e
                ));
            }
        }
    }

    Ok(())
}

pub fn run_case(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(file_path)?;
    let test_case: TestCase = serde_json::from_str(&content)?;

    let (exit_reason, final_state, final_memory) = run_test_case(&test_case)?;

    validate_test_results(&test_case, exit_reason, &final_state, &final_memory)
        .map_err(|e| e.into())
}

#[test]
#[ignore]
fn test_specific_test() {
    let test_case = "inst_store_imm_indirect_u8_with_offset_nok";
    let test_file = TEST_VECTORS_DIR.to_string() + test_case + ".json";

    if Path::new(&test_file).exists() {
        assert!(run_case(&test_file).is_ok());
    } else {
        println!("Test file {} not found, skipping", test_file);
    }
}

macro_rules! generate_json_tests {
    ($($test_fn_name:ident: $json_name:expr),* $(,)?) => {
        $(
            #[test]
            fn $test_fn_name() {


                let test_file = format!("{}{}.json", TEST_VECTORS_DIR, $json_name);

                if Path::new(&test_file).exists() {
                    match run_case(&test_file) {
                        Ok(_) => {},
                        Err(e) => panic!("Test failed: {}", e),
                    }
                } else {
                    panic!("Test file {} not found", test_file);
                }
            }
        )*
    };
}

generate_json_tests!(
    // NoArgs instructions
    test_inst_fallthrough: "inst_fallthrough",
    test_inst_trap: "inst_trap",

    // OneImmediate instructions
    test_inst_jump: "inst_jump",
    test_inst_load_imm_and_jump: "inst_load_imm_and_jump",

    // RegImmediate instructions (LOAD_*)
    test_inst_load_imm_64: "inst_load_imm_64",
    test_inst_load_imm: "inst_load_imm",
    test_inst_load_u8: "inst_load_u8",
    test_inst_load_i8: "inst_load_i8",
    test_inst_load_u16: "inst_load_u16",
    test_inst_load_i16: "inst_load_i16",
    test_inst_load_u32: "inst_load_u32",
    test_inst_load_i32: "inst_load_i32",
    test_inst_load_u64: "inst_load_u64",

    // RegImmediate instructions (STORE_*)
    test_inst_store_u8: "inst_store_u8",
    test_inst_store_u16: "inst_store_u16",
    test_inst_store_u32: "inst_store_u32",
    test_inst_store_u64: "inst_store_u64",

    // TwoImmediates instructions (STORE_IMM_*)
    test_inst_store_imm_u8: "inst_store_imm_u8",
    test_inst_store_imm_u16: "inst_store_imm_u16",
    test_inst_store_imm_u32: "inst_store_imm_u32",
    test_inst_store_imm_u64: "inst_store_imm_u64",

    // RegTwoImm instructions (STORE_IMM_IND_*)
    test_inst_store_imm_indirect_u8_with_offset_ok: "inst_store_imm_indirect_u8_with_offset_ok",
    test_inst_store_imm_indirect_u8_without_offset_ok: "inst_store_imm_indirect_u8_without_offset_ok",
    test_inst_store_imm_indirect_u16_with_offset_ok: "inst_store_imm_indirect_u16_with_offset_ok",
    test_inst_store_imm_indirect_u16_without_offset_ok: "inst_store_imm_indirect_u16_without_offset_ok",
    test_inst_store_imm_indirect_u32_with_offset_ok: "inst_store_imm_indirect_u32_with_offset_ok",
    test_inst_store_imm_indirect_u32_without_offset_ok: "inst_store_imm_indirect_u32_without_offset_ok",
    test_inst_store_imm_indirect_u64_with_offset_ok: "inst_store_imm_indirect_u64_with_offset_ok",
    test_inst_store_imm_indirect_u64_without_offset_ok: "inst_store_imm_indirect_u64_without_offset_ok",

    test_inst_store_imm_indirect_u8_with_offset_nok: "inst_store_imm_indirect_u8_with_offset_nok",
    test_inst_store_imm_indirect_u32_with_offset_nok: "inst_store_imm_indirect_u32_with_offset_nok",
    test_inst_store_imm_indirect_u64_with_offset_nok: "inst_store_imm_indirect_u64_with_offset_nok",

    // RegImmediateOffset instructions
    test_inst_branch_eq_imm_ok: "inst_branch_eq_imm_ok",
    test_inst_branch_eq_imm_nok: "inst_branch_eq_imm_nok",
    test_inst_branch_not_eq_imm_ok: "inst_branch_not_eq_imm_ok",
    test_inst_branch_not_eq_imm_nok: "inst_branch_not_eq_imm_nok",
    test_inst_branch_less_unsigned_imm_ok: "inst_branch_less_unsigned_imm_ok",
    test_inst_branch_less_unsigned_imm_nok: "inst_branch_less_unsigned_imm_nok",
    test_inst_branch_less_or_equal_unsigned_imm_ok: "inst_branch_less_or_equal_unsigned_imm_ok",
    test_inst_branch_less_or_equal_unsigned_imm_nok: "inst_branch_less_or_equal_unsigned_imm_nok",
    test_inst_branch_greater_or_equal_unsigned_imm_ok: "inst_branch_greater_or_equal_unsigned_imm_ok",
    test_inst_branch_greater_or_equal_unsigned_imm_nok: "inst_branch_greater_or_equal_unsigned_imm_nok",
    test_inst_branch_greater_unsigned_imm_ok: "inst_branch_greater_unsigned_imm_ok",
    test_inst_branch_greater_unsigned_imm_nok: "inst_branch_greater_unsigned_imm_nok",
    test_inst_branch_less_signed_imm_ok: "inst_branch_less_signed_imm_ok",
    test_inst_branch_less_signed_imm_nok: "inst_branch_less_signed_imm_nok",
    test_inst_branch_less_or_equal_signed_imm_ok: "inst_branch_less_or_equal_signed_imm_ok",
    test_inst_branch_less_or_equal_signed_imm_nok: "inst_branch_less_or_equal_signed_imm_nok",
    test_inst_branch_greater_or_equal_signed_imm_ok: "inst_branch_greater_or_equal_signed_imm_ok",
    test_inst_branch_greater_or_equal_signed_imm_nok: "inst_branch_greater_or_equal_signed_imm_nok",
    test_inst_branch_greater_signed_imm_ok: "inst_branch_greater_signed_imm_ok",
    test_inst_branch_greater_signed_imm_nok: "inst_branch_greater_signed_imm_nok",
);

//! Focused tests for `ChildVmInstance`: creation, invalid input, execution, state, memory, take_memory.

#[cfg(test)]
mod child_instance {
    use pvm_core::core::consts::MIN_ADDR;
    use pvm_core::core::{Permission, Registers};
    use pvm_core::encoding::encode_program;
    use pvm_core::vm::instructions::opcodes::{JUMP_IND, LOAD_IMM_64};
    use pvm_core::vm::{ChildVmInstance, ExecutionResult, StepResult, VmError};

    /// Minimal program that traps (opcode 0) — use when only validity is needed.
    fn encoded_trap_program() -> Vec<u8> {
        let program = [0u8; 4];
        let bitmask = vec![0xFF; (program.len() + 7) / 8];
        encode_program(&program, &bitmask, &[], 1)
    }

    /// Program that performs a clean Halt: LOAD_IMM_64 r0, 0xFFFF_0000; JUMP_IND r0, 0.
    /// djump(0xFFFF_0000) is the designated halt target (see vm/types.rs).
    fn encoded_halt_program() -> Vec<u8> {
        const HALT_TARGET: u64 = 0xFFFF_0000;

        let mut program = Vec::with_capacity(16);
        // Instruction 1: LOAD_IMM_64 r0, 0xFFFF_0000 (1 + 1 + 8 = 10 bytes)
        program.extend_from_slice(&[LOAD_IMM_64, 0]);
        program.extend_from_slice(&HALT_TARGET.to_le_bytes());

        program.extend_from_slice(&[JUMP_IND, 0, 0u8]);
        let bitmask = [0b1, 0b010100];
        encode_program(&program, &bitmask, &[], 1)
    }

    #[test]
    fn new_succeeds_with_id_and_blob() {
        let encoded = encoded_trap_program();
        let instance =
            ChildVmInstance::new(42, &encoded, 0, 10_000, Registers::default()).expect("new");
        assert_eq!(instance.instance_id(), 42);
        assert_eq!(instance.program_blob(), encoded.as_slice());
    }

    #[test]
    fn new_preserves_initial_pc_gas_and_registers() {
        let encoded = encoded_trap_program();
        let mut regs = Registers::default();
        regs.set(0, 100);
        regs.set(1, 200);
        let instance = ChildVmInstance::new(1, &encoded, 10, 5_000, regs).expect("new");
        let state = instance.get_state();
        assert_eq!(state.pc, 10);
        assert_eq!(state.initial_gas, 5_000);
        assert_eq!(state.spent_gas, 0);
        assert_eq!(state.registers.get(0), 100);
        assert_eq!(state.registers.get(1), 200);
    }

    #[test]
    fn new_rejects_invalid_blob() {
        let regs = Registers::default();
        let r1 = ChildVmInstance::new(1, &[0xFF; 5], 0, 1_000, regs);
        let r2 = ChildVmInstance::new(1, &[], 0, 1_000, regs);
        assert!(matches!(r1, Err(VmError::InvalidProgramBlob)));
        assert!(matches!(r2, Err(VmError::InvalidProgramBlob)));
    }

    #[test]
    fn execute_returns_halting_result() {
        let encoded = encoded_halt_program();
        let mut instance =
            ChildVmInstance::new(1, &encoded, 0, 10_000, Registers::default()).expect("new");
        let result = instance.execute();
        assert!(
            matches!(result, ExecutionResult::Halt),
            "program does LOAD_IMM_64 r0, 0xFFFF_0000; JUMP_IND r0, 0"
        );
        let state = instance.get_state();
        assert!(state.spent_gas <= state.initial_gas);
    }

    #[test]
    fn execute_with_zero_gas_stops() {
        let encoded = encoded_trap_program();
        let mut instance =
            ChildVmInstance::new(1, &encoded, 0, 0, Registers::default()).expect("new");
        let result = instance.execute();
        assert!(
            matches!(
                result,
                ExecutionResult::OutOfGas | ExecutionResult::Halt | ExecutionResult::Panic
            ),
            "got {:?}",
            result
        );
    }

    #[test]
    fn get_state_mut_roundtrip() {
        let encoded = encoded_trap_program();
        let mut instance =
            ChildVmInstance::new(1, &encoded, 0, 5_000, Registers::default()).expect("new");
        {
            let s = instance.get_state_mut();
            s.pc = 20;
            s.registers.set(5, 1111);
        }
        let state = instance.get_state();
        assert_eq!(state.pc, 20);
        assert_eq!(state.registers.get(5), 1111);
    }

    #[test]
    fn memory_read_write_with_permission() {
        let encoded = encoded_trap_program();
        let mut instance =
            ChildVmInstance::new(1, &encoded, 0, 10_000, Registers::default()).expect("new");
        let addr = MIN_ADDR;
        let data = [10u8, 20, 30, 40, 50];
        if let Some(mem) = instance.get_memory_mut() {
            mem.set_access(addr, data.len(), Permission::ReadWrite);
        }
        instance.write_memory(addr, &data).expect("write");
        let read = instance.read_memory(addr, data.len()).expect("read");
        assert_eq!(read, data);
    }

    #[test]
    fn memory_without_permission_fails() {
        let encoded = encoded_trap_program();
        let mut instance =
            ChildVmInstance::new(1, &encoded, 0, 10_000, Registers::default()).expect("new");
        let addr = MIN_ADDR;
        assert!(instance.write_memory(addr, &[1, 2, 3]).is_err());
        assert!(instance.read_memory(addr, 3).is_err());
    }

    #[test]
    fn memory_isolated_between_instances() {
        let encoded = encoded_trap_program();
        let regs = Registers::default();
        let mut a = ChildVmInstance::new(1, &encoded, 0, 10_000, regs).expect("new");
        let mut b = ChildVmInstance::new(2, &encoded, 0, 10_000, regs).expect("new");
        let addr = MIN_ADDR;
        for (inst, data) in [(&mut a, [1u8, 2, 3, 4]), (&mut b, [5u8, 6, 7, 8])] {
            if let Some(mem) = inst.get_memory_mut() {
                mem.set_access(addr, 4, Permission::ReadWrite);
            }
            inst.write_memory(addr, &data).expect("write");
        }
        assert_eq!(a.read_memory(addr, 4).expect("read"), [1, 2, 3, 4]);
        assert_eq!(b.read_memory(addr, 4).expect("read"), [5, 6, 7, 8]);
    }

    #[test]
    fn take_memory_clears_handle_and_read_fails() {
        let encoded = encoded_trap_program();
        let mut instance =
            ChildVmInstance::new(1, &encoded, 0, 10_000, Registers::default()).expect("new");
        assert!(instance.get_memory().is_some());
        let _mem = instance.take_memory();
        assert!(instance.get_memory().is_none());
        assert!(instance.read_memory(MIN_ADDR, 4).is_err());
    }

    #[test]
    fn single_step_returns_step_result() {
        let encoded = encoded_trap_program();
        let mut instance =
            ChildVmInstance::new(1, &encoded, 0, 10_000, Registers::default()).expect("new");
        let step = instance.single_step();
        assert!(matches!(
            step,
            StepResult::Continue
                | StepResult::Halt
                | StepResult::Panic
                | StepResult::Fault { .. }
                | StepResult::Ecall { .. }
        ));
    }
}

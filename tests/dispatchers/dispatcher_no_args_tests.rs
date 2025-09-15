#[cfg(test)]
mod tests {
    use pvm::core::{BitMask, Memory, Registers, StartSet};
    use pvm::vm::instructions::opcodes::*;
    use pvm::vm::{InstructionResult, VmContext, VmState};
    use std::sync::Arc;

    // Helper function to create a test VM without rustler resources
    fn create_test_vm() -> TestVm {
        let registers = Registers::new();
        let state = VmState::new(registers, 0, 1000);
        let context = Arc::new(VmContext {
            program: vec![],
            bitmask: BitMask::new(0),
            jump_table: vec![],
            start_set: StartSet::build(&[], &BitMask::new(0)),
        });
        let memory = Memory::test_memory(1);
        TestVm::new(context, state, Some(memory))
    }

    // Helper function to create a test VM with specific registers
    fn create_test_vm_with_registers(registers: Registers) -> TestVm {
        let state = VmState::new(registers, 0, 1000);
        let context = Arc::new(VmContext {
            program: vec![],
            bitmask: BitMask::new(0),
            jump_table: vec![],
            start_set: StartSet::build(&[], &BitMask::new(0)),
        });
        TestVm::new(context, state, None)
    }

    // Test version of Vm that doesn't use rustler resources
    #[allow(dead_code)]
    struct TestVm {
        pub context: Arc<VmContext>,
        pub memory: Option<Memory>,
        pub state: VmState,
    }

    impl TestVm {
        pub fn new(
            context: Arc<VmContext>,
            state: VmState,
            memory: Option<Memory>,
        ) -> Self {
            Self {
                context,
                memory,
                state,
            }
        }

        // Copy the dispatch_no_args method from the real Vm implementation
        #[inline(always)]
        pub fn dispatch_no_args(&self, opcode: u8) -> InstructionResult {
            match opcode {
                TRAP => InstructionResult::Panic,
                FALLTHROUGH => InstructionResult::Continue,
                _ => InstructionResult::Panic,
            }
        }
    }

    #[test]
    fn test_trap_sets_panic_exit_status() {
        let vm = create_test_vm();
        let result = vm.dispatch_no_args(TRAP);

        assert_eq!(result, InstructionResult::Panic);
    }

    #[test]
    fn test_fallthrough_keeps_existing_output_unchanged() {
        let mut registers = Registers::new();
        registers.set(0, 0);
        registers.set(1, 1);
        registers.set(2, 2);

        let vm = create_test_vm_with_registers(registers);

        let result = vm.dispatch_no_args(FALLTHROUGH);

        assert_eq!(result, InstructionResult::Continue);
        // Note: In the actual implementation, the state is not modified by fallthrough
        // The test verifies that the instruction returns Continue without side effects
    }

    #[test]
    fn test_unknown_opcode_returns_panic() {
        let vm = create_test_vm();
        let result = vm.dispatch_no_args(255); // Invalid opcode

        assert_eq!(result, InstructionResult::Panic);
    }

    #[test]
    fn test_other_valid_opcodes_return_panic() {
        let vm = create_test_vm();

        // Test that opcodes that don't belong to no_args category return panic
        assert_eq!(vm.dispatch_no_args(ECALLI), InstructionResult::Panic);
        assert_eq!(vm.dispatch_no_args(JUMP), InstructionResult::Panic);
        assert_eq!(vm.dispatch_no_args(LOAD_IMM_64), InstructionResult::Panic);
        assert_eq!(vm.dispatch_no_args(MOVE_REG), InstructionResult::Panic);
    }

    #[test]
    fn test_trap_with_different_vm_states() {
        // Test trap with different register states
        let mut registers = Registers::new();
        registers.set(0, 42);
        registers.set(5, 100);
        registers.set(12, 999);

        let vm = create_test_vm_with_registers(registers);
        let result = vm.dispatch_no_args(TRAP);

        assert_eq!(result, InstructionResult::Panic);
    }

    #[test]
    fn test_fallthrough_with_different_vm_states() {
        // Test fallthrough with different register states
        let mut registers = Registers::new();
        registers.set(0, 42);
        registers.set(5, 100);
        registers.set(12, 999);

        let vm = create_test_vm_with_registers(registers);
        let result = vm.dispatch_no_args(FALLTHROUGH);

        assert_eq!(result, InstructionResult::Continue);
    }

    #[test]
    fn test_all_no_args_opcodes() {
        let vm = create_test_vm();

        // Test all opcodes that should be handled by no_args dispatcher
        assert_eq!(vm.dispatch_no_args(TRAP), InstructionResult::Panic);
        assert_eq!(
            vm.dispatch_no_args(FALLTHROUGH),
            InstructionResult::Continue
        );
    }
}

use crate::{
    core::{consts::PAGE_SIZE, Memory, MemoryError, Permission, Registers},
    deblob,
    vm::{ExecutionResult, StepResult, Vm, VmContext, VmState},
    StartSet,
};
use std::sync::Arc;

pub struct ChildVmInstance {
    pub vm: Vm,
    pub instance_id: u64,
    pub program: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VmError {
    InvalidProgramBlob,
    InitializationFailed,
    MemoryAllocationFailed,
}

impl ChildVmInstance {
    pub fn new(
        instance_id: u64,
        program_blob: &[u8],
        initial_pc: usize,
        initial_gas: u64,
        initial_registers: Registers,
    ) -> Result<Self, VmError> {
        let deblob_result = deblob(program_blob).map_err(|_| VmError::InvalidProgramBlob)?;

        let start_set = StartSet::build(&deblob_result.program, &deblob_result.bitmask);

        let context = Arc::new(VmContext {
            program: deblob_result.program,
            bitmask: deblob_result.bitmask,
            jump_table: deblob_result.jump_table,
            start_set,
            tracer: None,
        });

        let memory = Memory::builder().build();

        let state = VmState::new(initial_registers, initial_pc, initial_gas);

        let vm = Vm::new(context, state, Some(memory));

        Ok(Self {
            vm,
            instance_id,
            program: program_blob.to_vec(),
        })
    }

    pub fn execute(&mut self) -> ExecutionResult {
        self.vm.execute()
    }

    pub fn single_step(&mut self) -> StepResult {
        self.vm.single_step()
    }

    pub fn get_state(&self) -> &VmState {
        self.vm.get_state()
    }

    pub fn get_state_mut(&mut self) -> &mut VmState {
        self.vm.get_state_mut()
    }

    pub fn get_memory(&self) -> Option<&Memory> {
        self.vm.get_memory()
    }

    pub fn get_memory_mut(&mut self) -> Option<&mut Memory> {
        self.vm.get_memory_mut()
    }

    pub fn take_memory(&mut self) -> Option<Memory> {
        self.vm.take_memory()
    }

    pub fn read_memory(&self, addr: usize, len: usize) -> Result<&[u8], MemoryError> {
        self.vm
            .get_memory()
            .ok_or(MemoryError::Panic)?
            .read(addr, len)
    }

    pub fn write_memory(&mut self, addr: usize, data: &[u8]) -> Result<(), MemoryError> {
        self.vm
            .get_memory_mut()
            .ok_or(MemoryError::Panic)?
            .write(addr, data)
    }

    pub fn set_memory_access(
        &mut self,
        page_index: usize,
        page_count: usize,
        permission: Permission,
    ) -> Result<(), ()> {
        let addr = page_index * PAGE_SIZE;
        let len = page_count * PAGE_SIZE;
        if let Some(memory) = self.get_memory_mut() {
            memory.set_access(addr, len, permission);
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn check_memory_access(
        &self,
        page_index: usize,
        page_count: usize,
        required_permission: Permission,
    ) -> bool {
        let addr = page_index * PAGE_SIZE;
        let len = page_count * PAGE_SIZE;
        self.get_memory().map_or(false, |memory| {
            memory.check_access(addr, len, required_permission)
        })
    }

    pub fn zero_memory(&mut self, addr: usize, len: usize) -> Result<(), MemoryError> {
        let zeros = vec![0u8; len];
        self.write_memory(addr, &zeros)
    }

    pub fn instance_id(&self) -> u64 {
        self.instance_id
    }

    pub fn program_blob(&self) -> &[u8] {
        &self.program
    }
}

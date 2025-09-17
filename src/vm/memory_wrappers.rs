use crate::{
    core::MemoryError,
    vm::{InstructionResult, Vm},
};

impl Vm {
    #[inline(always)]
    pub fn read_memory(&self, addr: usize, len: usize) -> Result<&[u8], MemoryError> {
        match self
            .memory
            .as_ref()
            .expect("Memory not available")
            .read(addr, len)
        {
            Ok(data) => Ok(data),
            Err(MemoryError::Fault { page_addr }) => Err(MemoryError::Fault { page_addr }),
            Err(MemoryError::Panic) => Err(MemoryError::Panic),
        }
    }

    #[inline(always)]
    pub fn write_memory(&mut self, addr: usize, data: &[u8]) -> InstructionResult {
        match self
            .memory
            .as_mut()
            .expect("Memory not available")
            .write(addr, data)
        {
            Ok(()) => InstructionResult::Continue,
            Err(MemoryError::Fault { page_addr }) => InstructionResult::Fault { page: page_addr },
            Err(MemoryError::Panic) => InstructionResult::Panic,
        }
    }
}

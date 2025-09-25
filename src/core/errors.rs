use crate::core::memory::MemoryError;

/// Core error types for the PVM
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PvmError {
    Memory(MemoryError),
    InvalidInstruction,
    InvalidAccess,
    StackOverflow,
    StackUnderflow,
    HeapOverflow,
    OutOfGas,
    Panic,
}

impl From<MemoryError> for PvmError {
    fn from(err: MemoryError) -> Self {
        PvmError::Memory(err)
    }
}

/// Result type for PVM operations
pub type PvmResult<T> = Result<T, PvmError>;

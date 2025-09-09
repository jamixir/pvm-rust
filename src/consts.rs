pub const ZP: usize = 0x1000; // page size (4KB)
pub const ZZ: usize = 0x1000; // min allowed address (4KB)
pub const ZI: usize = 0x100_0000; // layout buffer size (16MB)
pub const MAX_ADDR: usize = 0x1_0000_0000; // 4GB

pub const PAGE_SIZE: usize = ZP;
pub const MIN_ADDR: usize = ZZ;
pub const MEMORY_SIZE: usize = MAX_ADDR;
pub const LAYOUT_BUFFER_SIZE: usize = ZI;
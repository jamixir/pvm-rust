use crate::core::consts::{MEMORY_SIZE, MIN_ADDR, PAGES_PER_ACCESS_WORD, PAGE_SIZE};
use memmap::{MmapMut, MmapOptions};

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Permission {
    None = 0b00,
    Read = 0b01,
    ReadWrite = 0b11,
}

impl From<u8> for Permission {
    fn from(value: u8) -> Self {
        match value {
            0b00 => Permission::None,
            0b01 => Permission::Read,
            0b11 => Permission::ReadWrite,
            _ => Permission::None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemoryError {
    Fault { page_addr: usize },
    Panic,
}

#[derive(Debug)]
pub struct Memory {
    mmap: MmapMut,
    access: Vec<u64>,
    pub current_heap_pointer: usize,
    pub max_heap_pointer: usize,
}

impl Default for Memory {
    fn default() -> Self {
        Self::builder().build()
    }
}

pub struct MemoryBuilder {
    memory: Memory,
}

impl Memory {
    #[inline]
    pub fn builder() -> MemoryBuilder {
        let mmap = MmapOptions::new().len(MEMORY_SIZE).map_anon().unwrap();
        let total_pages = MEMORY_SIZE / PAGE_SIZE;
        let access_entries = total_pages.div_ceil(PAGES_PER_ACCESS_WORD);
        let memory = Memory {
            mmap,
            access: vec![0; access_entries],
            current_heap_pointer: MIN_ADDR,
            max_heap_pointer: MEMORY_SIZE,
        };
        MemoryBuilder { memory }
    }

    pub fn test_memory(num_pages: usize) -> Memory {
        let mmap = MmapOptions::new()
            .len(num_pages * PAGE_SIZE)
            .map_anon()
            .unwrap();
        let access_entries = num_pages.div_ceil(PAGES_PER_ACCESS_WORD);
        Memory {
            mmap,
            access: vec![u64::MAX; access_entries], // read and write are allowed
            current_heap_pointer: 0,
            max_heap_pointer: num_pages * PAGE_SIZE,
        }
    }

    #[inline(always)]
    pub fn read(&self, addr: usize, len: usize) -> Result<&[u8], MemoryError> {
        if len == 0 {
            return Ok(&[]);
        }
        if addr < MIN_ADDR {
            return Err(MemoryError::Panic);
        }
        if self.check_access(addr, len, Permission::Read) {
            Ok(&self.mmap[addr..addr + len])
        } else {
            let page_addr = addr & !(PAGE_SIZE - 1);
            Err(MemoryError::Fault { page_addr })
        }
    }

    #[inline(always)]
    pub fn write(&mut self, addr: usize, data: &[u8]) -> Result<(), MemoryError> {
        if data.is_empty() {
            return Ok(());
        }
        if addr < MIN_ADDR {
            return Err(MemoryError::Panic);
        }
        if self.check_access(addr, data.len(), Permission::ReadWrite) {
            self.mmap[addr..addr + data.len()].copy_from_slice(data);
            Ok(())
        } else {
            let page_addr = addr & !(PAGE_SIZE - 1);
            Err(MemoryError::Fault { page_addr })
        }
    }

    #[inline(always)]
    pub fn check_access(&self, addr: usize, len: usize, required_permission: Permission) -> bool {
        // Check for overflow in addr + len
        let end_addr = match addr.checked_add(len) {
            Some(end) => end,
            None => return false,
        };

        if end_addr > self.mmap.len() {
            return false;
        }

        let page_start = addr >> 12;
        let page_end: usize = end_addr.div_ceil(PAGE_SIZE);
        let required_bits = required_permission as u64;

        unsafe {
            for page in page_start..page_end {
                let entry_index = page >> 5; // page / 32

                let bit_offset = (page & 31) << 1; // (page % 32) * 2
                let entry = *self.access.get_unchecked(entry_index);
                let page_permission = (entry >> bit_offset) & 0b11;
                if (page_permission & required_bits) != required_bits {
                    return false;
                }
            }
        }

        true
    }

    #[inline]
    pub fn set_access(&mut self, addr: usize, len: usize, permission: Permission) {
        let start_page = addr >> 12;
        let end_page = (addr + len).div_ceil(PAGE_SIZE);
        let permission_bits = permission as u64;

        unsafe {
            for page in start_page..end_page {
                let entry_idx = page >> 5; // (page / 32)
                let bit_offset = (page & 31) << 1; // (page % 32) * 2
                let mask = 0b11u64 << bit_offset;
                let entry = self.access.get_unchecked_mut(entry_idx);
                // Clear the 2 bits for this page
                *entry &= !mask;
                // Set the new permissions
                *entry |= permission_bits << bit_offset;
            }
        }
    }
}

impl MemoryBuilder {
    #[inline(always)]
    pub fn get_mut_slice(&mut self, addr: usize, len: usize) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.memory.mmap.as_mut_ptr().add(addr), len) }
    }

    #[inline(always)]
    pub fn set_access(&mut self, addr: usize, len: usize, permission: Permission) {
        self.memory.set_access(addr, len, permission);
    }

    #[inline]
    pub fn set_heap_bounds(&mut self, current: usize, max: usize) {
        self.memory.current_heap_pointer = current;
        self.memory.max_heap_pointer = max;
    }

    #[inline(always)]
    pub fn build(self) -> Memory {
        self.memory
    }
}

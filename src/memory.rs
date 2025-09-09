// src/memory.rs

use crate::consts::{MEMORY_SIZE, MIN_ADDR, PAGE_SIZE};
use memmap::{MmapMut, MmapOptions};

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Permission {
    None = 0b00,
    Read = 0b01,
    ReadWrite = 0b11,
}

pub struct Memory {
    mmap: MmapMut,
    access: Vec<u64>,
    current_heap_pointer: usize,
    max_heap_pointer: usize,
}

pub struct MemoryBuilder {
    memory: Memory,
}

impl Memory {
    #[inline]
    pub fn builder() -> MemoryBuilder {
        let mmap = unsafe { MmapOptions::new().len(MEMORY_SIZE).map_anon().unwrap() };
        let total_pages = MEMORY_SIZE / PAGE_SIZE;
        let access_entries = (total_pages + 31) / 32;
        let memory = Memory {
            mmap,
            access: vec![0; access_entries],
            current_heap_pointer: MIN_ADDR,
            max_heap_pointer: MEMORY_SIZE,
        };
        MemoryBuilder { memory }
    }

    #[inline(always)]
    pub fn read(&self, addr: usize, len: usize) -> &[u8] {
        &self.mmap[addr..addr + len]
    }

    #[inline(always)]
    pub fn write(&mut self, addr: usize, data: &[u8]) {
        self.mmap[addr..addr + data.len()].copy_from_slice(data);
    }

    #[inline(always)]
    pub fn check_access(&self, addr: usize, len: usize, required_permission: Permission) -> bool {
        let page_start = addr / PAGE_SIZE;
        let page_end: usize = (addr + len + PAGE_SIZE - 1) / PAGE_SIZE;
        let required_bits = required_permission as u64;

        unsafe {
            for page in page_start..page_end {
                let entry_index = page >> 5; // (page / 32)

                let bit_offset = (page & 31) << 1; // (page %32) * 2
                let entry = *self.access.get_unchecked(entry_index);
                let page_permission = ((entry >> bit_offset) & 0b11);
                if (page_permission & required_bits) != required_bits {
                    return false;
                }
            }
        }

        true
    }

    #[inline]
    pub fn set_access(&mut self, addr: usize, len: usize, permissions: Permission) {
        let start_page = addr / PAGE_SIZE;
        let end_page = (addr + len + PAGE_SIZE - 1) / PAGE_SIZE;
        let permission_bits = permissions as u64;
        

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

    #[inline]
    pub fn size(&self) -> usize {
        self.mmap.len()
    }
}

impl MemoryBuilder {
    #[inline(always)]
    pub fn get_mut_slice(&mut self, addr: usize, len: usize) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.memory.mmap.as_mut_ptr().add(addr), len) }
    }

    #[inline(always)]
    pub fn set_access(&mut self, addr: usize, len: usize, permissions: Permission) {
        self.memory.set_access(addr, len, permissions);
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

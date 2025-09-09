use memmap::MmapMut;
use rustler::{Env, Term};
use rustler::types::atom::ok;
use rustler::Encoder;
use std::fs::File;
use std::io::Result;

use crate::memory::Memory;

pub struct Vm {
    pub registers: [u64; 13],
    pub memory: Memory,
    pub program: Vec<u8>,
    pub bitmask: Vec<u64>,
    pub pc: usize,
    pub gas: u64,
}

impl Vm {
    pub fn new(program: Vec<u8>) -> Self {
        Self {
            registers: [0; 13],
            memory: Memory::builder().build(),
            program,
            bitmask: vec![0; 13],
            pc: 0,
            gas: 0,
        }
    }
}
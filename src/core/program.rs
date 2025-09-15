use std::ops::Index;

use rustler::{Decoder, Encoder, Env, Term};

#[derive(Debug)]
pub struct Program {
    pub program: Vec<u8>,
    pub size: usize,
}

impl Program {
    pub fn new(data: &[u8]) -> Self {
        let size = data.len();
        let mut program = Self {
            program: data.to_vec(),
            size,
        };
        program.pad();
        program
    }

    pub fn from_vec(data: Vec<u8>) -> Self {
        let size = data.len();
        let mut program = Self {
            program: data,
            size,
        };
        program.pad();
        program
    }

    pub fn pad(&mut self) {
        self.program.extend(vec![0u8; 32]);
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn as_safe_slice(&self) -> &[u8] {
        &self.program
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.program[..self.size]
    }
}

impl AsRef<[u8]> for Program {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl Index<usize> for Program {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.as_slice()[index]
    }
}

impl Decoder<'_> for Program {
    fn decode(term: Term) -> rustler::NifResult<Self> {
        let list: Vec<u8> = term.decode()?;
        let size = list.len();
        Ok(Program {
            program: list,
            size,
        })
    }
}

impl Encoder for Program {
    fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
        self.program.encode(env)
    }
}

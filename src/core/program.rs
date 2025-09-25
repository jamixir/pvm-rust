use std::ops::Index;

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

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.program[..self.size]
    }
}

impl Index<usize> for Program {
    type Output = u8;

    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        unsafe { self.program.get_unchecked(index) }
    }
}

impl Index<std::ops::Range<usize>> for Program {
    type Output = [u8];

    #[inline(always)]
    fn index(&self, range: std::ops::Range<usize>) -> &Self::Output {
        unsafe { self.program.get_unchecked(range) }
    }
}

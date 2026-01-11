use crate::core::{BitMask, Program};

#[cfg_attr(feature = "std", derive(thiserror::Error))]
#[derive(Debug, Clone, PartialEq)]
pub enum DecodeError {
    #[error("Insufficient bytes: needed {needed}, available {available}")]
    InsufficientBytes { needed: usize, available: usize },
    #[error("Invalid integer encoding")]
    InvalidIntegerEncoding,
    #[error("Invalid program length")]
    InvalidProgramLength,
}

pub struct Deblob {
    pub program: Program,
    pub bitmask: BitMask,
    pub jump_table: Vec<usize>,
}

pub fn deblob(bin: &[u8]) -> Result<Deblob, DecodeError> {
    let (jump_len, rest) = decode_next_integer(bin)?;
    let (z, rest) = decode_next_le(rest, 1)?;
    let (prog_len, rest) = decode_next_integer(rest)?;

    let (jump_table, rest) = decode_jump_table(rest, jump_len as usize, z)?;

    if rest.len() < prog_len as usize {
        return Err(DecodeError::InsufficientBytes {
            needed: prog_len as usize,
            available: rest.len(),
        });
    }

    let (program_bytes, bitmask_bytes) = rest.split_at(prog_len as usize);
    let program = Program::new(program_bytes);

    let bitmask = BitMask::from_bytes(bitmask_bytes, prog_len as usize);

    Ok(Deblob {
        program,
        bitmask,
        jump_table,
    })
}

fn decode_next_integer(bin: &[u8]) -> Result<(u64, &[u8]), DecodeError> {
    if bin.is_empty() {
        return Err(DecodeError::InsufficientBytes {
            needed: 1,
            available: 0,
        });
    }

    match bin[0] {
        0 => Ok((0, &bin[1..])),
        1..=127 => Ok((bin[0] as u64, &bin[1..])),
        255 => {
            if bin.len() < 9 {
                return Err(DecodeError::InsufficientBytes {
                    needed: 9,
                    available: bin.len(),
                });
            }
            let value = u64::from_le_bytes(bin[1..9].try_into().unwrap());
            Ok((value, &bin[9..]))
        }
        128..=254 => {
            let (l, a_l) = determine_l_and_a_l(bin[0])?;
            if bin.len() < 1 + l {
                return Err(DecodeError::InsufficientBytes {
                    needed: 1 + l,
                    available: bin.len(),
                });
            }
            let h = (bin[0] - a_l) as u64;
            let x_rest = decode_le(&bin[1..1 + l], l)?;
            Ok((h << (8 * l) | x_rest, &bin[1 + l..]))
        }
    }
}

fn determine_l_and_a_l(byte0: u8) -> Result<(usize, u8), DecodeError> {
    if byte0 >= 254 {
        Ok((7, 254))
    } else if byte0 >= 252 {
        Ok((6, 252))
    } else if byte0 >= 248 {
        Ok((5, 248))
    } else if byte0 >= 240 {
        Ok((4, 240))
    } else if byte0 >= 224 {
        Ok((3, 224))
    } else if byte0 >= 192 {
        Ok((2, 192))
    } else if byte0 >= 128 {
        Ok((1, 128))
    } else {
        Err(DecodeError::InvalidIntegerEncoding)
    }
}
#[inline(always)]
pub fn decode_le(bin: &[u8], l: usize) -> Result<u64, DecodeError> {
    if bin.len() < l {
        return Err(DecodeError::InsufficientBytes {
            needed: l,
            available: bin.len(),
        });
    }
    let mut buf = [0u8; 8];
    buf[..l].copy_from_slice(&bin[..l]);
    Ok(u64::from_le_bytes(buf))
}

#[inline(always)]
fn decode_next_le(bin: &[u8], l: usize) -> Result<(u64, &[u8]), DecodeError> {
    if bin.len() < l {
        return Err(DecodeError::InsufficientBytes {
            needed: l,
            available: bin.len(),
        });
    }
    let mut buf = [0u8; 8];
    buf[..l].copy_from_slice(&bin[0..l]);
    let value = u64::from_le_bytes(buf);
    Ok((value, &bin[l..]))
}

fn decode_jump_table(
    mut bin: &[u8],
    size: usize,
    z: u64,
) -> Result<(Vec<usize>, &[u8]), DecodeError> {
    if size == 0 {
        return Ok((Vec::new(), bin));
    }

    let mut values: Vec<usize> = Vec::with_capacity(size);
    let z_size = z as usize;

    for _ in 0..size {
        let (value, rest) = decode_next_le(bin, z_size)?;
        values.push(value as usize);
        bin = rest;
    }

    Ok((values, bin))
}

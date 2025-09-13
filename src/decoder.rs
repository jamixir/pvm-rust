pub struct Deblob<'a> {
    pub program: &'a [u8],
    pub bitmask: Vec<u64>,
    pub jump_table: Vec<u64>,
}

pub fn deblob(bin: &[u8]) -> Deblob {
    let (jump_len, rest) = decode_next_integer(bin);
    let (z, rest) = decode_next_le(rest, 1);
    let (prog_len, rest) = decode_next_integer(rest);

    let (jump_table, rest) = decode_jump_table(rest, jump_len as usize, z);

    let (program_bytes, bitmask_bytes) = rest.split_at(prog_len as usize);
    let mut bitmask = pack_bitmask_le(bitmask_bytes);
    add_trap_bit(&mut bitmask, prog_len as usize);

    Deblob {
        program: program_bytes,
        bitmask,
        jump_table,
    }
}

pub fn pack_bitmask_le(bytes: &[u8]) -> Vec<u64> {
    let mut words: Vec<u64> = bytes
        .chunks_exact(8)
        .map(|chunk| u64::from_le_bytes(chunk.try_into().unwrap()))
        .collect();

    let remainder = bytes.chunks_exact(8).remainder();
    if !remainder.is_empty() {
        let mut buf = [0u8; 8];
        buf[..remainder.len()].copy_from_slice(&remainder);
        words.push(u64::from_le_bytes(buf));
    }
    words
}

pub fn add_trap_bit(bitmask: &mut Vec<u64>, trap_pos: usize) {
    let word_index = trap_pos >> 6;
    let bit_index = trap_pos & 63;

    if word_index >= bitmask.len() {
        bitmask.resize(word_index + 1, 0);
    }
    bitmask[word_index] |= 1 << bit_index;
}

#[inline(always)]
fn decode_next_integer(bin: &[u8]) -> (u64, &[u8]) {
    if bin.is_empty() {
        panic!("decode_next_integer: empty input");
    }

    match bin[0] {
        0 => (0, &bin[1..]),

        1..=127 => (bin[0] as u64, &bin[1..]),

        255 => {
            if bin.len() < 9 {
                panic!("decode_next_integer: not enough bytes for 8-byte integer");
            }
            let value = u64::from_le_bytes(bin[1..9].try_into().unwrap());
            (value, &bin[9..])
        }

        128..=254 => {
            let (l, a_l) = determine_l_and_a_l(bin[0]);
            if bin.len() < 1 + l {
                panic!("decode_next_integer: not enough bytes for medium integer");
            }
            let h = (bin[0] - a_l) as u64;
            let x_rest = decode_le(&bin[1..1 + l], l);
            let value = (h << (8 * l)) | x_rest;

            (value, &bin[1 + l..])
        }
    }
}

fn determine_l_and_a_l(byte0: u8) -> (usize, u8) {
    if byte0 >= 254 {
        (7, 254)
    } else if byte0 >= 252 {
        (6, 252)
    } else if byte0 >= 248 {
        (5, 248)
    } else if byte0 >= 240 {
        (4, 240)
    } else if byte0 >= 224 {
        (3, 224)
    } else if byte0 >= 192 {
        (2, 192)
    } else if byte0 >= 128 {
        (1, 128)
    } else {
        panic!("Invalid byte for determine_l_and_a_l: {}", byte0);
    }
}

#[inline(always)]
fn decode_le(bin: &[u8], l: usize) -> u64 {
    let mut buf = [0u8; 8];
    buf[..l].copy_from_slice(&bin[0..l]);
    u64::from_le_bytes(buf)
}

#[inline(always)]
fn decode_next_le(bin: &[u8], l: usize) -> (u64, &[u8]) {
    let mut buf = [0u8; 8];
    buf[..l].copy_from_slice(&bin[0..l]);
    let value = u64::from_le_bytes(buf);
    (value, &bin[l..])
}

fn decode_jump_table(mut bin: &[u8], size: usize, z: u64) -> (Vec<u64>, &[u8]) {
    if size == 0 {
        return (Vec::new(), bin);
    }

    let mut values = Vec::with_capacity(size);
    let z_size = z as usize;

    for _ in 0..size {
        let (value, rest) = decode_next_le(bin, z_size);
        values.push(value);
        bin = rest;
    }

    (values, bin)
}

#[cfg(test)]
mod tests {
    use crate::{decoder::deblob, encoder::encode_program};

    fn encode_decode(
        program: &[u8],
        bitmask: &[u8],
        jump_table: &[u64],
        z: u64,
    ) -> (Vec<u8>, Vec<u64>, Vec<u64>) {
        let encoded = encode_program(program, bitmask, jump_table, z);
        let blob = deblob(&encoded);
        (blob.program.to_vec(), blob.bitmask, blob.jump_table)
    }

    #[test]
    fn test_empty_jump_table() {
        let program = &[60, 171, 142, 73, 61, 2, 3];
        let bitmask = &[137];
        let jump_table: &[u64] = &[];
        let z = 1;

        let (decoded_program, decoded_bitmask, decoded_jump_table) =
            encode_decode(program, bitmask, jump_table, z);

        assert_eq!(decoded_program, program);
        assert_eq!(decoded_jump_table, jump_table);
        assert_eq!(decoded_bitmask.len(), 1); // single word may hold first 8 bits
                                              // Could also compare bits manually if needed
    }

    #[test]
    fn test_jump_table() {
        let program = &[1, 2, 3, 4];
        let bitmask = &[0xFF];
        let jump_table: &[u64] = &[10, 20, 30];
        let z = 1;

        let (decoded_program, decoded_bitmask, decoded_jump_table) =
            encode_decode(program, bitmask, jump_table, z);

        assert_eq!(decoded_program, program);
        assert_eq!(decoded_jump_table, jump_table);
        assert_eq!(decoded_bitmask.len(), 1);
    }

    #[test]
    fn test_larger_z() {
        let program = &[1, 2, 3, 4];
        let bitmask = &[0xFF];
        let jump_table: &[u64] = &[256, 512, 1024]; // requires 2 bytes each
        let z = 2;

        let (decoded_program, _decoded_bitmask, decoded_jump_table) =
            encode_decode(program, bitmask, jump_table, z);

        assert_eq!(decoded_program, program);
        assert_eq!(decoded_jump_table, jump_table);
    }

    #[test]
    fn test_max_jump_and_program() {
        let program = &[1u8; 100];
        let bitmask = &[0xAA, 0x55];
        let jump_table: &[u64] = &[0, 1, 2, 255, 1024];
        let z = 2;

        let (decoded_program, decoded_bitmask, decoded_jump_table) =
            encode_decode(program, bitmask, jump_table, z);

        assert_eq!(decoded_program, program);
        assert_eq!(decoded_jump_table, jump_table);
        assert!(!decoded_bitmask.is_empty());
    }

    #[test]
    fn test_trap_bit_set() {
        let program = &[0x01, 0x02, 0x03];
        let bitmask = &[0b10100000]; // 1 byte
        let jump_table: &[u64] = &[];
        let z = 1;

        let (_decoded_program, decoded_bitmask, _decoded_jump_table) =
            encode_decode(program, bitmask, jump_table, z);

        // Trap bit is always added at program_len
        let trap_bit_pos = program.len();
        let word_index = trap_bit_pos / 64;
        let bit_index = trap_bit_pos % 64;
        assert_eq!(
            (decoded_bitmask[word_index] >> bit_index) & 1,
            1,
            "Trap bit not set correctly"
        );
    }
}

#[cfg(test)]
mod bitmask_skip_tests {
    use super::*;
    use crate::utils::skip;

    fn compute_skips(bitmask: &[u64]) -> Vec<usize> {
        let mut skips = Vec::new();
        let mut pc = 0;

        loop {
            let skip_distance = skip(bitmask, pc);
            if skip_distance.is_none() {
                break;
            }
            let skip_distance = skip_distance.unwrap();

            if pc + skip_distance < bitmask.len() * 64 {
                skips.push(skip_distance);
                pc += skip_distance + 1; //
            } else {
                break;
            }
        }

        skips
    }

    #[test]
    fn bitmask_packing_and_skip_consistency() {
        let bitmask_bytes = &[0b1010_0001, 0b0000_0100];
        let packed = pack_bitmask_le(bitmask_bytes);

        let expected_skips = vec![4, 1, 2];

        let actual_skips = compute_skips(&packed);

        assert_eq!(
            expected_skips, actual_skips,
            "Packed bitmask skips mismatch"
        );
    }

    #[test]
    fn bitmask_with_trap_and_skips() {
        let bitmask_bytes = &[0b1010_1011, 0b0001_0000];
        let prog_len = 13; // cut program length to force trap in last bits
        let mut packed = pack_bitmask_le(bitmask_bytes);
        add_trap_bit(&mut packed, prog_len);

        let actual_skips = compute_skips(&packed);

        let expected_skips = vec![0, 1, 1, 1, 4, 0];

        assert_eq!(
            expected_skips, actual_skips,
            "Skips including trap mismatch"
        );
    }

    #[test]
    fn concrete_single_u64_bitmask_examples() {
        // Example 1: Single u64 word (8 bytes)
        // Little-endian representation: 0x8101010101010101
        // Binary (LE): 0000_0001 0000_0001 0000_0001 0000_0001 0000_0001 0000_0001 0000_0001 1000_0001
        // Bit positions (0-indexed): 0, 8, 16, 24, 32, 40, 48, 56, 63
        // First bit is always 1 (program starts from instruction)
        let bitmask_bytes = &[0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x81]; // 8 bytes = 1 u64
        let mut packed = pack_bitmask_le(bitmask_bytes);
        let prog_len = 60; // Program shorter than full bitmask
        add_trap_bit(&mut packed, prog_len);

        let expected_skips = vec![7, 7, 7, 7, 7, 7, 7, 3, 2];
        let actual_skips = compute_skips(&packed);
        assert_eq!(expected_skips, actual_skips, "Single u64 skip mismatch");
    }

    #[test]
    fn concrete_trap_bit_effects() {
        let bitmask_bytes = &[0x05]; // Binary: 0000_0101 (bits 0 and 2 set)

        // Test 1:(trap at bit 3)
        let mut packed1 = pack_bitmask_le(bitmask_bytes);
        add_trap_bit(&mut packed1, 3);
        // Set bits: 0, 2, 3 (trap) -> skip distances: 1, 0
        let expected_skips1 = vec![1, 0];
        let actual_skips1 = compute_skips(&packed1);
        assert_eq!(expected_skips1, actual_skips1, "Short program trap effect");

        // Test 2:  (trap at bit 5)
        let mut packed2 = pack_bitmask_le(bitmask_bytes);
        add_trap_bit(&mut packed2, 5);
        // Set bits: 0, 2, 5 (trap) -> skip distances: 1, 2
        let expected_skips2 = vec![1, 2];
        let actual_skips2 = compute_skips(&packed2);
        assert_eq!(expected_skips2, actual_skips2, "Medium program trap effect");

        // Test 3: (trap at bit 10)
        let mut packed3 = pack_bitmask_le(bitmask_bytes);
        add_trap_bit(&mut packed3, 10);
        // Set bits: 0, 2, 10 (trap) -> skip distances: 1, 7
        let expected_skips3 = vec![1, 7];
        let actual_skips3 = compute_skips(&packed3);
        assert_eq!(expected_skips3, actual_skips3, "Long program trap effect");
    }
}

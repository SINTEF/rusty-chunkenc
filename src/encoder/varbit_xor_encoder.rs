/// Writes a f64 as a Prometheus varbit xor encoded number.
///
/// It also needs the previous value, the previous leading and trailing bits count.
///
/// The first time it is called, use 0xff (or 255) for the leading bits counts,
/// and 0 for the trailing bits count.
pub fn write_varbit_xor<W: bitstream_io::BitWrite>(
    value: f64,
    previous_value: f64,
    previous_leading_bits_count: u8,
    previous_trailing_bits_count: u8,
    bit_writer: &mut W,
) -> std::io::Result<(u8, u8)> {
    let delta = value.to_bits() ^ previous_value.to_bits();

    if delta == 0 {
        bit_writer.write_bit(false)?;
        return Ok((previous_leading_bits_count, previous_trailing_bits_count));
    }
    bit_writer.write_bit(true)?;

    let mut new_leading = delta.leading_zeros() as u8;
    let new_trailing = delta.trailing_zeros() as u8;

    // Weird clamping that I don't reproduce, but it's there
    if new_leading >= 32 {
        new_leading = 31;
    }

    // If we reuse the previous leading and trailing bit counts
    if previous_leading_bits_count != 0xff
        && new_leading >= previous_leading_bits_count
        && new_trailing >= previous_trailing_bits_count
    {
        bit_writer.write_bit(false)?;
        bit_writer.write(
            64_u32 - (previous_leading_bits_count as u32) - previous_trailing_bits_count as u32,
            delta >> previous_trailing_bits_count,
        )?;
        return Ok((previous_leading_bits_count, previous_trailing_bits_count));
    }

    bit_writer.write_bit(true)?;
    bit_writer.write(5, new_leading)?;
    let sigbits = (64_u64 - new_leading as u64) - new_trailing as u64;
    // Overflow 64 to 0 is fine because if 0 sigbits, we would have written a "same number"
    // bit a bit earlier.
    // The reason is that only 6 bits are available, and the maximum value is 63.
    let encoded_sigbits = if sigbits > 63 { 0 } else { sigbits };
    bit_writer.write(6, encoded_sigbits)?;
    bit_writer.write(sigbits as u32, delta >> new_trailing)?;

    Ok((new_leading, new_trailing))
}

#[cfg(test)]
mod tests {
    use core::f64;

    use crate::varbit_xor::read_varbit_xor;

    use super::*;
    use bitstream_io::{BigEndian, BitWrite, BitWriter};
    use rand::{Rng, SeedableRng};

    fn generate_random_test_data(seed: u64) -> Vec<Vec<f64>> {
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

        let mut test_cases = Vec::with_capacity(128);
        for _ in 0..128 {
            let vec_size = rng.gen_range(1..129);
            let mut vec = Vec::with_capacity(vec_size);

            let mut value: f64 = rng.gen();
            vec.push(value);

            for _ in 1..vec_size {
                if rng.gen_bool(0.33) {
                    value += 1.0;
                } else if rng.gen_bool(0.33) {
                    value = rng.gen();
                }
                vec.push(value);
            }
            test_cases.push(vec);
        }
        test_cases
    }

    #[test]
    fn test_write_varbit_xor() {
        let mut test_cases = generate_random_test_data(42);

        // add just a test case with the weird clamping
        test_cases.push(vec![f64::MAX, 0.0, f64::MIN, f64::MAX, f64::MIN]);

        for test_case in test_cases {
            let mut buffer: Vec<u8> = Vec::new();

            // Writing first
            let mut bit_writer = BitWriter::endian(&mut buffer, BigEndian);

            let mut value = 0.0;
            let mut leading = 0xff;
            let mut trailing = 0;

            for number in &test_case {
                let (new_leading, new_trailing) =
                    write_varbit_xor(*number, value, leading, trailing, &mut bit_writer).unwrap();
                value = *number;
                leading = new_leading;
                trailing = new_trailing;
            }

            bit_writer.byte_align().unwrap();

            // Read again
            value = 0.0;
            leading = 0;
            trailing = 0;

            let mut cursor: (&[u8], usize) = (&buffer, 0);

            for number in test_case {
                let (new_cursor, (new_value, new_leading, new_trailing)) =
                    read_varbit_xor(value, leading, trailing)(cursor).unwrap();
                cursor = new_cursor;
                assert_eq!(new_value, number);
                value = new_value;
                leading = new_leading;
                trailing = new_trailing;
            }
        }
    }
}

/// Writes a i64 as a Prometheus varbit timestamp.
pub fn write_varbit_ts<W: bitstream_io::BitWrite>(
    value: i64,
    writer: &mut W,
) -> std::io::Result<()> {
    match value {
        0 => writer.write_bit(false)?,
        // 1 to 14 bits
        -8191..=8192 => {
            writer.write_out::<2, u8>(0b10)?;
            writer.write_out::<14, u64>(value as u64 & 0x3FFF)?;
        }
        // 15 to 17 bits
        -65535..=65536 => {
            writer.write_out::<3, u8>(0b110)?;
            writer.write_out::<17, u64>(value as u64 & 0x1FFFF)?;
        }
        // 18 to 20 bits
        -524287..=524288 => {
            writer.write_out::<4, u8>(0b1110)?;
            writer.write_out::<20, u64>(value as u64 & 0x0FFFFF)?;
        }
        _ => {
            writer.write_out::<4, u8>(0b1111)?;
            writer.write_out::<64, u64>(value as u64)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {

    use crate::varbit_ts::read_varbit_ts;

    use super::*;
    use bitstream_io::{BigEndian, BitWrite, BitWriter};
    use rand::{Rng, SeedableRng};

    fn generate_random_test_data(seed: u64) -> Vec<Vec<i64>> {
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

        let mut test_cases = Vec::with_capacity(128);
        for _ in 0..128 {
            let vec_size = rng.gen_range(1..129);
            let mut vec = Vec::with_capacity(vec_size);

            let mut value: i64 = if rng.gen_bool(0.5) {
                rng.gen_range(-100000000..1000000)
            } else {
                rng.gen_range(-10000..10000)
            };
            vec.push(value);

            for _ in 1..vec_size {
                if rng.gen_bool(0.33) {
                    value += 1;
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
    fn test_write_varbit_ts() {
        let mut test_cases = generate_random_test_data(42);

        // add just a test case with the weird clamping
        test_cases.push(vec![i64::MAX, 0, i64::MIN, i64::MAX, i64::MIN]);

        for test_case in test_cases {
            let mut buffer: Vec<u8> = Vec::new();

            // Writing first
            let mut bit_writer = BitWriter::endian(&mut buffer, BigEndian);

            for number in &test_case {
                write_varbit_ts(*number, &mut bit_writer).unwrap();
            }

            bit_writer.byte_align().unwrap();

            // Read again
            let mut cursor: (&[u8], usize) = (&buffer, 0);
            for number in test_case {
                let (new_cursor, new_value) = read_varbit_ts(cursor).unwrap();
                cursor = new_cursor;
                assert_eq!(new_value, number);
            }
        }
    }
}

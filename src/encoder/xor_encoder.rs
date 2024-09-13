use bitstream_io::{BigEndian, BitWrite, BitWriter};
use smallvec::SmallVec;

use crate::xor::{XORChunk, XORSample};

use super::{
    uvarint_encoder::write_uvarint, varbit_ts_encoder::write_varbit_ts,
    varbit_xor_encoder::write_varbit_xor, varint_encoder::write_varint,
};

fn write_first_sample<W: std::io::Write>(
    first_sample: &XORSample,
    writer: &mut W,
) -> std::io::Result<()> {
    let XORSample { timestamp, value } = first_sample;

    write_varint(*timestamp, writer)?;

    // Classic Float64 for the value
    writer.write_all(&value.to_be_bytes())?;

    Ok(())
}

#[derive(Debug)]
struct XORReadIterator {
    pub timestamp: i64,
    pub value: f64,
    pub leading_bits_count: u8,
    pub trailing_bits_count: u8,
    pub timestamp_delta: i64,
}

fn write_second_sample<W: bitstream_io::BitWrite>(
    second_sample: &XORSample,
    first_sample: &XORSample,
    writer: &mut W,
) -> std::io::Result<XORReadIterator> {
    let timestamp = second_sample.timestamp;
    let value = second_sample.value;

    let timestamp_delta = timestamp - first_sample.timestamp;
    if timestamp_delta < 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "samples aren't sorted by timestamp ascending",
        ));
    }

    // I didn't find a more beautiful way to write the uvarint in the bitstream directly,
    // we use SmallVec so at least it's allocated on the stack and not the heap.
    let mut uvarint_bytes = SmallVec::<u8, 9>::new();
    write_uvarint(timestamp_delta as u64, &mut uvarint_bytes)?;
    writer.write_bytes(&uvarint_bytes)?;

    let (leading, trailing) = write_varbit_xor(value, first_sample.value, 0xff, 0, writer)?;

    Ok(XORReadIterator {
        timestamp,
        value,
        leading_bits_count: leading,
        trailing_bits_count: trailing,
        timestamp_delta,
    })
}

fn write_n_sample<W: bitstream_io::BitWrite>(
    previous_iterator: &XORReadIterator,
    sample: &XORSample,
    writer: &mut W,
) -> std::io::Result<XORReadIterator> {
    let timestamp = sample.timestamp;
    let value = sample.value;

    let timestamp_delta = timestamp - previous_iterator.timestamp;
    if timestamp_delta < 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "samples aren't sorted by timestamp ascending",
        ));
    }
    let timestamp_delta_of_delta = timestamp_delta - previous_iterator.timestamp_delta;

    write_varbit_ts(timestamp_delta_of_delta, writer)?;

    let (leading_bits_count, trailing_bits_count) = write_varbit_xor(
        value,
        previous_iterator.value,
        previous_iterator.leading_bits_count,
        previous_iterator.trailing_bits_count,
        writer,
    )?;

    Ok(XORReadIterator {
        timestamp,
        value,
        leading_bits_count,
        trailing_bits_count,
        timestamp_delta,
    })
}

impl XORChunk {
    /// Writes the XOR chunk to the writer.
    pub fn write<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        // Write the number of samples first
        let samples = self.samples();

        let num_samples = samples.len();
        let num_samples_u16 = u16::try_from(num_samples).map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "too many samples for one chunk",
            )
        })?;
        writer.write_all(&num_samples_u16.to_be_bytes())?;

        if num_samples == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "at least one sample is required",
            ));
        }
        let first_sample = &samples[0];
        write_first_sample(first_sample, writer)?;

        if num_samples > 1 {
            let mut bit_writer = BitWriter::endian(writer, BigEndian);

            let second_sample = &samples[1];
            let mut iterator = write_second_sample(second_sample, first_sample, &mut bit_writer)?;

            for sample in &samples[2..] {
                iterator = write_n_sample(&iterator, sample, &mut bit_writer)?;
            }

            // Add 0 bits padding
            bit_writer.byte_align()?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use std::vec;

    use crate::xor::read_xor_chunk_data;

    use super::*;
    use rand::{Rng, SeedableRng};

    fn generate_random_test_data(seed: u64) -> Vec<Vec<XORSample>> {
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

        let mut test_cases = Vec::with_capacity(128);
        for _ in 0..128 {
            let mut timestamp: i64 = rng.gen_range(1234567890..1357908642);
            let vec_size = rng.gen_range(1..129);
            let mut vec = Vec::with_capacity(vec_size);

            let mut value: f64 = if rng.gen_bool(0.5) {
                rng.gen_range(-100000000.0..1000000.0)
            } else {
                rng.gen_range(-10000.0..10000.0)
            };
            vec.push(XORSample { timestamp, value });

            for _ in 1..vec_size {
                timestamp += rng.gen_range(1..30);
                if rng.gen_bool(0.33) {
                    value += 1.0;
                } else if rng.gen_bool(0.33) {
                    value = rng.gen();
                }
                vec.push(XORSample { timestamp, value });
            }
            test_cases.push(vec);
        }
        test_cases
    }

    #[test]
    fn test_write_xor_chunk() {
        let mut test_cases = generate_random_test_data(42);

        // add a test case with the unusually extreme values
        test_cases.push(vec![
            XORSample {
                timestamp: i64::MIN + 1,
                value: f64::MAX,
            },
            XORSample {
                timestamp: 0,
                value: 0.0,
            },
            XORSample {
                timestamp: 2,
                value: f64::MIN,
            },
            XORSample {
                timestamp: 3,
                value: f64::MAX,
            },
            XORSample {
                timestamp: i64::MAX - 1,
                value: f64::MIN,
            },
        ]);

        // add a test with only one sample
        test_cases.push(vec![XORSample {
            timestamp: 1234567890,
            value: 42.0,
        }]);

        // add a test with only two samples
        test_cases.push(vec![
            XORSample {
                timestamp: 1234567890,
                value: 42.0,
            },
            XORSample {
                timestamp: 1234567891,
                value: 42.0,
            },
        ]);

        let mut buffer: Vec<u8> = Vec::new();

        let mut chunks = Vec::with_capacity(test_cases.len());

        // Write
        for test_case in &test_cases {
            let chunk = XORChunk::new(test_case.clone());
            chunk.write(&mut buffer).unwrap();
            let mut tmp_buffer = Vec::new();
            chunk.write(&mut tmp_buffer).unwrap();
            chunks.push(chunk);
        }

        // Read again
        let mut cursor: &[u8] = &buffer;
        for (i, _test_case) in test_cases.iter().enumerate() {
            let (new_cursor, parsed_chunk) = read_xor_chunk_data(cursor).unwrap();
            let test_chunk = &chunks[i];
            assert_eq!(parsed_chunk.samples(), test_chunk.samples());
            cursor = new_cursor;
        }
    }

    #[test]
    fn test_write_xor_chunk_errors() {
        let test_cases = vec![
            vec![],
            // Two samples not sorted
            vec![
                XORSample {
                    timestamp: 10,
                    value: 42.0,
                },
                XORSample {
                    timestamp: -10,
                    value: 42.0,
                },
            ],
            // Three samples not sorted
            vec![
                XORSample {
                    timestamp: 9,
                    value: 42.0,
                },
                XORSample {
                    timestamp: 10,
                    value: 42.0,
                },
                XORSample {
                    timestamp: 9,
                    value: 43.0,
                },
            ],
            // 65536 samples
            [0; 65536]
                .iter()
                .enumerate()
                .map(|(i, _v)| XORSample {
                    timestamp: i as i64,
                    value: i as f64,
                })
                .collect::<Vec<XORSample>>(),
        ];

        let mut buffer: Vec<u8> = Vec::new();

        for test_case in &test_cases {
            let chunk = XORChunk::new(test_case.clone());
            assert!(chunk.write(&mut buffer).is_err());
        }
    }
}

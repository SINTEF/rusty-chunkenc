use crate::{
    chunk::{Chunk, ChunkType},
    crc32c::write_crc32c,
};

use super::uvarint_encoder::write_uvarint;

fn write_chunk_type<W: std::io::Write>(
    chunk_type: ChunkType,
    writer: &mut W,
) -> std::io::Result<()> {
    match chunk_type {
        ChunkType::XOR => {
            writer.write_all(&[1u8])?;
        }
        ChunkType::Histogram => {
            writer.write_all(&[2u8])?;
        }
        ChunkType::FloatHistogram => {
            writer.write_all(&[3u8])?;
        }
    }
    Ok(())
}

impl Chunk {
    /// Writes the chunk to the writer in the Prometheus format.
    pub fn write<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        // A chunk starts by its size, which we don't know yet
        // So… we can't stream the chunk writing, we have to
        // write it all in memory first.

        let mut buffer: Vec<u8> = Vec::with_capacity(32);

        match self {
            Chunk::XOR(xor_chunk) => {
                write_chunk_type(ChunkType::XOR, &mut buffer)?;
                xor_chunk.write(&mut buffer)?;
            }
            Chunk::Histogram(histogram_chunk) => {
                write_chunk_type(ChunkType::Histogram, &mut buffer)?;
                histogram_chunk.write(&mut buffer)?;
            }
            Chunk::FloatHistogram(float_histogram_chunk) => {
                write_chunk_type(ChunkType::FloatHistogram, &mut buffer)?;
                float_histogram_chunk.write(&mut buffer)?;
            }
        }

        let chunk_len = buffer.len() as u64 - 1;

        write_uvarint(chunk_len, writer)?;
        writer.write_all(&buffer)?;
        write_crc32c(&buffer, writer)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        chunk::read_chunk,
        xor::{XORChunk, XORSample},
    };

    use super::*;
    use rand::{Rng, SeedableRng};

    fn generate_random_test_data(seed: u64, count: usize) -> Vec<Chunk> {
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

        let mut test_cases = Vec::with_capacity(count);
        for _ in 0..count {
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
            test_cases.push(Chunk::XOR(XORChunk::new(vec)));
        }
        test_cases
    }

    #[test]
    fn test_write_chunk() {
        let test_cases = generate_random_test_data(1234, 128);

        let mut buffer: Vec<u8> = Vec::new();

        // Write
        for test_case in &test_cases {
            test_case.write(&mut buffer).unwrap();
        }

        // Read again
        let mut cursor: &[u8] = &buffer;
        for test_case in test_cases {
            let (new_cursor, parsed_chunk) = read_chunk(cursor).unwrap();
            assert_eq!(parsed_chunk, test_case);
            cursor = new_cursor;
        }
    }

    #[test]
    fn test_wrong_crc32c() {
        let test_cases = generate_random_test_data(1234, 1);
        let test_case = &test_cases[0];
        let mut buffer: Vec<u8> = Vec::new();
        test_case.write(&mut buffer).unwrap();
        // check that it's read correctly first
        let (_, parsed_chunk) = read_chunk(&buffer).unwrap();
        assert_eq!(&parsed_chunk, test_case);

        // Now corrupt the CRC32C
        let buffer_len = buffer.len();
        buffer[buffer_len - 4] = !buffer[buffer_len - 4];

        let error = read_chunk(&buffer).unwrap_err();
        assert!(error.to_string().contains("Verify"));
    }
}

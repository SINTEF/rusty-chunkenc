use crate::chunks::ChunksDiskFormat;

impl ChunksDiskFormat {
    pub fn write<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        // start with the magic code, the version, and the padding
        writer.write_all(&[0x85, 0xBD, 0x40, 0xDD, 1, 0, 0, 0])?;

        if self.chunks().is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "at least one chunk is required",
            ));
        }

        // Write all the chunks one by one
        for chunk in self.chunks() {
            chunk.write(writer)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use crate::{read_chunks, Chunk, XORSample};

    use super::*;

    #[test]
    fn test_write_chunks_disk_format() {
        let chunk_disk_format = ChunksDiskFormat::new(
            vec![
                Chunk::new_xor(vec![
                    XORSample {
                        timestamp: 7200000,
                        value: 12000.0,
                    },
                    XORSample {
                        timestamp: 7201000,
                        value: 12001.0,
                    },
                ]),
                Chunk::new_xor(vec![
                    XORSample {
                        timestamp: 7200000,
                        value: 123.45,
                    },
                    XORSample {
                        timestamp: 7201000,
                        value: 123.46,
                    },
                ]),
            ],
            None,
        );

        // Serialise the chunks
        let mut buffer: Vec<u8> = Vec::new();
        chunk_disk_format.write(&mut buffer).unwrap();

        // Parse a chunk from a buffer
        let (_, parsed_chunk_disk_format) = read_chunks(&buffer, None).unwrap();
        println!("parsed_chunks: {:?}", parsed_chunk_disk_format);
        assert_eq!(parsed_chunk_disk_format, chunk_disk_format);
    }

    #[test]
    fn test_without_chunks() {
        let chunk_disk_format = ChunksDiskFormat::new(vec![], None);

        // Serialise the chunks
        let mut buffer: Vec<u8> = Vec::new();
        let error = chunk_disk_format.write(&mut buffer).unwrap_err();
        assert!(error.to_string().contains("at least one chunk is required"));
    }
}

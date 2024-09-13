mod chunk_data;
mod index_data;
mod test_data;

#[cfg(test)]
mod tests {
    use rusty_chunkenc::{
        chunks::read_chunks,
        index, uvarint, varbit,
        xor::{read_xor_chunk_data, XORChunk, XORSample},
    };

    use super::*;

    #[test]
    fn test_read_varbit_int() {
        let test_data = &test_data::TEST_DATA;
        assert_eq!(test_data.varbit_ints.len(), 33);

        for varbit_int in &test_data.varbit_ints {
            let (_, value) = varbit::read_varbit_int((&varbit_int.e, 0)).unwrap();
            assert_eq!(value, varbit_int.v);
        }
    }

    #[test]
    fn test_read_varbit_uint() {
        let test_data = &test_data::TEST_DATA;
        assert_eq!(test_data.varbit_uints.len(), 17);

        for varbit_uint in &test_data.varbit_uints {
            let (_, value) = varbit::read_varbit_uint((&varbit_uint.e, 0)).unwrap();
            assert_eq!(value, varbit_uint.v);
        }
    }

    #[test]
    fn test_read_uvarint() {
        let test_data = &test_data::TEST_DATA;
        assert_eq!(test_data.uvarints.len(), 17);

        for uvarint in &test_data.uvarints {
            let (_, value) = uvarint::read_uvarint(&uvarint.e).unwrap();
            assert_eq!(value, uvarint.v);
        }
    }

    #[test]
    fn test_read_chunk() {
        for chunk in &test_data::TEST_DATA.chunks {
            let (remaining_data, parsed_chunk) = read_xor_chunk_data(&chunk.e).unwrap();
            assert!(remaining_data.is_empty());
            let parsed_samples = parsed_chunk.samples();
            for (i, sample) in chunk.s.iter().enumerate() {
                assert_eq!(parsed_samples[i].timestamp, sample.ts);
                assert_eq!(parsed_samples[i].value, sample.v);
            }
        }
    }

    #[test]
    fn test_write_xor_chunk() {
        for test_chunk in &test_data::TEST_DATA.chunks {
            let mut buffer: Vec<u8> = Vec::new();
            // convert the test data chunk to an internal xor chunk
            let samples = test_chunk.s.iter().map(|sample| XORSample {
                timestamp: sample.ts,
                value: sample.v,
            });
            let new_xor_chunk = XORChunk::new(samples.collect());
            new_xor_chunk.write(&mut buffer).unwrap();
            assert_eq!(&buffer, &test_chunk.e);
        }
    }

    #[test]
    fn test_read_chunks_disk_format() {
        let chunk_data = chunk_data::CHUNK_DATA;

        let (remaining_data, chunks_disk_format) = read_chunks(chunk_data, None).unwrap();
        assert_eq!(remaining_data.len(), 0);
        assert_eq!(chunks_disk_format.version(), 1);
        assert_eq!(chunks_disk_format.chunks().len(), 9600);

        let lol = chunks_disk_format
            .chunks()
            .get(100)
            .unwrap()
            .block_chunk_ref();
        println!("lol: {:?}", lol);
    }

    #[test]
    fn test_read_index_disk_format() {
        let index_data = &index_data::INDEX_DATA;

        let (remaining_data, index_disk_format) =
            index::read_index_disk_format(index_data).unwrap();

        assert!(remaining_data.is_empty());

        for serie in index_disk_format.series() {
            for chunk in serie.chunks() {
                println!("chunk file index: {:?}", chunk.file_index());
                println!("chunk file offset: {:?}", chunk.file_offset());
            }
        }
    }

    /*#[test]
    fn test_folder() {
        // Example of folder
        let path = "/prometheus/01J5SNVY6NDETAXEY3Q4YW2HGC";

        let folder = Folder::parse_folder(path).unwrap();
        println!("folder: {:?}", folder);
    }*/

    #[test]
    fn single_chunk_example() {
        let chunk = rusty_chunkenc::Chunk::new_xor(vec![
            rusty_chunkenc::XORSample {
                timestamp: 7200000,
                value: 12000.0,
            },
            rusty_chunkenc::XORSample {
                timestamp: 7201000,
                value: 12001.0,
            },
        ]);

        // Serialise the chunk
        let mut buffer: Vec<u8> = Vec::new();
        chunk.write(&mut buffer).unwrap();

        assert_eq!(
            buffer,
            [
                0x12, 0x01, 0x00, 0x02, 0x80, 0xF4, 0xEE, 0x06, 0x40, 0xC7, 0x70, 0x00, 0x00, 0x00,
                0x00, 0x00, 0xE8, 0x07, 0xF0, 0x0C, 0x1F, 0xCE, 0x4F, 0xA7
            ]
        );

        // Parse a chunk from a buffer
        let (_, parsed_chunk) = rusty_chunkenc::read_chunk(&buffer).unwrap();
        println!("parsed_chunk: {:?}", parsed_chunk);
        assert_eq!(parsed_chunk, chunk);
    }

    #[test]
    fn chunks_example() {
        let chunk_disk_format = rusty_chunkenc::ChunksDiskFormat::new(
            vec![
                rusty_chunkenc::Chunk::new_xor(vec![
                    rusty_chunkenc::XORSample {
                        timestamp: 7200000,
                        value: 12000.0,
                    },
                    rusty_chunkenc::XORSample {
                        timestamp: 7201000,
                        value: 12001.0,
                    },
                ]),
                rusty_chunkenc::Chunk::new_xor(vec![
                    rusty_chunkenc::XORSample {
                        timestamp: 7200000,
                        value: 123.45,
                    },
                    rusty_chunkenc::XORSample {
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
        let (_, parsed_chunk_disk_format) = rusty_chunkenc::read_chunks(&buffer, None).unwrap();
        println!("parsed_chunks: {:?}", parsed_chunk_disk_format);
        assert_eq!(parsed_chunk_disk_format, chunk_disk_format);
    }
}

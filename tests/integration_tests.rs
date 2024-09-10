mod chunk_data;
mod index_data;
mod test_data;

#[cfg(test)]
mod tests {
    use rusty_chunkenc::{chunk, chunks, index, uvarint, varbit, xor::read_xor_chunk_data};

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
        let chunk_data = &test_data::TEST_DATA.chunks[0].e;

        let (remaining_data, chunk) = read_xor_chunk_data(chunk_data).unwrap();
        println!("chunk: {:?}", chunk);
    }

    #[test]
    fn test_read_chunks_disk_format() {
        let chunk_data = chunk_data::CHUNK_DATA;

        let (remaining_data, chunks_disk_format) = chunks::read_chunks(0, chunk_data).unwrap();
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
}

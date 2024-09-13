use nom::{number::complete::be_u32, IResult};

pub fn read_crc32c(input: &[u8]) -> IResult<&[u8], u32> {
    // Golang serialises the CRC32 as big endian unsigned 32 bits number
    // https://cs.opensource.google/go/go/+/refs/tags/go1.23.0:src/hash/crc32/crc32.go;l=223-228
    be_u32(input)
}

#[inline]
pub fn compute_crc32c(input: &[u8]) -> u32 {
    ::crc32c::crc32c(input)
}

pub fn write_crc32c<W: std::io::Write>(input: &[u8], writer: &mut W) -> std::io::Result<()> {
    let crc32c = compute_crc32c(input);
    writer.write_all(&crc32c.to_be_bytes())?;
    Ok(())
}

pub fn assert_crc32c_on_data(
    input: &[u8],
    skip_front: usize,
    data_length: usize,
    expected_crc32c: u32,
) -> IResult<&[u8], ()> {
    // It's also important to note that Prometheus uses the CRC32 Castagnoli variant.
    let chunk_type_and_chunk_data = &input[skip_front..skip_front + data_length];
    let computed_crc32c = compute_crc32c(chunk_type_and_chunk_data);

    if expected_crc32c != computed_crc32c {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Verify,
        )));
    }

    Ok((input, ()))
}

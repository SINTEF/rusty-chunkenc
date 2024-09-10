use std::num::NonZeroUsize;

use nom::{
    bytes::complete::take, multi::length_count, number::complete::be_i32, IResult, InputTake,
};

use crate::{
    crc32c::{assert_crc32c_on_data, read_crc32c},
    uvarint::read_uvarint,
};

fn read_number_of_symbols(input: &[u8]) -> IResult<&[u8], usize> {
    let (remaining_input, len) = be_i32(input)?;
    if len < 0 {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Verify,
        )));
    }
    Ok((remaining_input, len as usize))
}

fn read_symbol(input: &[u8]) -> IResult<&[u8], String> {
    let (remaining_input, str_len) = read_uvarint(input)?;
    let (remaining_input, str) = take(str_len)(remaining_input)?;

    // convert to utf-8
    let str = String::from_utf8(str.to_vec()).map_err(|_| {
        nom::Err::Error(nom::error::Error::new(
            remaining_input,
            nom::error::ErrorKind::Verify,
        ))
    })?;

    Ok((remaining_input, str))
}

fn read_symbols(input: &[u8]) -> IResult<&[u8], Vec<String>> {
    length_count(read_number_of_symbols, read_symbol)(input)
}

pub fn read_symbol_table(input: &[u8]) -> IResult<&[u8], Vec<String>> {
    let (remaining_input, len) = be_i32(input)?;

    // Check if there is enough data to read the chunk, the nom way
    let chunk_size: usize = len as usize;
    if let Some(needed) = chunk_size
        .checked_sub(remaining_input.len())
        .and_then(NonZeroUsize::new)
    {
        return Err(nom::Err::Incomplete(nom::Needed::Size(needed)));
    }

    let (remaining_input, symbol_table_data) = remaining_input.take_split(chunk_size);
    let (remaining_input, expected_crc32c) = read_crc32c(remaining_input)?;
    assert_crc32c_on_data(input, 4, chunk_size, expected_crc32c)?;

    let (tmp_remaining_input, symbol_table) = read_symbols(symbol_table_data)?;
    assert!(tmp_remaining_input.is_empty());

    Ok((remaining_input, symbol_table))
}

use std::num::NonZeroUsize;

use nom::{number::complete::be_u64, sequence::tuple, IResult};

use crate::crc32c::{assert_crc32c_on_data, read_crc32c};

#[derive(Debug)]
pub struct IndexTableOfContent {
    pub symbols: Option<usize>,
    pub series: Option<usize>,
    pub label_indices_start: Option<usize>,
    pub label_offset_table: Option<usize>,
    pub postings_start: Option<usize>,
    pub postings_offset_table: Option<usize>,
}

static TOC_SIZE: usize = 8 * 6 + 4;
static TOC_SIZE_WITHOUT_CRC32C: usize = 8 * 6;

pub fn read_toc(input: &[u8]) -> IResult<&[u8], IndexTableOfContent> {
    let (
        remaining_input,
        (
            symbols,
            series,
            label_indices_start,
            label_offset_table,
            postings_start,
            postings_offset_table,
            expected_crc32c,
        ),
    ) = tuple((be_u64, be_u64, be_u64, be_u64, be_u64, be_u64, read_crc32c))(input)?;

    assert_crc32c_on_data(input, 0, TOC_SIZE_WITHOUT_CRC32C, expected_crc32c)?;

    Ok((
        remaining_input,
        IndexTableOfContent {
            symbols: if symbols == 0 {
                None
            } else {
                Some(symbols as usize)
            },
            series: if series == 0 {
                None
            } else {
                Some(series as usize)
            },
            label_indices_start: if label_indices_start == 0 {
                None
            } else {
                Some(label_indices_start as usize)
            },
            label_offset_table: if label_offset_table == 0 {
                None
            } else {
                Some(label_offset_table as usize)
            },
            postings_start: if postings_start == 0 {
                None
            } else {
                Some(postings_start as usize)
            },
            postings_offset_table: if postings_offset_table == 0 {
                None
            } else {
                Some(postings_offset_table as usize)
            },
        },
    ))
}

pub fn read_toc_at_end(input: &[u8]) -> IResult<&[u8], IndexTableOfContent> {
    if input.len() < TOC_SIZE {
        return Err(nom::Err::Incomplete(nom::Needed::Size(
            NonZeroUsize::new(TOC_SIZE).unwrap(),
        )));
    }
    let toc_index_start = input.len() - TOC_SIZE;
    let toc_input_start = &input[toc_index_start..];
    let (remaining_input, toc) = read_toc(toc_input_start)?;
    Ok((remaining_input, toc))
}

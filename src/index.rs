use std::num::NonZeroUsize;

use nom::{branch::alt, bytes::complete::tag, sequence::tuple, IResult};

use crate::{
    series::{read_series, Serie},
    symbol_table::read_symbol_table,
    toc::read_toc_at_end,
};

pub struct IndexDiskFormat {
    series: Vec<Serie>,
}

impl IndexDiskFormat {
    pub fn new(series: Vec<Serie>) -> Self {
        Self { series }
    }

    pub fn series(&self) -> &Vec<Serie> {
        &self.series
    }
}

static HEADER_LENGTH: usize = 5;

pub fn read_index_disk_format(input: &[u8]) -> IResult<&[u8], IndexDiskFormat> {
    let (remaining_input, (_, index_disk_format)) = tuple((
        // Index on disk start with 0xBA AA D7 00
        tag([0xBA, 0xAA, 0xD7, 0x00]),
        alt((read_version_one, read_version_two)),
    ))(input)?;

    Ok((remaining_input, index_disk_format))
}

// Looks like version 1 and version 2 are the same for what we want to parse.
fn read_simple_sections(input: &[u8]) -> IResult<&[u8], IndexDiskFormat> {
    let (remaining_input, toc) = read_toc_at_end(input)?;

    let symbols = if let Some(symbols) = toc.symbols {
        if let Some((_, symbols_input)) = input.split_at_checked(symbols - HEADER_LENGTH) {
            let (_, tmp_symbols) = read_symbol_table(symbols_input)?;
            tmp_symbols
        } else {
            return Err(nom::Err::Incomplete(nom::Needed::Size(
                NonZeroUsize::new(symbols - HEADER_LENGTH - input.len()).unwrap(),
            )));
        }
    } else {
        Vec::new()
    };

    let series = if let Some(series_start) = toc.series {
        if let Some((_, series_input)) = input.split_at_checked(series_start - HEADER_LENGTH) {
            // Try to find the end of the series data section
            let series_end = toc.label_indices_start.unwrap_or_else(|| {
                toc.label_offset_table.unwrap_or_else(|| {
                    toc.postings_start
                        .unwrap_or_else(|| toc.postings_offset_table.unwrap_or(0))
                })
            });

            let (_, tmp_series) = read_series(series_start, series_end)(series_input)?;
            tmp_series
        } else {
            return Err(nom::Err::Incomplete(nom::Needed::Size(
                NonZeroUsize::new(series_start - HEADER_LENGTH - input.len()).unwrap(),
            )));
        }
    } else {
        Vec::new()
    };

    //println!("toc: {:?}", toc);
    //println!("symbols: {:?}", symbols);
    //println!("series: {:?}", series);

    // Apply the symbol table to the series
    let series_finalised: Vec<Serie> = series
        .into_iter()
        .map(|s| s.finalise(&symbols))
        .collect::<Result<Vec<Serie>, ()>>()
        .map_err(|_| {
            nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Verify))
        })?;

    Ok((remaining_input, IndexDiskFormat::new(series_finalised)))
}

pub fn read_version_one(input: &[u8]) -> IResult<&[u8], IndexDiskFormat> {
    let (remaining_input, (_, index_disk_format)) =
        tuple((tag([1u8]), read_simple_sections))(input)?;

    Ok((remaining_input, index_disk_format))
}

pub fn read_version_two(input: &[u8]) -> IResult<&[u8], IndexDiskFormat> {
    let (remaining_input, (_, index_disk_format)) =
        tuple((tag([2u8]), read_simple_sections))(input)?;

    Ok((remaining_input, index_disk_format))
}

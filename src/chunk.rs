use std::num::NonZeroUsize;

use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{consumed, value},
    sequence::tuple,
    IResult, InputTake, ToUsize,
};

use crate::{
    crc32c::{assert_crc32c_on_data, read_crc32c},
    histogram::{
        read_float_histogram_chunk_data, read_histogram_chunk_data, FloatHistogramChunk,
        HistogramChunk,
    },
    uvarint::read_uvarint,
    xor::{read_xor_chunk_data, XORChunk, XORSample},
};

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum ChunkType {
    #[allow(clippy::upper_case_acronyms)]
    XOR,
    Histogram,
    FloatHistogram,
}

struct ChunkHeader {
    chunk_size: u64,
    chunk_type: ChunkType,
}

pub(crate) trait ChunkWithBlockChunkRef {
    fn block_chunk_ref(&self) -> Option<u64>;
    fn compute_block_chunk_ref(&mut self, file_index: u64, chunks_addr: *const u8);
}

/// A Prometheus chunk.
///
/// It can be a XOR chunk, a histogram chunk, or a float histogram chunk.
///
/// For now, only the XOR chunk type is fully implemented.
#[derive(Debug, PartialEq)]
pub enum Chunk {
    XOR(XORChunk),
    Histogram(HistogramChunk),
    FloatHistogram(FloatHistogramChunk),
}

impl Chunk {
    /// Creates a Chunk of type XOR.
    pub fn new_xor(samples: Vec<XORSample>) -> Self {
        Self::XOR(XORChunk::new(samples))
    }

    /// Returns the XOR chunk if it's a XOR chunk.
    pub fn as_xor(self) -> Option<XORChunk> {
        match self {
            Chunk::XOR(xor_chunk) => Some(xor_chunk),
            _ => None,
        }
    }

    /// Retuns the block chunk reference.
    pub fn block_chunk_ref(&self) -> Option<u64> {
        match self {
            Chunk::XOR(xor_chunk) => xor_chunk.block_chunk_ref(),
            Chunk::Histogram(histogram_chunk) => histogram_chunk.block_chunk_ref(),
            Chunk::FloatHistogram(float_histogram_chunk) => float_histogram_chunk.block_chunk_ref(),
        }
    }

    pub(crate) fn compute_chunk_ref(&mut self, file_index: u64, chunks_addr: *const u8) {
        match self {
            Chunk::XOR(xor_chunk) => {
                xor_chunk.compute_block_chunk_ref(file_index, chunks_addr);
            }
            Chunk::Histogram(histogram_chunk) => {
                histogram_chunk.compute_block_chunk_ref(file_index, chunks_addr);
            }
            Chunk::FloatHistogram(float_histogram_chunk) => {
                float_histogram_chunk.compute_block_chunk_ref(file_index, chunks_addr);
            }
        }
    }
}

fn read_chunk_type(input: &[u8]) -> IResult<&[u8], ChunkType> {
    alt((
        value(ChunkType::XOR, tag([1u8])),
        value(ChunkType::Histogram, tag([2u8])),
        value(ChunkType::FloatHistogram, tag([3u8])),
    ))(input)
}

fn read_chunk_header(input: &[u8]) -> IResult<&[u8], ChunkHeader> {
    let (remaining_input, (chunk_size, chunk_type)) =
        tuple((read_uvarint, read_chunk_type))(input)?;

    Ok((
        remaining_input,
        ChunkHeader {
            chunk_size,
            chunk_type,
        },
    ))
}

fn parse_chunk_data(
    addr: *const u8,
    chunk_type: ChunkType,
    chunk_data: &[u8],
) -> IResult<&[u8], Chunk> {
    match chunk_type {
        ChunkType::XOR => {
            let (remaining_input, mut xor_chunk) = read_xor_chunk_data(chunk_data)?;
            xor_chunk.set_addr(addr);
            Ok((remaining_input, Chunk::XOR(xor_chunk)))
        }
        ChunkType::Histogram => {
            let (remaining_input, histogram_chunk) = read_histogram_chunk_data(chunk_data)?;
            Ok((remaining_input, Chunk::Histogram(histogram_chunk)))
        }
        ChunkType::FloatHistogram => {
            let (remaining_input, float_histogram_chunk) =
                read_float_histogram_chunk_data(chunk_data)?;
            Ok((
                remaining_input,
                Chunk::FloatHistogram(float_histogram_chunk),
            ))
        }
    }
}

/// Reads a chunk from the input data.
///
/// Returns the remaining input data and the chunk.
pub fn read_chunk(input: &[u8]) -> IResult<&[u8], Chunk> {
    let addr = input.as_ptr();

    let (remaining_input, (consumed_header_bytes, chunk_header)) =
        consumed(read_chunk_header)(input)?;

    // Check if there is enough data to read the chunk, the nom way
    let chunk_size: usize = chunk_header.chunk_size.to_usize();
    if let Some(needed) = chunk_size
        .checked_sub(remaining_input.len())
        .and_then(NonZeroUsize::new)
    {
        return Err(nom::Err::Incomplete(nom::Needed::Size(needed)));
    }

    // Extract the data section
    let (remaining_input, chunk_data) = remaining_input.take_split(chunk_size);

    // Before we parse the chunk data, we read and check the CRC32 Castagnoli checksum
    let (remaining_input, chunk_crc32c) = read_crc32c(remaining_input)?;

    // We need to get the size of the header because it has a variable length
    // and we use the end of the header in the CRC32 calculation.
    // The CRC32C is computed on the type and the data, but not the size
    const CHUNK_TYPE_SIZE: usize = 1;
    let header_length = consumed_header_bytes.len();
    assert_crc32c_on_data(
        input,
        header_length - CHUNK_TYPE_SIZE,
        chunk_size + CHUNK_TYPE_SIZE,
        chunk_crc32c,
    )?;

    // Finaly, we can parse the chunk data
    let (remaining_chunk_data_input, chunk) =
        parse_chunk_data(addr, chunk_header.chunk_type, chunk_data)?;

    // https://github.com/prometheus/prometheus/pull/14854
    if !remaining_chunk_data_input.is_empty() {
        // The bug is that a whole byte of 0 is used for padding.
        let (remaining_chunk_data_input, _) = tag([0u8; 1])(remaining_chunk_data_input)?;
        assert!(remaining_chunk_data_input.is_empty());
    }

    // We jungled a bit between the input buffers because we wanted to check the CRC32 checksum
    // before we parsed the chunk data. Sorry about that.

    Ok((remaining_input, chunk))
}

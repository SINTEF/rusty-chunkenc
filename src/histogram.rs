use nom::IResult;

use crate::chunk::ChunkWithBlockChunkRef;

#[derive(Debug)]
pub struct HistogramChunk {}

impl ChunkWithBlockChunkRef for HistogramChunk {
    fn block_chunk_ref(&self) -> Option<u64> {
        None
    }

    fn compute_block_chunk_ref(&mut self, _file_index: u64, _chunks_addr: *const u8) {}
}

pub fn read_histogram_chunk_data(input: &[u8]) -> IResult<&[u8], HistogramChunk> {
    // An exercice left to the reader
    Ok((input, HistogramChunk {}))
}

#[derive(Debug)]
pub struct FloatHistogramChunk {}

impl ChunkWithBlockChunkRef for FloatHistogramChunk {
    fn block_chunk_ref(&self) -> Option<u64> {
        None
    }

    fn compute_block_chunk_ref(&mut self, _file_index: u64, _chunks_addr: *const u8) {}
}

pub fn read_float_histogram_chunk_data(input: &[u8]) -> IResult<&[u8], FloatHistogramChunk> {
    // An exercice left to the reader
    Ok((input, FloatHistogramChunk {}))
}

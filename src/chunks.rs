use nom::{bytes::complete::tag, multi::many1, sequence::tuple, IResult};

use crate::chunk::{read_chunk, Chunk};

/// A Prometheus chunks disk format.
///
/// It contains a version number, always 1 for now, and a list of chunks.
#[derive(Debug)]
pub struct ChunksDiskFormat {
    version: u8,
    chunks: Vec<Chunk>,
    file_index: Option<u64>,
    addr: Option<*const u8>,
}

impl ChunksDiskFormat {
    /// Creates a new chunks disk format with the given chunks and file index.
    ///
    /// The file index is used to compute the chunk references.
    /// Set it to None if you don't use the chunk references.
    pub fn new(chunks: Vec<Chunk>, file_index: Option<u64>) -> Self {
        Self {
            version: 1,
            chunks,
            file_index,
            addr: None,
        }
    }

    /// Returns the version number of the chunks disk format (always 1).
    pub fn version(&self) -> u8 {
        self.version
    }

    /// Returns the chunks.
    pub fn chunks(&self) -> &[Chunk] {
        &self.chunks
    }

    fn set_addr(&mut self, addr: *const u8) {
        self.addr = Some(addr);
    }

    fn set_file_index(&mut self, file_index: u64) {
        self.file_index = Some(file_index);
    }

    fn compute_chunk_refs(&mut self) {
        let chunks_addr = match self.addr {
            Some(addr) => addr,
            None => return,
        };
        let file_index = match self.file_index {
            Some(file_index) => file_index,
            None => return,
        };
        for chunk in self.chunks.iter_mut() {
            chunk.compute_chunk_ref(file_index, chunks_addr);
        }
    }
}

impl PartialEq for ChunksDiskFormat {
    fn eq(&self, other: &Self) -> bool {
        self.version == other.version
            && self.chunks == other.chunks
            && self.file_index == other.file_index
    }
}

fn read_chunks_disk_format(input: &[u8]) -> IResult<&[u8], ChunksDiskFormat> {
    let (remaining_input, (_, mut chunks_disk_format)) = tuple((
        // Chunks on disk start with 0x85BD40DD
        tag([0x85, 0xBD, 0x40, 0xDD]),
        read_version_one,
    ))(input)?;

    chunks_disk_format.set_addr(input.as_ptr());

    Ok((remaining_input, chunks_disk_format))
}

fn read_version_one(input: &[u8]) -> IResult<&[u8], ChunksDiskFormat> {
    let (remaining_input, (_, _, chunks)) = tuple((
        // Read the version byte, that is 1
        tag([1u8]),
        // 3 bytes of 0 for padding
        tag([0u8; 3]),
        // Chunks follow each other
        many1(read_chunk),
    ))(input)?;

    Ok((
        remaining_input,
        ChunksDiskFormat {
            version: 1,
            chunks,
            file_index: None,
            addr: None,
        },
    ))
}

/// Reads the chunks disk format from the input data.
///
/// The file index is used to compute the chunk references.
/// Set it to None, if you don't use the chunk references.
///
/// It returns the remaining input data and the chunks disk format.
pub fn read_chunks(input: &[u8], file_index: Option<u64>) -> IResult<&[u8], ChunksDiskFormat> {
    let (remaining_input, mut chunks_disk_format) = read_chunks_disk_format(input)?;
    if let Some(file_index) = file_index {
        chunks_disk_format.set_file_index(file_index);
    }
    chunks_disk_format.compute_chunk_refs();
    Ok((remaining_input, chunks_disk_format))
}

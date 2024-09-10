use nom::{
    bits, bytes,
    number::complete::{be_f64, be_u16},
    sequence::tuple,
    IResult,
};

use crate::{
    chunk::ChunkWithBlockChunkRef, uvarint::read_uvarint, varbit_xor::read_varbit_xor,
    varint::read_varint,
};
use crate::{varbit_ts::read_varbit_ts, NomBitInput};

#[derive(Debug)]
pub struct XORChunk {
    samples: Vec<XORSample>,
    block_chunk_ref: Option<u64>,
    addr: Option<*const u8>,
}

impl ChunkWithBlockChunkRef for XORChunk {
    fn block_chunk_ref(&self) -> Option<u64> {
        self.block_chunk_ref
    }
    fn compute_block_chunk_ref(&mut self, file_index: u64, chunks_addr: *const u8) {
        match self.addr {
            Some(addr) => {
                self.block_chunk_ref =
                    Some((file_index << 32) | (addr as u64 - chunks_addr as u64));
            }
            None => self.block_chunk_ref = None,
        }
    }
}

impl XORChunk {
    pub fn set_addr(&mut self, addr: *const u8) {
        self.addr = Some(addr);
    }

    pub fn samples(&self) -> &[XORSample] {
        &self.samples
    }
}

#[derive(Debug)]
pub struct XORSample {
    pub timestamp: i64,
    pub value: f64,
}

#[derive(Debug)]
pub struct XORIterator {
    pub timestamp: i64,
    pub value: f64,
    pub leading_bits_count: u8,
    pub trailing_bits_count: u8,
    pub timestamp_delta: u64,
}

pub fn read_first_sample(input: &[u8]) -> IResult<&[u8], XORSample> {
    let (remaining_input, (timestamp, value)) = tuple((read_varint, be_f64))(input)?;
    Ok((remaining_input, XORSample { timestamp, value }))
}

pub fn read_second_sample<'a>(
    first_timestamp: i64,
    first_value: f64,
) -> impl Fn(NomBitInput<'a>) -> IResult<NomBitInput<'a>, XORIterator> {
    move |input: NomBitInput<'a>| {
        let (
            remaining_input,
            (timestamp_delta, (value, new_leading_bits_count, new_trailing_bits_count)),
        ) = tuple((bytes(read_uvarint), read_varbit_xor(first_value, 0, 0)))(input)?;

        let timestamp = first_timestamp
            + i64::try_from(timestamp_delta).map_err(|_| {
                nom::Err::Error(nom::error::Error::new(
                    remaining_input,
                    nom::error::ErrorKind::TooLarge,
                ))
            })?;

        Ok((
            remaining_input,
            XORIterator {
                timestamp,
                value,
                leading_bits_count: new_leading_bits_count,
                trailing_bits_count: new_trailing_bits_count,
                timestamp_delta,
            },
        ))
    }
}

pub fn read_n_sample<'a>(
    previous_iterator: &XORIterator,
) -> impl Fn(NomBitInput<'a>) -> IResult<NomBitInput<'a>, XORIterator> {
    let previous_timestamp = previous_iterator.timestamp;
    let previous_value = previous_iterator.value;
    let previous_leading_bits_count = previous_iterator.leading_bits_count;
    let previous_trailing_bits_count = previous_iterator.trailing_bits_count;
    let previous_timestamp_delta = previous_iterator.timestamp_delta;

    move |input: NomBitInput<'a>| {
        let (
            remaining_input,
            (timestamp_delta_of_delta, (value, new_leading_bits_count, new_trailing_bits_count)),
        ) = tuple((
            read_varbit_ts,
            read_varbit_xor(
                previous_value,
                previous_leading_bits_count,
                previous_trailing_bits_count,
            ),
        ))(input)?;

        let timestamp_delta = ((previous_timestamp_delta as i64) + timestamp_delta_of_delta) as u64;
        let timestamp = previous_timestamp + timestamp_delta as i64;

        Ok((
            remaining_input,
            XORIterator {
                timestamp,
                value,
                leading_bits_count: new_leading_bits_count,
                trailing_bits_count: new_trailing_bits_count,
                timestamp_delta,
            },
        ))
    }
}

fn read_following_samples<'a>(
    first_timestamp: i64,
    first_value: f64,
    num_samples: u16,
) -> impl Fn(NomBitInput<'a>) -> IResult<NomBitInput<'a>, Vec<XORSample>> {
    move |input: NomBitInput<'a>| {
        let mut samples = Vec::with_capacity(num_samples as usize);
        samples.push(XORSample {
            timestamp: first_timestamp,
            value: first_value,
        });

        if num_samples > 1 {
            let (remaining_input_bits, iterator) =
                read_second_sample(first_timestamp, first_value)(input)?;

            samples.push(XORSample {
                timestamp: iterator.timestamp,
                value: iterator.value,
            });

            let mut iterator = iterator;
            let mut remaining_input_bits = remaining_input_bits;
            for _ in 2..num_samples {
                let (tmp_remaining_input_bits, tmp_iterator) =
                    read_n_sample(&iterator)(remaining_input_bits)?;
                iterator = tmp_iterator;
                remaining_input_bits = tmp_remaining_input_bits;

                samples.push(XORSample {
                    timestamp: iterator.timestamp,
                    value: iterator.value,
                });
            }

            return Ok((remaining_input_bits, samples));
        }
        Ok((input, samples))
    }
}

pub fn read_xor_chunk_data(input: &[u8]) -> IResult<&[u8], XORChunk> {
    let (remaining_input, (num_samples, first_sample)) = tuple((be_u16, read_first_sample))(input)?;

    let (remaining_input, all_samples) = bits(read_following_samples(
        first_sample.timestamp,
        first_sample.value,
        num_samples,
    ))(remaining_input)?;

    //println!("all samples: {:?}", all_samples);
    //panic!("stop");

    Ok((
        remaining_input,
        XORChunk {
            samples: all_samples,
            block_chunk_ref: None,
            addr: None,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_chunk() {
        // Long chunk with the bug
        // See https://github.com/prometheus/prometheus/pull/14854
        let input = b"\x00\x01\x80\xF4\xEE\x06\x40\xC7\x70\x00\x00\x00\x00\x00\x00";
        let (_, chunk) = read_xor_chunk_data(input).unwrap();
        assert_eq!(chunk.samples.len(), 1);
        assert_eq!(chunk.samples[0].timestamp, 7200000);
        assert_eq!(chunk.samples[0].value, 12000.0);

        // Correct chunk
        let input = b"\x00\x01\x80\xF4\xEE\x06\x40\xC7\x70\x00\x00\x00\x00\x00";
        let (_, chunk) = read_xor_chunk_data(input).unwrap();
        assert_eq!(chunk.samples.len(), 1);
        assert_eq!(chunk.samples[0].timestamp, 7200000);
        assert_eq!(chunk.samples[0].value, 12000.0);
    }
}

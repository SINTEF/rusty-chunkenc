use std::collections::BTreeMap;

use nom::{
    bytes::complete::take,
    combinator::{consumed, map},
    multi::many_m_n,
    sequence::tuple,
    IResult,
};

use crate::{
    crc32c::{assert_crc32c_on_data, read_crc32c},
    uvarint::read_uvarint,
    varint::read_varint,
};

#[derive(Debug)]
pub struct SerieLabel {
    pub name_ref: u32,
    pub value_ref: u32,
}

#[derive(Debug)]
pub struct SerieChunk {
    pub mint: i64,
    pub maxt: i64,
    pub data_ref: u64,
}

impl SerieChunk {
    pub fn file_index(&self) -> u64 {
        self.data_ref >> 32
    }
    pub fn file_offset(&self) -> u64 {
        self.data_ref & 0xFFFFFFFF
    }
}

#[derive(Debug)]
pub struct SerieTmp {
    pub labels: Vec<SerieLabel>,
    pub chunks: Vec<SerieChunk>,
}

impl SerieTmp {
    pub fn finalise(self, symbols: &[String]) -> Result<Serie, ()> {
        let labels = self
            .labels
            .into_iter()
            .map(|l| {
                let name = symbols.get(l.name_ref as usize);
                let value = symbols.get(l.value_ref as usize);
                if let (Some(name), Some(value)) = (name, value) {
                    Ok((name.clone(), value.clone()))
                } else {
                    Err(())
                }
            })
            .collect::<Result<BTreeMap<String, String>, ()>>()?;
        Ok(Serie {
            labels,
            chunks: self.chunks,
        })
    }
}

#[derive(Debug)]
pub struct Serie {
    pub labels: BTreeMap<String, String>,
    chunks: Vec<SerieChunk>,
}

impl Serie {
    pub fn chunks(&self) -> &[SerieChunk] {
        &self.chunks
    }

    pub fn get_xx_hash(&self) -> u64 {
        // Prometheus uses a zero seed
        let mut hasher = xxhash_rust::xxh64::Xxh64::new(0);

        for (name, value) in &self.labels {
            hasher.update(name.as_bytes());
            hasher.update(b"\xff");
            hasher.update(value.as_bytes());
            hasher.update(b"\xff");
        }

        hasher.digest()
    }
}

fn read_series_labels(input: &[u8]) -> IResult<&[u8], Vec<SerieLabel>> {
    let (remaining_input, len) = read_uvarint(input)?;
    let (remaining_input, labels) = many_m_n(
        len as usize,
        len as usize,
        map(
            tuple((read_uvarint, read_uvarint)),
            |(name_ref, value_ref)| SerieLabel {
                name_ref: name_ref as u32,
                value_ref: value_ref as u32,
            },
        ),
    )(remaining_input)?;
    Ok((remaining_input, labels))
}

fn read_series_chunks(input: &[u8]) -> IResult<&[u8], Vec<SerieChunk>> {
    let (remaining_input, len) = read_uvarint(input)?;
    let (remaining_input, mut chunks) = many_m_n(
        len as usize,
        len as usize,
        map(
            tuple((read_varint, read_uvarint, read_uvarint)),
            |(mint, maxt, data_ref)| SerieChunk {
                mint,
                maxt: (maxt as i64) + mint,
                data_ref,
            },
        ),
    )(remaining_input)?;

    for i in 1..chunks.len() {
        chunks[i].mint += chunks[i - 1].maxt;
        chunks[i].maxt += chunks[i - 1].maxt;
        chunks[i].data_ref += chunks[i - 1].data_ref;
    }

    Ok((remaining_input, chunks))
}

pub fn read_serie(input: &[u8]) -> IResult<&[u8], SerieTmp> {
    let (
        remaining_input,
        ((consumed_serie_len, serie_len), serie_labels, serie_chunks, expected_crc32c),
    ) = tuple((
        consumed(read_uvarint),
        read_series_labels,
        read_series_chunks,
        read_crc32c,
    ))(input)?;

    assert_crc32c_on_data(
        input,
        consumed_serie_len.len(),
        serie_len as usize,
        expected_crc32c,
    )?;

    Ok((
        remaining_input,
        SerieTmp {
            labels: serie_labels,
            chunks: serie_chunks,
        },
    ))
}

pub fn read_series(start: usize, end: usize) -> impl Fn(&[u8]) -> IResult<&[u8], Vec<SerieTmp>> {
    move |input: &[u8]| {
        let mut remaining_input = input;
        let mut series = Vec::new();
        let mut total_consumed = 0;

        // Handle initial padding
        let initial_padding = (16 - (start % 16)) % 16;
        if initial_padding > 0 {
            let (tmp_input, _) = take(initial_padding)(remaining_input)?;
            remaining_input = tmp_input;
            total_consumed += initial_padding;
        }

        loop {
            if total_consumed >= end - start {
                break;
            }
            let (tmp_input, serie) = read_serie(remaining_input)?;
            //println!("serie: {:?}", serie);
            series.push(serie);

            // Calculate and consume padding
            let consumed = remaining_input.len() - tmp_input.len();
            total_consumed += consumed;
            let padding = (16 - (consumed % 16)) % 16;
            if padding > 0 {
                let (padded_input, _) = take(padding)(tmp_input)?;
                remaining_input = padded_input;
                total_consumed += padding;
            } else {
                remaining_input = tmp_input;
            }
        }

        Ok((remaining_input, series))
    }
}

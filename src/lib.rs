//!
//! A Rust implementation of Prometheus' [`chunkenc`](https://pkg.go.dev/github.com/prometheus/prometheus/tsdb/chunkenc) library.
//!
//! ## Features
//!
//! - Parse Prometheus XOR-encoded chunks (that are heavily inspired by [Gorilla](https://www.vldb.org/pvldb/vol8/p1816-teller.pdf)).
//! - Serialise time series to Prometheus XOR-encoded chunks.
//! - Read Prometheus' cold data directly from the disk.
//! - Also comes with utilities to read and write `varint`, `uvarint`, `varbit`, `varbit_ts`, and `varbit_xor` numbers.
//!
//! ## Why?
//!
//! Prometheus uses XOR Chunks in its remote read API, and I wanted to understand how they work in detail. This crate enables [SensApp](https://github.com/sintef/sensapp) to stream data to Prometheus. SensApp is written in Rust, and I wanted a chunkenc Rust implementation.
//!
//! Also, writing a parser and a serialiser did sound fun.
//!
//! ## Acknowledgements
//!
//! This project is ported from Prometheus' [`chunkenc`](https://pkg.go.dev/github.com/prometheus/prometheus/tsdb/chunkenc), that used [`go-tzs`](https://github.com/dgryski/go-tsz), that is based on the [Gorilla](https://www.vldb.org/pvldb/vol8/p1816-teller.pdf) paper. The parsing heavily relies on [`nom`](https://crates.io/crates/nom).
//!
//! The project supports the [Smart Building Hub](https://smartbuildinghub.no/) research infrastructure project, which is funded by the [Norwegian Research Council](https://www.forskningsradet.no/).
//!
//! ## Example
//!
//! ```rust
//! let chunk_disk_format = rusty_chunkenc::ChunksDiskFormat::new(
//! vec![
//!     rusty_chunkenc::Chunk::new_xor(vec![
//!         rusty_chunkenc::XORSample {
//!             timestamp: 7200000,
//!             value: 12000.0,
//!         },
//!         rusty_chunkenc::XORSample {
//!             timestamp: 7201000,
//!             value: 12001.0,
//!         },
//!     ]),
//!     rusty_chunkenc::Chunk::new_xor(vec![
//!         rusty_chunkenc::XORSample {
//!             timestamp: 7200000,
//!             value: 123.45,
//!         },
//!         rusty_chunkenc::XORSample {
//!             timestamp: 7201000,
//!             value: 123.46,
//!         },
//!     ]),
//! ],
//! None,
//! );
//!
//! // Serialise the chunks
//! let mut buffer: Vec<u8> = Vec::new();
//! chunk_disk_format.write(&mut buffer).unwrap();
//!
//! // Parse a chunk from a buffer
//! let (_, parsed_chunk_disk_format) = rusty_chunkenc::read_chunks(&buffer, None).unwrap();
//! println!("parsed_chunks: {:?}", parsed_chunk_disk_format);
//! assert_eq!(parsed_chunk_disk_format, chunk_disk_format);
//! ```
//!
//! Or for a single chunk:
//!
//! ```rust
//! let chunk = rusty_chunkenc::Chunk::new_xor(vec![
//!     rusty_chunkenc::XORSample {
//!         timestamp: 7200000,
//!         value: 12000.0,
//!     },
//!     rusty_chunkenc::XORSample {
//!         timestamp: 7201000,
//!         value: 12001.0,
//!     },
//! ]);
//!
//! // Serialise the chunk
//! let mut buffer: Vec<u8> = Vec::new();
//! chunk.write(&mut buffer).unwrap();
//!
//! assert_eq!(
//!     buffer,
//!     [
//!         0x12, 0x01, 0x00, 0x02, 0x80, 0xF4, 0xEE, 0x06, 0x40, 0xC7, 0x70, 0x00, 0x00, 0x00,
//!         0x00, 0x00, 0xE8, 0x07, 0xF0, 0x0C, 0x1F, 0xCE, 0x4F, 0xA7
//!     ]
//! );
//!
//! // Parse a chunk from a buffer
//! let (_, parsed_chunk) = rusty_chunkenc::read_chunk(&buffer).unwrap();
//! println!("parsed_chunk: {:?}", parsed_chunk);
//! ```

/// Single Prometheus chunk.
pub mod chunk;
/// Prometheus chunks disk format.
pub mod chunks;
mod crc32c;
mod encoder;
mod errors;
/// WIP: Parse all prometheus data from the prometheus folder.
pub mod folder;
/// Histogram and Float Histogram chunks, not implemented yet.
pub mod histogram;
/// WIP: Prometheus index files
pub mod index;
mod series;
mod symbol_table;
mod toc;
/// Golang's uvarint.
pub mod uvarint;
/// Prometheus's varbit encoding.
pub mod varbit;
/// Prometheus's varbit timestamp encoding.
pub mod varbit_ts;
/// Prometheus's varbit xor encoding.
pub mod varbit_xor;
/// Golang's varint.
pub mod varint;
/// XOR chunk.
pub mod xor;

type NomBitInput<'a> = (&'a [u8], usize);

// Re-exports
pub use chunk::read_chunk;
pub use chunk::Chunk;

pub use chunks::read_chunks;
pub use chunks::ChunksDiskFormat;

pub use xor::XORSample;

pub mod chunk;
pub mod chunks;
pub mod crc32c;
pub mod histogram;
pub mod index;
pub mod series;
pub mod symbol_table;
pub mod toc;
pub mod uvarint;
pub mod varbit;
pub mod varbit_ts;
pub mod varbit_xor;
pub mod varint;
pub mod xor;

type NomBitInput<'a> = (&'a [u8], usize);

/// Write a u64 as a Golang uvarint.
pub fn write_uvarint<W: std::io::Write>(value: u64, writer: &mut W) -> std::io::Result<()> {
    let mut x: u64 = value;
    while x >= 0x80 {
        writer.write_all(&[(x as u8) | 0x80])?;
        x >>= 7;
    }
    writer.write_all(&[x as u8])?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::uvarint::read_uvarint;
    use rand::{Rng, SeedableRng};

    use super::*;

    #[test]
    fn test_write_uvarint() {
        let mut buffer: Vec<u8> = Vec::new();
        let mut writer = std::io::Cursor::new(&mut buffer);

        let mut numbers = vec![
            0,
            1,
            7,
            8,
            63,
            64,
            511,
            512,
            4095,
            4096,
            262143,
            262144,
            33554431,
            33554432,
            72057594037927935,
            72057594037927936,
            u64::MAX,
        ];

        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        // Add some random numbers
        for _ in 0..100 {
            let number: u64 = rng.gen();
            numbers.push(number);
        }

        // Write
        for number in &numbers {
            write_uvarint(*number, &mut writer).unwrap();
        }

        // Read
        let mut cursor = &buffer[..];
        for number in numbers {
            let (new_cursor, read_number) = read_uvarint(cursor).unwrap();
            assert_eq!(read_number, number);
            cursor = new_cursor;
        }
    }
}

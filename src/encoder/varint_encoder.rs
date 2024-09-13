use super::uvarint_encoder::write_uvarint;

/// Write a i64 as a Golang varint.
pub fn write_varint<W: std::io::Write>(value: i64, writer: &mut W) -> std::io::Result<()> {
    let x = value;
    let mut ux = (x as u64) << 1;
    if x < 0 {
        ux = !ux;
    }
    write_uvarint(ux, writer)
}

#[cfg(test)]
mod tests {
    use crate::varint::read_varint;
    use rand::{Rng, SeedableRng};

    use super::*;

    #[test]
    fn test_write_varint() {
        let mut buffer: Vec<u8> = Vec::new();
        let mut writer = std::io::Cursor::new(&mut buffer);

        let mut numbers = vec![
            i64::MIN,
            -36028797018963968,
            -36028797018963967,
            -16777216,
            -16777215,
            -131072,
            -131071,
            -2048,
            -2047,
            -256,
            -255,
            -32,
            -31,
            -4,
            -3,
            -1,
            0,
            1,
            4,
            5,
            32,
            33,
            256,
            257,
            2048,
            2049,
            131072,
            131073,
            16777216,
            16777217,
            36028797018963968,
            36028797018963969,
            i64::MAX,
        ];

        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        // Add some random numbers
        for _ in 0..100 {
            let number: i64 = rng.gen();
            numbers.push(number);
        }

        // Write
        for number in &numbers {
            write_varint(*number, &mut writer).unwrap();
        }

        // Read
        let mut cursor = &buffer[..];
        for number in numbers {
            let (new_cursor, read_number) = read_varint(cursor).unwrap();
            assert_eq!(read_number, number);
            cursor = new_cursor;
        }
    }
}

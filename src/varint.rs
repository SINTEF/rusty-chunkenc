use nom::IResult;

use crate::uvarint::read_uvarint;

pub use crate::encoder::varint_encoder::write_varint;

/// Parses a Golang varint.
pub fn read_varint(input: &[u8]) -> IResult<&[u8], i64> {
    let (remaining_input, uvarint_value) = read_uvarint(input)?;

    let value = (uvarint_value >> 1) as i64;
    if uvarint_value & 1 != 0 {
        Ok((remaining_input, !value))
    } else {
        Ok((remaining_input, value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_boring_values() {
        let input = b"\x00";
        let (_, value) = read_varint(input).unwrap();
        assert_eq!(value, 0);

        let input = b"\x01";
        let (_, value) = read_varint(input).unwrap();
        assert_eq!(value, -1);

        let input = b"\x02";
        let (_, value) = read_varint(input).unwrap();
        assert_eq!(value, 1);

        let input = b"\x7f";
        let (_, value) = read_varint(input).unwrap();
        assert_eq!(value, -64);

        let input = b"\x80\x01";
        let (_, value) = read_varint(input).unwrap();
        assert_eq!(value, 64);

        let input = b"\xff\x01";
        let (_, value) = read_varint(input).unwrap();
        assert_eq!(value, -128);

        let input = b"\xac\x02";
        let (_, value) = read_varint(input).unwrap();
        assert_eq!(value, 150);

        let input = b"\x80\x80\x01";
        let (_, value) = read_varint(input).unwrap();
        assert_eq!(value, 8192);

        let input = b"\x80\x80\x02";
        let (_, value) = read_varint(input).unwrap();
        assert_eq!(value, 16384);

        let input = b"\x81\x80\x02";
        let (_, value) = read_varint(input).unwrap();
        assert_eq!(value, -16385);
    }

    #[test]
    fn test_with_weird_data() {
        let input = "hello world".as_bytes();
        let (_, value) = read_varint(input).unwrap();
        assert_eq!(value, 52);
    }

    #[test]
    fn test_with_overflows() {
        // Classic overflow
        let input = b"\x80\x80\x80\x80\x80\x80\x80\x80\x80\x80\x01";
        let result = read_varint(input);
        assert!(result.is_err());

        // More subtle overflow
        let input = b"\x80\x80\x80\x80\x80\x80\x80\x80\x80\x02";
        let result = read_varint(input);
        assert!(result.is_err());
    }
}

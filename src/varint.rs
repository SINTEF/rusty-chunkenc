use nom::IResult;

use crate::uvarint::read_uvarint;

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

    /*#[test]
    fn test_with_golangs_test_values() {
        const TEST_VALUES: [i64; 18] = [
            -1 << 63,
            //(-1 << 63) + 1,
            i64::MIN,
            -1,
            0,
            1,
            2,
            10,
            20,
            63,
            64,
            65,
            127,
            128,
            129,
            255,
            256,
            257,
            //(1 << 63) - 1,
            i64::MAX,
        ];

        for value in TEST_VALUES {
    }*/

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

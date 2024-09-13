use nom::{bytes::complete::take, IResult};

pub use crate::encoder::uvarint_encoder::write_uvarint;

/// Parses a Golang uvarint.
pub fn read_uvarint(input: &[u8]) -> IResult<&[u8], u64> {
    let mut input_pointer = input;
    let mut x: u64 = 0;
    let mut s: usize = 0;

    for i in 0..10 {
        let (new_input_pointer, byte_buffer) = take(1usize)(input_pointer)?;
        input_pointer = new_input_pointer;
        let byte = byte_buffer[0];

        if byte < 0x80 {
            if i == 9 && byte > 1 {
                return Err(nom::Err::Error(nom::error::Error::new(
                    input,
                    nom::error::ErrorKind::TooLarge,
                )));
            }
            return Ok((input_pointer, x | (byte as u64) << s));
        }

        x |= ((byte & 0x7f) as u64) << s;
        s += 7;
    }

    Err(nom::Err::Error(nom::error::Error::new(
        input,
        nom::error::ErrorKind::TooLarge,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_boring_values() {
        let input = b"\x00";
        let (_, value) = read_uvarint(input).unwrap();
        assert_eq!(value, 0);

        let input = b"\x01";
        let (_, value) = read_uvarint(input).unwrap();
        assert_eq!(value, 1);

        let input = b"\x7f";
        let (_, value) = read_uvarint(input).unwrap();
        assert_eq!(value, 127);

        let input = b"\x80\x01";
        let (_, value) = read_uvarint(input).unwrap();
        assert_eq!(value, 128);

        let input = b"\xff\x01";
        let (_, value) = read_uvarint(input).unwrap();
        assert_eq!(value, 255);

        let input = b"\xac\x02";
        let (_, value) = read_uvarint(input).unwrap();
        assert_eq!(value, 300);

        let input = b"\x80\x80\x01";
        let (_, value) = read_uvarint(input).unwrap();
        assert_eq!(value, 16384);
    }

    #[test]
    fn test_with_weird_data() {
        let input = "hello world".as_bytes();
        let (_, value) = read_uvarint(input).unwrap();
        assert_eq!(value, 104);
    }

    #[test]
    fn test_with_overflows() {
        // Classic overflow
        let input = b"\x80\x80\x80\x80\x80\x80\x80\x80\x80\x80\x01";
        let result = read_uvarint(input);
        assert!(result.is_err());

        // More subtle overflow
        let input = b"\x80\x80\x80\x80\x80\x80\x80\x80\x80\x02";
        let result = read_uvarint(input);
        assert!(result.is_err());
    }
}

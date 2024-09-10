use nom::{
    bits::complete::{bool, take},
    IResult,
};

use crate::NomBitInput;

fn read_leading_bits_count(input: NomBitInput) -> IResult<NomBitInput, u8> {
    // The leading bits count is 5 bits long.
    take(5usize)(input)
}

fn read_middle_bits_count(input: NomBitInput) -> IResult<NomBitInput, u8> {
    // The middle bits count is 6 bits long.
    let (remaining_input, middle_bits_count): (NomBitInput, u8) = take(6usize)(input)?;

    // As prometheus uses 64 bits floats, the number of middle bits can be up to 64.
    // However, the max value on 6 bits is 63.
    // There, prometheus has a small trick: it overflows and 0 actually means 64.
    // It works because numbers with zero bits are not serialized through this.
    // Every saved bit counts!
    if middle_bits_count == 0 {
        return Ok((remaining_input, 64));
    }

    Ok((remaining_input, middle_bits_count))
}

pub fn read_varbit_xor<'a>(
    previous_value: f64,
    previous_leading_bits_count: u8,
    previous_trailing_bits_count: u8,
) -> impl Fn(NomBitInput<'a>) -> IResult<NomBitInput<'a>, (f64, u8, u8)> {
    move |input: NomBitInput<'a>| {
        // Read the bit saying whether we use the previous value or not
        let (remaining_input, different_value_bit) = bool(input)?;
        if !different_value_bit {
            return Ok((
                remaining_input,
                (
                    previous_value,
                    previous_leading_bits_count,
                    previous_trailing_bits_count,
                ),
            ));
        }

        let leading_bits_count: u8;
        let middle_bits_count: u8;
        let trailing_bits_count: u8;

        // Read the bit saying whether we reuse the previous leading and trailing bits count or not
        let (remaining_input, different_leading_and_trailing_bits_count) = bool(remaining_input)?;
        let mut remaining_input = remaining_input;
        if different_leading_and_trailing_bits_count {
            let (tmp_remaining_input, tmp_leading_bits_count) =
                read_leading_bits_count(remaining_input)?;
            let (tmp_remaining_input, tmp_middle_bits_count) =
                read_middle_bits_count(tmp_remaining_input)?;
            remaining_input = tmp_remaining_input;
            leading_bits_count = tmp_leading_bits_count;
            middle_bits_count = tmp_middle_bits_count;
            trailing_bits_count = 64 - leading_bits_count - middle_bits_count;
        } else {
            leading_bits_count = previous_leading_bits_count;
            trailing_bits_count = previous_trailing_bits_count;
            middle_bits_count = 64 - leading_bits_count - trailing_bits_count;
        }

        // Read the right number of bits
        let (remaining_input, value_bits): (NomBitInput, u64) =
            take(middle_bits_count)(remaining_input)?;

        // Compute the new value
        let new_value =
            f64::from_bits(previous_value.to_bits() ^ (value_bits << trailing_bits_count));

        Ok((
            remaining_input,
            (new_value, leading_bits_count, trailing_bits_count),
        ))
    }
}

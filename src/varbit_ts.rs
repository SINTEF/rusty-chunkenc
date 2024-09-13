use nom::{
    bits::complete::{bool, take},
    IResult,
};

use crate::NomBitInput;

pub use crate::encoder::varbit_ts_encoder::write_varbit_ts;

/// Reads a varbit-encoded integer from the input.
///
/// Prometheus' varbitint starts with a bucket category of variable length.
/// It consists of 1 bits and a final 0, up to 8 bits.
/// When it's 8 bits long, the final 0 is skipped.
///
/// It consists of 9 categories.
fn read_varbit_ts_bucket(input: NomBitInput) -> IResult<NomBitInput, u8> {
    let mut remaining_input = input;

    for i in 0..4 {
        let (new_remaining_input, bit) = bool(remaining_input)?;
        remaining_input = new_remaining_input;
        // If we read a 0, it's a sign that we reached the end of the bucket category.
        if !bit {
            return Ok((remaining_input, i));
        }
    }

    // If we read 4 bits already, there is no final 0.
    Ok((remaining_input, 4))
}

#[inline]
fn varbit_ts_bucket_to_num_bits(bucket: u8) -> u8 {
    match bucket {
        0 => 0,
        1 => 14,
        2 => 17,
        3 => 20,
        4 => 64,
        _ => unreachable!("Invalid bucket value"),
    }
}

/// Reads a Prometheus varbit timestamp encoded number from the input.
pub fn read_varbit_ts(input: NomBitInput) -> IResult<NomBitInput, i64> {
    let (remaining_input, bucket) = read_varbit_ts_bucket(input)?;
    let num_bits = varbit_ts_bucket_to_num_bits(bucket);

    // Shortcut for the 0 use case as nothing more has to be read.
    if bucket == 0 {
        return Ok((remaining_input, 0));
    }

    let (remaining_input, mut value): (_, i64) = take(num_bits)(remaining_input)?;
    if num_bits != 64 && value > (1 << (num_bits - 1)) {
        value -= 1 << num_bits;
    }

    Ok((remaining_input, value))
}

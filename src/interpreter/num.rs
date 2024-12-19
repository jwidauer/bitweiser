use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ParseIntError {
    #[error("The input string was empty.")]
    Empty,
    #[error("The input contained an invalid digit at index {0}.")]
    InvalidDigit(usize),
    #[error("The input was too large to fit in the target integer type.")]
    Overflow,
}

const fn can_not_overflow<const RADIX: u32>(digits: &[u8]) -> bool {
    RADIX <= 16 && digits.len() <= std::mem::size_of::<u64>() * 2
}

// This function is used in the lexer to parse numbers in different bases.
// It's a simplified version of the `from_str_radix` function from the standard library.
pub(super) fn from_slice_radix<const RADIX: u32>(mut digits: &[u8]) -> Result<u64, ParseIntError> {
    use ParseIntError as PIE;

    assert!(2 <= RADIX && RADIX <= 36);

    if digits.is_empty() {
        return Err(PIE::Empty);
    }

    let mut result = 0;
    if can_not_overflow::<RADIX>(digits) {
        let mut loc = 0;
        while let [c, rest @ ..] = digits {
            let x = (*c as char).to_digit(RADIX).ok_or(PIE::InvalidDigit(loc))?;
            result = result * (RADIX as u64) + x as u64;
            digits = rest;
            loc += 1;
        }
    } else {
        let mut loc = 0;
        while let [c, rest @ ..] = digits {
            let x = (*c as char).to_digit(RADIX).ok_or(PIE::InvalidDigit(loc))?;
            result = result
                .checked_mul(RADIX as u64)
                .and_then(|v| v.checked_add(x as u64))
                .ok_or(PIE::Overflow)?;
            digits = rest;
            loc += 1;
        }
    }
    Ok(result)
}

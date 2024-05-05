use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};
use std::num::ParseFloatError;
use std::str::FromStr;

#[derive(Debug, Default, Clone, Copy)]
pub struct NonNegativeF32(f32);

impl From<NonNegativeF32> for f32 {
    fn from(value: NonNegativeF32) -> Self {
        value.0
    }
}

impl From<f32> for NonNegativeF32 {
    fn from(value: f32) -> Self {
        assert!(value >= 0.);
        NonNegativeF32(value)
    }
}

impl Display for NonNegativeF32 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl FromStr for NonNegativeF32 {
    type Err = ParseNonNegativeFloatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match f32::from_str(s) {
            Ok(n) if n >= 0. => Ok(NonNegativeF32(n)),
            Ok(_) => Err(ParseNonNegativeFloatError::Negative),
            Err(e) => Err(ParseNonNegativeFloatError::ParseFloatError(e)),
        }
    }
}

#[derive(Debug)]
pub enum ParseNonNegativeFloatError {
    ParseFloatError(ParseFloatError),
    Negative,
}

impl Error for ParseNonNegativeFloatError {}

impl Display for ParseNonNegativeFloatError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ParseNonNegativeFloatError::ParseFloatError(e) => Display::fmt(e, f),
            ParseNonNegativeFloatError::Negative => f.write_str("negative value"),
        }
    }
}

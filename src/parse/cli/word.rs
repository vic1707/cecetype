use super::ErrorKind;
use ::core::{
    char::ParseCharError,
    fmt,
    num::{ParseFloatError, ParseIntError},
};

/// Word parsing error.
#[derive(Debug, ::thiserror::Error)]
pub enum ParseError {
    #[error("invalid char: {0}")]
    Char(#[from] ParseCharError),
    #[error("invalid bool")]
    Bool,
    #[error("invalid integer: {0}")]
    Int(#[from] ParseIntError),
    #[error("invalid float: {0}")]
    Float(#[from] ParseFloatError),
}

/// A word from CLI input: bare or quoted.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum Word<'w> {
    Bare(&'w str),
    Quoted(&'w str),
}

impl fmt::Display for Word<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bare(str) | Self::Quoted(str) => str.fmt(f),
        }
    }
}

impl<'w> Word<'w> {
    pub const fn bare(self) -> Result<&'w str, ErrorKind<'w>> {
        match self {
            Self::Bare(ident) => Ok(ident),
            Self::Quoted(_) => Err(ErrorKind::UnexpectedQuotedWord),
        }
    }

    pub const fn quoted(self) -> Result<&'w str, ErrorKind<'w>> {
        match self {
            Self::Quoted(ident) => Ok(ident),
            Self::Bare(_) => Err(ErrorKind::ExpectedQuotedWord),
        }
    }

    pub fn parse_bool(self) -> Result<bool, ErrorKind<'w>> {
        let ident = self.bare()?;
        if ident.eq_ignore_ascii_case("true") || ident.eq_ignore_ascii_case("yes") {
            Ok(true)
        } else if ident.eq_ignore_ascii_case("false") || ident.eq_ignore_ascii_case("no") {
            Ok(false)
        } else {
            Err(ParseError::Bool.into())
        }
    }

    pub fn parse_char(self) -> Result<char, ErrorKind<'w>> {
        Ok(self.quoted()?.parse().map_err(ParseError::Char)?)
    }
    pub const fn parse_string(self) -> Result<&'w str, ErrorKind<'w>> {
        self.quoted()
    }

    pub fn parse_u8(self) -> Result<u8, ErrorKind<'w>> {
        Ok(self.bare()?.parse().map_err(Into::<ParseError>::into)?)
    }
    pub fn parse_u16(self) -> Result<u16, ErrorKind<'w>> {
        Ok(self.bare()?.parse().map_err(Into::<ParseError>::into)?)
    }
    pub fn parse_u32(self) -> Result<u32, ErrorKind<'w>> {
        Ok(self.bare()?.parse().map_err(Into::<ParseError>::into)?)
    }
    pub fn parse_u64(self) -> Result<u64, ErrorKind<'w>> {
        Ok(self.bare()?.parse().map_err(Into::<ParseError>::into)?)
    }
    pub fn parse_u128(self) -> Result<u128, ErrorKind<'w>> {
        Ok(self.bare()?.parse().map_err(Into::<ParseError>::into)?)
    }

    pub fn parse_i8(self) -> Result<i8, ErrorKind<'w>> {
        Ok(self.bare()?.parse().map_err(Into::<ParseError>::into)?)
    }
    pub fn parse_i16(self) -> Result<i16, ErrorKind<'w>> {
        Ok(self.bare()?.parse().map_err(Into::<ParseError>::into)?)
    }
    pub fn parse_i32(self) -> Result<i32, ErrorKind<'w>> {
        Ok(self.bare()?.parse().map_err(Into::<ParseError>::into)?)
    }
    pub fn parse_i64(self) -> Result<i64, ErrorKind<'w>> {
        Ok(self.bare()?.parse().map_err(Into::<ParseError>::into)?)
    }
    pub fn parse_i128(self) -> Result<i128, ErrorKind<'w>> {
        Ok(self.bare()?.parse().map_err(Into::<ParseError>::into)?)
    }

    pub fn parse_f32(self) -> Result<f32, ErrorKind<'w>> {
        Ok(self.bare()?.parse().map_err(Into::<ParseError>::into)?)
    }
    pub fn parse_f64(self) -> Result<f64, ErrorKind<'w>> {
        Ok(self.bare()?.parse().map_err(Into::<ParseError>::into)?)
    }
}

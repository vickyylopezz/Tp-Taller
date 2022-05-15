/// Represents the possible errors that can occur while parsing some
/// bencoded string.
#[derive(Debug, PartialEq, Eq)]
pub enum ParserError {
    Empty,
    InvalidEncoding,
    InvalidInteger(String),
    InvalidByteStringLength,
}

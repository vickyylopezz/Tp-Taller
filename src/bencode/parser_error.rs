/// Represents the possible errors that can occur while parsing some
/// bencoded string.
#[derive(Debug, PartialEq, Eq)]
pub enum ParserError {
    Empty,
    InvalidEncoding(usize, &'static str), //TODO: Remove str
    InvalidInteger(String),
    InvalidByteStringLength,
}

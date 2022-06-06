/// Represents the possible errors that can occur while encoding.
#[derive(Debug, PartialEq)]
pub enum EncoderError {
    InvalidHexadecimal,
    InvalidUTF8,
}

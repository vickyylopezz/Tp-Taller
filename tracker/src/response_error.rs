use std::{error::Error, fmt::Display};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResponseError {
    ValueNotExpected,
    EventNotExpected,
}

impl Error for ResponseError {}

impl Display for ResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            ResponseError::ValueNotExpected => "Value not expected",
            ResponseError::EventNotExpected => "Event not expected",
        };
        writeln!(f, "{}", msg)
    }
}

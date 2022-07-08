use std::fmt;

/// Represents the possible errors that can occur while receiving
/// a response from the tracker.
#[derive(Debug, PartialEq, Eq)]
pub enum TrackerResponseError {
    InvalidResponse,
    ReadStream,
    Parse,
}

impl fmt::Display for TrackerResponseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TrackerResponseError::InvalidResponse => {
                write!(f, "The response from the tracker is invalid")
            }
            TrackerResponseError::ReadStream => {
                write!(f, "An error ocurred while reading from the stream")
            }
            TrackerResponseError::Parse => write!(f, "An error ocurred while parsing the response"),
        }
    }
}

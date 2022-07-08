use std::fmt;

/// Represents the possible errors that can occur while doing
/// a request to the tracker.
#[derive(Debug, PartialEq, Eq)]
pub enum TrackerRequestError {
    EncoderError,
    WriteError,
    NoPortError,
    Host,
    InvalidTcpStream,
    InvalidAdress,
    InvalidQuerystring,
    WriteStream,
}

impl fmt::Display for TrackerRequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TrackerRequestError::EncoderError => {
                write!(f, "An error occurred encoding the request")
            }
            TrackerRequestError::WriteError => write!(f, "An error ocurred generating the request"),
            TrackerRequestError::NoPortError => write!(f, "Couldn't establish connection to port"), // TODO: Handle
            TrackerRequestError::Host => write!(f, "Couldn't parse host"),
            TrackerRequestError::InvalidTcpStream => {
                write!(f, "An error ocurred trying to establish the connection")
            }
            TrackerRequestError::InvalidAdress => write!(f, "Couldn't obtain local addres"),
            TrackerRequestError::InvalidQuerystring => {
                write!(f, "An error ocurred generating the query")
            }
            TrackerRequestError::WriteStream => write!(f, "Couldn't write to stream'"),
        }
    }
}

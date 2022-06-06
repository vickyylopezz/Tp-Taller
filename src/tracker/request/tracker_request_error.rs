/// Represents the possible errors that can occur while doing
/// a request to the tracker.
#[derive(Debug, PartialEq, Eq)]
pub enum TrackerRequestError {
    EncoderError,
    WriteError,
    NoPortError,
}

/// Represents the possible errors that can occur while receiving
/// a response from the tracker.
#[derive(Debug, PartialEq, Eq)]

pub enum TrackerResponseError {
    InvalidResponse,
}

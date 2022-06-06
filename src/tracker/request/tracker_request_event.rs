/// Represents the possible events that can occur while doing
/// a request to the tracker.
#[derive(Debug, PartialEq, Eq)]
pub enum TrackerRequestEvent {
    Started,
    Completed,
    Stopped,
}

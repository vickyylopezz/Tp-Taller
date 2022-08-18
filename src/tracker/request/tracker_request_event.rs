
use serde::{Serialize, Deserialize};

/// Represents the possible events that can occur while doing
/// a request to the tracker.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum TrackerRequestEvent {
    Started,
    Completed,
    Stopped,
}

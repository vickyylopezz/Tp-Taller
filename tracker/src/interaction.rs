use bittorrent::tracker::request::tracker_request_event::TrackerRequestEvent;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Interaction {
    pub date: String,
    pub event: TrackerRequestEvent,
}

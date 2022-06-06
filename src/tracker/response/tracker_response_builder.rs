use crate::peer::Peer;
use crate::tracker::response::tracker_response::TrackerResponse;
use crate::tracker::response::tracker_response::TrackerResponseMode;

use super::tracker_response::ResponseData;

pub struct TrackerResponseBuilder {
    interval: i64,
    complete: i64,
    incomplete: i64,
    peers: Vec<Peer>,
    min_interval: Option<i64>,
}

impl Default for TrackerResponseBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl TrackerResponseBuilder {
    pub fn new() -> Self {
        Self {
            interval: 0,
            complete: 0,
            incomplete: 0,
            peers: Vec::new(),
            min_interval: None,
        }
    }

    pub fn interval(&'_ mut self, i: i64) -> &'_ mut Self {
        self.interval = i;
        self
    }

    pub fn complete(&'_ mut self, c: i64) -> &'_ mut Self {
        self.complete = c;
        self
    }

    pub fn incomplete(&'_ mut self, inc: i64) -> &'_ mut Self {
        self.incomplete = inc;
        self
    }

    pub fn peers(&'_ mut self, p: Vec<Peer>) -> &'_ mut Self {
        self.peers = p;
        self
    }

    pub fn min_interval(&'_ mut self, mi: i64) -> &'_ mut Self {
        self.min_interval = Some(mi);
        self
    }

    pub fn response_data(self) -> ResponseData {
        ResponseData {
            interval: self.interval,
            complete: self.complete,
            incomplete: self.incomplete,
            peers: self.peers,
            min_interval: self.min_interval,
        }
    }

    pub fn build(self) -> TrackerResponse {
        TrackerResponse(TrackerResponseMode::Response(self.response_data()))
    }
}

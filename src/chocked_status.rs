use crate::peer;

/// Represents the possible status of a peer.
#[derive(Debug, PartialEq, Eq)]
pub enum ChokedStatus {
    Choked,
    Unchoked,
}

pub enum PeerState {
    ChokingAndInterested(peer::Peer),
    ChokingAndNotInterested(peer::Peer),
    NotChokingAndNotInterested(peer::Peer),
    NotChokingAndInterested(peer::Peer),
}

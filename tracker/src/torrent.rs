use std::{collections::HashMap, net::Ipv4Addr};

use bittorrent::{
    bencode::bencoded_value::BencodedValue,
    tracker::request::tracker_request_event::TrackerRequestEvent,
};
use chrono::Local;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

use crate::{interaction::Interaction, response_error::ResponseError};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PeerState {
    Active,
    Inactive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Torrent {
    pub info_hash: String,
    pub peers: Vec<PeerTracker>,
    pub upload_date: String,
}

impl Torrent {
    pub fn peers(&mut self) -> &mut Vec<PeerTracker> {
        &mut self.peers
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PeerTracker {
    pub peer_id: String,
    pub ip: String,
    pub port: u16,
    pub uploaded: u32,
    pub downloaded: u32,
    pub left: u32,
    pub interactions: Vec<Interaction>,
    pub state: PeerState,
}

impl PeerTracker {
    fn new() -> Self {
        PeerTracker {
            peer_id: String::new(),
            ip: String::new(),
            port: 0,
            uploaded: 0,
            downloaded: 0,
            left: 0,
            interactions: Vec::new(),
            state: PeerState::Active,
        }
    }
}

/// Sets the info of a peer that will be stored.
pub fn set_peer(
    peer_id: &str,
    ip: &str,
    port: &str,
    parameters: HashMap<&str, &str>,
    peers: &mut Vec<PeerTracker>,
) {
    match peers
        .iter()
        .position(|p| p.peer_id == peer_id && p.ip == ip && p.port.to_string() == port)
    {
        Some(index) => {
            for (key, value) in parameters {
                set_peer_parameter(key, value, &mut peers[index]).unwrap();
            }
        }
        None => {
            let mut peer = PeerTracker::new();

            for (key, value) in parameters {
                set_peer_parameter(key, value, &mut peer).unwrap();
            }
            peers.push(peer);
        }
    }

}

/// Sets the values of the attributes of a peer.
fn set_peer_parameter(key: &str, value: &str, peer: &mut PeerTracker) -> Result<(), ResponseError> {
    match key {
        "info_hash" => {Ok(())},
        "peer_id" => {
            peer.peer_id = value.parse().unwrap();
            Ok(())
        }
        "ip" => {
            peer.ip = value.parse().unwrap();
            Ok(())
        }
        "port" => {
            peer.port = value.parse().unwrap();
            Ok(())
        }
        "downloaded" => {
            peer.downloaded = value.parse().unwrap();
            Ok(())
        }
        "uploaded" => {
            peer.uploaded = value.parse().unwrap();
            Ok(())
        }
        "left" => {
            peer.left = value.parse().unwrap();
            Ok(())
        }
        "event" => match value {
            "started" => {
                peer.state = PeerState::Active;
                peer.interactions.push(Interaction {
                    date: Local::now().to_string(),
                    event: TrackerRequestEvent::Started,
                });
                Ok(())
            }
            "stopped" => {
                peer.state = PeerState::Inactive;
                peer.interactions.push(Interaction {
                    date: Local::now().to_string(),
                    event: TrackerRequestEvent::Stopped,
                });
                Ok(())
            }
            "completed" => {
                peer.state = PeerState::Active;
                peer.interactions.push(Interaction {
                    date: Local::now().to_string(),
                    event: TrackerRequestEvent::Completed,
                });
                Ok(())
            }

            _ => Err(ResponseError::EventNotExpected),
        },
        _ => Err(ResponseError::ValueNotExpected),
    }
}

/// Encodes a list of peers in a dictonary mode.
pub fn dictionary_mode(torrent_peers: Vec<PeerTracker>) -> Vec<u8> {
    let mut peers = Vec::new();

    for p in torrent_peers {
        peers.push(BencodedValue::Dictionary(vec![
            (
                BencodedValue::ByteString("peer id".into()),
                BencodedValue::ByteString(p.peer_id.into()),
            ),
            (
                BencodedValue::ByteString("ip".into()),
                BencodedValue::ByteString(p.ip.into()),
            ),
            (
                BencodedValue::ByteString("port".into()),
                BencodedValue::Integer(p.port as i64),
            ),
        ]));
    }

    BencodedValue::Dictionary(vec![
        (
            BencodedValue::ByteString("interval".into()),
            BencodedValue::Integer(900),
        ),
        (
            BencodedValue::ByteString("peers".into()),
            BencodedValue::List(peers),
        ),
    ])
    .encode()
}

/// Encodes a list of peers in a binary mode.
pub fn binary_mode(torrent_peers: Vec<PeerTracker>) -> Vec<u8> {
    let mut peers = Vec::<u8>::new();

    for p in torrent_peers {
        let be_port: [u8; 2] = p.port.to_be_bytes();
        let ip: Ipv4Addr = p.ip.parse().unwrap();
        let be_ip: [u8; 4] = u32::from(ip).to_be_bytes();
        peers.append(&mut be_ip.to_vec());
        peers.append(&mut be_port.to_vec());
    }

    BencodedValue::Dictionary(vec![
        (
            BencodedValue::ByteString("interval".into()),
            BencodedValue::Integer(900),
        ),
        (
            BencodedValue::ByteString("peers".into()),
            BencodedValue::ByteString(peers),
        ),
    ])
    .encode()
}

/// Returns the amount of peers that the tracker will return to a client
fn get_amount_of_peers(peers: &[PeerTracker], mut numwant: i32) -> i32 {
    let mut counter = 0;
    for p in peers {
        if p.state == PeerState::Active {
            counter += 1;
        }
    }
    if numwant > counter {
        numwant = counter
    }

    numwant
}

/// Returns the list of peers that will be send to a client.
/// The peers in the list are active.
pub fn get_peers(all_peers: Vec<PeerTracker>, numwant: String) -> Vec<PeerTracker> {
    let count = get_amount_of_peers(&all_peers, numwant.parse().unwrap());
    let mut peers = Vec::<PeerTracker>::with_capacity(count as usize);

    let mut rng = thread_rng();
    let mut idx;
    let mut i = 0;
    let mut vec = Vec::new();
    while i < count {
        idx = rng.gen_range(0, count);
        while vec.contains(&idx) {
            idx = rng.gen_range(0, count);
        }
        vec.push(idx);

        if idx > (all_peers.len() - 1) as i32 {
            idx = 0;
        }

        if all_peers[idx as usize].state == PeerState::Active {
            peers.push(all_peers[idx as usize].clone());
            i += 1;
        }
    }

    peers
}

/// Sets a new state for a peer.
pub fn _set_state(peer: &mut PeerTracker, new_state: PeerState) {
    peer.state = new_state;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn have_one_active_peer_and_get_one_peer() {
        let mut peers = Vec::<PeerTracker>::new();

        let active_peer = PeerTracker::new();
        peers.push(active_peer.clone());

        let mut inactive_peer = PeerTracker::new();
        _set_state(&mut inactive_peer, PeerState::Inactive);
        peers.push(inactive_peer);

        let get = get_peers(peers, "50".to_string());

        assert_eq!(get.len(), 1);
        assert_eq!(get[0], active_peer);
    }

    #[test]
    fn have_non_active_peers_and_get_zero_peers() {
        let mut peers = Vec::<PeerTracker>::new();

        let mut inactive_peer_1 = PeerTracker::new();
        _set_state(&mut inactive_peer_1, PeerState::Inactive);
        peers.push(inactive_peer_1);

        let mut inactive_peer_2 = PeerTracker::new();
        _set_state(&mut inactive_peer_2, PeerState::Inactive);
        peers.push(inactive_peer_2);

        let get = get_peers(peers, "50".to_string());

        assert_eq!(get.len(), 0);
    }
}

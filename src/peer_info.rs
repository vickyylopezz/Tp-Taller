use std::net::IpAddr;

use crate::{client::bitfield, connection, peer::peer_handler::Peer};

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct PeerInfo {
    pub bitfield: bitfield::BitField,
    client: connection::ConnectionStatus,
    pub peer: connection::ConnectionStatus,
    pub peer_id: Vec<u8>,
    pub port: u16,
    pub ip: Option<IpAddr>,
}

impl PeerInfo {
    pub fn new(bitfield: bitfield::BitField, peer: Peer) -> Self {
        Self {
            bitfield,
            client: connection::ConnectionStatus::new(),
            peer: connection::ConnectionStatus::new(),
            peer_id: peer.peer_id.unwrap().to_vec(),
            port: peer.port,
            ip: peer.ip,
        }
    }

    pub fn have(&mut self, index: u32) {
        self.bitfield.set_piece(index as usize);
    }

    pub fn unchoke_from_peer(&mut self) {
        self.peer.1 = connection::ChokeStatus::Unchoked;
    }

    pub fn choke_from_peer(&mut self) {
        self.peer.1 = connection::ChokeStatus::Choked;
    }

    pub fn _interested_from_client(&mut self) {
        self.client.0 = connection::InterestStatus::Interested;
    }

    pub fn update_bitfield(&mut self, b: Vec<u8>) {
        self.bitfield = bitfield::BitField::new_from_vec(b, self.bitfield.pieces());
    }

    pub fn connection_status(&self) -> connection::ConnectionStatus {
        self.peer.clone()
    }
}

use crate::chocked_status::ChokedStatus;
use crate::peer::Peer;

/// Represents a constructor of a Peer.
pub struct PeerBuilder {
    peer_id: Option<[u8; 20]>,
    ip: String,
    port: i64,
    choked: ChokedStatus,
    interested: bool,
    bitfield: Vec<bool>,
}

impl Default for PeerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl PeerBuilder {
    pub fn new() -> Self {
        Self {
            peer_id: None,
            ip: String::new(),
            port: 0,
            choked: ChokedStatus::Choked,
            interested: false,
            bitfield: Vec::new(),
        }
    }

    /// Sets the identifier of the peer.
    pub fn peer_id(&'_ mut self, p: [u8; 20]) -> &'_ mut Self {
        self.peer_id = Some(p);
        self
    }

    /// Sets the ip address of the peer.
    pub fn ip(&'_ mut self, ip: Vec<u8>) -> &'_ mut Self {
        self.ip = String::from_utf8_lossy(&ip).to_string();
        self
    }

    /// Sets the port number of the peer.
    pub fn port(&'_ mut self, po: i64) -> &'_ mut Self {
        self.port = po;
        self
    }

    /// Initialices the atributes of the peer.
    pub fn build(self) -> Peer {
        Peer {
            peer_id: self.peer_id,
            ip: self.ip,
            port: self.port,
            choked: self.choked,
            interested: self.interested,
            bitfield: self.bitfield,
        }
    }
}

use std::{
    net::{IpAddr, Ipv4Addr},
    str::FromStr,
};

use crate::peer::peer_handler::Peer;

/// Represents a constructor of a Peer.
pub struct PeerBuilder {
    /// unique ID for the peer
    pub peer_id: Option<[u8; 20]>,
    /// peer's IP address
    pub ip: Option<IpAddr>,
    /// peer's port number
    pub port: u16,
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
            ip: None,
            port: 0,
        }
    }

    /// Sets the identifier of the peer.
    pub fn peer_id(&'_ mut self, p: [u8; 20]) -> &'_ mut Self {
        self.peer_id = Some(p);
        self
    }

    /// Sets the ip address of the peer.
    pub fn ip(&'_ mut self, ip: Vec<u8>) -> &'_ mut Self {
        if ip.len() == 4 {
            self.ip = Some(IpAddr::V4(Ipv4Addr::new(ip[0], ip[1], ip[2], ip[3])));
        } else {
            let peer_ip = match String::from_utf8_lossy(&ip) {
                std::borrow::Cow::Borrowed(it) => it.to_string(),
                std::borrow::Cow::Owned(it) => it,
            };

            self.ip = match IpAddr::from_str(&peer_ip) {
                Ok(it) => Some(it),
                Err(_) => None,
            };
        }

        //self.ip = String::from_utf8_lossy(&ip).to_string();
        self
    }

    /// Sets the port number of the peer.
    pub fn port(&'_ mut self, po: u16) -> &'_ mut Self {
        self.port = po;
        self
    }

    /// Initialices the atributes of the peer.
    pub fn build(self) -> Peer {
        Peer {
            peer_id: self.peer_id,
            ip: self.ip,
            port: self.port,
        }
    }
}

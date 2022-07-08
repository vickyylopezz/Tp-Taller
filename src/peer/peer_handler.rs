use std::collections::HashMap;
use std::net::IpAddr;

use crate::bencode::bencoded_value::BencodedValue;
use crate::peer::peer_builder::PeerBuilder;

static PEERS_REQUIRED_KEYS: [&[u8]; 2] = [b"ip", b"port"];

/// Contains the data of a peer. Only the ip and
/// port fields are required.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Peer {
    /// unique ID for the peer
    pub peer_id: Option<[u8; 20]>,
    /// peer's IP address
    pub ip: Option<IpAddr>,
    /// peer's port number
    pub port: u16,
}

impl Peer {
    /// Creates a new Peer structure from an ip and a port.
    /// Returns [`Some`] if no errors occur while building the
    /// instance; otherwise returns [`None`].
    pub fn new_byte_string(ip: &[u8], port: &[u8], peer_id: [u8; 20]) -> Option<Self> {
        let mut peer_build = PeerBuilder::new();

        //Port
        peer_build.port(port[0] as u16 * 256 + port[1] as u16);

        //Peer id
        peer_build.peer_id(peer_id);

        //Ip
        let ip = BencodedValue::ByteString(ip.to_vec()).byte_string()?;
        peer_build.ip(ip);

        Some(peer_build.build())
    }

    /// Creates a new Peer structure from a bencoded dictionary.
    /// Returns [`Some`] if no errors occur while building the
    /// instance; otherwise returns [`None`].
    pub fn new_dict(peers: Vec<(BencodedValue, BencodedValue)>, peer_id: [u8; 20]) -> Option<Self> {
        let dict = peers
            .into_iter()
            .collect::<HashMap<BencodedValue, BencodedValue>>();
        let mut required = PEERS_REQUIRED_KEYS
            .iter()
            .map(|v| BencodedValue::ByteString(v.to_vec()));
        let has_required = required.all(|k| dict.contains_key(&k));
        if has_required {
            let mut peer_build = PeerBuilder::new();
            peer_build.peer_id(peer_id);
            for (k, v) in dict {
                if let BencodedValue::ByteString(s) = k {
                    build_peer_fields(&mut peer_build, &s[..], v)?;
                } else {
                    return None;
                }
            }
            Some(peer_build.build())
        } else {
            None
        }
    }
}

/// Helper function for building the Peer struct. Returns [`None`]
/// if there is an error building some of the fields
fn build_peer_fields<'a>(
    peer: &'a mut PeerBuilder,
    field: &'a [u8],
    value: BencodedValue,
) -> Option<()> {
    match field {
        b"peer id" => {
            let p = value.byte_string()?;
            peer.peer_id(p[0..20].try_into().ok()?);
        }
        b"ip" => {
            let ip = value.byte_string()?;
            peer.ip(ip);
        }

        b"port" => {
            let po = value.integer()?;
            peer.port(po as u16);
        }
        _ => return None,
    }
    Some(())
}

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;

    use super::*;
    #[test]
    fn create_a_peer_with_ip_and_port_and_id() {
        let peer_1 = Peer::new_byte_string(
            &[127, 0, 0, 1],
            &[21, 43],
            [
                0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
                0u8, 0u8, 0u8, 0u8,
            ],
        )
        .unwrap();

        let peer_2 = Peer {
            peer_id: Some([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
            ip: Some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
            port: 5419,
        };

        assert_eq!(peer_1, peer_2)
    }
}

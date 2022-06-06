use std::collections::HashMap;

use crate::bencode::bencoded_value::BencodedValue;
use crate::chocked_status::ChokedStatus;
use crate::peer_builder::PeerBuilder;

static PEERS_REQUIRED_KEYS: [&[u8]; 2] = [b"ip", b"port"];

/// Contains the data of a peer. Only the ip and
/// port fields are required.
#[derive(Debug, PartialEq, Eq)]
pub struct Peer {
    /// unique ID for the peer
    pub peer_id: Option<[u8; 20]>,
    /// peer's IP address
    pub ip: String,
    /// peer's port number
    pub port: i64,
    /// peer's posibility of interact with other peers
    pub choked: ChokedStatus,
    /// peer's interest of interact with other peers
    pub interested: bool,
    /// Bitmap of the containing pieces
    pub bitfield: Vec<bool>,
}

impl Peer {
    /// Creates a new Peer structure from an ip and a port.
    /// Returns [`Some`] if no errors occur while building the
    /// instance; otherwise returns [`None`].
    pub fn new_byte_string(ip: &[u8], port: &[u8]) -> Option<Self> {
        let mut peer_build = PeerBuilder::new();
        build_peer_fields(
            &mut peer_build,
            b"ip",
            BencodedValue::ByteString(ip.to_vec()),
        );
        build_peer_fields(
            &mut peer_build,
            b"port",
            BencodedValue::ByteString(port.to_vec()),
        );
        Some(peer_build.build())
    }

    /// Creates a new Peer structure from a bencoded dictionary.
    /// Returns [`Some`] if no errors occur while building the
    /// instance; otherwise returns [`None`].
    pub fn new_dict(peers: Vec<(BencodedValue, BencodedValue)>) -> Option<Self> {
        let dict = peers
            .into_iter()
            .collect::<HashMap<BencodedValue, BencodedValue>>();
        let mut required = PEERS_REQUIRED_KEYS
            .iter()
            .map(|v| BencodedValue::ByteString(v.to_vec()));
        let has_required = required.all(|k| dict.contains_key(&k));
        if has_required {
            let mut peer_build = PeerBuilder::new();
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
            peer.peer_id(p[0..20].try_into().unwrap()); //Chequear unwrap
        }

        b"ip" => {
            let ip = value.byte_string()?;
            peer.ip(ip);
        }

        b"port" => {
            let po = value.integer()?;
            peer.port(po);
        }
        _ => return None,
    }
    Some(())
}

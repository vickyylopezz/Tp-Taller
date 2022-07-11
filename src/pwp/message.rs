use crate::utils::from_u32_be;

/// Represents the possible messages that peers can send between them.
#[derive(Debug, PartialEq, Eq)]

pub enum PWPMessage {
    KeepAlive,
    Choke,
    Unchoke,
    Interested,
    NotInterested,
    Have(u32),
    Bitfield(Vec<u8>),
    Request(u32, u32, u32),
    Piece(u32, u32, Vec<u8>),
    Cancel(u32, u32, u32),
    Handshake(Vec<u8>, Vec<u8>),
}

impl PWPMessage {
    /// Creates a new message using the Peer Wire Protocol
    /// Creates a new instance of [`PWPMessage`] from a `Vec<(BencodedValue,
    /// BencodedValue)>`.  Returns [`Some`] if no errors occur while
    /// building the instance; otherwise returns [`None`].
    pub fn new(msg_id: &u8, buf: &mut &[u8]) -> Option<PWPMessage> {
        let msg = match *msg_id {
            0 => PWPMessage::Choke,
            1 => PWPMessage::Unchoke,
            2 => PWPMessage::Interested,
            3 => PWPMessage::NotInterested,
            4 => PWPMessage::Have(from_u32_be(&mut &buf[..])?),
            5 => PWPMessage::Bitfield(buf.to_owned()),
            6 => {
                let index = from_u32_be(&mut &buf[0..4]);
                let begin = from_u32_be(&mut &buf[4..8]);
                let length = from_u32_be(&mut &buf[8..12]);
                PWPMessage::Request(index?, begin?, length?)
            }
            7 => {
                let index = from_u32_be(&mut &buf[0..4]);
                let begin = from_u32_be(&mut &buf[4..8]);
                let block = buf[8..].to_owned();
                PWPMessage::Piece(index?, begin?, block)
            }
            8 => {
                let index = from_u32_be(&mut &buf[0..4]);
                let begin = from_u32_be(&mut &buf[4..8]);
                let length = from_u32_be(&mut &buf[8..12]);
                PWPMessage::Cancel(index?, begin?, length?)
            }
            b'T' => {
                let info_hash = buf[28..48].to_owned();
                let peer_id = buf[48..].to_owned();
                PWPMessage::Handshake(info_hash, peer_id)
            }
            b'K' => PWPMessage::KeepAlive,

            _ => return None,
        };
        Some(msg)
    }
}

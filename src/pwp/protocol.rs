use crate::peer;
use crate::pwp::message::PWPMessage;
use crate::utils;
use std::{
    io::{Read, Write},
    net::TcpStream,
};

#[derive(Debug)]
pub enum PWPError {
    ReadError, // TODO: Remove, redundant
    WrongSizeRead,
    MissingPeerID, // Should be an error?
    Connection,
    Handshake,
    PeerConnection,
    Read,
    EmptyBytes,
}

#[derive(Debug)]
pub struct PWPStream(pub TcpStream);

impl PWPStream {
    pub fn connect(peer: &peer::Peer, info_hash: Vec<u8>) -> Result<Self, PWPError> {
        let socket = format!("{}:{}", peer.ip, peer.port);
        let mut stream = TcpStream::connect(socket).map_err(|_| PWPError::Connection)?;
        let peer_id = peer.peer_id.ok_or(PWPError::MissingPeerID)?;
        let msg = handshake_msg(info_hash, &peer_id[..]);

        stream.write_all(&msg).map_err(|_| PWPError::Handshake)?;
        println!("Handshake sent");
        Ok(PWPStream(stream))
    }
    /// msg format <length prefix><message ID><payload>
    pub fn send(&mut self, msg: PWPMessage) -> Result<(), PWPError> {
        let bytes = match msg {
            PWPMessage::KeepAlive => 0u32.to_be_bytes().into(),
            PWPMessage::Choke => {
                let mut b: Vec<u8> = vec![0u8, 0u8, 0u8, 1u8];
                b.push(0);
                b
            }
            PWPMessage::Unchoke => {
                let mut b: Vec<u8> = vec![0u8, 0u8, 0u8, 1u8];
                b.push(1);
                b
            }
            PWPMessage::Interested => {
                let mut b: Vec<u8> = vec![0u8, 0u8, 0u8, 1u8];
                b.push(2);
                b
            }
            PWPMessage::NotInterested => {
                let mut b: Vec<u8> = vec![0u8, 0u8, 0u8, 1u8];
                b.push(3);
                b
            }
            PWPMessage::Have(piece_index) => {
                let mut b = vec![0u8, 0u8, 0u8, 5u8, 4u8];
                b.append(&mut piece_index.to_be_bytes().into());
                b
            }
            PWPMessage::Bitfield(mut bitfield) => {
                let len = 1u32 + bitfield.len() as u32;
                let mut b: Vec<u8> = len.to_be_bytes().into();
                b.push(5u8);
                b.append(&mut bitfield);
                b
            }
            PWPMessage::Request(index, begin, length) => {
                let mut b: Vec<u8> = 13u32.to_be_bytes().into();
                b.push(6);
                utils::append!(
                    b,
                    index.to_be_bytes().to_vec(),
                    begin.to_be_bytes().to_vec(),
                    length.to_be_bytes().to_vec()
                )
            }
            PWPMessage::Piece(index, begin, mut block) => {
                let mut b: Vec<u8> = (9u32 + block.len() as u32).to_be_bytes().into();
                b.push(7);
                b.append(&mut index.to_be_bytes().into());
                b.append(&mut begin.to_be_bytes().into());
                b.append(&mut block);
                b
            }
            PWPMessage::Cancel(index, begin, length) => {
                let mut b: Vec<u8> = 13u32.to_be_bytes().into();
                b.push(8);
                b.append(&mut index.to_be_bytes().into());
                b.append(&mut begin.to_be_bytes().into());
                b.append(&mut length.to_be_bytes().into());
                b
            }
            PWPMessage::Handshake(_, _) => todo!(),
            PWPMessage::Empty => todo!(),
        };
        self.0
            .write_all(&bytes)
            .map_err(|_| PWPError::PeerConnection)
    }

    fn read_bytes(&self, bytes_to_read: u32) -> Result<Vec<u8>, PWPError> {
        let mut buf = vec![];
        let stream = &self.0;
        let mut take = stream.take(bytes_to_read as u64);
        let bytes_read = take.read_to_end(&mut buf);
        match bytes_read {
            Ok(n) => {
                if (n as u32) == bytes_to_read {
                    Ok(buf)
                } else {
                    Err(PWPError::WrongSizeRead)
                }
            }
            Err(_) => Err(PWPError::WrongSizeRead),
        }
    }

    /// Interpretates the stream of bytes recieved from the peer.
    pub fn read(&mut self) -> Result<PWPMessage, PWPError> {
        let array = match self.read_bytes(4) {
            Ok(it) => it,
            Err(e) => match e {
                PWPError::EmptyBytes => vec![253],
                PWPError::Read => return Err(PWPError::Read),
                _ => return Err(PWPError::WrongSizeRead),
            },
        };
        if array[0] == 253 {
            return Ok(PWPMessage::new(&b'E', &mut &array[..]));
        }
        let msg_len = from_u32_be(&mut &array[..]).ok_or(PWPError::WrongSizeRead)?;

        let new_msg = if msg_len > 0 {
            let msg = self.read_bytes(msg_len).unwrap();
            PWPMessage::new(&msg[0], &mut &msg[1..])
        } else {
            PWPMessage::new(&b'K', &mut &array[..])
        };

        Ok(new_msg)
    }

    pub fn read_handshake(&mut self) -> Result<PWPMessage, PWPError> {
        let msg_len = 68; // 49 + 19 --> 49 + len(pstr)
        let msg = match self.read_bytes(msg_len) {
            Ok(it) => it,
            Err(_) => return Err(PWPError::WrongSizeRead),
        };

        let new_msg = PWPMessage::new(&msg[4], &mut &msg[0..]);

        println!("Handshake recieved");
        Ok(new_msg)
    }
}

fn handshake_msg(info_hash: Vec<u8>, peer_id: &[u8]) -> Vec<u8> {
    let mut pstr = b"BitTorrent protocol".to_vec();
    let mut pstrlen = vec![pstr.len() as u8];
    let mut reserved = vec![0u8; 8];
    let mut hash = info_hash;
    utils::append!(pstrlen, pstr, reserved, hash, peer_id.to_vec())
}

// TODO: Move to utils
fn from_u32_be(array: &mut &[u8]) -> Option<u32> {
    let (int_bytes, rest) = array.split_at(std::mem::size_of::<u32>());
    *array = rest;
    Some(u32::from_be_bytes(int_bytes.try_into().ok()?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_handshake_msg_correctly() {
        let got = handshake_msg(vec![0u8; 20], &[0u8; 20]);
        let want = vec![
            19, b'B', b'i', b't', b'T', b'o', b'r', b'r', b'e', b'n', b't', b' ', b'p', b'r', b'o',
            b't', b'o', b'c', b'o', b'l', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];

        assert_eq!(got, want);
    }
}

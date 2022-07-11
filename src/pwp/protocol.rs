use crate::pwp::message::PWPMessage;
use crate::utils;
use crate::{peer::peer_handler::Peer, pwp::protocol_error::ProtocolError};
use std::net::SocketAddr;
use std::{io::Read, io::Write, net::TcpStream};

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
    MappingError,
}

#[derive(Debug)]
pub struct PWPStream(pub TcpStream);

impl PWPStream {
    pub fn connect(peer: &Peer, info_hash: Vec<u8>) -> Result<Self, ProtocolError> {
        let ip = match peer.ip {
            Some(it) => it,
            None => return Err(ProtocolError::Connection),
        };
        let socket = SocketAddr::new(ip, peer.port);
        // println!("{}", socket);
        let mut stream = TcpStream::connect(socket).map_err(|_| ProtocolError::Connection)?;
        // println!("{:?}", stream);
        let peer_id = peer.peer_id.ok_or(ProtocolError::MissingPeerID)?;
        let msg = handshake_msg(info_hash, &peer_id[..]);
        //let mut stream = stream;
        stream
            .write_all(&msg)
            .map_err(|_| ProtocolError::Handshake)?;

        // println!("Handshake sent");

        Ok(PWPStream(stream))
    }

    /// msg format <length prefix><message ID><payload>
    ///
    /// We have decided to keep the fully match arm without any modularization because it's much
    /// less complex to understand what is going on with each message.
    ///
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
            PWPMessage::Handshake(info_hash, peer_id) => handshake_msg(info_hash, &peer_id),
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
            Err(_) => Err(PWPError::ReadError),
        }
    }

    /// Interpretates the stream of bytes recieved from the peer.
    pub fn read(&mut self) -> Result<PWPMessage, PWPError> {
        let array = self.read_bytes(4)?;
        if array[3] == 0 {
            return PWPMessage::new(&b'K', &mut &array[..]).ok_or(PWPError::MappingError);
        }
        let msg_len = utils::from_u32_be(&mut &array[..]).ok_or(PWPError::WrongSizeRead)?;

        if msg_len > 0 {
            let msg = self.read_bytes(msg_len)?;
            PWPMessage::new(&msg[0], &mut &msg[1..]).ok_or(PWPError::MappingError)
        } else {
            PWPMessage::new(&b'K', &mut &array[..]).ok_or(PWPError::MappingError)
        }
    }

    /// Reads a handshake message and returns it.
    pub fn read_handshake(&mut self) -> Result<PWPMessage, PWPError> {
        let msg_len = 68; // 49 + 19 --> 49 + len(pstr)
        let msg = match self.read_bytes(msg_len) {
            Ok(it) => it,
            Err(_) => return Err(PWPError::WrongSizeRead),
        };

        PWPMessage::new(&msg[4], &mut &msg[0..]).ok_or(PWPError::MappingError)
    }

    pub fn new(stream: TcpStream) -> Self {
        Self(stream)
    }
}

fn handshake_msg(info_hash: Vec<u8>, peer_id: &[u8]) -> Vec<u8> {
    let mut pstr = b"BitTorrent protocol".to_vec();
    let mut pstrlen = vec![pstr.len() as u8];
    let mut reserved = vec![0u8; 8];
    let mut hash = info_hash;
    utils::append!(pstrlen, pstr, reserved, hash, peer_id.to_vec())
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

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
    Empty, // Port is needed?
}

impl PWPMessage {
    pub fn new(msg_id: &u8, buf: &mut &[u8]) -> PWPMessage {
        match *msg_id {
            0 => PWPMessage::Choke,
            1 => PWPMessage::Unchoke,
            2 => PWPMessage::Interested,
            3 => PWPMessage::NotInterested,
            4 => PWPMessage::Have(from_u32_be(&mut &buf[..]).unwrap()),
            5 => PWPMessage::Bitfield(buf.to_owned()),
            6 => {
                let index = from_u32_be(&mut &buf[0..4]);
                let begin = from_u32_be(&mut &buf[4..8]);
                let length = from_u32_be(&mut &buf[8..12]);
                PWPMessage::Request(index.unwrap(), begin.unwrap(), length.unwrap())
            }
            7 => {
                let index = from_u32_be(&mut &buf[0..4]);
                let begin = from_u32_be(&mut &buf[4..8]);
                let block = buf[8..].to_owned();
                PWPMessage::Piece(index.unwrap(), begin.unwrap(), block)
            }
            8 => {
                let index = from_u32_be(&mut &buf[0..4]);
                let begin = from_u32_be(&mut &buf[4..8]);
                let length = from_u32_be(&mut &buf[8..12]);
                PWPMessage::Cancel(index.unwrap(), begin.unwrap(), length.unwrap())
            }
            b'T' => {
                let info_hash = buf[28..48].to_owned();
                let peer_id = buf[48..].to_owned();
                PWPMessage::Handshake(info_hash, peer_id)
            }
            b'E' => PWPMessage::Empty,
            b'K' => PWPMessage::KeepAlive,

            //9 => PWPMessage::Port, //review parameters of the port in case is needed
            _ => panic!("Bad message id: {}", msg_id), //manage the error in a better way
        }
    }
}

// TODO: Move to utils
fn from_u32_be(array: &mut &[u8]) -> Option<u32> {
    let (int_bytes, rest) = array.split_at(std::mem::size_of::<u32>());
    *array = rest;
    Some(u32::from_be_bytes(int_bytes.try_into().ok()?))
}

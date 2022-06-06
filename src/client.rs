use crate::client_error::ClientError;
use crate::peer::Peer;
use crate::piece::Piece;
use crate::pwp::message::PWPMessage;
use crate::pwp::protocol::PWPStream;
use crate::torrent::info::Info;
use crate::torrent::metainfo::{self, Metainfo};
use crate::tracker::tracker_handler::{hash_info, TrackerHandler};

use std::fs::File;
use std::{env, fs};

/// Represents a bittorrent client.
#[derive(Debug, PartialEq, Eq)]
pub struct Client<'a> {
    /// List of peers that have some piece from the file to be downloaded
    peers: Option<&'a Vec<Peer>>,
    /// File name of the file to be downloaded
    file: String,
}
impl Default for Client<'_> {
    fn default() -> Self {
        Self::new()
    }
}
impl Client<'_> {
    pub fn new() -> Self {
        Client {
            file: get_file(),
            peers: None,
        }
    }

    /// Connects the bittorrent client with a peer and downloads the piece received
    pub fn run(self) {
        let metainfo = metainfo::read_torrent(File::open(self.file).unwrap()).unwrap();
        let mut tracker = TrackerHandler::new(&metainfo);
        let peers = get_peers(&mut tracker);
        for p in peers {
            println!("Connected to peer {}:{}", p.ip, p.port);
            if connect_to_peer(p, &metainfo).is_ok() {
                break;
            }
            println!("Failed connection");
        }
    }
}

/// Returns the torrent file entered by the user
fn get_file() -> String {
    let args: Vec<String> = env::args().collect();
    let file_name = &args[1];

    file_name.trim().to_string()
}

/// Returns the list of peers of the tracker response
fn get_peers<'a>(tracker: &'a mut TrackerHandler) -> &'a Vec<Peer> {
    // Send request to the tracker and get the response of the tracker
    tracker.manage_interaction().unwrap();

    // Get list of peers that have the pieces of the file to be downloaded
    tracker.get_peers().unwrap()
}

/// Connect the bittorrent client with a peer from the list of peers
/// and returns the piece from it
fn connect_to_peer(peer: &Peer, metainfo: &Metainfo) -> Result<(), ClientError> {
    // TCP connection
    // Client Handshake
    let mut stream = PWPStream::connect(peer, hash_info(metainfo.info.bencode()).to_vec()).unwrap();

    // Peer Handshake
    let handshake_msg = stream
        .read_handshake()
        .map_err(|_| ClientError::HandshakeError)?;

    if let PWPMessage::Handshake(has_info, _) = handshake_msg {
        if has_info != hash_info(metainfo.info.bencode()) {
            return Err(ClientError::HandshakeError);
        }
    }

    //Bitfield
    stream.read().map_err(|_| ClientError::BitfieldError)?;
    println!("Bitfield recieved");

    //Interested
    stream.send(PWPMessage::Interested).unwrap();
    println!("Interested sent");

    //Unchoke
    stream.read().unwrap();
    println!("Unchoke recieved");

    let Info(i) = &metainfo.info;
    let info = match i {
        crate::torrent::info::InfoMode::Empty => todo!(),
        crate::torrent::info::InfoMode::SingleFile(it) => it,
    };

    let mut piece = Piece::new(
        info.piece_length,
        253,
        info.piece_length,
        info.pieces[253 * 20..253 * 20 + 20].to_vec(),
    );
    let requests = info.piece_length as u32 / 16384;

    let mut file = File::create(info.name.clone()).unwrap();
    for i in 0..requests - 1 {
        stream
            .send(PWPMessage::Request(253, i * 16384, 16384))
            .unwrap();
        let data = match stream.read().unwrap() {
            PWPMessage::Piece(_, _, data) => data,
            _ => todo!(),
        };
        piece.store(&mut file, i, data).unwrap();
        println!("Block {} of piece 253 recieved", i);
    }

    let lenght = {
        if info.piece_length as u32 % 16384 > 0 {
            info.piece_length as u32 % 16384
        } else {
            16384
        }
    };
    let _request_msg = stream.send(PWPMessage::Request(253, (requests - 1) * 16384, lenght)); // Obtener parámetros del RequestPiece
    let data = match stream.read().unwrap() {
        PWPMessage::Piece(_, _, data) => data,
        _ => todo!(),
    };

    println!("Block {} of piece 253 recieved", requests - 1);
    file.set_len(info.piece_length as u64).unwrap();
    piece.store(&mut file, requests - 1, data).unwrap();
    fs::rename(
        info.name.clone(),
        "piece253-".to_owned() + &info.name.clone(),
    )
    .unwrap();

    println!("Piece 253 downloaded");

    Ok(())
}

use log::{error, info};
use std::fs::File;
use std::io::{self, Read, Seek, Write};
use std::path::Path;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};

use crate::client::bitfield::BitField;
use crate::client::torrent_file::TorrentFile;
use crate::download::bitfield_download::{BitFieldDownload, Status};

use crate::log::logger::LogHandle;
use crate::peer::peer_handler::Peer;
use crate::pwp::message::PWPMessage;
use crate::pwp::protocol::PWPStream;
use crate::storage::piece::Piece;
use crate::utils;

const BLOCK_SIZE: u32 = 16384; //2^14

pub struct HandlerDownload;

pub enum HandlerMessage {
    Piece(usize),
    HaveAllPieces,
    PeerConnected(Peer),
    PeerNotConnected,
    Have,
    Bitfield,
    Unchoke,
    NotAllPieces,
    Terminate,
}

pub enum ClientHandlerMessage {
    WrongHash(u32, u32), // Piece index, block
    Terminate,
}

impl HandlerDownload {
    pub fn new(
        logger: LogHandle,
        torrents: Arc<Mutex<Vec<TorrentFile>>>,
        i: usize,
        directory: String,
        torrent: TorrentFile,
    ) -> Self {
        let (ui_sender, ui_receiver) = mpsc::channel();
        let info = utils::get_info_from_torrentfile(torrent.metainfo.info.clone());

        listen_peers(
            Arc::new(Mutex::new(ui_receiver)),
            torrents,
            i,
            directory.clone(),
            logger.clone(),
            torrent.clone(),
        );

        thread::spawn(move || {
            let bit = match BitFieldDownload::new(torrent.pieces_ammount as usize) {
                Ok(b) => b,
                Err(_) => return,
            };
            let data = match torrent.response.clone() {
                Some(data) => data,
                None => return,
            };
            let handler_bitfield = Arc::new(Mutex::new(bit));
            let mut threads = Vec::<JoinHandle<()>>::new();
            for p in data.peers {
                let info_hash =
                    utils::hash_info(&torrent.clone().metainfo.info.clone().bencode()).to_vec();

                let handler_bitfield = Arc::clone(&handler_bitfield);

                let file_name = info.name.clone();
                let pieces = info.pieces.clone();

                let ui_sender = ui_sender.clone();
                let directory = directory.clone();
                let log_handle = logger.clone();
                let torrent = torrent.clone();
                let thread = thread::spawn(move || {
                    let mut stream =
                        match stream_peers(p, info_hash, log_handle.clone(), ui_sender.clone()) {
                            Some(stream) => stream,
                            None => return,
                        };
                    let mut peer_bitfield = match BitField::new(torrent.pieces_ammount as usize) {
                        Ok(bit) => bit,
                        Err(_) => return,
                    };
                    (stream, peer_bitfield) =
                        match stream_unchoke(stream, peer_bitfield, torrent.clone()) {
                            Some(stream) => stream,
                            None => return,
                        };

                    let mut handle_miss = Vec::new();
                    if let Ok(bit) = handler_bitfield.lock() {
                        handle_miss = bit.get_missing()
                    }
                    let piece_index = match piece_miss(handle_miss, peer_bitfield.get_available()) {
                        Some(index) => index,
                        None => return,
                    };
                    let piece = Piece::new(
                        info.piece_length as i64,
                        piece_index as i64,
                        pieces[piece_index as usize * 20..piece_index as usize * 20 + 20].to_vec(),
                        file_name.clone(),
                        directory.clone(),
                    );
                    if let Ok(mut bit) = handler_bitfield.lock() {
                        bit.set_piece(piece.index as usize, Status::InProgress);
                    }
                    download_piece(
                        stream,
                        piece,
                        ui_sender,
                        directory,
                        log_handle.clone(),
                        torrent,
                        (handler_bitfield, peer_bitfield),
                    );
                });

                threads.push(thread);
            }

            for t in threads {
                t.join().unwrap();
            }
        });

        Self
    }
}

fn piece_miss(handle_miss: Vec<usize>, peer_miss: Vec<usize>) -> Option<usize> {
    for elem in handle_miss.iter() {
        for elem2 in peer_miss.iter() {
            if elem == elem2 {
                return Some(*elem);
            }
        }
    }
    None
}

fn stream_unchoke(
    mut stream: PWPStream,
    mut peer_bitfield: BitField,
    torrent: TorrentFile,
) -> Option<(PWPStream, BitField)> {
    let mut unchoke = false;
    while !unchoke {
        match stream.read() {
            Ok(it) => match it {
                PWPMessage::Unchoke => unchoke = true,
                PWPMessage::Have(piece) => {
                    peer_bitfield.set_piece(piece as usize);
                    match stream.send(PWPMessage::Interested) {
                        Ok(_) => (),
                        Err(_) => return None,
                    }
                }
                PWPMessage::Bitfield(bitfield) => {
                    peer_bitfield =
                        BitField::new_from_vec(bitfield, torrent.pieces_ammount as usize);
                    match stream.send(PWPMessage::Interested) {
                        Ok(_) => (),
                        Err(_) => return None,
                    }
                }
                _ => break,
            },
            Err(_) => return None,
        }
    }
    Some((stream, peer_bitfield))
}

fn stream_peers(
    p: Peer,
    info_hash: Vec<u8>,
    mut log_handle: LogHandle,
    ui_sender: Sender<HandlerMessage>,
) -> Option<PWPStream> {
    match connect_to_useful_peer(p.clone(), info_hash) {
        Some(it) => {
            let ip =
                p.ip.map(|ip| ip.to_string())
                    .unwrap_or_else(|| "-".to_string());
            info!("Connected to peer: {}", ip);
            log_handle.info(&format!("Connected to peer: {}", ip));

            match ui_sender.send(HandlerMessage::PeerConnected(p)) {
                Ok(_) => (),
                Err(_) => return None,
            }
            Some(it)
        }
        None => {
            let ip =
                p.ip.map(|ip| ip.to_string())
                    .unwrap_or_else(|| "-".to_string());
            info!("Couldn't connect to peer: {}", ip);
            log_handle.info(&format!("Couldn't connect to peer: {}", ip));
            None
        }
    }
}
fn listen_peers(
    shared_ui_rx: Arc<Mutex<Receiver<HandlerMessage>>>,
    torrents: Arc<Mutex<Vec<TorrentFile>>>,
    i: usize,
    directory: String,
    logger: LogHandle,
    torrent: TorrentFile,
) -> Option<JoinHandle<()>> {
    let mut log_handle = logger;
    let info = utils::get_info_from_torrentfile(torrent.metainfo.info);
    Some(thread::spawn(move || loop {
        if let Ok(receiver) = shared_ui_rx.lock() {
            match receiver.try_recv() {
                Ok(it) => {
                    match it {
                        HandlerMessage::Piece(index) => {
                            if let Ok(mut torrents) = torrents.lock() {
                                (*torrents)[i].bitfield.set_piece(index);
                            }
                        }
                        HandlerMessage::HaveAllPieces => {
                            store_file(
                                info.name.clone(),
                                info.piece_length as u64 * torrent.pieces_ammount as u64,
                                torrent.pieces_ammount as i32,
                                info.piece_length as u64,
                                &directory,
                                log_handle.clone(),
                            );
                            info!("Torrent {} downloaded!", info.name);
                            log_handle.info(&format!("Torrent {} downloaded", info.name));
                        }
                        HandlerMessage::PeerConnected(p) => {
                            if let Ok(mut torrents) = torrents.lock() {
                                (*torrents)[i].peers_connected.push(p);
                            }
                        }
                        _ => continue,
                    };
                }
                Err(_) => continue,
            }
        }
    }))
}
pub fn store_file(
    file_name: String,
    file_length: u64,
    pieces: i32,
    piece_length: u64,
    directory: &str,
    mut logger: LogHandle,
) {
    let mut file = match File::create(Path::new(&format!("{}{}", directory, file_name))) {
        Ok(file) => file,
        Err(_) => return,
    };

    match file.set_len(file_length) {
        Ok(_) => (),
        Err(_) => return,
    };

    for i in 0..pieces {
        let mut piece = match File::open(Path::new(&format!(
            "{}piece{}-{}",
            directory,
            i,
            file_name.clone()
        ))) {
            Ok(it) => it,
            Err(e) => {
                error!("{}, while storing piece: {}", e, i);
                logger.error(&format!("{}, while storing piece: {}", e, i));
                return;
            }
        };
        let mut buf = Vec::<u8>::new();
        match piece.read_to_end(&mut buf) {
            Ok(_) => (),
            Err(_) => return,
        };
        let offset = i as u64 * piece_length as u64;
        match file.seek(io::SeekFrom::Start(offset)) {
            Ok(_) => (),
            Err(_) => return,
        };
        match file.write_all(&buf) {
            Ok(_) => (),
            Err(_) => return,
        };
    }
}

fn download_piece(
    mut stream: PWPStream,
    mut piece: Piece,
    sender: Sender<HandlerMessage>,
    directory: String,
    logger: LogHandle,
    torrent: TorrentFile,
    bitfields: (Arc<Mutex<BitFieldDownload>>, BitField),
) {
    let bitfield = bitfields.0;
    let peer_bitfield = bitfields.1;
    let info = utils::get_info_from_torrentfile(torrent.metainfo.info.clone());
    let blocks_ammount = ((info.piece_length as f64) / (BLOCK_SIZE as f64)).ceil() as u32;

    for i in 0..blocks_ammount - 1 {
        match stream.send(PWPMessage::Request(piece.index as u32, i * 16384, 16384)) {
            Ok(_) => (),
            Err(_) => return,
        };
        match stream.read() {
            Ok(msg) => match msg {
                PWPMessage::Piece(_, _, block) => {
                    match piece.store(i, block) {
                        Ok(_) => (),
                        Err(_) => return,
                    };
                }
                _ => {
                    if let Ok(mut bit) = bitfield.lock() {
                        bit.set_piece(piece.index as usize, Status::NotDownload);
                    }
                    return;
                }
            },

            Err(_) => {
                if let Ok(mut bit) = bitfield.lock() {
                    bit.set_piece(piece.index as usize, Status::NotDownload);
                }
                return;
            }
        };
    }

    let lenght = {
        if piece.length as u32 % 16384 > 0 {
            piece.length as u32 % 16384
        } else {
            16384
        }
    };

    let _request_msg = stream.send(PWPMessage::Request(
        piece.index as u32,
        (blocks_ammount - 1) * 16384,
        lenght,
    ));
    if let Ok(msg) = stream.read() {
        match msg {
            PWPMessage::Piece(_, _, data) => {
                next_piece(
                    stream,
                    (piece, data),
                    sender,
                    directory,
                    logger,
                    torrent,
                    (bitfield, peer_bitfield),
                );
            }
            _ => {
                if let Ok(mut bit) = bitfield.lock() {
                    bit.set_piece(piece.index as usize, Status::NotDownload);
                }
            }
        }
    }
}

fn next_piece(
    stream: PWPStream,
    piece: (Piece, Vec<u8>),
    sender: Sender<HandlerMessage>,
    directory: String,
    mut logger: LogHandle,
    torrent: TorrentFile,
    bitfields: (Arc<Mutex<BitFieldDownload>>, BitField),
) {
    let data = piece.1;
    let mut piece = piece.0;
    let bitfield = bitfields.0;
    let peer_bitfield = bitfields.1;

    let info = utils::get_info_from_torrentfile(torrent.metainfo.info.clone());

    let blocks_ammount = ((info.piece_length as f64) / (BLOCK_SIZE as f64)).ceil() as u32;

    match piece.store(blocks_ammount - 1, data) {
        Ok(_) => (),
        Err(_) => return,
    };
    match sender.send(HandlerMessage::Piece(piece.index as usize)) {
        Ok(_) => (),
        Err(_) => return,
    };
    if let Ok(peer) = stream.0.peer_addr() {
        info!(
            "Downloaded piece {} from peer {} for {}",
            piece.index,
            peer.ip(),
            info.name
        );
        logger.info(&format!(
            "Downloaded piece {} from peer {} for {} ",
            piece.index,
            peer.ip(),
            info.name,
        ));
    }

    let mut piece_index = 0;
    if let Ok(mut bit) = bitfield.lock() {
        bit.set_piece(piece.index as usize, Status::Downloaded);
        let missing = bit.get_missing();
        if bit.has_all_pieces() {
            match sender.send(HandlerMessage::HaveAllPieces) {
                Ok(_) => (),
                Err(_) => return,
            };
            return;
        } else if missing.is_empty() {
            return;
        }
        let mut has_piece = false;
        for i in missing {
            if peer_bitfield.has_piece(i) {
                piece_index = i;
                has_piece = true;
                break;
            }
        }
        if !has_piece {
            return;
        }
    }

    let piece = Piece::new(
        piece.length,
        piece_index as i64,
        info.pieces[piece_index as usize * 20..piece_index as usize * 20 + 20].to_vec(),
        info.name.clone(),
        directory.to_string(),
    );
    if let Ok(mut bit) = bitfield.lock() {
        bit.set_piece(piece.index as usize, Status::InProgress);
    }
    download_piece(
        stream,
        piece,
        sender,
        directory,
        logger,
        torrent,
        (bitfield, peer_bitfield),
    );
}

// impl Drop for HandlerDownload {
//     fn drop(&mut self) {
//         self.sender.send(ClientHandlerMessage::Terminate).unwrap();
//         if let Some(thread) = self.handler.take() {
//             thread.join().unwrap();
//         }
//     }
// }

pub fn connect_to_useful_peer(peer: Peer, hash: Vec<u8>) -> Option<PWPStream> {
    let mut stream = match PWPStream::connect(&peer, hash.clone()) {
        Ok(it) => it,
        Err(_) => return None,
    };
    let handshake_msg = match stream.read_handshake() {
        Ok(it) => match it {
            PWPMessage::Handshake(info_hash, peer_id) => PWPMessage::Handshake(info_hash, peer_id),
            _ => return None,
        },
        Err(_) => return None,
    };
    if let PWPMessage::Handshake(has_info, _) = handshake_msg {
        if has_info != hash {
            return None;
        }
    }
    Some(stream)
}

use std::sync::{mpsc, Arc, Mutex};
use std::thread;

use log::info;

use crate::client::bitfield;
use crate::download::bitfield_download::{BitFieldDownload, Status};

use crate::log::logger::LogHandle;
use crate::peer::peer_handler::Peer;
use crate::peer_info::PeerInfo;
use crate::pwp::message::{self, PWPMessage};
use crate::pwp::protocol::PWPStream;
use crate::storage::piece::Piece;
use crate::threadpool;

const BLOCK_SIZE: u32 = 16384; //2^14

pub struct HandlerDownload {
    handler: Option<thread::JoinHandle<()>>,
    receiver: mpsc::Receiver<HandlerMessage>,
    sender: mpsc::Sender<ClientHandlerMessage>,
}

pub enum HandlerMessage {
    Piece(Piece),
    HaveAllPieces,
    PeerConnected(PeerInfo),
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
        info_hash: Vec<u8>,
        peers: Vec<Peer>,
        pieces_ammount: u32,
        piece_length: u32,
        logger: LogHandle,
        file_name: String,
        pieces: Vec<u8>,
    ) -> Self {
        let (sender, receiver) = mpsc::channel(); // Por este canal se envia lo que va desde el supervisor al cliente
        let (client_handler_tx, _client_handler_rx) = mpsc::channel();

        let handler = Some(thread::spawn(move || {
            let mut handle = logger.clone();
            let mut pool = threadpool::ThreadPool::new(peers.len(), handle.clone()).unwrap();
            let handler_bitfield = Arc::new(Mutex::new(
                BitFieldDownload::new(pieces_ammount as usize).unwrap(),
            ));
            let peer_number: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
            loop {
                let arc_bitfield = Arc::clone(&handler_bitfield);
                let tx_handler_client = mpsc::Sender::clone(&sender);

                let peer_number_clone = Arc::clone(&peer_number);
                let peers = peers.clone();
                let info_hash = info_hash.clone();

                let mut stream;
                let mut peer;
                if let Ok(mut peer_n) = peer_number_clone.lock() {
                    if *peer_n >= peers.len() {
                        *peer_n = 0;
                    }
                    stream = match connect_to_useful_peer(peers[*peer_n].clone(), info_hash.clone())
                    {
                        Some(it) => {
                            info!(
                                "Connected to peer: {}",
                                peers[*peer_n]
                                    .ip
                                    .map(|ip| ip.to_string())
                                    .unwrap_or_else(|| "-".into())
                            );
                            handle.info(&format!(
                                "Connected to peer: {}",
                                peers[*peer_n]
                                    .ip
                                    .map(|ip| ip.to_string())
                                    .unwrap_or_else(|| "-".into())
                            ));
                            let bitfield = bitfield::BitField::new(pieces_ammount as usize);
                            peer = PeerInfo::new(bitfield.unwrap(), peers[*peer_n].clone());
                            tx_handler_client
                                .send(HandlerMessage::PeerConnected(peer.clone()))
                                .unwrap();

                            *peer_n += 1;
                            it
                        }
                        None => {
                            tx_handler_client
                                .send(HandlerMessage::PeerNotConnected)
                                .unwrap();

                            *peer_n += 1;
                            continue;
                        }
                    };
                } else {
                    continue;
                }

                if let Ok(bit) = arc_bitfield.lock() {
                    if bit.has_all_pieces() {
                        info!("All pieces downloaded");
                        handle.info("All pieces downloaded");
                        tx_handler_client
                            .send(HandlerMessage::HaveAllPieces)
                            .unwrap();
                        break;
                    }
                    tx_handler_client
                        .send(HandlerMessage::NotAllPieces)
                        .unwrap();
                }

                // let bitfield = bitfield::BitField::new(pieces_ammount as usize);

                // let mut peer = PeerInfo::new(bitfield.unwrap(), peer);

                let pieces = pieces.clone();
                let file_name = file_name.clone();
                let mut handle_clone = handle.clone();
                pool.spawn(move || {
                    //let thr = thread::spawn(move || {
                    let mut cant_desc = 0;
                    let mut block_number = 0;
                    let mut am_unchoked = false;
                    let mut piece_miss = 0;
                    let mut vec_pieces = Vec::<Piece>::new();
                    loop {
                        let file_name = file_name.clone();

                        //Read peer messagess
                        match stream.read() {
                            Ok(msg) => {
                                match msg {
                                    message::PWPMessage::KeepAlive => {
                                        return;
                                    }
                                    message::PWPMessage::Choke => {
                                        info!("Client choked");
                                        handle_clone.info("Client choked");
                                        peer.choke_from_peer();
                                        return;
                                    }
                                    message::PWPMessage::Unchoke => {
                                        if !am_unchoked {
                                            am_unchoked = true;
                                            peer.unchoke_from_peer();
                                            tx_handler_client
                                                .send(HandlerMessage::Unchoke)
                                                .unwrap();

                                            info!("Client unchoked");
                                            handle_clone.info("Client unchoked");
                                            //Elegir pieza a descargar
                                            if let Ok(mut bit) = arc_bitfield.lock() {
                                                piece_miss = bit.get_missing()[0];
                                                let mut counter = 1;
                                                while !peer.bitfield.has_piece(piece_miss)
                                                    && counter < pieces_ammount
                                                {
                                                    piece_miss =
                                                        bit.get_missing()[counter as usize];
                                                    counter += 1;
                                                }
                                                bit.set_piece(piece_miss, Status::InProgress);
                                                let piece = Piece::new(
                                                    piece_length as i64,
                                                    piece_miss as i64,
                                                    pieces[piece_miss * 20..piece_miss * 20 + 20]
                                                        .to_vec(),
                                                    file_name,
                                                );

                                                vec_pieces.push(piece);
                                            }

                                            stream
                                                .send(PWPMessage::Request(
                                                    piece_miss as u32,
                                                    block_number * 16384,
                                                    16384,
                                                ))
                                                .unwrap();
                                            block_number += 1;
                                        }
                                    }

                                    message::PWPMessage::Piece(_, begin, data) => {
                                        vec_pieces[cant_desc]
                                            .add_block((begin) as i64 / BLOCK_SIZE as i64, data);
                                        let num_blocks =
                                            ((piece_length as f64) / (BLOCK_SIZE as f64)).ceil()
                                                as u32;
                                        if block_number > num_blocks - 1 {
                                            tx_handler_client
                                                .send(HandlerMessage::Piece(
                                                    vec_pieces[cant_desc].clone(),
                                                ))
                                                .unwrap(); //Le manda al cliente la pieza

                                            cant_desc += 1;

                                            if cant_desc < 5 {
                                                if let Ok(mut bit) = arc_bitfield.lock() {
                                                    bit.set_piece(piece_miss, Status::Downloaded);

                                                    if !bit.has_all_pieces() {
                                                        piece_miss = bit.get_missing()[0];
                                                        let mut counter = 1;
                                                        while !peer.bitfield.has_piece(piece_miss)
                                                            && counter < pieces_ammount
                                                        {
                                                            piece_miss =
                                                                bit.get_missing()[counter as usize];
                                                            counter += 1;
                                                        }
                                                        bit.set_piece(
                                                            piece_miss,
                                                            Status::InProgress,
                                                        );
                                                        let piece = Piece::new(
                                                            piece_length as i64,
                                                            piece_miss as i64,
                                                            pieces[piece_miss * 20
                                                                ..piece_miss * 20 + 20]
                                                                .to_vec(),
                                                            file_name,
                                                        );
                                                        vec_pieces.push(piece);
                                                    }
                                                }

                                                block_number = 0;
                                                stream
                                                    .send(PWPMessage::Request(
                                                        piece_miss as u32,
                                                        block_number * 16384,
                                                        16384,
                                                    ))
                                                    .unwrap();

                                                block_number += 1;
                                                continue;
                                            } else {
                                                break;
                                            }
                                        }
                                        if block_number < num_blocks {
                                            stream
                                                .send(PWPMessage::Request(
                                                    piece_miss as u32,
                                                    block_number * 16384,
                                                    16384,
                                                ))
                                                .unwrap();
                                            block_number += 1;
                                        }
                                    }
                                    message::PWPMessage::Have(index) => {
                                        tx_handler_client.send(HandlerMessage::Have).unwrap();

                                        peer.have(index);
                                        stream.send(message::PWPMessage::Interested).unwrap();
                                    }
                                    message::PWPMessage::Bitfield(b) => {
                                        tx_handler_client.send(HandlerMessage::Bitfield).unwrap();
                                        peer.update_bitfield(b);
                                        stream.send(message::PWPMessage::Interested).unwrap();
                                    }
                                    _ => (),
                                };
                            }
                            Err(_) => return,
                        }
                    }
                })
                .unwrap();
            }
            pool.join().unwrap();
        }));

        Self {
            handler,
            receiver,
            sender: client_handler_tx,
        }
    }

    pub fn send(
        &self,
        message: ClientHandlerMessage,
    ) -> Result<(), mpsc::SendError<ClientHandlerMessage>> {
        self.sender.send(message)
    }

    pub fn receive(&self) -> Result<HandlerMessage, mpsc::TryRecvError> {
        self.receiver.try_recv()
    }
}

impl Drop for HandlerDownload {
    fn drop(&mut self) {
        self.sender.send(ClientHandlerMessage::Terminate).unwrap();
        if let Some(thread) = self.handler.take() {
            thread.join().unwrap();
        }
    }
}

pub fn connect_to_useful_peer(peer: Peer, hash: Vec<u8>) -> Option<PWPStream> {
    let mut stream = match PWPStream::connect(&peer, hash.clone()) {
        Ok(it) => it,
        Err(_) => return None,
    };

    // Peer Handshake
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
fn _available_peers(peers: Vec<Peer>, info_hash: Vec<u8>) -> Vec<PWPStream> {
    peers
        .iter()
        .filter_map(|p| PWPStream::connect(p, info_hash.clone()).ok())
        .collect::<Vec<_>>() // Reintentarlo
                             // Salir si no se pudieron establecer las conexiones
}

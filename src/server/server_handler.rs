use crate::{client::bitfield::BitField, utils};
use log::{error, info};
use std::{
    fs,
    io::{Read, Seek, SeekFrom},
    net::{SocketAddr, TcpListener},
    sync::{Arc, Mutex},
    thread,
};

use crate::{
    client::torrent_file::TorrentFile,
    log::logger::LogHandle,
    pwp::{message::PWPMessage, protocol::PWPStream},
};

use super::server_error::ServerError;

#[cfg(not(feature = "server-demo"))]
const LISTENER_IP: &str = "0.0.0.0";

#[cfg(feature = "server-demo")]
const LISTENER_IP: &str = "127.0.0.1";

#[derive(Debug)]
/// Represents a bittorrent client.
pub struct Server {
    thread: Option<thread::JoinHandle<()>>,
    torrents: Arc<Mutex<Vec<TorrentFile>>>,
}

impl Server {
    pub fn new(torrents: Arc<Mutex<Vec<TorrentFile>>>) -> Self {
        Self {
            thread: None,
            torrents,
        }
    }

    /// Starts the server process, the sever will be listening for
    /// requests at the port specified in the config file
    pub fn run(
        &mut self,
        port: u16,
        download: String,
        mut logger: LogHandle,
    ) -> Result<(), ServerError> {
        let listener = TcpListener::bind(format!("{}:{}", LISTENER_IP, port))
            .map_err(|_| ServerError::StreamError)?; // el puerto lo tengo que configurar
        info!("Listening at: {}:{}", LISTENER_IP, port);
        logger.info(&format!("Listening at: {}:{}", LISTENER_IP, port));

        let torrents = self.torrents.clone();

        let thread = Some(thread::spawn(move || loop {
            match listener.accept() {
                Ok((stream, addr)) => {
                    handle_connection(
                        stream,
                        addr,
                        logger.clone(),
                        torrents.clone(),
                        download.clone(),
                    );
                }
                Err(_) => continue, //log
            };
        }));
        self.thread = thread;

        Ok(())
    }
}

// impl Drop for Server {
//     fn drop(&mut self) {
//         if let Some(sender) = self.sender.take() {
//             sender.send(ServerMessage::Terminate).unwrap();
//         }
//         if let Some(thread) = self.thread.take() {
//             thread.join().unwrap();
//         }
//     }
// }

/// Establishes the connection with the peer that sent a
/// handshake. Returns `None` if the connection is cut off or if the
/// handshake read is in someway invalid
fn init_connection(stream: std::net::TcpStream) -> Option<(PWPStream, PWPMessage, Vec<u8>)> {
    let mut pwp_stream = PWPStream(stream);
    let handshake = pwp_stream.read_handshake().ok()?;

    let handshake_msg = match handshake {
        PWPMessage::Handshake(h, p) => (PWPMessage::Handshake(h.clone(), p), h),
        _ => return None,
    };

    Some((pwp_stream, handshake_msg.0, handshake_msg.1))
}

/// Handles the interaction with the connected peer
fn handle_connection(
    stream: std::net::TcpStream,
    addr: SocketAddr,
    mut logger: LogHandle,
    torrents: Arc<Mutex<Vec<TorrentFile>>>,
    download: String,
) -> Option<thread::JoinHandle<()>> {
    if let Some((mut pwp_stream, handshake_msg, info_hash)) = init_connection(stream) {
        establish_connection(&mut pwp_stream, addr, handshake_msg, logger.clone());
        let bitfield = match generate_bitfield(&torrents, &info_hash, &mut logger) {
            Some(b) => b,
            None => return None,
        };
        send_bitfield(bitfield, &mut pwp_stream, addr, &mut logger);

        let child = thread::spawn(move || {
            let mut peer_interested = false;
            let mut am_choking = true;

            loop {
                match pwp_stream.read() {
                    Ok(msg) => {
                        match msg {
                            PWPMessage::KeepAlive => {
                                continue;
                            } // Si no recibo mensajes por dos minutos deberia cerrar la conexion
                            PWPMessage::Interested => {
                                peer_interested = true;
                            }
                            PWPMessage::NotInterested => {
                                peer_interested = false;
                            }
                            PWPMessage::Request(index, begin, length) => {
                                make_request(
                                    (am_choking, peer_interested),
                                    torrents.clone(),
                                    info_hash.clone().clone(),
                                    download.clone(),
                                    logger.clone(),
                                    (index, begin, length),
                                    &mut pwp_stream,
                                );
                            }
                            _ => continue,
                        }
                    }

                    Err(crate::pwp::protocol::PWPError::WrongSizeRead) => continue,
                    Err(_) => {
                        error!("Couldn't read from stream");
                        logger.error("Couldn't read from stream");
                        break;
                    }
                }

                if peer_interested && am_choking {
                    am_choking = false;
                    match pwp_stream.send(PWPMessage::Unchoke) {
                        Ok(_) => (),
                        Err(_) => return,
                    };
                }
            }
        });
        Some(child)
    } else {
        None
    }
}

/// Sends handshake to connected peer.
fn establish_connection(
    stream: &mut PWPStream,
    addr: SocketAddr,
    handshake: PWPMessage,
    mut logger: LogHandle,
) -> Option<()> {
    match stream.send(handshake) {
        Ok(_) => {
            info!("Connection established with: {}", addr);
            logger.info(&format!("Connection established with: {}", addr));
            Some(())
        }
        Err(_) => {
            error!("Couldn't establish connection with: {}", addr);
            logger.error(&format!("Couldn't establish connection with: {}", addr));
            None
        }
    }
}

/// Generates the bitfield, according to the pieces the client haves
#[cfg(not(feature = "server-demo"))]
fn generate_bitfield(
    torrents: &Arc<Mutex<Vec<TorrentFile>>>,
    info_hash: &[u8],
    logger: &mut LogHandle,
) -> Option<BitField> {
    if let Ok(t) = torrents.lock() {
        t.iter()
            .find(|t| t.get_info_hash() == *info_hash)
            .map(|t| t.bitfield.clone())
    } else {
        error!("Poisoned Mutex");
        logger.error("Poisoned Mutex");
        None
    }
}

/// This version is used for the demo of the server
#[cfg(feature = "server-demo")]
fn generate_bitfield(
    torrents: &Arc<Mutex<Vec<TorrentFile>>>,
    info_hash: &Vec<u8>,
    logger: &mut LogHandle,
) -> Option<BitField> {
    let mut bitfield;
    let pieces = if let Ok(t) = torrents.lock() {
        t.iter()
            .find(|t| t.get_info_hash() == *info_hash)
            .map(|t| t.pieces_ammount.clone())?
    } else {
        return None;
    };

    bitfield = match BitField::new(pieces) {
        Ok(b) => b,
        Err(_) => {
            error!("Invalid bitfield size");
            logger.error("Invalid bitfield size");
            return None;
        }
    };

    for i in 0..10 {
        bitfield.set_piece(i);
    }

    Some(bitfield)
}

/// Sends bitfield to the connected peer
fn send_bitfield(
    bitfield: BitField,
    stream: &mut PWPStream,
    addr: SocketAddr,
    logger: &mut LogHandle,
) -> Option<()> {
    match stream.send(PWPMessage::Bitfield(bitfield.bits())) {
        Ok(_) => {
            info!("Bitfield sent: to {}", addr);
            logger.info(&format!("Bitfield sent to: {}", addr));
            Some(())
        }
        Err(_) => {
            error!("Couldn't send bitfield to: {}", addr);
            logger.error(&format!("Couldn't send bitfield to: {}", addr));
            None
        }
    }
}

/// Sends the requested block to the peer
fn make_request(
    connection_state: (bool, bool),
    torrents: Arc<Mutex<Vec<TorrentFile>>>,
    info_hash: Vec<u8>,
    download: String,
    mut logger: LogHandle,
    params: (u32, u32, u32),
    stream: &mut PWPStream,
) -> Option<()> {
    let (index, begin, length) = params;
    let (am_choking, peer_interested) = connection_state;
    if !am_choking && peer_interested {
        let buf = block(
            &torrents,
            &info_hash,
            &mut logger,
            &download,
            index,
            begin,
            length,
        )?;
        stream
            .send(PWPMessage::Piece(index, begin, buf))
            .map_err(|_| {
                error!("Couldn't send block");
                logger.error("Couldn't send block");
            })
            .ok()?;
        let filename = filename(&torrents, &info_hash, &mut logger)?;
        // info!(
        //     "Block {} of Piece {} from {} sent",
        //     begin / length,
        //     index,
        //     filename,
        // );
        logger.info(&format!(
            "Block {} of Piece {} from {} sent",
            begin / length,
            index,
            filename,
        ));
    }
    Some(())
}

/// Obtains the torrent filename
fn filename(
    torrents: &Arc<Mutex<Vec<TorrentFile>>>,
    info_hash: &[u8],
    logger: &mut LogHandle,
) -> Option<String> {
    match torrents.lock() {
        Ok(t) => t
            .iter()
            .find(|t| t.get_info_hash() == *info_hash)
            .map(|t| utils::get_info_from_torrentfile(t.metainfo.info.clone()).name),
        Err(_) => {
            error!("Poisoned Mutex");
            logger.error("Poisoned Mutex");
            None
        }
    }
}

/// Obtains the requested block
fn block(
    torrents: &Arc<Mutex<Vec<TorrentFile>>>,
    info_hash: &[u8],
    logger: &mut LogHandle,
    download: &str,
    index: u32,
    begin: u32,
    length: u32,
) -> Option<Vec<u8>> {
    let filename = filename(torrents, info_hash, logger);
    let mut fi = fs::File::open(format!("{}piece{}-{}", download, index, filename?))
        .map_err(|e| {
            error!("{} occurred while reading piece: {}", e, index);
            logger.error(&format!("{} occurred while reading piece: {}", e, index));
        })
        .ok()?;

    fi.seek(SeekFrom::Start(begin as u64))
        .map_err(|e| {
            error!("{} occurred while reading piece: {}", e, index);
            logger.error(&format!("{} occurred while reading piece: {}", e, index));
        })
        .ok()?;

    let mut buf = Vec::with_capacity(length as usize);
    let mut handle = fi.take(length as u64);
    handle
        .read(&mut buf)
        .map_err(|e| {
            error!("{} occurred while writing block: {}", e, index);
            logger.error(&format!("{} occurred while writing block: {}", e, index));
        })
        .ok()?;

    Some(buf)
}

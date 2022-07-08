use std::{
    collections::HashMap,
    fs,
    io::{Read, Seek, SeekFrom},
    net::TcpListener,
    sync::{mpsc, Arc, Mutex, RwLock},
    thread,
    time::{Duration, Instant},
};

use log::{debug, info};

use crate::{
    client::torrent_file::TorrentFile,
    log::logger::LogHandle,
    pwp::{message::PWPMessage, protocol::PWPStream},
    threadpool,
};

use super::server_error::ServerError;

enum ServerMessage {
    Terminate,
}

#[derive(Debug)]
/// Represents a bittorrent client.
pub struct Server {
    thread: Option<thread::JoinHandle<()>>,
    torrents: Arc<Mutex<Vec<TorrentFile>>>,
    sender: Option<mpsc::Sender<ServerMessage>>,
}

impl Server {
    pub fn new(torrents: Arc<Mutex<Vec<TorrentFile>>>) -> Self {
        Self {
            thread: None,
            torrents,
            sender: None,
        }
    }

    pub fn run(
        &mut self,
        port: u16,
        download: String,
        mut logger: LogHandle,
    ) -> Result<(), ServerError> {
        let ip = "0.0.0.0";
        let listener =
            TcpListener::bind(format!("{}:{}", ip, port)).map_err(|_| ServerError::StreamError)?; // el puerto lo tengo que configurar
        info!("Listening at: {}:{}", ip, port);
        logger.info(&format!("Listening at: {}:{}", ip, port));
        let pool = threadpool::ThreadPool::new(5, logger.clone()).unwrap();
        let torrents = self.torrents.clone();
        let connections = Arc::new(RwLock::new(
            self.torrents
                .lock()
                .unwrap()
                .iter()
                .map(|t| (t.get_info_hash(), 0u8))
                .collect::<HashMap<Vec<u8>, u8>>(),
        ));

        let (tx, rx) = mpsc::channel();
        let thread = Some(thread::spawn(move || loop {
            match listener.accept() {
                Ok((stream, addr)) => {
                    if let Some((mut pwp_stream, info_hash)) = init_connection(stream, &connections)
                    {
                        pwp_stream
                            .send(PWPMessage::Handshake(info_hash.clone(), b"rustic".to_vec()))
                            .unwrap();
                        info!("Connection established with: {}", addr);
                        logger.info(&format!("Connection established with: {}", addr));
                        let bitfield = torrents
                            .lock()
                            .unwrap()
                            .iter()
                            .find(|t| t.get_info_hash() == info_hash)
                            .map(|t| t.bitfield.clone())
                            .unwrap();
                        pwp_stream
                            .send(PWPMessage::Bitfield(bitfield.bits()))
                            .unwrap();

                        let connections_clone = Arc::clone(&connections);
                        let torrents_clone = torrents.clone();
                        let mut log_handle = logger.clone();
                        let download_dir = download.clone();
                        pool.spawn(move || {
                            let mut peer_interested = false;
                            let mut am_choking = true;
                            let mut now = Instant::now();
                            loop {
                                match pwp_stream.read().unwrap() {
                                    PWPMessage::KeepAlive => {
                                        now = Instant::now();
                                        continue;
                                    } // Si no recibo mensajes por dos minutos deberia cerrar la conexion
                                    PWPMessage::Interested => {
                                        peer_interested = true;
                                        now = Instant::now();
                                    }
                                    PWPMessage::NotInterested => {
                                        peer_interested = false;
                                        now = Instant::now();
                                    }
                                    PWPMessage::Request(index, begin, length) => {
                                        if !am_choking && peer_interested {
                                            let filename: String = torrents_clone
                                                .lock()
                                                .unwrap()
                                                .iter()
                                                .find(|t| t.get_info_hash() == info_hash)
                                                .map(|t| t.file_name.clone())
                                                .unwrap();
                                            let mut fi = fs::File::open(format!(
                                                "{}Piece{}-{}",
                                                download_dir, index, filename
                                            ))
                                            .unwrap();

                                            fi.seek(SeekFrom::Start(begin as u64)).unwrap();
                                            let mut buf = Vec::with_capacity(length as usize);
                                            let mut handle = fi.take(length as u64);
                                            handle.read(&mut buf).unwrap();

                                            pwp_stream
                                                .send(PWPMessage::Piece(index, begin, buf))
                                                .unwrap();

                                            info!(
                                                "Block {} of Piece {} sent",
                                                begin / length,
                                                index
                                            );
                                            log_handle.info(&format!(
                                                "Block {} of Piece {} sent",
                                                begin / length,
                                                index
                                            ));
                                        }
                                        now = Instant::now();
                                    }
                                    _ => continue,
                                }

                                if peer_interested
                                    && am_choking
                                    && connections_clone.read().unwrap()[&info_hash] < 5
                                {
                                    am_choking = false;
                                    pwp_stream.send(PWPMessage::Unchoke).unwrap();
                                }

                                if now.elapsed() >= Duration::from_secs(300) {
                                    break; //log timeout
                                }
                                let downloaded = torrents_clone
                                    .lock()
                                    .unwrap()
                                    .iter()
                                    .find(|t| t.get_info_hash() == info_hash)
                                    .map(|t| t.bitfield.get_downloaded())
                                    .unwrap();
                                for i in downloaded.iter() {
                                    pwp_stream
                                        .send(PWPMessage::Have((*i).try_into().unwrap()))
                                        .unwrap();
                                }
                            }
                        })
                        .unwrap()
                    } else {
                        continue;
                    }
                }
                Err(_) => continue, //log
            };

            match rx.try_recv().unwrap() {
                ServerMessage::Terminate => break,
            };
        }));
        self.thread = thread;

        self.sender = Some(tx);
        Ok(())
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        if let Some(sender) = self.sender.take() {
            sender.send(ServerMessage::Terminate).unwrap();
        }
        if let Some(thread) = self.thread.take() {
            thread.join().unwrap();
        }
    }
}

fn init_connection(
    stream: std::net::TcpStream,
    connections: &Arc<RwLock<HashMap<Vec<u8>, u8>>>,
) -> Option<(PWPStream, Vec<u8>)> {
    let mut pwp_stream = PWPStream(stream);
    let handshake = pwp_stream.read_handshake().ok()?;
    let info_hash = match handshake {
        PWPMessage::Handshake(h, _) => h,
        _ => return None,
    };

    let associated = !connections
        .read()
        .unwrap()
        .keys()
        .any(|hash| *hash == info_hash);

    if !associated {
        return None;
    } else {
        *connections.write().unwrap().get_mut(&info_hash)? += 1;
        // The key exists
    }
    Some((pwp_stream, info_hash))
}


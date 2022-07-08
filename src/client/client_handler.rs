use log::info;

use crate::client::client_error::ClientError;
use crate::client::client_handler::ClientError::HandlerMessageError;
use crate::client::client_handler::ClientError::RequestNotSentError;
use crate::client::client_handler::ClientError::SentError;
use crate::client::client_handler::ClientError::ThreadError;
use crate::client::client_handler::ClientError::TrackerError;
use crate::client::torrent_file::TorrentFile;
use crate::config;
use crate::download::handler::HandlerDownload;
use crate::log::logger::LogHandle;
use crate::log::logger::Logger;
use crate::server::server_handler::Server;
use crate::storage::piece::Piece;
use crate::storage::store::Store;
use crate::storage::store::StoreClientMessage;
use crate::storage::store::StoreMessage;

use crate::torrent::info::Info;
use crate::tracker::handler::HandlerClientMessage;
use crate::tracker::handler::HandlerClientMessage::ResponseMessage;
use crate::tracker::handler::{Handler, HandlerMessage};
use crate::tracker::request::tracker_request::TrackerRequest;

use crate::tracker::response::tracker_response::TrackerResponseMode::Failure;
use crate::tracker::response::tracker_response::TrackerResponseMode::Response;
use crate::ui::render::LiveViewRawData;
use crate::ui::render::MainViewRawData;
use crate::ui::render::MessagesFromMain;
use crate::ui::render::RawData;
use crate::ui::render::Render;
use crate::ui::render::RequestMessage;
use crate::ui::render::TorrentViewRawData;
use crate::utils;
use std::ffi::OsStr;
use std::fs;
use std::fs::File;

use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::thread::JoinHandle;

#[derive(Debug)]
/// Represents a bittorrent client.
pub struct Client {
    /// List of torrent files requested to downloaded by the client.
    pub torrents: Arc<Mutex<Vec<TorrentFile>>>,
    pub render: Arc<Mutex<Render>>,
}

impl Client {
    /// Creates a new [`Client`] with a vector of [`TorrentFile`].
    pub fn new(render: Arc<Mutex<Render>>, config: &config::Config) -> Result<Self, ClientError> {
        let files = get_files(Path::new(&config.torrents()));
        let mut vec = Vec::<TorrentFile>::with_capacity(files.len());
        for file in &files {
            vec.push(TorrentFile::new(file.clone()).map_err(|_| ClientError::TorrentFileError)?);
        }

        //run(vec, render);

        Ok(Client {
            torrents: Arc::new(Mutex::new(vec)),
            render,
        })
    }

    /// Makes multiple requests concurrently to the traker
    /// and gets the tracker response of each request.
    fn handle_request(
        mut file: &mut TorrentFile,
        tracker_request: TrackerRequest,
        handler: &Handler,
    ) -> Result<(), ClientError> {
        handler
            .send(HandlerMessage::RequestMessage(tracker_request))
            .map_err(|_| ClientError::HandlerError)?;

        // Receive a Sent Message from Handler
        match handler.receive() {
            Ok(m) => {
                match m {
                    ResponseMessage(_) => Err(RequestNotSentError),
                    HandlerClientMessage::Sent => {
                        // Receive a TrackerResponse Message from Handler
                        match handler.receive() {
                            Ok(msg) => match msg {
                                ResponseMessage(response) => match response.0 {
                                    Failure => Err(TrackerError),
                                    Response(response_data) => {
                                        file.response = Some(response_data);
                                        Ok(())
                                    }
                                },
                                HandlerClientMessage::Sent => Err(SentError),
                                HandlerClientMessage::Terminate => Err(ThreadError),
                                _ => Err(TrackerError),
                            },
                            Err(_) => Err(HandlerMessageError),
                        }
                    }
                    HandlerClientMessage::Terminate => Err(ThreadError),
                    _ => Err(TrackerError),
                }
            }
            Err(_) => Err(HandlerMessageError),
        }
    }

    /// Downloads the pieces of the torrent file got by the peers concurrently in disk.
    fn handle_store(
        storage: &Store,
        mut log_handle: LogHandle,
        piece: Piece,
        file: &mut TorrentFile,
    ) -> Result<(), ClientError> {
        storage
            .send(StoreMessage::PieceMessage(piece))
            .map_err(|_| ClientError::StorageError)?;

        match storage.receive() {
            Ok(m) => match m {
                StoreClientMessage::Stored(piece_index, _) => {
                    file.bitfield.set_piece(piece_index as usize);
                    info!("Downloaded piece: {}", piece_index);
                    log_handle.info(&format!("Downloaded piece: {}", piece_index));
                }
                _ => return Err(ClientError::StorageError),
            },
            Err(_) => {
                return Err(ClientError::StorageError);
            }
        };

        Ok(())
    }

    /// Implements the flow of the program.
    /// Interacts with the tracker and the peers to downloaded the pieces of the torrents files.
    /// Also, converts the client to a serve when already has a piece.
    pub fn run(
        torrents: Arc<Mutex<Vec<TorrentFile>>>,
        render: Arc<Mutex<Render>>,
        mut config: config::Config,
    ) -> Result<(JoinHandle<()>, JoinHandle<()>), ClientError> {
        let torrents_clone = Arc::clone(&torrents);
        let data = get_info(
            &torrents_clone.lock().unwrap(),
            crate::ui::render::RequestMessage::MainView,
        );
        render
            .lock()
            .unwrap()
            .send_main(MessagesFromMain::MainViewMsg(MainViewRawData {
                raw_data: data,
            }))
            .unwrap();

        let thread = thread::spawn(move || loop {
            let msg = render.lock().unwrap().receive_main();

            match msg {
                Ok(it) => match it {
                    crate::ui::render::RequestMessage::MainView => {
                        let data = get_info(&torrents_clone.lock().unwrap(), it);
                        render
                            .lock()
                            .unwrap()
                            .send_main(MessagesFromMain::MainViewMsg(MainViewRawData {
                                raw_data: data,
                            }))
                            .unwrap();
                    }
                    crate::ui::render::RequestMessage::TorrentView => {
                        let data = get_info(&torrents_clone.lock().unwrap(), it);

                        render
                            .lock()
                            .unwrap()
                            .send_main(MessagesFromMain::TorrentViewMsg(TorrentViewRawData {
                                raw_data: data,
                            }))
                            .unwrap();
                    }
                    crate::ui::render::RequestMessage::LiveView(_) => {
                        let data = get_info(&torrents_clone.lock().unwrap(), it);
                        render
                            .lock()
                            .unwrap()
                            .send_main(MessagesFromMain::LiveViewMsg(LiveViewRawData {
                                raw_data: data[0].clone(),
                            }))
                            .unwrap();
                    }
                    crate::ui::render::RequestMessage::Terminate => continue,
                },
                Err(_) => continue,
            }
        });

        let torrents_clone = Arc::clone(&torrents);

        let port = config.tcp_port();
        let downloads_dir = config.downloads();
        let logs_dir = config.logs();

        let client_handler = Some(thread::spawn(move || {
            let log = File::create(Path::new(&format!("{}config.log", config.logs())))
                .map_err(|_| ClientError::LoggerError)
                .unwrap();

            let logger = Logger::new(log);
            let log_handle = logger.new_handler();

            //	    let mut server = Server::new(Arc::clone(&torrents_clone));
            //	    let server_thread = server.run(port, log_handle.clone()).unwrap();

            let handler = Handler::new(log_handle);

            let len = torrents_clone.lock().unwrap().len();
            for i in 0..len {
                let torrents_clone = Arc::clone(&torrents);

                let info_hash =
                    utils::hash_info(&(*torrents_clone.lock().unwrap())[i].metainfo.info.bencode());

                let tracker_request = TrackerRequest::new(
                    info_hash,
                    (*torrents_clone.lock().unwrap())[i]
                        .metainfo
                        .announce
                        .clone(),
                    port,
                );

                // -------------------- Logger --------------------
                let Info(mode) = (*torrents_clone.lock().unwrap())[i].metainfo.info.clone();
                let info = match mode {
                    crate::torrent::info::InfoMode::Empty => todo!(),
                    crate::torrent::info::InfoMode::SingleFile(it) => it,
                };

                let log = File::create(Path::new(&format!("{}{}.log", logs_dir, info.name)))
                    .map_err(|_| ClientError::LoggerError)
                    .unwrap();

                let logger = Logger::new(log);
                let logger_handler = logger.new_handler();

                // -------------------- Tracker --------------------

                Client::handle_request(
                    &mut (*torrents_clone.lock().unwrap())[i],
                    tracker_request,
                    &handler,
                )
                .map_err(|_| ClientError::HandlerError)
                .unwrap();

                let mut server = Server::new(Arc::clone(&torrents_clone));
                server
                    .run(port, downloads_dir.clone(), logger_handler.clone())
                    .unwrap();

                let downloads = downloads_dir.clone();
                let inside = thread::spawn(move || {
                    let data = match (*torrents_clone.lock().unwrap())[i].response.clone() {
                        Some(data) => data,
                        None => return,
                    };
                    let Info(mode) = (*torrents_clone.lock().unwrap())[i].metainfo.info.clone();
                    let info = match mode {
                        crate::torrent::info::InfoMode::Empty => todo!(),
                        crate::torrent::info::InfoMode::SingleFile(it) => it,
                    };
                    let handler_download = HandlerDownload::new(
                        info_hash.to_vec(),
                        data.peers,
                        (info.length / info.piece_length) as u32,
                        info.piece_length as u32,
                        logger_handler.clone(),
                        info.name.clone(),
                        info.pieces,
                    );

                    let storage = Store::new(downloads.clone(), logger_handler.clone());
                    loop {
                        match handler_download.receive() {
                            Ok(it) => match it {
                                crate::download::handler::HandlerMessage::Piece(piece) => {
                                    Client::handle_store(
                                        &storage,
                                        logger_handler.clone(),
                                        piece,
                                        &mut (*torrents_clone.lock().unwrap())[i],
                                    )
                                    .map_err(|_| ClientError::StorageError)
                                    .unwrap()
                                }
                                crate::download::handler::HandlerMessage::PeerConnected(peer) => {
                                    (*torrents_clone.lock().unwrap())[i]
                                        .peers_connected
                                        .push(peer)
                                }
                                crate::download::handler::HandlerMessage::Terminate => break,
                                crate::download::handler::HandlerMessage::HaveAllPieces => break,
                                _ => continue,
                            },
                            Err(_) => continue,
                        }
                    }

                    drop(handler_download);
                    //		    drop(server);
                    storage
                        .store_file(
                            info.name.clone(),
                            info.length as u64,
                            (info.length / info.piece_length) as i32,
                            info.piece_length as u64,
                            &downloads,
                        )
                        .map_err(|_| ClientError::StorageError)
                        .unwrap();

                    drop(storage);
                });
                drop(server);
                inside.join().unwrap();
            }

            drop(handler);
        }));

        Ok((client_handler.unwrap(), thread))
    }
}
pub fn get_info(torrents: &[TorrentFile], msg: RequestMessage) -> Vec<RawData> {
    let mut vec = Vec::new();
    match msg {
        RequestMessage::MainView => {
            for t in torrents {
                let Info(mode) = t.metainfo.info.clone();
                let info = match mode {
                    crate::torrent::info::InfoMode::Empty => todo!(),
                    crate::torrent::info::InfoMode::SingleFile(it) => it,
                };
                let mut peers = Vec::new();
                if let Some(res) = t.response.clone() {
                    peers = res.peers.clone();
                };
                let data = RawData::Main {
                    nombre: info.name,
                    hash_de_verificación: utils::hash_info(&t.metainfo.info.bencode()).to_vec(),
                    tamaño_total: info.length as u32,
                    cantidad_de_piezas: (info.length / info.piece_length) as u32,
                    cantidad_de_peers: peers.len() as u32,
                    piezas_faltantes: t.bitfield.get_missing().len() as u32,
                };
                vec.push(data);
            }
            vec
        }
        RequestMessage::TorrentView => {
            for t in torrents {
                let Info(mode) = t.metainfo.info.clone();
                let info = match mode {
                    crate::torrent::info::InfoMode::Empty => todo!(),
                    crate::torrent::info::InfoMode::SingleFile(it) => it,
                };
                let mut peers = Vec::new();
                if let Some(res) = t.response.clone() {
                    peers = res.peers.clone();
                };
                let data = RawData::Torrent {
                    nombre: info.name.clone(),
                    hash_de_verificación: utils::hash_info(&t.metainfo.info.bencode()).to_vec(),
                    tamaño_total: info.length as u32,
                    cantidad_de_piezas: (info.length / info.piece_length) as u32,
                    cantidad_de_peers: peers.len() as u32,
                    piezas_faltantes: t.bitfield.get_missing().len() as u32,
                    cantidad_conexiones_activas: Some(0),
                };
                vec.push(data);
            }
            vec
        }
        RequestMessage::LiveView(id) => {
            let t_clone = torrents.to_owned();
            let torrent: Vec<&TorrentFile> = t_clone
                .iter()
                .filter(|&t| utils::get_info_from_torrentfile(t.metainfo.info.clone()).name == id.0)
                .collect();
            let t = torrent[0];
            let Info(mode) = t.metainfo.info.clone();
            let info = match mode {
                crate::torrent::info::InfoMode::Empty => todo!(),
                crate::torrent::info::InfoMode::SingleFile(it) => it,
            };
            let data = RawData::Live {
                nombre: utils::get_info_from_torrentfile(t.metainfo.info.clone()).name,
                peers_activos: t.peers_connected.clone(),
                velocidad_subida: 0,
                cantidad_de_descargadas: (info.length / info.piece_length) as u32
                    - t.bitfield.get_missing().len() as u32,
                tamanio_pieza: info.piece_length as u32,
            };
            vec.push(data);

            vec
        }
        RequestMessage::Terminate => vec,
    }
}
/// Returns the torrent files entered by the user.
pub fn get_files<P: AsRef<Path>>(path: P) -> Vec<String> {
    let dir = fs::read_dir(path).unwrap();
    dir.into_iter()
        .filter(|r| r.is_ok())
        .map(|r| r.unwrap().path())
        .filter(|p| p.extension() == Some(OsStr::new("torrent")))
        .flat_map(|f| f.to_str().map(|s| s.to_string()))
        .collect()
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     #[test]
//     fn receive_a_torrent_file() {
//         let mut args = Vec::new();
//         args.push("target/debug/bittorrent".trim().to_string());
//         args.push("debian-11.3.0-arm64-netinst.iso.torrent".trim().to_string());

//         let mut files = Vec::new();
//         files.push("debian-11.3.0-arm64-netinst.iso.torrent".trim().to_string());

//         assert_eq!(files, get_files(args));
//     }

//     #[test]
//     fn receive_a_not_torrent_file() {
//         let mut args = Vec::new();
//         args.push("target/debug/bittorrent".trim().to_string());
//         args.push("debian-11.3.0-arm64-netinst.iso".trim().to_string());

//         let files: Vec<String> = Vec::new();

//         assert_eq!(get_files(args), files);
//     }

//     #[test]
//     fn receive_multiple_times_the_same_torrent_file() {
//         let mut args = Vec::new();
//         args.push("target/debug/bittorrent".trim().to_string());
//         args.push("debian-11.3.0-arm64-netinst.iso.torrent".trim().to_string());
//         args.push("debian-11.3.0-arm64-netinst.iso.torrent".trim().to_string());

//         let mut files = Vec::new();
//         files.push("debian-11.3.0-arm64-netinst.iso.torrent".trim().to_string());
//         files.push("debian-11.3.0-arm64-netinst.iso.torrent".trim().to_string());

//         assert_eq!(files, get_files(args));
//     }

//     #[test]
//     fn receive_multiple_torrent_files() {
//         let mut args = Vec::new();
//         args.push("target/debug/bittorrent".trim().to_string());
//         args.push(
//             "kubuntu-16.04.6-desktop-amd64.iso.torrent"
//                 .trim()
//                 .to_string(),
//         );
//         args.push("debian-11.3.0-arm64-netinst.iso.torrent".trim().to_string());

//         let mut files = Vec::new();
//         files.push(
//             "kubuntu-16.04.6-desktop-amd64.iso.torrent"
//                 .trim()
//                 .to_string(),
//         );
//         files.push("debian-11.3.0-arm64-netinst.iso.torrent".trim().to_string());

//         assert_eq!(files, get_files(args));
//     }
// }

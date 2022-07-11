use log::error;

use crate::client::client_error::ClientError;
use crate::client::torrent_file::TorrentFile;
use crate::config;
use crate::download::handler::HandlerDownload;

use crate::log::logger::LogHandle;
use crate::log::logger::Logger;
use crate::peer::peer_handler::Peer;
use crate::server::server_handler::Server;

use crate::tracker::handler::Handler;
use crate::tracker::request::tracker_request::TrackerRequest;

use crate::tracker::response::tracker_response::TrackerResponse;
use crate::tracker::response::tracker_response::TrackerResponseMode::Failure;
use crate::tracker::response::tracker_response::TrackerResponseMode::Response;
use crate::ui::render::LiveViewRawData;
use crate::ui::render::MainViewRawData;
use crate::ui::render::MessagesFromMain;
use crate::ui::render::RawData;
use crate::ui::render::Render;
use crate::ui::render::RequestMessage;
use crate::ui::render::TorrentId;
use crate::ui::render::TorrentViewRawData;
use crate::utils;
use std::ffi::OsStr;
use std::fs;

use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::path::Path;
use std::sync::mpsc::TryRecvError;
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
        let files = get_files(Path::new(&config.torrents())).ok_or(ClientError::FileError)?;
        let mut vec = Vec::<TorrentFile>::with_capacity(files.len());
        for file in files {
            vec.push(TorrentFile::new(file).map_err(|_| ClientError::TorrentFileError)?);
        }

        Ok(Client {
            torrents: Arc::new(Mutex::new(vec)),
            render,
        })
    }

    /// Implements the flow of the program.
    /// Interacts with the tracker and the peers to downloaded the pieces of the torrents files.
    /// Also, converts the client to a serve when already has a piece.
    pub fn run(
        torrents: Arc<Mutex<Vec<TorrentFile>>>,
        render: Arc<Mutex<Render>>,
        config: config::Config,
        logger: &Logger,
    ) -> Result<(JoinHandle<()>, JoinHandle<()>), ClientError> {
        //Listener ui
        let mut log_handle = logger.new_handler();
        let thread_listener_ui = listener_ui(Arc::clone(&torrents), render, log_handle.clone())
            .ok_or(ClientError::ThreadError)?;

        //Config

        //Thread cliente
        let client_handler = thread::spawn(move || {
            match start_client(config, log_handle.clone(), torrents) {
                Some(_) => (),
                None => {
                    error!("An error ocurred while downloading the torrents, closing client...");
                    log_handle.error(
                        "An error ocurred while downloading the torrents, closing client...",
                    );
                }
            };
        });
        Ok((client_handler, thread_listener_ui))
    }
}

/// Starts the main client thread, starting both the download and
/// serving processes
fn start_client(
    config: config::Config,
    logger: LogHandle,
    torrents: Arc<Mutex<Vec<TorrentFile>>>,
) -> Option<()> {
    //Server
    let mut server = Server::new(Arc::clone(&torrents));
    server
        .run(config.tcp_port(), config.logs(), logger.clone())
        .ok()?;
    download_torrents(torrents, config, logger)
}

/// Genrates a thread for each torrent, where the client will interact
/// with the associated peers
fn download_torrents(
    torrents: Arc<Mutex<Vec<TorrentFile>>>,
    config: config::Config,
    mut logger: LogHandle,
) -> Option<()> {
    let mut lock = match torrents.lock() {
        Ok(t) => t,
        Err(_) => {
            error!("Poisoned Mutex");
            logger.error("Poisoned Mutex");
            return None;
        }
    };
    let mut connected = false;
    for (i, torrent) in lock.iter_mut().enumerate() {
        let info_hash = utils::hash_info(&torrent.metainfo.info.bencode());
        let request = match handle_tracker(torrent, info_hash, &config, &logger) {
            Some(r) => r,
            None => continue,
        };

        let response = match torrent.response.as_mut() {
            Some(r) => r,
            None => continue,
        };

        if cfg!(feature = "server-demo") {
            let peer = Peer {
                peer_id: Some(request.peer_id),
                ip: Some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
                port: config.tcp_port(),
            };
            response.peers.push(peer);
        }
        HandlerDownload::new(
            logger.clone(),
            Arc::clone(&torrents),
            i,
            config.downloads(),
            torrent.clone(),
        );

        connected |= true;
    }

    if !connected {
        error!("Couldn't connect to any torrent");
        logger.error("Couldn't connect to any torrent");
        None
    } else {
        Some(()) // Could connect to at least one torrent
    }
}

/// Spawns a thread that will serve as a intermediary between the
/// client and the UI
pub fn listener_ui(
    torrents: Arc<Mutex<Vec<TorrentFile>>>,
    render: Arc<Mutex<Render>>,
    mut logger: LogHandle,
) -> Option<JoinHandle<()>> {
    let raw_data = get_info_wrapper(RequestMessage::MainView, &torrents, &mut logger)?;
    send_data_to_view(
        MessagesFromMain::MainViewMsg(MainViewRawData { raw_data }),
        &render,
        &mut logger,
    )?;

    let mut log_handle = logger.clone();
    Some(thread::spawn(move || loop {
        let msg = match render.lock() {
            Ok(r) => match r.receive_main() {
                Ok(m) => m,
                Err(TryRecvError::Empty) => continue,
                Err(TryRecvError::Disconnected) => break,
            },
            Err(_) => {
                error!("Poisoned Mutex");
                log_handle.error("Poisoned Mutex");
                break;
            }
        };
        handle_ui_msg(
            msg,
            Arc::clone(&torrents),
            Arc::clone(&render),
            log_handle.clone(),
        );
    }))
}

/// Gets the necessary information for the UI from the client
pub fn get_info(torrents: &[TorrentFile], msg: RequestMessage) -> Vec<RawData> {
    match msg {
        RequestMessage::MainView => main_view_msg(torrents),
        RequestMessage::TorrentView => torrent_view_msg(torrents),
        RequestMessage::LiveView(id) => live_view_msg(torrents, id),
        RequestMessage::Terminate => Vec::new(),
    }
}

/// Get information for the main view
fn main_view_msg(torrents: &[TorrentFile]) -> Vec<RawData> {
    let mut vec = Vec::new();
    for t in torrents {
        let info = t.metainfo.info().unwrap();
        let mut peers = Vec::new();
        if let Some(res) = t.response.clone() {
            peers = res.peers.clone();
        };
        let data = RawData::Main {
            name: info.name,
            authentication_hash: utils::hash_info(&t.metainfo.info.bencode()).to_vec(),
            total_size: info.length as u32,
            number_of_pieces: (info.length / info.piece_length) as u32,
            number_of_peers: peers.len() as u32,
            remaining_pieces: t.bitfield.get_missing().len() as u32,
        };
        vec.push(data);
    }
    vec
}
/// Get information for the torrent view
fn torrent_view_msg(torrents: &[TorrentFile]) -> Vec<RawData> {
    let mut vec = Vec::new();
    for t in torrents {
        let info = t.metainfo.info().unwrap();
        let mut peers = Vec::new();
        if let Some(res) = t.response.clone() {
            peers = res.peers.clone();
        };
        let data = RawData::Torrent {
            name: info.name.clone(),
            authentication_hash: utils::hash_info(&t.metainfo.info.bencode()).to_vec(),
            total_size: info.length as u32,
            number_of_pieces: (info.length / info.piece_length) as u32,
            number_of_peers: peers.len() as u32,
            remaining_pieces: t.bitfield.get_missing().len() as u32,
            active_connections: t.peers_connected.len(),
        };
        vec.push(data);
    }
    vec
}
/// Get information for the live view
fn live_view_msg(torrents: &[TorrentFile], id: TorrentId) -> Vec<RawData> {
    let mut vec = Vec::new();
    let t_clone = torrents.to_owned();
    let torrent: Vec<&TorrentFile> = t_clone
        .iter()
        .filter(|&t| utils::get_info_from_torrentfile(t.metainfo.info.clone()).name == id.0)
        .collect();
    let t = torrent[0];
    let info = t.metainfo.info().unwrap();
    let data = RawData::Live {
        name: utils::get_info_from_torrentfile(t.metainfo.info.clone()).name,
        active_peers: t.peers_connected.clone(),
        upload_speed: 0,
        downloaded_files: (info.length / info.piece_length) as u32
            - t.bitfield.get_missing().len() as u32,
        piece_size: info.piece_length as u32,
    };
    vec.push(data);

    vec
}

/// Returns the torrents in the folder specified in path. Returns
/// `None` if the path doesn't correspond to a directory or if there
/// isn't any torrent in the directory
pub fn get_files<P: AsRef<Path>>(path: P) -> Option<Vec<String>> {
    let dir = fs::read_dir(path).ok()?;
    Some(
        dir.into_iter()
            .filter(|r| r.is_ok())
            .map(|r| r.unwrap().path())
            .filter(|p| p.extension() == Some(OsStr::new("torrent")))
            .flat_map(|f| f.to_str().map(|s| s.to_string()))
            .collect(),
    )
}

/// Interaction with the tracker
fn handle_tracker(
    torrent: &mut TorrentFile,
    info_hash: [u8; 20],
    config: &config::Config,
    logger: &LogHandle,
) -> Option<TrackerRequest> {
    let mut tracker_request = TrackerRequest::new(
        info_hash,
        torrent.metainfo.announce.clone(),
        config.tcp_port(),
    );

    let handler = Handler::new(logger.clone(), &mut tracker_request);
    handler
        .tracker_response
        .as_ref()
        .and_then(|TrackerResponse(r)| match r {
            Failure => None,
            Response(response_data) => {
                torrent.response = Some(response_data.clone());
                Some(tracker_request)
            }
        })
}

/// Wrapper over get_info() that adds logging
fn get_info_wrapper(
    msg: RequestMessage,
    torrents: &Arc<Mutex<Vec<TorrentFile>>>,
    logger: &mut LogHandle,
) -> Option<Vec<RawData>> {
    Some(get_info(
        &torrents
            .lock()
            .map_err(|e| {
                error!("Poisoned Mutex");
                logger.error("Poisoned Mutex");
                e
            })
            .ok()?,
        msg,
    ))
}

/// Sends data from the client to the ui
fn send_data_to_view(
    msg: MessagesFromMain,
    render: &Arc<Mutex<Render>>,
    logger: &mut LogHandle,
) -> Option<()> {
    match render.lock() {
        Ok(r) => r.send_main(msg).ok(),
        Err(_) => {
            error!("Poisoned Mutex");
            logger.error("Poisoned Mutex");
            None
        }
    }
}

/// Acts according to the message sent from the ui
fn handle_ui_msg(
    m: RequestMessage,
    torrents: Arc<Mutex<Vec<TorrentFile>>>,
    render: Arc<Mutex<Render>>,
    mut logger: LogHandle,
) -> Option<()> {
    match m {
        RequestMessage::MainView => {
            let raw_data = get_info_wrapper(m, &torrents, &mut logger)?;
            let msg = MessagesFromMain::MainViewMsg(MainViewRawData { raw_data });
            send_data_to_view(msg, &render, &mut logger)
        }

        RequestMessage::TorrentView => {
            let raw_data = get_info_wrapper(m, &torrents, &mut logger)?;
            let msg = MessagesFromMain::TorrentViewMsg(TorrentViewRawData { raw_data });

            send_data_to_view(msg, &render, &mut logger)
        }
        RequestMessage::LiveView(_) => {
            let raw_data = get_info_wrapper(m, &torrents, &mut logger)?;
            let msg = MessagesFromMain::LiveViewMsg(LiveViewRawData {
                raw_data: raw_data[0].clone(),
            });
            send_data_to_view(msg, &render, &mut logger)
        }
        RequestMessage::Terminate => None,
    }
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

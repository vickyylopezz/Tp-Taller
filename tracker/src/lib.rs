pub mod http;

mod data;
mod interaction;
mod response_error;
mod threadpool;
mod torrent;
use std::collections::HashMap;

use std::fs::File;
use std::io::{BufWriter, Read, Write};

use std::net::TcpStream;
use std::str::from_utf8;
use std::sync::{Arc, Mutex};

use bittorrent::tracker::url_encoder::encoder::URLEncoded;
use chrono::Local;
use data::Data;
use http::builder::APIBuilder;
use http::server;

use torrent::{binary_mode, PeerTracker, Torrent};

use crate::http::endpoint::Endpoint;
use crate::http::error;
use crate::http::resource::Resource;
use crate::http::response::{Response, StatusCode};
use log::{debug, error};

#[derive(Debug, Clone)]
pub struct Tracker {
    torrents: Arc<Mutex<Data>>,
}

impl Tracker {
    pub fn new() -> Self {
        Self {
            torrents: Arc::new(Mutex::new(Data::new())),
        }
    }

    fn torrents(&self) -> Option<Vec<Torrent>> {
        match self.torrents.lock() {
            Ok(mut l) => Some(l.torrents().to_vec()),
            Err(_) => {
                error!("Poisoned Mutex");
                None
            }
        }
    }

    fn push(&mut self, torrent: Torrent) {
        match self.torrents.lock() {
            Ok(mut l) => l.push(torrent),
            Err(_) => {
                error!("Poisoned Mutex");
                // TODO: Handle
            }
        }
    }

    fn update_peers(
        &mut self,
        peer_id: &str,
        ip: &str,
        port: &str,
        parameters: HashMap<&str, &str>,
        pos: usize,
    ){
        if let Ok(mut l) = self.torrents.lock() {
            torrent::set_peer(peer_id, ip, port, parameters, l.torrents()[pos].peers());
        }
    }

    fn find(&self, info_hash: &str) -> Option<usize> {
        self.torrents()
            .iter()
            .flatten()
            .position(|r| r.info_hash == info_hash)
    }

    fn handle_response(&mut self, buf: &[u8], stream: &mut TcpStream) -> Option<Torrent> {
        // TODO: Extract
        let msg = from_utf8(buf).ok()?;
        let request: Vec<&str> = msg.split(' ').collect();
        let url: Vec<&str> = request[1].split("/announce?").collect::<Vec<&str>>()[1]
            .split('&')
            .collect();

        let parameters = make_parameters(url);
        let info_hash = make_info_hash(&parameters)?;
        let len = self.torrents()?.len();
        let torrent_pos = self.find(&info_hash).unwrap_or(len);
        if torrent_pos == len {
            self.push(Torrent {
                info_hash,
                upload_date: Local::now().to_string(),
                peers: Vec::new(),
            });
        }
        let compact = parameters.get("compact").unwrap_or(&"0");
        let numwant = parameters.get("numwant").unwrap_or(&"50");

        self.update_peers(
            parameters.get("peer_id")?,
            parameters.get("ip")?,
            parameters.get("port")?,
            parameters.clone(),
            torrent_pos,
        );

        let torrent = self.torrents()?[torrent_pos].clone(); 

        let peers = torrent
            .clone()
            .peers
            .into_iter()
            .filter(|p| {p.ip != *parameters.get("ip").unwrap()
                    || p.port != parameters.get("port").unwrap().parse::<u16>().unwrap()
            })
            .collect();
    
        send_response(stream, peers, compact.to_string(), numwant.to_string());

        Some(torrent)
    }
}

fn make_info_hash(parameters: &HashMap<&str, &str>) -> Option<String> {
    let mut info_hash = String::new();
    for h in URLEncoded(parameters["info_hash"].to_string()).decode()? {
        info_hash.push_str(&format!("{:x}", h));
    }
    Some(info_hash)
}

fn make_parameters(url: Vec<&str>) -> HashMap<&str, &str> {
    let mut parameters = HashMap::new();
    for u in url {
        let parameter = u.split('=').collect::<Vec<&str>>();
        parameters.insert(parameter[0], parameter[1]);
    }
    parameters
}

impl<'a> server::Handler for Tracker {
    fn handle_connection(&mut self, mut stream: TcpStream, endpoints: &APIBuilder<'static>) {
        let mut buf = [0; 2048];
        match stream.read(&mut buf) {
            Ok(_) => {}
            Err(_) => {
                error!("An error ocurred while handling the connection");
                return;
            }
        };
        //GET /announce?{querystring} HTTP/1.1\r\nHost: {}\r\n\r\n

        const GET_TRACKER: &[u8] = b"GET /announce?";

        if buf.starts_with(GET_TRACKER) {
            self.handle_response(&buf, &mut stream);
            return;
        }

        let parsed = match parse_endpoint(&buf) {
            Some(s) => s,
            None => {
                debug!("{}", from_utf8(&buf).unwrap());
                error!("Invalid endpoint");
                return;
            }
        };

        let r = match endpoints.get(Endpoint::new(&parsed)) {
            Ok(c) => {
                let torrents = match self.torrents() {
                    Some(t) => t,
                    None => return,
                };
                create_metadata(&torrents);
                Response::new(StatusCode::Code200)
                    .content_length(c.len())
                    .content(c)
                    .response()
            }
            Err(e) => match e {
                error::ApiError::ResourceNotFound(c, _) => Response::new(StatusCode::Code404)
                    .content_length(c.len())
                    .content(c)
                    .response(),
                error::ApiError::InvalidResourceRead(_) => todo!(), //log
            },
        };
        write_to_stream(&mut stream, &r);
    }
}

impl Default for Tracker {
    fn default() -> Self {
        Self::new()
    }
}

fn create_metadata(torrents: &[Torrent]) {
    File::create("../web/metadata.json").map_or_else(
        |e| error!("Error creating file: {}", e),
        |f| match serde_json::to_writer(BufWriter::new(f), torrents) {
            Ok(_) => {}
            Err(e) => {
                error!("Error writing json: {}", e)
            }
        },
    );
}

fn parse_endpoint(buf: &[u8]) -> Option<String> {
    from_utf8(buf)
        .ok()
        .and_then(|s| s.split(char::is_whitespace).find(|s| s.contains('/')))
        .map(|s| s.to_string())
}

pub fn build_endpoints<'a>() -> APIBuilder<'a> {
    let mut api = APIBuilder::new();

    const ROOT: &str = "/";
    const ANNOUNCE: &str = "/announce";
    const STATS: &str = "/stats";
    const ANNOUNCE_LOGIC: &str = "/announce.js";
    const CONNECTED_PEERS_LOGIC: &str = "/connected_peers.js";
    const TORRENTS_AMMOUNT_LOGIC: &str = "/torrents_ammount.js";
    const COMPLETED_PEERS_LOGIC: &str = "/completed_peers.js";
    const JSON: &str = "/prueba.json";

    api.add(Endpoint::new(ROOT), Resource::new("../web/home/home.html"));
    api.add(
        Endpoint::new(ANNOUNCE),
        Resource::new("../web/announce/announce.html"),
    );
    api.add(
        Endpoint::new(STATS),
        Resource::new("../web/stats/stats.html"),
    );
    api.add(
        Endpoint::new(ANNOUNCE_LOGIC),
        Resource::new("../web/announce/announce.js"),
    );
    api.add(
        Endpoint::new(CONNECTED_PEERS_LOGIC),
        Resource::new("../web/stats/connected_peers.js"),
    );
    api.add(
        Endpoint::new(TORRENTS_AMMOUNT_LOGIC),
        Resource::new("../web/stats/torrents_ammount.js"),
    );
    api.add(
        Endpoint::new(COMPLETED_PEERS_LOGIC),
        Resource::new("../web/stats/completed_peers.js"),
    );

    api.add(Endpoint::new(JSON), Resource::new("../web/prueba.json"));

    api.not_found(Resource::new("../web/404.html"));
    api
}

/// Creates and sends the response of the tracker.
fn send_response(
    stream: &mut TcpStream,
    peers: Vec<PeerTracker>,
    compact: String,
    numwant: String,
) {
    let status_line = "HTTP/1.1 200 OK";
    let content_type = "Content-Type: text/plain";
    let peers = torrent::get_peers(peers, numwant);
    let mut contents = if compact == "0" {
        torrent::dictionary_mode(peers)
    } else {
        binary_mode(peers)
    };

    let response = format!(
        "{}\r\n{}\r\nContent-Length: {}\r\n\r\n",
        status_line,
        content_type,
        contents.len(),
    );

    let mut response = response.as_bytes().to_vec();
    response.append(&mut contents);
    write_to_stream(stream, &response)
}

fn write_to_stream(stream: &mut TcpStream, data: &[u8]) {
    match stream.write_all(data) {
        Ok(_) => {}
        Err(_) => error!("Something went wrong writing to the stream"),
    };
    match stream.flush() {
        Ok(_) => {}
        Err(_) => error!("Something went wrong flushing the stream"),
    }
}
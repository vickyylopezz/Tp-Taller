use super::response::tracker_response::TrackerResponseMode;
use crate::bencode::parser;
use crate::peer::Peer;
use crate::torrent::metainfo::Metainfo;
use crate::tracker::request::tracker_request::TrackerRequest;
use crate::tracker::response::tracker_response::TrackerResponse;
use crate::tracker::tracker_handler_error::TrackerHandlerError;

use native_tls::TlsConnector;
use sha1::{Digest, Sha1};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::str;

/// Contains the interactions with the tracker and the data needed to interact with it.
#[derive(Debug, PartialEq, Eq)]
pub struct TrackerHandler<'a> {
    /// Request established with the tracker
    tracker_request: TrackerRequest,
    /// Response granted by the tracker
    tracker_response: TrackerResponse,
    /// Metainfo placeholder
    torrent: &'a Metainfo,
    /// Placeholder of the data granted by the tracker response
    response: Vec<u8>,
}

impl<'a> TrackerHandler<'a> {
    /// Creates a new TrackerResponse structure.
    pub fn new(torrent: &'a Metainfo) -> Self {
        Self {
            tracker_request: { TrackerRequest::new(hash_info(torrent.info.bencode())) },
            tracker_response: TrackerResponse(TrackerResponseMode::Failure),
            response: Vec::new(),
            torrent,
        }
    }

    /// Helper function for setting the request address. Returns [`TrackerHandlerError`]
    /// if there is an error while setting the ip
    fn set_request_addr(&mut self, stream: &TcpStream) -> Result<(), TrackerHandlerError> {
        let local_addr = stream.local_addr();
        let ip = match local_addr {
            Ok(it) => it.ip(),
            Err(_) => return Err(TrackerHandlerError::InvalidAdress),
        };

        self.tracker_request.set_addr(ip);
        Ok(())
    }

    fn parse_torrent_host<'b>(&self, host: &'b mut String) -> Option<&'b String> {
        if host.contains("https://") {
            *host = host.strip_prefix("https://").unwrap().to_string();
        } else if host.contains("http://") {
            *host = host.strip_prefix("http://").unwrap().to_string();
        } else {
            return None;
        }

        *host = host.strip_suffix("/announce").unwrap().to_string();
        let tls: Vec<&str> = host.split(':').collect();
        *host = tls[0].to_string();
        self.add_port(host);

        Some(host)
    }

    fn add_port<'c>(&self, host: &'c mut String) -> Option<&'c String> {
        if !host.contains(':') {
            host.push_str(":443");
        }

        Some(host)
    }

    /// Main flow for establishing the request with the tracker. Returns [`TrackerHandlerError`]
    /// if there is an error while doing the request.
    fn request(&mut self) -> Result<(), TrackerHandlerError> {
        let connector = TlsConnector::new().map_err(|_| TrackerHandlerError::InvalidTlsConnector);
        let mut announce = self.torrent.announce.clone();

        let host = match self.parse_torrent_host(&mut announce) {
            Some(it) => it,
            None => return Err(TrackerHandlerError::RequestError),
        };
        let stream = TcpStream::connect(host).unwrap();
        if self.set_request_addr(&stream).is_err() {
            return Err(TrackerHandlerError::InvalidAdress);
        }

        let tls: Vec<&str> = host.split(':').collect();
        let mut stream = connector.unwrap().connect(tls[0], stream).unwrap();

        let path = format!(
            "/announce{}",
            self.tracker_request
                .generate_querystring()
                .map_err(|_| TrackerHandlerError::InvalidQuerystring)?
                .get_querystring()
        );

        let header = format!("GET {} HTTP/1.0\r\nHost: {}\r\n\r\n", path, host);
        //println!("{}", header);
        println!("Tracker Request done");
        stream.write_all(header.as_bytes()).unwrap();
        let mut res = vec![];
        stream.read_to_end(&mut res).unwrap();

        let index = find_payload_index(&res).unwrap();
        self.response = res[index + 4..].to_vec();
        //println!("RESPONSE: {}", String::from_utf8_lossy(&self.response));
        Ok(())
    }

    /// Desencodes the bencoded answer and sets it to its tracker_response atribute.
    fn response(&mut self) -> Result<(), TrackerHandlerError> {
        let bencoded_dictionary = parser::parse(self.response.clone()).unwrap();
        self.tracker_response = TrackerResponse::new(bencoded_dictionary)
            .map_err(|_| TrackerHandlerError::ResponseError)?;

        Ok(())
    }

    /// Manages the interaction with the tracker.
    pub fn manage_interaction(&mut self) -> Result<(), TrackerHandlerError> {
        match self.request() {
            Ok(_) => match self.response() {
                Ok(_) => {
                    let response = match &self.tracker_response.0 {
                        TrackerResponseMode::Failure => todo!(),
                        TrackerResponseMode::Response(it) => it,
                    };
                    println!(
                        "Tracker Response recieved. There are {} peers",
                        response.peers.len()
                    );
                    Ok(())
                }
                Err(_) => Err(TrackerHandlerError::ResponseError),
            },
            Err(_) => Err(TrackerHandlerError::InteractionError),
        }
    }

    // Returns the list of peers of the response of the tracker
    pub fn get_peers(&self) -> Result<&Vec<Peer>, TrackerHandlerError> {
        match &self.tracker_response.0 {
            TrackerResponseMode::Failure => Err(TrackerHandlerError::ResponseError),
            TrackerResponseMode::Response(it) => Ok(&it.peers),
        }
    }

    // Dummy function made in order to test the creation of a handler
    pub fn is_ok(&self) -> bool {
        true
    }
}

/// Function made in order to hash an u8 vec.
/// It returns a 20-byte SHA1 hash
pub fn hash_info(buf: Vec<u8>) -> [u8; 20] {
    let mut hasher = Sha1::new();
    hasher.update(buf);
    hasher.finalize()[0..20].try_into().unwrap()
}

fn find_payload_index(buffer: &[u8]) -> Option<usize> {
    buffer
        .windows(4)
        .enumerate()
        .find(|(_, w)| matches!(*w, b"\r\n\r\n"))
        .map(|(i, _)| i)
}

#[cfg(test)]
mod tests {
    use crate::torrent::metainfo;

    use super::*;

    #[test]
    fn test_connection() {
        let metainfo: &[u8] = b"d8:announce3:url13:announce-listl1:a2:ab3:abce7:comment7:comment\
			 10:created by2:me13:creation datei0e8:encoding4:utf84:infod6:length\
			 i0e6:md5sum4:aaaa4:name4:file12:piece lengthi0e6:pieces5:aaaaa7:privatei1eee";

        let got = metainfo::read_torrent(metainfo).unwrap();

        let tracker_handler = TrackerHandler::new(&got);
        assert_eq!(tracker_handler.is_ok(), true);
    }
}

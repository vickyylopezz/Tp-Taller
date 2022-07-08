use crate::tracker::request::querystring::Querystring;
use crate::tracker::request::tracker_request_error::TrackerRequestError;
use crate::tracker::request::tracker_request_event::TrackerRequestEvent;
use crate::tracker::url_encoder::encoder::URLEncoded;
use log::debug;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::fmt::Write;
use std::net::{IpAddr, TcpStream};

/// This type is scoped on the handling of the request of the tracker.
#[derive(Debug, PartialEq, Eq)]
pub struct TrackerRequest {
    /// Describes the file(s) of the torrent. There are two.
    /// This value is going to be hashed with sha1.
    info_hash: [u8; 20],
    /// Announce URL of the tracker
    pub announce: String,
    /// Peers's unique identificator.
    pub peer_id: [u8; 20],
    /// Client's port.
    port: u16,
    /// The total amount uploaded since the client sent the 'started' event to the tracker
    uploaded: u64,
    /// The total amount downloaded since the client sent the 'started' event to the tracker
    downloaded: u64,
    /// The number of bytes the client still has to download
    left: u64,
    /// Ip of the client
    ip: Option<IpAddr>,
    /// Possible status of the request (started, stopped, completed)
    event: TrackerRequestEvent,
    /// Indicates if the client accepts a compact response
    compact: u8,
}

impl TrackerRequest {
    /// Creates a new TrackerRequest structure.
    pub fn new(info: [u8; 20], announce: String, port: u16) -> Self {
        Self {
            info_hash: info,
            peer_id: {
                let random_chars: Vec<u8> = (&mut thread_rng())
                    .sample_iter(Alphanumeric)
                    .take(12)
                    .collect();
                ["-AZ2060-".as_bytes(), &random_chars[..]].concat()[0..20]
                    .try_into()
                    .unwrap()
            },
            port,
            ip: None,
            uploaded: 0,
            downloaded: 0,
            left: 0,
            event: TrackerRequestEvent::Started,
            announce,
            compact: 0,
        }
    }

    /// Establish a TCP conection with the first port in the range [6881, 6889]
    /// and sets its port number with that port.
    fn set_port(&mut self, ip: IpAddr) {
        let mut current_port: u16 = 6881;
        loop {
            if TcpStream::connect((ip, current_port)).is_ok() {
                self.port = current_port;
            }

            if current_port == 6889 {
                break;
            }

            current_port += 1;
        }
    }

    /// Sets the ip address direction of the client.    
    pub fn set_addr(&mut self, ip: IpAddr) {
        self.set_port(ip);
        self.ip = Some(ip);
    }

    /// Generates the querystring needed to do the request to the tracker.
    pub fn generate_querystring(&self) -> Result<Querystring, TrackerRequestError> {
        let mut querystring = "?".to_string();
        querystring.push_str("info_hash=");

        let mut url;

        querystring.push_str(match URLEncoded::new().urlencode(&self.info_hash) {
            Ok(it) => {
                url = it.get_url();
                &url
            }
            Err(_) => return Err(TrackerRequestError::EncoderError),
        });

        querystring.push_str("&peer_id=");
        querystring.push_str(match URLEncoded::new().urlencode(&self.peer_id) {
            Ok(it) => {
                url = it.get_url();
                &url
            }
            Err(_) => return Err(TrackerRequestError::EncoderError),
        });
        if let Some(ip) = &self.ip {
            write!(querystring, "&ip={}", ip).map_err(|_| TrackerRequestError::WriteError)?;
        }
        // write!(querystring, "&port={}", self.port).map_err(|_| TrackerRequestError::WriteError)?;
        write!(querystring, "&port={}", self.port).map_err(|_| TrackerRequestError::WriteError)?;
        write!(querystring, "&downloaded={}", self.downloaded)
            .map_err(|_| TrackerRequestError::WriteError)?;

        write!(querystring, "&uploaded={}", self.uploaded)
            .map_err(|_| TrackerRequestError::WriteError)?;
        write!(querystring, "&left={}", self.left).map_err(|_| TrackerRequestError::WriteError)?;
        write!(
            querystring,
            "&event={}",
            match self.event {
                TrackerRequestEvent::Started => "started",
                TrackerRequestEvent::Completed => "completed",
                TrackerRequestEvent::Stopped => "stopped",
            }
        )
        .map_err(|_| TrackerRequestError::WriteError)?;
        //write!(querystring, "&compact={}", self.compact).map_err(|_| TrackerRequestError::WriteError)?;

        Ok(Querystring(querystring))
    }
}

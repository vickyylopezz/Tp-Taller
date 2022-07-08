use crate::tracker::response::tracker_response_builder::TrackerResponseBuilder;
use crate::tracker::response::tracker_response_error::TrackerResponseError;
use crate::{bencode::bencoded_value::BencodedValue, peer::peer_handler::Peer};

use std::collections::HashMap;

// static RESPONSE_REQUIRED_KEYS: [&[u8]; 4] = [b"interval", b"complete", b"incomplete", b"peers"];
static RESPONSE_REQUIRED_KEYS: [&[u8]; 2] = [b"interval", b"peers"];

/// Represents the possible variants of the response of the tracker.
#[derive(Debug, PartialEq, Eq)]
pub enum TrackerResponseMode {
    /// Placeholder.
    Failure,
    /// Response data mode.
    Response(ResponseData),
}

/// Wrapper over the the [`TrackerResponseMode`] enum.
#[derive(Debug, PartialEq, Eq)]
pub struct TrackerResponse(pub TrackerResponseMode);

/// Contains the data from the response of the tracker.
/// Every key is required.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ResponseData {
    /// Interval in seconds that the client should wait between sending regular requests to the tracker.
    pub interval: i64,
    /// Number of peers with the entire file.
    pub complete: i64,
    /// Number of non-seeder peers.
    pub incomplete: i64,
    /// List of the file requesting peers.
    pub peers: Vec<Peer>,
    /// Minimum announce interval.
    pub min_interval: Option<i64>,
}

impl TrackerResponse {
    /// Creates a new TrackerResponse structure from a bencoded dictionary.
    /// Returns [`Some`] if no errors occur while building the instance; otherwise returns [`None`].
    pub fn new(
        bencoded_value: BencodedValue,
        peer_id: [u8; 20],
    ) -> Result<Self, TrackerResponseError> {
        let dict = bencoded_value
            .dictionary()
            .ok_or(TrackerResponseError::InvalidResponse)?
            .into_iter()
            .collect::<HashMap<BencodedValue, BencodedValue>>();
        let mut required = RESPONSE_REQUIRED_KEYS
            .iter()
            .map(|v| BencodedValue::ByteString(v.to_vec()));
        let has_requiered = required.all(|k| dict.contains_key(&k));

        if has_requiered {
            let mut tracker_response = TrackerResponseBuilder::new();
            for (k, v) in dict {
                if let BencodedValue::ByteString(s) = k {
                    build_response_fields(&mut tracker_response, &s[..], v, peer_id);
                } else {
                    return Err(TrackerResponseError::InvalidResponse);
                }
            }
            Ok(tracker_response.build())
        } else {
            Err(TrackerResponseError::InvalidResponse)
        }
    }
}

/// Helper function for building the TrackerResponse struct.
/// Returns [`None`] if there is an error building some of the fields.
fn build_response_fields(
    tracker_response: &mut TrackerResponseBuilder,
    field: &[u8],
    value: BencodedValue,
    peer_id: [u8; 20],
) -> Option<()> {
    match field {
        b"interval" => {
            let interval = value.integer()?;
            tracker_response.interval(interval);
        }

        b"complete" => {
            let complete = value.integer()?;
            tracker_response.complete(complete);
        }

        b"incomplete" => {
            let incomplete = value.integer()?;
            tracker_response.incomplete(incomplete);
        }

        b"peers" => {
            if let BencodedValue::List(list) = value {
                let mut peers = Vec::new();
                for p in list {
                    let dict = p.dictionary()?;
                    let peer = Peer::new_dict(dict, peer_id)?;
                    peers.push(peer);
                }
                tracker_response.peers(peers);
            } else if let BencodedValue::ByteString(b) = value {
                let mut peers: Vec<Peer> = Vec::new();
                let peer_ammount = b.len() / 6;
                let mut i = 0;
                loop {
                    if i >= peer_ammount * 6 {
                        break;
                    }

                    // |a|b|c|d|e|f|
                    // ip --> |a|b|c|d|
                    // port --> |e|f|
                    let ip = &b[i..i + 4];
                    let port = &b[i + 4..i + 6];

                    i += 6;

                    let peer = Peer::new_byte_string(ip, port, peer_id)?;
                    peers.push(peer);
                }
                tracker_response.peers(peers);
            }
        }
        b"min interval" => {
            let min_interval = value.integer()?;
            tracker_response.min_interval(min_interval);
        }
        _ => return None,
    };
    Some(())
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::bencode::parser;

    // #[test]
    // fn normal_response_with_dictionary_mode() {
    //     let response = "d8:intervali1800e8:completei1900e10:incompletei1700e5:peersld2:ip13:192.168.189.14:porti20111eeee".into();
    //     let bencoded_dictionary = parser::parse(response).unwrap();
    //     let tracker_response = TrackerResponse::new(bencoded_dictionary).unwrap();

    //     // let mut vec_peers = Vec::new();
    //     // let peer = Peer {
    //     //     peer_id: None,
    //     //     ip: "192.168.189.1".parse().unwrap(),
    //     //     port: 20111,
    //     // };
    //     // vec_peers.push(peer);
    //     // let response = TrackerResponse(TrackerResponseMode::Response(ResponseData {
    //     //     interval: 1800,
    //     //     complete: 1900,
    //     //     incomplete: 1700,
    //     //     peers: vec_peers,
    //     // }));
    //     // assert_eq!(tracker_response, response);
    // }

    #[test]
    fn response_with_few_keys() {
        let response = "d8:intervali1800e5:peersld2:ip13:192.168.189.14:porti20111eeee".into();
        let bencoded_dictionary = parser::parse(response).unwrap();

        assert_eq!(
            TrackerResponse::new(bencoded_dictionary, [0u8; 20]).unwrap_err(),
            TrackerResponseError::InvalidResponse
        );
    }
}

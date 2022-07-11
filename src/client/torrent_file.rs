use std::fs::File;

use crate::client::bitfield::BitField;
use crate::client::torrent_file_error::TorrentFileError;
use crate::peer::peer_handler::Peer;
use crate::torrent::info::Info;
use crate::torrent::metainfo::{self, Metainfo};
use crate::tracker::response::tracker_response::ResponseData;
use crate::utils;

#[derive(Debug, PartialEq, Eq, Clone)]
/// Represents a torrent file.
pub struct TorrentFile {
    /// Name of the file to be downloaded.
    pub file_name: String,
    /// Metainfo of the of the torrent file requested.
    pub metainfo: Metainfo,
    /// Tracker's bitfield of the torrent file requested.
    pub bitfield: BitField,
    /// Tracker's response of the torrent file requested.
    pub response: Option<ResponseData>,
    /// Amount of active connections as listener. Tiene que ser un Arc<Mutex>
    pub count_connections: i32,
    pub peers_connected: Vec<Peer>,
    pub pieces_ammount: usize,
}

impl TorrentFile {
    /// Creates a new [`TorrentFile`].
    pub fn new(file_name: String) -> Result<Self, TorrentFileError> {
        let file = File::open(file_name.clone()).map_err(|_| TorrentFileError::FileError)?;

        let metainfo =
            metainfo::read_torrent(&file).map_err(|_| TorrentFileError::MetainfoError)?;
        let Info(mode) = metainfo.info.clone();
        let info = match mode {
            crate::torrent::info::InfoMode::Empty => todo!(),
            crate::torrent::info::InfoMode::SingleFile(it) => it,
        };

        Ok(TorrentFile {
            file_name,
            metainfo,
            bitfield: BitField::new(info.length as usize / info.piece_length as usize)
                .map_err(|_| TorrentFileError::BitFieldError)?,
            response: None,
            count_connections: 0,
            peers_connected: Vec::new(),
            pieces_ammount: info.length as usize / info.piece_length as usize,
        })
    }

    pub fn get_info_hash(&self) -> Vec<u8> {
        utils::hash_info(&self.metainfo.info.bencode()).to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn create_a_torrent_file() {
        let got = TorrentFile::new("debian-11.3.0-arm64-netinst.iso.torrent".to_string()).unwrap();

        let file = File::open("debian-11.3.0-arm64-netinst.iso.torrent".to_string()).unwrap();
        let metainfo = metainfo::read_torrent(&file).unwrap();
        let Info(mode) = metainfo.info.clone();
        let info = match mode {
            crate::torrent::info::InfoMode::Empty => todo!(),
            crate::torrent::info::InfoMode::SingleFile(it) => it,
        };

        let want = TorrentFile {
            file_name: "debian-11.3.0-arm64-netinst.iso.torrent".to_string(),
            metainfo,
            bitfield: BitField::new(info.length as usize / info.piece_length as usize).unwrap(),
            response: None,
            count_connections: 0,
            peers_connected: Vec::new(),
            pieces_ammount: info.length as usize / info.piece_length as usize,
        };

        assert_eq!(got, want);
    }
}

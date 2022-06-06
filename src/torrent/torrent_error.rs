use crate::bencode::parser_error;

#[derive(Debug)]
pub enum TorrentError {
    File(std::io::Error),
    InvalidTorrent,
    Parse(parser_error::ParserError),
}

impl Eq for TorrentError {}

impl PartialEq for TorrentError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (TorrentError::File(e1), TorrentError::File(e2)) => e1.kind() == e2.kind(),
            (TorrentError::InvalidTorrent, TorrentError::InvalidTorrent) => true,
            (TorrentError::Parse(e1), TorrentError::Parse(e2)) => e1 == e2,
            _ => false,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]

pub enum ClientError {
    InvalidInfo,
    ReadTorrentError,
    HandshakeError,
    BitfieldError,
}

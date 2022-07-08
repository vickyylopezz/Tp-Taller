#[derive(Debug)]
pub enum ProtocolError {
    ReadError, // TODO: Remove, redundant
    WrongSizeRead,
    MissingPeerID, // Should be an error?
    Connection,
    Handshake,
    PeerConnection,
    Read,
    EmptyBytes,
    MappingError,
}

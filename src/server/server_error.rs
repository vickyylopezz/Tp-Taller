#[derive(Debug, PartialEq, Eq)]
/// Represents the possible errors when running a Server.
pub enum ServerError {
    StreamError,
    InteractionError,
    BitFieldError,
    UnchockedError,
    ChockedError,
    PieceError,
    NotExpectedMessageError,
    HandshakeError,
}

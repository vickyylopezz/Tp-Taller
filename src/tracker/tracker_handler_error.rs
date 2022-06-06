/// Represents the possible errors that can occur while handling
/// the tracker.
#[derive(Debug, PartialEq, Eq)]
pub enum TrackerHandlerError {
    InvalidTlsConnector,
    InvalidTcpStream,
    InvalidQuerystring,
    InvalidConnection,
    InvalidAdress,
    ResponseError,
    RequestError,
    InteractionError,
}

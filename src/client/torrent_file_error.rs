#[derive(Debug, PartialEq, Eq)]
/// Represents the possible errors when creatin a TorrentFile object.
pub enum TorrentFileError {
    FileError,
    MetainfoError,
    BitFieldError,
    ConvertionError,
}

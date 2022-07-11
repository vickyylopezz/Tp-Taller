use crate::bencode::bencoded_value::BencodedValue;
use crate::torrent::info_builder::InfoBuilder;
use std::collections::HashMap;

static FILEINFO_REQUIRED_KEYS: [&[u8]; 4] = [b"piece length", b"pieces", b"name", b"length"];

/// This enum represents all the possible variants of the info
/// dictionary. The empty variant has no use outside initialization.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum InfoMode {
    /// Placeholder, only used for initialization
    Empty,
    /// Single file mode (The torrent doesn't have a directory structure)
    SingleFile(SingleFileData),
    // TODO: Missing MultipleFile variant
}

/// Wrapper over the the [`InfoMode`] enum.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Info(pub InfoMode);

/// Container for the data in the dictionary associated with the info
/// key when in Single File Mode. The fields: `length`, `name`,
/// `piece_length` and `pieces` must be always present
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SingleFileData {
    /// Length of the file to be downloaded in bytes
    pub length: i64,
    /// 32 character hexadecimal string corresponding to the MD5 sum
    /// of the file. This is not used by BitTorrent, but it's included
    /// for greater compatibility
    pub md5sum: Option<Vec<u8>>,
    /// The filename (advisory)
    pub name: String,
    /// Number of bytes in each piece
    pub piece_length: i64,
    /// String consisting of the concatenation of all 20-byte SHA1
    /// hash values (one per piece)
    pub pieces: Vec<u8>,
    /// If true the client *must* publish its presence to get other
    /// peers only via the trackers explicitly described in the
    /// metainfo file. Else if its false or [`None`] the client may
    /// obtain peers from other peers (One could interpret private as
    /// "no external peer source")
    pub private: Option<bool>,
}

impl Info {
    /// Creates a new instance of [`Info`] from a `Vec<(BencodedValue,
    /// BencodedValue)>`.  Returns [`Some`] if no errors occur while
    /// building the instance; otherwise returns [`None`].
    pub fn new(info: Vec<(BencodedValue, BencodedValue)>) -> Option<Self> {
        let dict = info
            .into_iter()
            .collect::<HashMap<BencodedValue, BencodedValue>>();

        let mut required = FILEINFO_REQUIRED_KEYS
            .iter()
            .map(|v| BencodedValue::ByteString(v.to_vec()));

        let has_required = required.all(|k| dict.contains_key(&k));
        if has_required {
            let mut info_build = InfoBuilder::new();
            for (k, v) in dict {
                // This will never be None, a key is always a
                // BencodedValue::ByteString variant
                k.byte_string()
                    .and_then(|s| build_info_fields(&mut info_build, &s[..], v))?;
            }
            Some(info_build.build())
        } else {
            None
        }
    }
    /// Bencodes the contents of the struct returning a byte string
    pub fn bencode(&self) -> Vec<u8> {
        match self.0 {
            InfoMode::Empty => Vec::new(),
            InfoMode::SingleFile(ref s) => bencode_single_file(s),
        }
    }

    pub fn info(&self) -> Option<SingleFileData> {
        match &self.0 {
            InfoMode::Empty => None,
            InfoMode::SingleFile(s) => Some(s.clone()),
        }
    }
}

/// Helper function for bencoding the [`Info`] struct when the mode is
/// single file
fn bencode_single_file(data: &SingleFileData) -> Vec<u8> {
    let mut str = format!(
        "d6:lengthi{}e4:name{}:{}12:piece lengthi{}e",
        data.length,
        data.name.len(),
        data.name,
        data.piece_length
    );
    if let Some(b) = data.private {
        str = format!("{}7:privatei{}e", str, b as u8)
    }

    // TODO: REFACTOR
    let mut byte_string = str.as_bytes().to_vec();
    if let Some(ref sum) = data.md5sum {
        byte_string.append(&mut b"6:md5sum".to_vec());
        byte_string.append(&mut sum.len().to_string().into_bytes());
        byte_string.push(b':'); // ':' in ascii
        byte_string.append(&mut sum.clone());
    }
    byte_string.append(&mut b"6:pieces".to_vec());
    byte_string.append(&mut data.pieces.len().to_string().into_bytes());
    byte_string.push(b':');
    byte_string.append(&mut data.pieces.clone());
    byte_string.push(b'e');
    byte_string
}

/// Helper function for building the [`Info Struct`]. Returns [`None`]
/// if there is an error building some of the fields
fn build_info_fields<'a>(
    info: &'a mut InfoBuilder,
    field: &'a [u8],
    value: BencodedValue,
) -> Option<()> {
    match field {
        b"piece length" => {
            let l = value.integer()?;
            info.piece_length(l);
        }
        b"pieces" => {
            let bytes = value.byte_string()?;
            info.pieces(bytes);
        }
        b"name" => {
            let bytes = value.byte_string()?;
            info.name(String::from_utf8(bytes).ok()?);
        }
        b"length" => {
            let l = value.integer()?;
            info.length(l);
        }
        b"private" => {
            let private = value.integer()?;
            let is_private = match private {
                1 => Some(true),
                0 => Some(false),
                _ => return None,
            };
            info.private(is_private);
        }
        b"md5sum" => {
            let bytes = value.byte_string()?;
            info.md5sum(Some(bytes));
        }
        _ => return None,
    }
    Some(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn new_info_in_single_file_mode() {
        let dict = vec![
            (
                BencodedValue::ByteString(b"length".to_vec()),
                BencodedValue::Integer(0),
            ),
            (
                BencodedValue::ByteString(b"md5sum".to_vec()),
                BencodedValue::ByteString(b"aaaa".to_vec()),
            ),
            (
                BencodedValue::ByteString(b"name".to_vec()),
                BencodedValue::ByteString(b"file".to_vec()),
            ),
            (
                BencodedValue::ByteString(b"piece length".to_vec()),
                BencodedValue::Integer(0),
            ),
            (
                BencodedValue::ByteString(b"pieces".to_vec()),
                BencodedValue::ByteString(b"aaaaa".to_vec()),
            ),
            (
                BencodedValue::ByteString(b"private".to_vec()),
                BencodedValue::Integer(1),
            ),
        ];

        let info = Info::new(dict);
        let is_single_file = info.map(|i| {
            let Info(mode) = i;
            match mode {
                InfoMode::Empty => false,
                InfoMode::SingleFile(_) => true,
            }
        });

        assert_eq!(is_single_file, Some(true));
    }

    #[test]
    fn new_returns_none_if_required_field_is_missing() {
        let dict = vec![
            (
                BencodedValue::ByteString(b"md5sum".to_vec()),
                BencodedValue::ByteString(b"aaaa".to_vec()),
            ),
            (
                BencodedValue::ByteString(b"name".to_vec()),
                BencodedValue::ByteString(b"file".to_vec()),
            ),
            (
                BencodedValue::ByteString(b"piece length".to_vec()),
                BencodedValue::Integer(0),
            ),
            (
                BencodedValue::ByteString(b"pieces".to_vec()),
                BencodedValue::ByteString(b"aaaaa".to_vec()),
            ),
            (
                BencodedValue::ByteString(b"private".to_vec()),
                BencodedValue::Integer(1),
            ),
        ];

        let info = Info::new(dict);
        assert_eq!(None, info);
    }

    #[test]
    fn bencode_single_info_file_data() {
        let dict = vec![
            (
                BencodedValue::ByteString(b"length".to_vec()),
                BencodedValue::Integer(0),
            ),
            (
                BencodedValue::ByteString(b"md5sum".to_vec()),
                BencodedValue::ByteString(b"aaaa".to_vec()),
            ),
            (
                BencodedValue::ByteString(b"name".to_vec()),
                BencodedValue::ByteString(b"file".to_vec()),
            ),
            (
                BencodedValue::ByteString(b"piece length".to_vec()),
                BencodedValue::Integer(0),
            ),
            (
                BencodedValue::ByteString(b"pieces".to_vec()),
                BencodedValue::ByteString(b"aaaaa".to_vec()),
            ),
            (
                BencodedValue::ByteString(b"private".to_vec()),
                BencodedValue::Integer(1),
            ),
        ];
        let info = Info::new(dict);
        let got = info.map(|i| i.bencode());
        let want = Some(
            b"d6:lengthi0e4:name4:file12:piece lengthi0e7:privatei1e6:md5sum4:aaaa6:pieces5:aaaaae"
                .to_vec(),
        );

        assert_eq!(got, want);
    }
}

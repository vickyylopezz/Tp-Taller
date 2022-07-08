use super::info::{self, Info};
use super::metainfo_builder::MetainfoBuilder;
use super::torrent_error::TorrentError;
use crate::bencode::{bencoded_value::BencodedValue, parser};

use std::collections::HashMap;
use std::io;

static METAINFO_REQUIERED_KEYS: [&[u8]; 2] = [b"info", b"announce"];

/// Contains the metadata from the torrent file. Only the announce and
/// info fields are required.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Metainfo {
    /// Announce URL of the tracker
    pub announce: String,
    /// Extension to the oficial specification
    pub announce_list: Option<Vec<String>>,
    /// Comments from the author
    pub comment: Option<String>,
    /// Name and version of the program used to create the .torrent
    pub created_by: Option<String>,
    /// Creation time of the torrent, in standard UNIX epoch format
    pub creation_date: Option<i64>,
    /// String encoding format used for generating the pieces part of
    /// the info dictionary
    pub encoding: Option<String>,
    /// Describes the file(s) of the torrent. There are two
    /// possibilities: single file and multifile.
    pub info: Info,
}

impl Metainfo {
    /// Creates a new Metainfo structure from a bencoded dictionary.
    /// Returns [`Some`] if no errors occur while building the
    /// instance; otherwise returns [`None`].
    fn new(bencoded_value: BencodedValue) -> Option<Self> {
        let dict = bencoded_value
            .dictionary()?
            .into_iter()
            .collect::<HashMap<BencodedValue, BencodedValue>>();

        METAINFO_REQUIERED_KEYS
            .iter()
            .map(|v| BencodedValue::ByteString(v.to_vec()))
            .all(|k| dict.contains_key(&k))
            .then(|| {
                let mut metainfo = MetainfoBuilder::new();
                for (k, v) in dict {
                    // This will never be None, a key is always a
                    // BencodedValue::ByteString variant
                    k.byte_string()
                        .and_then(|s| build_metainfo_fields(&mut metainfo, &s[..], v))?;
                }

                Some(metainfo.build())
            })
            .flatten()
    }
}

/// Reads torrent and returns `Result<Metainfo, TorrentError>`
///
/// # Errors
///
/// This function will return an error if an error occurs while
/// reading or parsing the read contents. Also when an error occurs
/// while building the [`Metainfo`] structure
pub fn read_torrent<R: io::Read>(mut reader: R) -> Result<Metainfo, TorrentError> {
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf).map_err(TorrentError::File)?;
    let bencoded_dictionary = parser::parse(buf).map_err(TorrentError::Parse)?;

    Metainfo::new(bencoded_dictionary).ok_or(TorrentError::InvalidTorrent)
}

/// Helper function for building the Metainfo struct. Returns [`None`]
/// if there is an error building some of the fields
fn build_metainfo_fields(
    metainfo: &mut MetainfoBuilder,
    field: &[u8],
    v: BencodedValue,
) -> Option<()> {
    match field {
        b"info" => {
            let dict = v.dictionary()?;
            let info = info::Info::new(dict)?;

            metainfo.info(info);
        }
        b"announce" => {
            let bytes = v.byte_string()?;
            metainfo.announce(String::from_utf8(bytes).ok()?);
        }
        b"announce-list" => {
            let announce_list = v
                .list()
                .into_iter()
                .flatten()
                .filter_map(BencodedValue::list)
                .flatten()
                .map(|e| {
                    let bytes = e.byte_string()?;
                    String::from_utf8(bytes).ok()
                })
                .collect::<Option<Vec<String>>>()?;

            metainfo.announce_list(Some(announce_list));
        }
        b"creation date" => {
            let date = v.integer()?;
            metainfo.creation_date(Some(date));
        }
        b"comment" => {
            let bytes = v.byte_string()?;
            metainfo.comment(Some(String::from_utf8(bytes).ok()?));
        }
        b"created by" => {
            let bytes = v.byte_string()?;
            metainfo.created_by(Some(String::from_utf8(bytes).ok()?));
        }
        b"encoding" => {
            let bytes = v.byte_string()?;
            metainfo.encoding(Some(String::from_utf8(bytes).ok()?));
        }

        b"httpseeds" => {}
        _ => return None,
    }
    Some(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_metainfo_from_torrent_single_file() {
        let metainfo = b"d8:announce3:url13:announce-listll1:ael2:abel3:abcee7:comment7:comment\
			 10:created by2:me13:creation datei0e8:encoding4:utf84:infod6:length\
			 i0e6:md5sum4:aaaa4:name4:file12:piece lengthi0e6:pieces5:aaaaa7:privatei1eee";
        let got = read_torrent(&metainfo[..]).unwrap();
        let info = parser::parse(
            b"d6:lengthi0e6:md5sum4:aaaa4:name4:file12:piece lengthi0e6:pieces5:aaaaa7:privatei1ee"
                .to_vec(),
        )
        .unwrap()
        .dictionary()
        .unwrap();
        let want = Metainfo {
            announce: "url".into(),
            announce_list: Some(vec!["a".into(), "ab".into(), "abc".into()]),
            comment: Some("comment".into()),
            created_by: Some("me".into()),
            creation_date: Some(0),
            encoding: Some("utf8".into()),
            info: Info::new(info).unwrap(),
        };
        assert_eq!(got, want);
    }

    #[test]
    fn read_metainfo_with_only_required_fields() {
        let metainfo = b"d8:announce3:url4:infod6:length\
			 i0e4:name4:file12:piece lengthi0e6:pieces5:aaaaaee";
        let got = read_torrent(&metainfo[..]).unwrap();
        let info =
            parser::parse(b"d6:lengthi0e4:name4:file12:piece lengthi0e6:pieces5:aaaaae".to_vec())
                .unwrap()
                .dictionary()
                .unwrap();
        let want = Metainfo {
            announce: "url".into(),
            announce_list: None,
            comment: None,
            created_by: None,
            creation_date: None,
            encoding: None,
            info: Info::new(info).unwrap(),
        };
        assert_eq!(got, want);
    }

    #[test]
    fn reading_something_different_from_dictionary_returns_error() {
        let metainfo = b"le";
        let got = read_torrent(&metainfo[..]).unwrap_err();
        assert_eq!(got, TorrentError::InvalidTorrent);
    }

    #[test]
    fn reading_torrent_missing_required_field_returns_error() {
        let metainfo = b"4:infod6:lengthi0e4:name4:file12:piece lengthi0e6:pieces5:aaaaaee";
        let got = read_torrent(&metainfo[..]).unwrap_err();
        assert_eq!(got, TorrentError::InvalidTorrent);
    }

    #[test]
    fn reading_torrent_with_wrong_value_types() {
        let metainfo = b"d8:announce3:url13:announce-listl1:a2:ab3:abce7:comment7:comment\
			 10:created by2:me13:creation datei0e8:encoding4:utf84:infoleee";
        let got = read_torrent(&metainfo[..]).unwrap_err();
        assert_eq!(got, TorrentError::InvalidTorrent);
    }

    #[test]
    fn reading_torrent_with_extra_field_returns_error() {
        let metainfo = b"d5:extrai0e8:announce3:url4:infod6:length\
			 i0e4:name4:file12:piece lengthi0e6:pieces5:aaaaaee";
        let got = read_torrent(&metainfo[..]).unwrap_err();
        assert_eq!(got, TorrentError::InvalidTorrent);
    }

    #[test]
    fn read_torrent_file() {
        let f = std::fs::File::open("test.torrent").unwrap();
        read_torrent(f).unwrap();
    }
}

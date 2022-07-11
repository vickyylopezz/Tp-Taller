use std::{collections::HashMap, io};

/// All posible configuration parameters
const KEYS: [&str; 4] = ["port", "logs_dir", "downloads_dir", "torrents_dir"];

/// This type encapsulates the configuration parameters specified in
/// the configuration file
#[derive(Debug, PartialEq, Eq)]
pub struct Config {
    /// Port where the connections to other peers are listened
    tcp_port: u16,
    /// Directory where logs are going to be stored
    logs_directory: String,
    /// Directory where the downloads are going to be stored
    downloads_directory: String,
    /// Directory where the torrents are stored
    torrent_dir: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ConfigError {
    FileNotFound,
    InvalidFileContent,
    MissingValues,
    InvalidPortNumber,
}

impl Config {
    /// Creates a new instance of Config from, a type that implements
    /// [`io::Read`]. Returns an error when the file contains invalid
    /// UTF-8, or when some parameter is invalid or missing.
    pub fn new<F: io::Read>(mut file: F) -> Result<Self, ConfigError> {
        let mut buf = String::new();
        file.read_to_string(&mut buf)
            .map_err(|_| ConfigError::InvalidFileContent)?;

        let config_dict = buf
            .lines()
            .flat_map(|l| l.split_once('='))
            .collect::<HashMap<&str, &str>>();
        let all_keys = KEYS.iter().all(|k| config_dict.contains_key(k));
        if all_keys {
            Ok(Self {
                tcp_port: config_dict[KEYS[0]]
                    .parse()
                    .map_err(|_| ConfigError::InvalidPortNumber)?,
                logs_directory: config_dict[KEYS[1]].to_string(),
                downloads_directory: config_dict[KEYS[2]].to_string(),
                torrent_dir: config_dict[KEYS[3]].to_string(),
            })
        } else {
            Err(ConfigError::MissingValues)
        }
    }

    pub fn tcp_port(&self) -> u16 {
        self.tcp_port
    }

    pub fn logs(&self) -> String {
        self.logs_directory.clone()
    }

    pub fn downloads(&self) -> String {
        self.downloads_directory.clone()
    }

    pub fn torrents(&self) -> String {
        self.torrent_dir.clone()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            tcp_port: 6881,
            logs_directory: String::new(),
            downloads_directory: String::new(),
            torrent_dir: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_config_is_created_correctly() {
        let p = b"port=80\nlogs_dir=/home/test\ndownloads_dir=/home/downloads\ntorrents_dir=/home/torrents\nmode=server";
        let got = Config::new(&p[..]).unwrap();
        let want = Config {
            tcp_port: 80,
            logs_directory: String::from("/home/test"),
            downloads_directory: String::from("/home/downloads"),
            torrent_dir: String::from("/home/torrents"),
        };

        assert_eq!(got, want);
    }
}

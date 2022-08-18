use std::ops::{Index, IndexMut};

use serde::{Deserialize, Serialize};

use crate::torrent::Torrent;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Data {
    torrents: Vec<Torrent>,
}

impl Data {
    pub fn new() -> Self {
        Data {
            torrents: Vec::new(),
        }
    }

    pub fn torrents(&'_ mut self) -> &'_ mut [Torrent] {
        &mut self.torrents
    }

    pub fn push(&mut self, torrent: Torrent) {
        self.torrents.push(torrent);
    }

    pub fn _len(&self) -> usize {
        self.torrents.len()
    }

    pub fn _iter(&self) -> std::slice::Iter<Torrent> {
        self.torrents.iter()
    }
}

impl Index<usize> for Data {
    type Output = Torrent;

    fn index(&self, index: usize) -> &Self::Output {
        &self.torrents[index]
    }
}

impl IndexMut<usize> for Data {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.torrents[index]
    }
}

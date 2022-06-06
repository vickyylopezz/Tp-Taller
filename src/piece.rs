use std::{fs::File, io::Write};

use sha1::{Digest, Sha1};

use crate::block::Block;

static BLOCK_SIZE: i64 = 16384; // 2^14

#[derive(Debug, PartialEq, Eq)]
pub enum PieceError {}

/// Represents a portion of the data to be downloaded which is described in the metainfo
/// of the torrent file and can be verified by a SHA1 hash. It is made of many Blocks.
#[derive(Debug, PartialEq)]
pub struct Piece {
    pub length: i64,
    pub piece_length: i64,
    pub index: i64,
    pub blocks: Vec<Block>,
    pub hash: Vec<u8>,
}

impl Piece {
    /// Returns a pice of the file to be downloaded.
    /// Creates de Blocks of the piece.
    pub fn new(length: i64, index: i64, piece_length: i64, hash: Vec<u8>) -> Self {
        let mut blocks: Vec<Block> = vec![];
        let num_blocks = ((length as f64) / (BLOCK_SIZE as f64)).ceil() as i64;

        for i in 0..num_blocks {
            let block_length = {
                if i < num_blocks - 1 {
                    BLOCK_SIZE
                } else {
                    length - (BLOCK_SIZE * (num_blocks - 1))
                }
            };

            let block = Block::new(i, block_length);
            blocks.push(block);
        }

        Piece {
            length,
            piece_length,
            index,
            hash,
            blocks,
        }
    }

    /// Verifies that the piece has all the blocks of data.
    pub fn have_all_blocks(&self) -> bool {
        for block in self.blocks.iter() {
            if block.data.is_none() {
                return false;
            }
        }
        true
    }

    /// Validates that the data of the piece matches with its SHA1 hash.
    /// In that case, writes it in a file.
    fn write_piece(&mut self, file: &mut File, piece_data: Vec<u8>) {
        let mut hasher = Sha1::new();
        hasher.update(&piece_data);
        let result = hasher.finalize();
        println!("Hash of the piece: {:x?}", result);

        if self.hash == result[..] {
            file.write_all(&piece_data).unwrap();
        }
    }

    /// Writes a piece of the file to be downloaded in a file.
    pub fn store(
        &mut self,
        file: &mut File,
        block_index: u32,
        data: Vec<u8>,
    ) -> Result<(), PieceError> {
        let block = &mut self.blocks[block_index as usize];
        block.data = Some(data);

        if self.have_all_blocks() {
            // Concatenates data from different blocks
            let mut piece_data = vec![];
            for block in self.blocks.iter() {
                piece_data.extend(block.data.clone().unwrap());
            }
            self.write_piece(file, piece_data);
        }
        Ok(())
    }
}

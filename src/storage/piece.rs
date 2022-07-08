use std::{fs::File, io::Write};

use sha1::{Digest, Sha1};

use crate::storage::block::Block;

static BLOCK_SIZE: i64 = 16384; // 2^14

#[derive(Debug, PartialEq, Eq)]
pub enum PieceError {
    FewBlocks,
    File,
    Write,
    Block,
    DifferentHash,
}

/// Represents a portion of the data to be downloaded which is described in the metainfo
/// of the torrent file and can be verified by a SHA1 hash. It is made of many Blocks.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Piece {
    /// Length of the piece.
    pub length: i64,
    /// Index of the piece in the torrent file.
    pub index: i64,
    /// List of blocks of the piece.
    pub blocks: Vec<Block>,
    /// Hash of the piece. It is used to verify if the piece was correctly downloaded.
    pub hash: Vec<u8>,
    /// File name of the torrent file to be downloaded.
    pub file_name: String,
}

impl Piece {
    /// Returns a pice of the file to be downloaded.
    /// Creates de Blocks of the piece.
    pub fn new(length: i64, index: i64, hash: Vec<u8>, file_name: String) -> Self {
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

            let block = Block::new(i, block_length, index);
            blocks.push(block);
        }

        Piece {
            length,
            index,
            hash,
            blocks,
            file_name,
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

    /// Adds a block of data to the lists of blocks of the piece.
    pub fn add_block(&mut self, block_index: i64, data: Vec<u8>) {
        let block = &mut self.blocks[block_index as usize];
        block.data = Some(data);
    }
}

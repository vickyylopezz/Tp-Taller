/// Represents a portion of a piece to be downloaded.
/// Many blocks make up one piece of the file to be download.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Block {
    pub index: i64,
    pub piece_index: i64,
    pub length: i64,
    pub data: Option<Vec<u8>>,
}

impl Block {
    pub fn new(index: i64, length: i64, piece_index: i64) -> Self {
        Block {
            index,
            length,
            data: None,
            piece_index,
        }
    }

    pub fn add_data(&mut self, data: Vec<u8>) {
        self.data = Some(data);
    }
}
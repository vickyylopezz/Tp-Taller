use super::info::{Info, InfoMode, SingleFileData};

pub struct InfoBuilder {
    length: i64,
    md5sum: Option<Vec<u8>>,
    name: String,
    piece_length: i64,
    pieces: Vec<u8>,
    private: Option<bool>,
}

impl InfoBuilder {
    pub fn new() -> Self {
        Self {
            length: 0,
            md5sum: None,
            name: String::new(),
            piece_length: 0,
            pieces: Vec::new(),
            private: None,
        }
    }

    pub fn length(&'_ mut self, l: i64) -> &'_ mut Self {
        self.length = l;
        self
    }

    pub fn md5sum(&'_ mut self, md5: Option<Vec<u8>>) -> &'_ mut Self {
        self.md5sum = md5;
        self
    }

    pub fn name(&'_ mut self, n: String) -> &'_ mut Self {
        self.name = n;
        self
    }

    pub fn piece_length(&'_ mut self, l: i64) -> &'_ mut Self {
        self.piece_length = l;
        self
    }

    pub fn pieces(&'_ mut self, p: Vec<u8>) -> &'_ mut Self {
        self.pieces = p;
        self
    }

    pub fn private(&'_ mut self, is_private: Option<bool>) -> &'_ mut Self {
        self.private = is_private;
        self
    }

    pub fn single_file(self) -> SingleFileData {
        SingleFileData {
            length: self.length,
            md5sum: self.md5sum,
            name: self.name,
            piece_length: self.piece_length,
            pieces: self.pieces,
            private: self.private,
        }
    }

    pub fn build(self) -> Info {
        Info(InfoMode::SingleFile(self.single_file()))
    }
}

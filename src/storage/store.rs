use super::piece::{Piece, PieceError};

use crate::log::logger;

use sha1::{Digest, Sha1};

use std::fs::File;
use std::io;
use std::io::Read;
use std::io::Seek;
use std::io::Write;
use std::path::Path;
use std::sync::mpsc;
use std::sync::mpsc::RecvError;
use std::sync::mpsc::SendError;
use std::thread;

pub enum StoreError {
    File,
}

#[derive(Debug, PartialEq, Eq)]
pub enum StoreMessage {
    PieceMessage(Piece),
    Terminate,
}

#[derive(Debug, PartialEq, Eq)]
pub enum StoreClientMessage {
    PieceError(PieceError),
    Stored(i64, String),
    Terminate,
}

#[derive(Debug)]
pub struct Store {
    store_handler: Option<thread::JoinHandle<()>>,
    store_queue: mpsc::Sender<StoreMessage>, // Client -> Storage
    store_receiver: mpsc::Receiver<StoreClientMessage>, // Client -> Client
}

impl Store {
    pub fn new(directory: String, logger: logger::LogHandle) -> Self {
        let (store_queue, rx) = mpsc::channel();
        let (tx, store_receiver) = mpsc::channel();
        let store_channel = tx;

        let store_handler = Some(thread::spawn(move || loop {
            let piece = rx.recv().unwrap();
            let tx = store_channel.clone();
            //let pool = threadpool::ThreadPool::new(2, &mut logger).unwrap(); // There is no reason for 5

            match piece {
                StoreMessage::PieceMessage(p) =>
                //pool.spawn(move || {
                {
                    match store_piece(&p, &directory) {
                        // Log in console
                        Ok(_) => {
                            tx.send(StoreClientMessage::Stored(p.index, p.file_name))
                                .unwrap();
                        }
                        Err(e) => tx.send(StoreClientMessage::PieceError(e)).unwrap(),
                    }
                }
                //})
                //.unwrap()
                ,
                StoreMessage::Terminate => break,
            };
        }));

        Store {
            store_handler,
            store_queue,
            store_receiver,
        }
    }

    pub fn store_file(
        &self,
        file_name: String,
        file_length: u64,
        pieces: i32,
        piece_length: u64,
        directory: &str,
    ) -> Result<(), StoreError> {
        let mut file = File::create(Path::new(&format!("{}{}", directory, file_name.clone())))
            .map_err(|_| StoreError::File)?;
        file.set_len(file_length).map_err(|_| StoreError::File)?;

        for i in 0..pieces {
            let mut piece = File::open(Path::new(&format!(
                "{}piece{}-{}",
                directory,
                i,
                file_name.clone()
            )))
            .map_err(|_| StoreError::File)?;
            let mut buf = Vec::<u8>::new();
            piece.read_to_end(&mut buf).map_err(|_| StoreError::File)?;
            let offset = i as u64 * piece_length as u64;
            file.seek(io::SeekFrom::Start(offset))
                .map_err(|_| StoreError::File)?;
            file.write_all(&buf).map_err(|_| StoreError::File)?;
        }
        Ok(())
    }

    pub fn receive(&self) -> Result<StoreClientMessage, RecvError> {
        self.store_receiver.recv()
    }

    pub fn send(&self, msg: StoreMessage) -> Result<(), SendError<StoreMessage>> {
        self.store_queue.send(msg)
    }
}

impl Drop for Store {
    fn drop(&mut self) {
        self.store_queue.send(StoreMessage::Terminate).unwrap();

        if let Some(thread) = self.store_handler.take() {
            thread.join().unwrap();
        }
    }
}

fn have_all_blocks(p: &Piece) -> bool {
    for block in p.blocks.iter() {
        if block.data.is_none() {
            return false;
        }
    }
    true
}

fn store_piece(piece: &Piece, directory: &str) -> Result<(), PieceError> {
    if have_all_blocks(piece) {
        // Blocks union
        let mut piece_data = vec![];
        for block in piece.blocks.iter() {
            piece_data.extend(match block.data.clone() {
                Some(it) => it,
                None => return Err(PieceError::Block),
            });
        }

        // Hash piece
        let mut hasher = Sha1::new();
        hasher.update(&piece_data);
        let result = hasher.finalize();

        //println!("entre if hashs: {:?}", result);

        if piece.hash == result[..] {
            let path = &(directory.to_owned()
                + &"piece".to_owned()
                + &piece.index.to_string()
                + "-"
                + &piece.file_name.clone());
            let mut file = File::create(Path::new(path)).map_err(|_| PieceError::File)?;
            //store
            match file.write_all(&piece_data) {
                Ok(_) => return Ok(()),
                Err(_) => return Err(PieceError::Write),
            }
        } else {
            return Err(PieceError::DifferentHash);
        }
    }

    Err(PieceError::FewBlocks)
}

#[cfg(test)]
mod tests {

    // use crate::log::logger::Logger;

    // use super::*;

    // #[test]
    // fn store_pieces() {
    //     let mut p1 = Piece::new(
    //         16384,
    //         4,
    //         vec![
    //             41, 226, 220, 251, 177, 111, 99, 187, 2, 84, 223, 117, 133, 161, 91, 182, 251, 94,
    //             146, 125,
    //         ],
    //         "test1.txt".to_string(),
    //     );
    //     let mut p2 = Piece::new(
    //         16384,
    //         5,
    //         vec![
    //             41, 226, 220, 251, 177, 111, 99, 187, 2, 84, 223, 117, 133, 161, 91, 182, 251, 94,
    //             146, 125,
    //         ],
    //         "test1.txt".to_string(),
    //     );

    //     p1.add_block(0, vec![0, 0, 0]);
    //     p2.add_block(0, vec![0, 0, 0]);

    //     let log = File::create("log.txt").unwrap();

    //     let logger = Logger::new(log);
    //     let logger_handler = logger.new_handler();
    //     let store = Store::new(logger_handler);

    //     store.send(StoreMessage::PieceMessage(p1)).unwrap();
    //     store.send(StoreMessage::PieceMessage(p2)).unwrap();

    //     assert_eq!(
    //         store.receive(),
    //         Ok(StoreClientMessage::Stored(4, "test1.txt".to_string()))
    //     );
    // }
}

use crate::log::logger;
use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

pub type Job = Box<dyn FnOnce() + Send + 'static>;
pub type Receiver = Arc<Mutex<mpsc::Receiver<WorkerMessage>>>;

pub enum WorkerMessage {
    NewJob(Job),
    Terminate,
}

#[derive(Debug)]
pub enum WorkerError {
    NoThread,
    WorkerPanic,
}

#[derive(Debug)]
pub struct WorkerId(pub usize);

#[derive(Debug)]
pub struct Worker {
    pub id: WorkerId,
    pub thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    pub fn new(id: WorkerId, rx: Receiver, mut logger: logger::LogHandle) -> Self {
        let thread = thread::spawn(move || loop {
            let lock = match rx.lock() {
                Ok(l) => l,
                Err(_) => {
                    logger.error("Poisoned Mutex");
                    panic!("Poisoned Mutex")
                }
            };
            let message = match lock.recv() {
                Ok(r) => r,
                Err(_) => {
                    logger.error("Channel Disconnected");
                    panic!("Channel Disconnected")
                }
            };
            match message {
                WorkerMessage::NewJob(job) => {
                    // println!("\n\nWORKER: {}\n\n", id.0);
                    job()
                }
                WorkerMessage::Terminate => break,
            }
        });
        Self {
            id,
            thread: Some(thread),
        }
    }

    pub fn join(&mut self) -> Result<(), WorkerError> {
        if let Some(thread) = self.thread.take() {
            thread.join().map_err(|_| WorkerError::WorkerPanic)
        } else {
            Err(WorkerError::NoThread)
        }
    }
}

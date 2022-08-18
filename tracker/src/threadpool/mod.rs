mod error;
mod worker;

use log::error;
use std::sync::{mpsc, Arc, Mutex};
use worker::Worker;
use worker::WorkerId;
use worker::WorkerMessage;
/// Threadpool used to execute various jobs concurrently. Spawns a
/// specific number of threads.
#[derive(Debug)]
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<WorkerMessage>,
}

impl ThreadPool {
    /// Creates a new threadpool of size `size`. Returns an error if
    /// size is 0.
    pub fn new(size: usize) -> Result<Self, error::ThreadPoolError> {
        if size == 0 {
            return Err(error::ThreadPoolError::PoolCreationError);
        }
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);
        (0..size)
            .map(WorkerId)
            .for_each(|id| workers.push(Worker::new(id, Arc::clone(&receiver))));

        Ok(Self { workers, sender })
    }

    /// Sends a new job to the workers queue
    pub fn spawn<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        if self.sender.send(WorkerMessage::NewJob(job)).is_err() {
            error!("All receivers dropped");
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        self.workers.iter().for_each(|_| {
            self.sender.send(WorkerMessage::Terminate).unwrap();
        });

        self.workers.iter_mut().for_each(|w| {
            if let Some(thread) = w.thread.take() {
                thread.join().unwrap();
            }
        })
    }
}

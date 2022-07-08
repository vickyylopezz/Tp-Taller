mod worker;
use crate::log::logger;
use crate::threadpool::worker::WorkerMessage;
use crate::threadpool::worker::{Worker, WorkerId};
use std::sync::{mpsc, Arc, Mutex};

#[derive(Debug)]
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<WorkerMessage>,
}

#[derive(Debug)]
pub enum ThreadPoolError {
    PoolCreationError,
}

impl ThreadPool {
    pub fn new(size: usize, logger: logger::LogHandle) -> Result<Self, ThreadPoolError> {
        if size == 0 {
            return Err(ThreadPoolError::PoolCreationError);
        }
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);
        (0..size)
            .map(WorkerId)
            .for_each(|id| workers.push(Worker::new(id, Arc::clone(&receiver), logger.clone())));

        Ok(Self { workers, sender })
    }

    pub fn spawn<F>(&self, f: F) -> Result<(), mpsc::SendError<WorkerMessage>>
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(WorkerMessage::NewJob(job))
    }

    pub fn join(&mut self) -> Result<(), worker::WorkerError> {
        for e in self.workers.iter_mut() {
            e.join()?;
        }
        Ok(())
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
#[cfg(test)]
mod tests {}

use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

//use log::{debug, error};
use log::error;

pub type Job = Box<dyn FnOnce() + Send + 'static>;
pub type Receiver = Arc<Mutex<mpsc::Receiver<WorkerMessage>>>;

pub enum WorkerMessage {
    NewJob(Job),
    Terminate,
}

#[derive(Debug)]
pub struct WorkerId(pub usize);

#[derive(Debug)]
pub struct Worker {
    pub id: WorkerId,
    pub thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    pub fn new(id: WorkerId, rx: Receiver) -> Self {
        let thread = thread::spawn(move || loop {
            let message = match rx.lock() {
                Ok(l) => match l.recv() {
                    Ok(v) => v,
                    Err(_) => {
                        error!("All senders dropped");
                        return;
                    }
                },
                Err(_) => {
                    error!("Poisoned mutex");
                    return;
                }
            };
            match message {
                WorkerMessage::NewJob(job) => {
                    //let WorkerId(worker_id) = id;
                    //debug!("Worker {} got a job; executing.", worker_id);
                    job()
                }
                WorkerMessage::Terminate => {
                    //let WorkerId(worker_id) = id;
                    //debug!("Worker {} was told to terminate.", worker_id);
                    break;
                }
            }
        });
        Self {
            id,
            thread: Some(thread),
        }
    }
}

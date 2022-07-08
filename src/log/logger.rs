use std::{io, sync::mpsc, thread};

enum _LoggerError {
    Recv,
}

enum MessageType {
    Info(String),
    Error(String),
    Debug(String),
    Terminate,
}

#[derive(Debug, Clone)]
pub struct LogHandle(mpsc::Sender<MessageType>);

#[derive(Debug)]
pub struct Logger {
    queue: mpsc::Sender<MessageType>,
    thread: Option<thread::JoinHandle<()>>,
}

impl Logger {
    pub fn new<W: io::Write + Send + 'static>(mut w: W) -> Self {
        // TODO: Error channel
        let (queue, receiver) = mpsc::channel();
        let thread = Some(thread::spawn(move || loop {
            let msg = receiver.recv().unwrap(); //.map_or(MessageType::Terminate, |m| m); // could add some context
            match msg {
                MessageType::Info(s) => w.write_all(s.as_bytes()).unwrap(),
                MessageType::Error(s) => w.write_all(s.as_bytes()).unwrap(),
                MessageType::Debug(s) => w.write_all(s.as_bytes()).unwrap(),
                MessageType::Terminate => break,
            };
        }));

        Self { queue, thread }
    }

    pub fn new_handler(&self) -> LogHandle {
        LogHandle(self.queue.clone())
    }
}

impl LogHandle {
    pub fn info(&mut self, msg: &str) {
        let s = format!("INFO: {}\n", msg);
        self.0.send(MessageType::Info(s)).unwrap()
    }

    pub fn error(&mut self, msg: &str) {
        let s = format!("ERROR: {}\n", msg);
        self.0.send(MessageType::Error(s)).unwrap()
    }

    pub fn debug(&mut self, msg: &str) {
        let s = format!("DEBUG: {}\n", msg);
        self.0.send(MessageType::Debug(s)).unwrap()
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        self.queue.send(MessageType::Terminate).unwrap();
        if let Some(thread) = self.thread.take() {
            thread.join().unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::{BufRead, BufReader};
    const PATH: &'static str = "tmp.txt";

    #[test]
    fn log_to_file() {
        let fo = fs::File::create(PATH).unwrap();
        {
            let logger = Logger::new(fo);
            let mut handler = logger.new_handler();
            handler.info("Test log");
            handler.error("Test log");
            handler.debug("Test log");
        }

        let fi = fs::File::open(PATH).unwrap();

        let got = BufReader::new(fi)
            .lines()
            .collect::<Result<Vec<String>, _>>();

        fs::remove_file(PATH).unwrap();

        let want: Vec<String> = vec![
            "INFO: Test log".into(),
            "ERROR: Test log".into(),
            "DEBUG: Test log".into(),
        ];

        assert_eq!(got.unwrap(), want);
    }
}

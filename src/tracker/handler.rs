use super::request::tracker_request::TrackerRequest;
use super::request::tracker_request_error::TrackerRequestError;
use super::response::tracker_response::TrackerResponse;
use super::response::tracker_response_error::TrackerResponseError;
use super::tracker_handler_error::TrackerHandlerError;
use crate::bencode::parser;
use crate::log::logger;
use crate::threadpool;

use log::{error, info};
use native_tls::TlsConnector;
use std::io::{Read, Write};
use std::net::TcpStream;

use std::sync::mpsc;
use std::sync::mpsc::RecvError;
use std::sync::mpsc::SendError;
use std::{str, thread};

#[derive(Debug, PartialEq, Eq)]
pub enum HandlerMessage {
    RequestMessage(TrackerRequest),
    Terminate,
}

#[derive(Debug, PartialEq, Eq)]
pub enum HandlerClientMessage {
    ResponseMessage(TrackerResponse),
    RequestError(TrackerRequestError),
    ResponseError(TrackerResponseError),
    Sent,
    Terminate,
}

#[derive(Debug)]
pub struct Handler {
    pub request_handler: Option<thread::JoinHandle<()>>,
    request_queue: mpsc::Sender<HandlerMessage>, // Cliente -> Handler
    response_receiver: mpsc::Receiver<HandlerClientMessage>, // Handler -> Cliente
}

impl Handler {
    pub fn new(mut logger: logger::LogHandle) -> Self {
        let (request_queue, rx) = mpsc::channel();
        let (tx, response_receiver) = mpsc::channel();
        let request_channel = tx;

        let request_handler = Some(thread::spawn(move || {
            let mut pool = threadpool::ThreadPool::new(2, logger.clone()).unwrap();
            loop {
                let request = rx.recv().unwrap();
                let tx = request_channel.clone();
                let mut handle = logger.clone();
                match request {
                    HandlerMessage::RequestMessage(mut req) => pool
                        .spawn(move || {
                            match make_request(&mut req) {
                                // Log in console
                                Ok(s) => {
                                    tx.send(HandlerClientMessage::Sent).unwrap();
                                    info!("Tracker request sent");
                                    handle.info("Tracker request sent");
                                    match receive_response(s) {
                                        Ok(response) => match parse_response(response, req.peer_id)
                                        {
                                            Ok(r) => {
                                                info!("Response received");
                                                handle.info("Response received");
                                                tx.send(HandlerClientMessage::ResponseMessage(r))
                                                    .unwrap()
                                            }
                                            Err(e) => {
                                                error!("{}", e);
                                                handle.error(&format!("{}", e));
                                                tx.send(HandlerClientMessage::ResponseError(e))
                                                    .unwrap()
                                            }
                                        },
                                        Err(e) => {
                                            error!("{}", e);
                                            handle.error(&format!("{}", e));
                                            tx.send(HandlerClientMessage::ResponseError(e)).unwrap()
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("{}", e);
                                    handle.error(&format!("{}", e));
                                    tx.send(HandlerClientMessage::RequestError(e)).unwrap()
                                }
                            };
                        })
                        .unwrap(),
                    HandlerMessage::Terminate => break,
                };
            }

            pool.join().unwrap();
        }));

        Handler {
            request_handler,
            request_queue,
            response_receiver,
        }
    }

    pub fn receive(&self) -> Result<HandlerClientMessage, RecvError> {
        self.response_receiver.recv()
    }

    pub fn send(&self, msg: HandlerMessage) -> Result<(), SendError<HandlerMessage>> {
        self.request_queue.send(msg)
    }
}

impl Drop for Handler {
    fn drop(&mut self) {
        self.request_queue.send(HandlerMessage::Terminate).unwrap();

        if let Some(thread) = self.request_handler.take() {
            thread.join().unwrap();
        }
    }
}

fn parse_torrent_host(host: &mut String) -> Option<&String> {
    if host.contains("https://") {
        *host = host.strip_prefix("https://")?.to_string();
        *host = host.strip_suffix("/announce")?.to_string();
        let tls: Vec<&str> = host.split(':').collect();
        *host = tls[0].to_string();
        host.push_str(":443");
    } else if host.contains("http://") {
        *host = host.strip_prefix("http://")?.to_string();
        *host = host.strip_suffix("/announce")?.to_string();
    } else {
        return None;
    }

    Some(host)
}

trait ReadWrite: Read + Write {}

impl<T: Read + Write> ReadWrite for T {}
pub struct Stream {
    stream: Box<dyn ReadWrite>,
}

fn make_request(request: &mut TrackerRequest) -> Result<Stream, TrackerRequestError> {
    let mut announce = request.announce.clone();

    if announce.contains("https://") {
        let connector = TlsConnector::new().map_err(|_| TrackerHandlerError::InvalidTlsConnector);
        let host = match parse_torrent_host(&mut announce) {
            Some(it) => it,
            None => return Err(TrackerRequestError::Host),
        };
        let stream = match TcpStream::connect(host) {
            Ok(it) => it,
            Err(_) => return Err(TrackerRequestError::InvalidTcpStream),
        };
        let local_addr = stream.local_addr();
        let ip = match local_addr {
            Ok(it) => it.ip(),
            Err(_) => return Err(TrackerRequestError::InvalidAdress),
        };
        request.set_addr(ip);
        let tls: Vec<&str> = host.split(':').collect();
        let mut stream = connector.unwrap().connect(tls[0], stream).unwrap();
        let path = format!(
            "/announce{}",
            request
                .generate_querystring()
                .map_err(|_| TrackerRequestError::InvalidQuerystring)?
                .get_querystring()
        );
        let header = format!("GET {} HTTP/1.0\r\nHost: {}\r\n\r\n", path, host);
        match stream.write_all(header.as_bytes()) {
            Ok(_) => Ok({
                Stream {
                    stream: Box::new(stream),
                }
            }),
            Err(_) => Err(TrackerRequestError::WriteStream),
        }
    } else {
        let host = match parse_torrent_host(&mut announce) {
            Some(it) => it,
            None => return Err(TrackerRequestError::Host),
        };
        let mut stream = match TcpStream::connect(host) {
            Ok(it) => it,
            Err(_) => return Err(TrackerRequestError::InvalidTcpStream),
        };
        let local_addr = stream.local_addr();
        let ip = match local_addr {
            Ok(it) => it.ip(),
            Err(_) => return Err(TrackerRequestError::InvalidAdress),
        };
        request.set_addr(ip);
        let path = format!(
            "/announce{}",
            request
                .generate_querystring()
                .map_err(|_| TrackerRequestError::InvalidQuerystring)?
                .get_querystring()
        );
        let header = format!("GET {} HTTP/1.0\r\nHost: {}\r\n\r\n", path, host);
        match stream.write_all(header.as_bytes()) {
            Ok(_) => Ok({
                Stream {
                    stream: Box::new(stream),
                }
            }),
            Err(_) => Err(TrackerRequestError::WriteStream),
        }
    }
}

fn receive_response(mut stream: Stream) -> Result<Vec<u8>, TrackerResponseError> {
    let mut res = vec![];

    match stream.stream.read_to_end(&mut res) {
        Ok(_) => {
            let index = find_payload_index(&res).unwrap();
            Ok(res[index + 4..].to_vec())
        }
        Err(_) => Err(TrackerResponseError::ReadStream),
    }
}

fn parse_response(
    response: Vec<u8>,
    peer_id: [u8; 20],
) -> Result<TrackerResponse, TrackerResponseError> {
    let bencoded_dictionary = parser::parse(response).unwrap();

    TrackerResponse::new(bencoded_dictionary, peer_id).map_err(|_| TrackerResponseError::Parse)
}

fn find_payload_index(buffer: &[u8]) -> Option<usize> {
    buffer
        .windows(4)
        .enumerate()
        .find(|(_, w)| matches!(*w, b"\r\n\r\n"))
        .map(|(i, _)| i)
}

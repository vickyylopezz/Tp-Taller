use std::{
    io,
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream, ToSocketAddrs},
};

use log::{error, info};

use crate::threadpool;

use super::builder::APIBuilder;

/// An interface for handling incoming http connections
pub trait Handler: Clone + Send {
    fn handle_connection(&mut self, stream: TcpStream, endpoints: &APIBuilder<'static>);
}

/// Http server
pub struct HttpServer {
    listener: TcpListener,
    pool: threadpool::ThreadPool,
}

impl HttpServer {
    /// Creates a new HTTP server that listens at the specified socket
    /// address. Returns and error if an error occurs binding the
    /// listener to the socket
    pub fn new<A: ToSocketAddrs>(socket: A) -> Result<Self, io::Error> {
        let listener = TcpListener::bind(socket)?;
        // The specified value is set at compile time so it shouldn't fail
        let pool = threadpool::ThreadPool::new(5).unwrap();
        Ok(Self { listener, pool })
    }

    /// Spawns a new thread for each incoming connection, using the
    /// specified endpoints and handler.
    pub fn listen<H: 'static + Handler>(self, handler: Box<H>, endpoints: APIBuilder<'static>) {
        for result in self.listener.incoming() {
            let stream = match result {
                Ok(s) => s,
                Err(_) => {
                    error!("Couldn't establish connection");
                    continue;
                }
            };
            let connected_to = stream
                .peer_addr()
                .unwrap_or_else(|_| SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0));
            info!("Established connection to: {}", connected_to);
            let mut h = handler.clone();
            let api = endpoints.clone();
            self.pool.spawn(move || {
                h.handle_connection(stream, &api);
            });
        }
    }
}

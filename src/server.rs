use std::net::{SocketAddr, TcpListener};
use std::str::FromStr;
use std::thread;
use crate::error::Error;
use crate::connection::HttpConnection;

pub struct Server {
    socket: Option<TcpListener>,
    socket_addr: SocketAddr,
}

impl Server {
    pub fn new(addr: &str) -> Result<Self, Error> {
        let socket_addr = SocketAddr::from_str(addr)?;
        Ok(Server {
            socket: None,
            socket_addr
        })
    }

    pub fn run(&mut self) -> Result<(), Error> {
        self.socket = Some(TcpListener::bind(self.socket_addr)?);

        loop {
            let (stream, addr) = self.socket.as_ref().expect("socket is none").accept()?;
            
            thread::spawn(move || {
                HttpConnection::init(stream);

            });
        }
    }
}



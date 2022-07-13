use crate::connection::HttpConnection;
use crate::error::Error;
use crate::threadpool::ThreadPool;
use std::net::{SocketAddr, TcpListener};
use std::str::FromStr;

pub struct Server {
    socket: Option<TcpListener>,
    socket_addr: SocketAddr,
    threadpool: ThreadPool,
}

const NUM_THREADS: usize = 10;

impl Server {
    pub fn new(addr: &str) -> Result<Self, Error> {
        let socket_addr = SocketAddr::from_str(addr)?;
        Ok(Server {
            socket: None,
            socket_addr,
            threadpool: ThreadPool::new(NUM_THREADS),
        })
    }

    pub fn run(&mut self) -> Result<(), Error> {
        self.socket = Some(TcpListener::bind(self.socket_addr)?);

        loop {
            let (stream, addr) = self.socket.as_ref().expect("socket is none").accept()?;
            self.threadpool.execute(move || {
                HttpConnection::init(stream);
            });
        }
    }
}

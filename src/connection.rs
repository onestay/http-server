use crate::response;
use std::io::Read;
use std::net::{Shutdown, TcpStream};
use crate::response::HttpResponse;
use crate::util::HttpStatusCode;
use super::parser::Parser;
const BUFFER_SIZE: usize = 4096;

pub(crate) struct HttpConnection {
    buffer: [u8; BUFFER_SIZE],
    tcp_stream: TcpStream,
    parser: Parser,
}

impl HttpConnection {
    fn read_from_socket(mut self) {
        loop {
            match self.tcp_stream.read(&mut self.buffer) {
                Ok(bytes_read) => {
                    if bytes_read == 0 {
                        break;
                    }
                match self.parser.feed(&self.buffer[..bytes_read]){
                    Ok(res) => {
                        if res {
                            let request = self.parser.finish().unwrap();
                            let response = response::HttpResponse::ok();
                            response.send();
                            self.parser = Parser::new();
                        }
                    },
                    Err(e) => {
                        println!("{:?}", e);
                        self.tcp_stream.shutdown(Shutdown::Both).unwrap();
                        break;
                    }
                }
                }
                Err(err) => println!("Error reading from socket {}", err)
            }
        }
    }
    pub(crate) fn init(tcp_stream: TcpStream) {
        let conn = HttpConnection {
            buffer: [0; BUFFER_SIZE],
            tcp_stream,
            parser: Parser::new()
        };
        conn.read_from_socket();
    }

}

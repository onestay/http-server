use std::collections::HashMap;
use std::io::Write;
use std::net::TcpStream;
use std::io::Error as IoError;
use std::io::Read;
use std::fs::File;

use crate::util::HttpStatusCode;

pub(crate) struct HttpResponse {
    status: HttpStatusCode,
    headers: HashMap<String, String>,
}

const DEFAULT_HEADER_CAP: usize = 5;

impl HttpResponse {
    pub(crate) fn new(status: HttpStatusCode) -> Self {
        HttpResponse {
            status,
            headers: HashMap::with_capacity(DEFAULT_HEADER_CAP),
        }
    }

    pub(crate) fn ok() -> Self {
        HttpResponse::new(HttpStatusCode::OK)
    }

    pub(crate) fn not_found() -> Self {
        HttpResponse::new(HttpStatusCode::NotFound)
    }

    pub(crate) fn send(self) {}

    pub(crate) fn with_body<B: Body>(body: B) {}
}

pub(crate) trait Body {
    fn size(&self) -> Option<u64>;
    fn write(self, stream: &mut TcpStream) -> Result<(), IoError>;
}

impl Body for String {
    fn size(&self) -> Option<u64> {
        Some(self.len() as u64)
    }

    fn write(self, stream: &mut TcpStream) -> Result<(), IoError> {
        stream.write_all(self.as_bytes())
    }
}

impl Body for File {
    fn size(&self) -> Option<u64> {
        match self.metadata() {
            Ok(metadata) => Some(metadata.len()),
            Err(_) => None
        }
    }

    fn write(mut self, stream: &mut TcpStream) -> Result<(), IoError> {
        let mut buf: [u8; 4096] = [0; 4096];
        todo!()
    }

}
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::net::TcpStream;
use std::os::unix::prelude::AsRawFd;
use std::path::Path;

use bytes::Bytes;
use libc::{off_t, size_t};
use time::{format_description::well_known::Rfc2822, OffsetDateTime};

use crate::util::HttpStatusCode;

#[derive(Debug)]
pub(crate) struct HttpResponse<'a, B: IntoBody> {
    body: Option<B>,
    status_code: HttpStatusCode,
    version_minor: u8,
    version_major: u8,
    headers: HashMap<String, String>,
    tcp_stream: &'a TcpStream,
}

impl<'a, B: IntoBody> HttpResponse<'a, B> {
    pub(crate) fn new(tcp_stream: &'a TcpStream, status_code: HttpStatusCode) -> Self {
        HttpResponse {
            status_code,
            version_major: 1,
            version_minor: 1,
            headers: HashMap::new(),
            tcp_stream,
            body: None,
        }
    }

    // FIXME: this is very primitive
    pub(crate) fn set_header(&mut self, key: String, value: String) {
        self.headers.insert(key, value);
    }
    // sends the request with the body
    pub(crate) fn body(mut self, body: B) {
        self.body = Some(body);
        let req = self.format_request();
        let req = req.as_bytes();
        self.tcp_stream.write_all(req).unwrap();
        self.tcp_stream.write_all(&self.body.unwrap().into_bytes()).unwrap();
    }
    // this response function is kinda special since we don't set the body of HttpRequest since we use sendfile directly with the socket
    pub(crate) fn send_file(mut self, path: &Path) {
        println!("{:?}", path);
        match File::open(path.file_name().unwrap()) {
            Ok(file) => {
                let file_size = file.metadata().unwrap().len();
                self.headers.insert("content-length".to_string(), file_size.to_string());
                self.headers.insert("content-type".to_string(), "text/html".to_string());
                let in_fd = file.as_raw_fd();
                let out_fd = self.tcp_stream.as_raw_fd();
                let mut offset: off_t = 0;
                let req = self.format_request();
                self.tcp_stream.write_all(req.as_bytes()).unwrap();
                unsafe {
                    let result = libc::sendfile(out_fd, in_fd, &mut offset, file_size as size_t);
                    if result == -1 {
                        println!("Error with sendfile");
                    }
                }
            }
            Err(e) => {
                panic!("{}", e);
            }
        }
    }
    fn populate_headers(&mut self) {
        if let Some(body) = &self.body {
            self.headers.insert("content-length".to_string(), body.size().to_string());
            self.headers.insert("content type".to_string(), body.content_type().to_string());
        }
        self.headers.insert("server".to_string(), "http-server".to_string());
        let current_date_time = OffsetDateTime::now_utc().format(&Rfc2822).unwrap();
        self.headers.insert("date".to_string(), current_date_time);
        self.headers.insert("connection".to_string(), "close".to_string());
    }

    fn format_request(&mut self) -> String {
        self.populate_headers();
        let mut response = format!("HTTP/{}.{} {} \r\n", self.version_major, self.version_minor, self.status_code);
        for (key, value) in self.headers.iter() {
            response += &format!("{}: {}\r\n", key, value);
        }
        response += "\r\n";

        response
    }
}


impl IntoBody for String {
    fn size(&self) -> usize {
        self.len()
    }

    fn into_bytes(self) -> Bytes {
        Bytes::from(self)
    }

    fn content_type(&self) -> &'static str {
        "text/plain"
    }
}

impl IntoBody for &'static str {
    fn size(&self) -> usize {
        self.len()
    }

    fn into_bytes(self) -> Bytes {
        Bytes::from(self)
    }

    fn content_type(&self) -> &'static str {
        "text/plain"
    }
}

pub(crate) trait IntoBody {
    fn size(&self) -> usize;
    fn into_bytes(self) -> Bytes;
    fn content_type(&self) -> &'static str;
}

pub(crate) fn send_400_response(stream: &mut TcpStream) {
    let response = "HTTP/1.1 400\r\nConnection: close\r\n\r\n";
    stream.write_all(response.as_bytes()).unwrap();
}
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::net::TcpStream;
use std::os::unix::prelude::AsRawFd;
use std::path::{Path, PathBuf};

use bytes::Bytes;
use libc::{off_t, size_t};
use log::info;
use time::{format_description::well_known::Rfc2822, OffsetDateTime};
use crate::request::HttpRequest;

use crate::util::{HttpMethod, HttpStatusCode};

#[derive(Debug)]
pub(crate) struct HttpResponse<'a> {
    status_code: HttpStatusCode,
    version_minor: u8,
    version_major: u8,
    headers: HashMap<String, String>,
    tcp_stream: &'a mut TcpStream,
    request: &'a HttpRequest
}

impl<'a> HttpResponse<'a> {
    pub(crate) fn new(request: &'a HttpRequest, tcp_stream: &'a mut TcpStream) -> Self {
        HttpResponse {
            status_code: HttpStatusCode::OK,
            version_major: 1,
            version_minor: 1,
            headers: HashMap::new(),
            tcp_stream,
            request,
        }
    }

    pub(crate) fn send(mut self) {
        if self.request.method() != &HttpMethod::Get {
            self.status_code = HttpStatusCode::MethodNotAllowed;
            self.body("This server does not yet support methods other than GET");
            return;
        }

        self.send_file()
    }



    // FIXME: this is very primitive
    pub(crate) fn set_header(&mut self, key: String, value: String) {
        self.headers.insert(key, value);
    }
    // sends the request with the body
    pub(crate) fn body<B: IntoBody>(mut self, body: B) {
        let req = self.format_request(Some(&body));
        let req = req.as_bytes();
        self.tcp_stream.write_all(req).unwrap();
        self.log_request();
        body.write_to_socket(self.tcp_stream);
    }

    fn log_request(&mut self) {
        info!("{} \"{} {} HTTP/1.1\" {} {}", self.tcp_stream.peer_addr().unwrap(), self.request.method(), self.request.target(), self.status_code, self.headers.get("Content-Length").unwrap_or(&" ".to_string()));
    }

    // this response function is kinda special since we don't set the body of HttpRequest since we use sendfile directly with the socket
    pub(crate) fn send_file(mut self) {
        let mut path = PathBuf::from("static");
        path.push(self.request.target().strip_prefix("/").unwrap());

        if !path.exists() {
            self.status_code = HttpStatusCode::NotFound;
            self.body("File not found");
            return;
        }
        let req = self.format_request(Some(&path));
        self.tcp_stream.write_all(req.as_bytes()).unwrap();
        self.log_request();
        path.write_to_socket(self.tcp_stream);
    }
    fn populate_headers<B: IntoBody>(&mut self, body: Option<&B>) {
        if let Some(body) = body {
            self.headers.insert("Content-Length".to_string(), body.size().to_string());
            self.headers.insert("Content-Type".to_string(), body.content_type().to_string());
        }
        self.headers.insert("Server".to_string(), "http-server".to_string());
        let current_date_time = OffsetDateTime::now_utc().format(&Rfc2822).unwrap();
        self.headers.insert("Date".to_string(), current_date_time);
        self.headers.insert("Connection".to_string(), "close".to_string());
    }

    fn format_request<B: IntoBody>(&mut self, body: Option<&B>) -> String {
        self.populate_headers(body);
        let mut response = format!("HTTP/{}.{} {} \r\n", self.version_major, self.version_minor, self.status_code);
        for (key, value) in self.headers.iter() {
            response += &format!("{}: {}\r\n", key, value);
        }
        response += "\r\n";

        response
    }
}

impl IntoBody for std::path::PathBuf {
    fn size(&self) -> usize {
        self.metadata().unwrap().len() as usize
    }


    fn write_to_socket(self, stream: &mut TcpStream) {
        let file = File::open(self).unwrap();
        let in_fd = file.as_raw_fd();
        let out_fd = stream.as_raw_fd();
        let mut offset: off_t = 0;
        let size = file.metadata().unwrap().len();
        unsafe {
            let result = libc::sendfile(out_fd, in_fd, &mut offset,  size as size_t);
            if result == -1 {
                println!("Error with sendfile");
            }
        }

    }

    fn content_type(&self) -> &'static str {
        let extension = self.extension();
        if let Some(extension) = extension {
            if extension == "html" {
                return "text/html";
            }

            if extension == "js" {
                return "text/javascript";
            }

            if extension == "css" {
                return "text/css";
            }

            if extension == "jpg" || extension == "jpeg" {
                return "image/jpeg";
            }

            if extension == "png" {
                return "image/png";
            }
        }

        "application/octet-stream"
    }
}

impl IntoBody for String {
    fn size(&self) -> usize {
        self.len()
    }

    fn write_to_socket(self, _: &mut TcpStream) {
        todo!()
    }

    fn content_type(&self) -> &'static str {
        "text/plain"
    }
}

impl IntoBody for &'static str {
    fn size(&self) -> usize {
        self.len()
    }

    fn write_to_socket(self, stream: &mut TcpStream) {
        stream.write_all(self.as_bytes());
    }

    fn content_type(&self) -> &'static str {
        "text/plain"
    }
}

pub(crate) trait IntoBody {
    fn size(&self) -> usize;
    fn write_to_socket(self, _: &mut TcpStream);
    fn content_type(&self) -> &'static str;
}

pub(crate) fn send_400_response(stream: &mut TcpStream) {
    let response = "HTTP/1.1 400\r\nConnection: close\r\n\r\n";
    stream.write_all(response.as_bytes()).unwrap();
}
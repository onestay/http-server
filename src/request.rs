use std::collections::HashMap;
use std::hash::Hash;
use std::net::SocketAddr;
use crate::util::HttpMethod;

pub(crate) struct HttpRequest {
    src_addr: Option<SocketAddr>,
    target: String,
    headers: HashMap<String, String>,
    method: HttpMethod,
}

impl HttpRequest {
   pub(crate) fn new(target: String, headers: HashMap<String, String>, method: HttpMethod) -> Self {
       HttpRequest {
          src_addr: None,
           target,
           headers,
           method
       }
   }
    pub(crate) fn target(&self) -> &str {
        &self.target
    }

    pub(crate) fn header(&self, key: &str) -> Option<&String> {
       self.headers.get(key)
    }

    pub(crate) fn method(&self) -> &HttpMethod {
        &self.method
    }
}
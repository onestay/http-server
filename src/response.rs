use std::collections::HashMap;

use crate::util::HttpStatusCode;

pub(crate) struct HttpResponse<B> {
    status: HttpStatusCode,
    body: Option<B>,
    headers: HashMap<String, String>,
}

const DEFAULT_HEADER_CAP: usize = 5;

impl<B> HttpResponse<B> {
    pub(crate) fn new(status: HttpStatusCode) -> Self {
        HttpResponse {
            status,
            body: None,
            headers: HashMap::with_capacity(DEFAULT_HEADER_CAP),
        }
    }
    pub(crate) fn send(self) {}
}



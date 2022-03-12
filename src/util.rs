use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use lazy_static::lazy_static;

lazy_static! {
    pub(crate) static ref INVALID_TOKEN_CHARACTERS: HashSet<char> = {
        let mut m = HashSet::new();
        m.insert('(');
        m.insert(')');
        m.insert(',');
        m.insert('/');
        m.insert(':');
        m.insert(';');
        m.insert('<');
        m.insert('=');
        m.insert('>');
        m.insert('?');
        m.insert('@');
        m.insert('[');
        m.insert('\\');
        m.insert(']');
        m.insert('{');
        m.insert('}');
        m.insert('"');

        m
    };
}

// FIXME: add CONNECT
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum HttpMethod {
    Get,
    Put,
    Head,
    Post,
    Trace,
    Delete,
    Options,
    Invalid,
}

impl Display for HttpMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpMethod::Get => { write!(f, "GET") }
            HttpMethod::Put => { write!(f, "PUT") }
            HttpMethod::Head => { write!(f, "HEAD") }
            HttpMethod::Post => { write!(f, "POST") }
            HttpMethod::Trace => { write!(f, "TRACE") }
            HttpMethod::Delete => { write!(f, "DELETE") }
            HttpMethod::Options => { write!(f, "OPTIONS") }
            HttpMethod::Invalid => { write!(f, "INVALID") }
        }
    }
}

#[derive(Debug, PartialEq)]
#[allow(dead_code)]
#[non_exhaustive]
pub(crate) enum HttpStatusCode {
    Continue,
    SwitchingProtocols,
    OK,
    Accepted,
    NonAuthoritativeInformation,
    NoContent,
    NotModified,
    NotFound,
    InternalServerError,
}

impl Display for HttpStatusCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
       let status_code = match self {
           HttpStatusCode::Continue => 100,
           HttpStatusCode::SwitchingProtocols => 101,
           HttpStatusCode::OK => 200,
           HttpStatusCode::Accepted => 202,
           HttpStatusCode::NonAuthoritativeInformation => 203,
           HttpStatusCode::NoContent => 204,
           HttpStatusCode::NotModified => 304,
           HttpStatusCode::NotFound => 404,
           HttpStatusCode::InternalServerError => 500,
           
       };
        write!(f, "{}", status_code)
    }
}

pub(crate) const CR: char = '\r';
pub(crate) const LF: char = '\n';
pub(crate) const SP: char = ' ';

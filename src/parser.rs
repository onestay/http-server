use std::collections::HashMap;
use crate::util::*;
use crate::request::HttpRequest;

const INITIAL_TARGET_CAP: usize = 50;

#[derive(Debug)]
pub(crate) enum ParserError {
    InvalidMethod,
    InvalidVersion,
    ExpectedSpace(&'static str),
    UnexpectedChar(&'static str),
    NotReady,
}

#[derive(Debug, PartialEq)]
enum ReqLineState {
    Method,
    FirstSpaceBeforeUrl,
    SpacesBeforeUrl,
    InUrl,
    SpaceAfterUrl,
    H,
    HT,
    HTT,
    HTTP,
    Slash,
    VersionFirstMajor,
    VersionMajor,
    VersionFirstMinor,
    VersionMinor,
    AlmostDone,
}

#[derive(Debug, PartialEq)]
enum HeaderState {
    Name,
    OWSBeforeValue,
    FirstValue,
    Value,
    OWSAfterValue,
    HeadersAlmostDone,
    AlmostDone
}

#[derive(Debug, PartialEq)]
enum State {
    RequestLine,
    Header,
    Done,
}

pub(crate) struct Parser {
    method: String,
    header_map: HashMap<String, String>,
    current_header_name: String,
    current_header_value: String,
    state: State,
    req_line_state: ReqLineState,
    header_state: HeaderState,
    method_parsed: HttpMethod,
    request_target: String,
    http_version_major: u8,
    http_version_minor: u8,
}

impl Parser {
    pub(crate) fn new() -> Self {
        Parser {
            method: String::with_capacity(7),
            method_parsed: HttpMethod::Invalid,
            request_target: String::with_capacity(INITIAL_TARGET_CAP),
            http_version_major: 0,
            http_version_minor: 0,
            header_map: HashMap::new(),
            req_line_state: ReqLineState::Method,
            state: State::RequestLine,
            header_state: HeaderState::Name,
            current_header_name: String::with_capacity(INITIAL_TARGET_CAP),
            current_header_value: String::with_capacity(INITIAL_TARGET_CAP)
        }
    }

    fn is_token(ch: &char) -> bool {
        ch.is_ascii() && !INVALID_TOKEN_CHARACTERS.contains(ch)
    }

    // VCHAR (0x21-7E) or obs-text (0x80-FF)
    fn is_valid_field_content_char(ch: u8) -> bool {
        (0x21..=0x7E).contains(&ch) || ch >= 0x80
    }

    fn parse_request_line(&mut self, ch: char) -> Result<bool, ParserError> {
        match self.req_line_state {
            ReqLineState::Method => {
                self.method.push(ch);
                if self.method.len() == 3 {
                    if self.method == "GET" {
                        self.method_parsed = HttpMethod::Get;
                    } else if self.method == "PUT" {
                        self.method_parsed = HttpMethod::Put;
                    }
                } else if self.method.len() == 4 {
                    if self.method == "HEAD" {
                        self.method_parsed = HttpMethod::Head
                    } else if self.method == "POST" {
                        self.method_parsed = HttpMethod::Post
                    }
                } else if self.method.len() == 5 && self.method == "TRACE" {
                    self.method_parsed = HttpMethod::Trace
                } else if self.method.len() == 6 && self.method == "DELETE" {
                    self.method_parsed = HttpMethod::Delete
                } else if self.method.len() == 7 && self.method == "OPTIONS" {
                    self.method_parsed = HttpMethod::Options
                }

                if self.method_parsed != HttpMethod::Invalid {
                    self.req_line_state = ReqLineState::FirstSpaceBeforeUrl;
                } else if self.method.len() == 7 {
                    return Err(ParserError::InvalidMethod);
                }
            }
            ReqLineState::FirstSpaceBeforeUrl => {
                if ch != SP {
                    return Err(ParserError::ExpectedSpace("no space between METHOD and TARGET"));
                }
                self.req_line_state = ReqLineState::InUrl
            }
            ReqLineState::SpacesBeforeUrl => {}
            ReqLineState::InUrl => {
                if !ch.is_ascii_control() && ch != SP {
                    self.request_target.push(ch)
                } else if ch == SP {
                    self.req_line_state = ReqLineState::H;
                }
            }
            ReqLineState::SpaceAfterUrl => {}
            ReqLineState::H => {
                if ch != 'H' {
                    return Err(ParserError::UnexpectedChar("H after TARGET"));
                }
                self.req_line_state = ReqLineState::HT;
            }
            ReqLineState::HT => {
                if ch != 'T' {
                    return Err(ParserError::UnexpectedChar("T after H"));
                }
                self.req_line_state = ReqLineState::HTT;
            }
            ReqLineState::HTT => {
                if ch != 'T' {
                    return Err(ParserError::UnexpectedChar("T after T"));
                }
                self.req_line_state = ReqLineState::HTTP;
            }
            ReqLineState::HTTP => {
                if ch != 'P' {
                    return Err(ParserError::UnexpectedChar("P after T"));
                }
                self.req_line_state = ReqLineState::Slash;
            }
            ReqLineState::Slash => {
                if ch != '/' {
                    return Err(ParserError::UnexpectedChar("/ after P"));
                }
                self.req_line_state = ReqLineState::VersionFirstMajor;
            }
            ReqLineState::VersionFirstMajor => {
                if !ch.is_ascii_digit() {
                    return Err(ParserError::UnexpectedChar("DIGIT after P"));
                }
                self.http_version_major = ch as u8 - b'0';
                if self.http_version_major != 1 {
                    return Err(ParserError::InvalidVersion);
                }

                self.req_line_state = ReqLineState::VersionMajor;
            }
            ReqLineState::VersionMajor => {
                if ch == '.' {
                    self.req_line_state = ReqLineState::VersionFirstMinor;
                } else if ch.is_ascii_digit() {
                    self.http_version_major *= 10 + (ch as u8 - b'0');
                } else {
                    return Err(ParserError::UnexpectedChar("DIGIT after DIGIT"));
                }
            }
            ReqLineState::VersionFirstMinor => {
                if !ch.is_ascii_digit() {
                    return Err(ParserError::UnexpectedChar("VERSION MINOR after ."));
                }
                self.http_version_minor = ch as u8 - b'0';
                self.req_line_state = ReqLineState::VersionMinor;
            }
            ReqLineState::VersionMinor => {
                if ch == CR {
                    self.req_line_state = ReqLineState::AlmostDone;
                } else if ch.is_ascii_digit() {
                    self.http_version_minor *= 10 + (ch as u8 - b'0');
                } else {
                    return Err(ParserError::UnexpectedChar("DIGIT after DIGIT"));
                }
            }
            ReqLineState::AlmostDone => {
                if ch != LF {
                    return Err(ParserError::UnexpectedChar("LF after CR"));
                }
                println!("{} {} HTTP/{}.{}", self.method_parsed, self.request_target, self.http_version_major, self.http_version_minor);
                return Ok(true);
            }
        }

        Ok(false)
    }

    // FIXME: handle obs-fold
    // https://datatracker.ietf.org/doc/html/rfc7230#section-3.2
    fn parse_headers(&mut self, ch: char) -> Result<bool, ParserError> {
        match self.header_state {
            HeaderState::Name => {
                if ch == CR && self.current_header_name.is_empty() {
                    self.header_state = HeaderState::HeadersAlmostDone;
                } else if Parser::is_token(&ch) {
                    self.current_header_name.push(ch);
                } else if ch == ':' {
                    self.header_state = HeaderState::OWSBeforeValue;
                } else {
                    return Err(ParserError::UnexpectedChar("token expected for header field name"));
                }
            }
            HeaderState::OWSBeforeValue => {
                if ch == ' ' {
                    self.header_state = HeaderState::FirstValue;
                } else if Parser::is_valid_field_content_char(ch as u8) {
                    self.current_header_value.push(ch);
                    self.header_state = HeaderState::Value;
                } else {
                    return Err(ParserError::UnexpectedChar("in OWSBeforeValue"));
                }
            }
            HeaderState::FirstValue => {
                if Parser::is_valid_field_content_char(ch as u8) {
                    self.current_header_value.push(ch);
                    self.header_state = HeaderState::Value;
                }
            }
            HeaderState::Value => {
                // FIXME: handle OWS after field-value
                if ch == ' ' || ch == '\t' || Parser::is_valid_field_content_char(ch as u8) {
                    self.current_header_value.push(ch);
                } else if ch == CR {
                    self.header_state = HeaderState::AlmostDone;
                }
            }
            HeaderState::OWSAfterValue => {}
            // In this state we got one 'CR'
            HeaderState::AlmostDone => {
                if ch == LF {
                    // we got one complete header. Push it to the HeaderMap
                    //FIXME: find a way to not clone here
                    self.header_map.insert(self.current_header_name.to_ascii_lowercase(), self.current_header_value.to_ascii_lowercase());
                    self.current_header_value.clear();
                    self.current_header_name.clear();
                    self.header_state = HeaderState::Name;
                } else {
                    return Err(ParserError::UnexpectedChar("expected LF after CR"));
                }
            }
            // In this state we got one 'CR' from HeaderState::Name
            // meaning we already have one CRLF
            HeaderState::HeadersAlmostDone => {
                return if ch == LF {
                    Ok(true)
                } else {
                    Err(ParserError::UnexpectedChar("expected LF after CR"))
                }
            }
        }
        Ok(false)
    }

    pub(crate) fn feed(&mut self, buffer: &[u8]) -> Result<bool, ParserError> {
        for i in buffer.iter() {
            if !i.is_ascii() {
                panic!()
            }

            let ch = *i as char;

            match self.state {
                State::RequestLine => {
                    if self.parse_request_line(ch)? {
                        self.state = State::Header;
                    }
                }
                State::Header => {
                    if self.parse_headers(ch)? {
                        println!("{:?}", self.header_map);
                        self.state = State::Done;
                        return Ok(true);
                    }
                }
                // when we are done the data being fed is irrelevant
                State::Done => return Ok(true)
            }
        }

        Ok(false)
    }

    pub(crate) fn finish(self) -> Result<HttpRequest, ParserError> {
        if self.state != State::Done {
            return Err(ParserError::NotReady);
        }
        Ok(HttpRequest::new(self.request_target, self.header_map, self.method_parsed))
    }
}

mod test {
    use crate::parser::Parser;

    #[test]
    fn test_parse_headers_valid() {
        let mut parser = Parser::new();
        let headers = "hello: world\r\nhoware: y ou_doing\r\n\r\n43e6tygse";
        for c in headers.chars() {
            let res = parser.parse_headers(c).expect("shouldn't error");
            if res {
                break;
            }
        }
        assert_eq!(parser.header_map.len(), 2);
        assert_eq!(parser.header_map.get("hello").unwrap(), "world");
        assert_eq!(parser.header_map.get("howare").unwrap(), "y ou_doing");
    }

    #[test]
    #[should_panic(expected = "token expected for header field name")]
    fn test_parse_headers_invalid() {
        let mut parser = Parser::new();
        let headers = "in<v:alid\r\n\r\n";
        for c in headers.chars() {
            let res = parser.parse_headers(c).unwrap();
            if res {
                break;
            }
        }
    }
}
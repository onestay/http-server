use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid addr")]
    InvalidAddr(#[from] std::net::AddrParseError),
    #[error("io error")]
    IOError(#[from] std::io::Error)
}
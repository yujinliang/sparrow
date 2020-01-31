#![allow(dead_code)] 
use crate::mysql;

pub type FrontendResult<T> = std::result::Result<T, FrontendError>;

#[derive(Debug)]
pub enum FrontendError {
    Io(async_std::io::Error),
    ErrMysql(mysql::errors::MysqlError),
    ProxyAuthDenied,
    ProxyAuthOldinClientProtocol41,
}

impl From<async_std::io::Error> for FrontendError {
    fn from(e : async_std::io::Error) -> Self {
        FrontendError::Io(e)
    }
}

impl From<mysql::errors::MysqlError> for FrontendError {
    fn from(e : mysql::errors::MysqlError) -> Self {
        FrontendError::ErrMysql(e)
    }
}

impl std::error::Error for FrontendError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            FrontendError::ProxyAuthDenied => None,
            FrontendError::ProxyAuthOldinClientProtocol41 => None,
            FrontendError::Io(e) => e.source(),
            FrontendError::ErrMysql(e) => e.source(),
        }
    }
}
impl std::fmt::Display for FrontendError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {  
            match self {
                FrontendError::ProxyAuthDenied => write!(f, "proxy auth denied!"),
                FrontendError::ProxyAuthOldinClientProtocol41 => write!(f, "Too old than CapabilityFlags::CLIENT_PROTOCOL_41!"),
                FrontendError::Io(e) => e.fmt(f),
                FrontendError::ErrMysql(e) => e.fmt(f),
            }
        }
}
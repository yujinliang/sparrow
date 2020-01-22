#![allow(dead_code)] 

pub type ProxyResult<T> = std::result::Result<T, ProxyError>;

#[derive(Debug)]
pub enum ProxyError {
    Io(async_std::io::Error),
    Other(Box<dyn std::error::Error+Send+Sync>),
}

impl From<async_std::io::Error> for ProxyError {
    fn from(e : async_std::io::Error) -> Self {
        ProxyError::Io(e)
    }
}

impl std::error::Error for ProxyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ProxyError::Io(e) => e.source(),
            ProxyError::Other(e) => e.source(),
        }
    }
}
impl std::fmt::Display for ProxyError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {  
            match self {
                ProxyError::Io(e) => e.fmt(f),
                ProxyError::Other(e) => e.fmt(f),
            }
        }
}
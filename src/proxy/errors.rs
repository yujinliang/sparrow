#![allow(dead_code)] 
use failure::{Fail};

pub type ProxyResult<T> = std::result::Result<T, ProxyError>;

#[derive(Debug, Fail)]
pub enum ProxyError {

    #[fail(display = "async_std::io::Error: {}", other)]
    Io { other: async_std::io::Error,},

    //#[fail(display = "std::option::NoneError: {}", other)]
    //NoneErr { other: std::option::NoneError,}

}

impl From<async_std::io::Error> for ProxyError {
    fn from(e : async_std::io::Error) -> Self {
        ProxyError::Io{other:e}
    }
}

//impl From<std::option::NoneError> for ProxyError {
  //  fn from(e : std::option::NoneError) -> Self {
   //     ProxyError::NoneErr{other:e}
  //  }
//}

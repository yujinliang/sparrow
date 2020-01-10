#![allow(dead_code)] 
use failure::{Fail};

pub type MysqlResult<T> = std::result::Result<T, MysqlError>;

#[derive(Debug, Fail)]
pub enum MysqlError {
    #[fail(display = "invalid mysql packet header")]
    InvalidPacketHeader,

    #[fail(display = "mysql packet sequence mismatch")]
    MismatchPacketSequence,

    #[fail(display = "Incomplete mysql packet payload")]
    IncompletePacketPayload,

    #[fail(display = "mysql  zero payload")]
    PacketlZeroPayload,

    #[fail(display = "std::io::Error: {}", other)]
    Io { other: std::io::Error,}

}

impl From<std::io::Error> for MysqlError {
    fn from(e : std::io::Error) -> Self {
            MysqlError::Io{other:e}
    }
}
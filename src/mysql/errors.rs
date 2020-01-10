use failure::{Fail};

pub type MysqlResult<T> = std::result::Result<T, MysqlError>;

#[derive(Debug, Fail)]
pub enum MysqlError {
    #[fail(display = "invalid mysql packet header")]
    InvalidMysqlPacketHeader,

    #[fail(display = "invalid mysql packet payload len")]
    InvalidMysqlPacketPayloadLen,

    #[fail(display = "invalid mysql packet Sequence")]
    InvalidMysqlPacketSequence,

    #[fail(display = "invalid mysql packet payload")]
    InvalidMysqlPacketPayload,

    #[fail(display = "invalid mysql  zero payload")]
    ErrMysqlZeroPayload,

    #[fail(display = "std::io::Error: {}", other)]
    Io { other: std::io::Error,}

}

impl From<std::io::Error> for MysqlError {
    fn from(e : std::io::Error) -> Self {
            MysqlError::Io{other:e}
    }
}
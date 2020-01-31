#![allow(dead_code)] 

pub type MysqlResult<T> = std::result::Result<T, MysqlError>;

#[derive(Debug)]
pub enum MysqlError {
    MismatchPacketSequence,
    PacketlZeroPayload,
    OkPacketWrongSize,
    OkPacketILL,
    ErrPacketWrongSize,
    ErrPacketILL,
    ErUnknownCmd,
    Io (async_std::io::Error),
}

impl From<async_std::io::Error> for MysqlError {
    fn from(e : async_std::io::Error) -> Self {
            MysqlError::Io(e)
    }
}

impl std::error::Error for MysqlError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            MysqlError::MismatchPacketSequence =>  None,
            MysqlError::PacketlZeroPayload => None,
            MysqlError::OkPacketILL => None,
            MysqlError::ErUnknownCmd => None,
            MysqlError::OkPacketWrongSize => None,
            MysqlError::ErrPacketWrongSize => None,
            MysqlError::ErrPacketILL => None,
            MysqlError::Io(e) => e.source(),
        }
    }
}

impl std::fmt::Display for MysqlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {  
        match self {
            MysqlError::MismatchPacketSequence => write!(f, "MysqlError::MismatchPacketSequence!"),
            MysqlError::PacketlZeroPayload => write!(f, "MysqlError::PacketlZeroPayload!"),
            MysqlError::OkPacketWrongSize => write!(f, "MysqlError::OkPacketWrongSize!"),
            MysqlError::OkPacketILL => write!(f, "MysqlError::OkPacketILL!"),
            MysqlError::ErUnknownCmd => write!(f, "MysqlError::ErUnknownCmd!"),
            MysqlError::ErrPacketWrongSize => write!(f, "MysqlError::ErrPacketWrongSize!"),
            MysqlError::ErrPacketILL => write!(f, "MysqlError::ErrPacketILL!"),
            MysqlError::Io(e) => e.fmt(f),  
        }
    }
}
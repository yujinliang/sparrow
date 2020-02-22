#![allow(dead_code)] 
use crate::mysql::errors::MysqlError;

pub type BackendResult<T> = std::result::Result<T, BackendError>;

#[derive(Debug)]
pub enum BackendError {
    InnerErrPipeEmpty,
    InnerErrOfflineOrQuit,
    PoolErrClusterIdNotFound(String),
    PoolErrNodeNotFound(String),
    PoolErrConnGrowFailed(String),
    IO(async_std::io::Error),
    Mysql(MysqlError),
}

impl From<async_std::io::Error> for BackendError {
    fn from(e : async_std::io::Error) -> Self {
        BackendError::IO(e)
    }
}
impl From<MysqlError> for BackendError {
    fn from(e : MysqlError) -> Self {
        BackendError::Mysql(e)
    }
}
impl std::error::Error for BackendError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            BackendError::InnerErrPipeEmpty => None,
            BackendError::PoolErrClusterIdNotFound(..) => None,
            BackendError::PoolErrNodeNotFound(..) => None,
            BackendError::InnerErrOfflineOrQuit => None,
            BackendError::PoolErrConnGrowFailed(..) => None,
            BackendError::IO(e) => e.source(),
            BackendError::Mysql(e) => e.source(),
        }
    }
}
impl std::fmt::Display for BackendError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {  
            match self {
                BackendError::InnerErrPipeEmpty => write!(f, "node conn pipe is empty!"),
                BackendError::PoolErrClusterIdNotFound(id) => write!(f, "cluster_id: {:?} not exist!", id),
                BackendError::PoolErrNodeNotFound(id) => write!(f, "node: {:?}  not exist!", id),
                BackendError::InnerErrOfflineOrQuit => write!(f, "node offline or quit!"),
                BackendError::PoolErrConnGrowFailed(id) => write!(f, "node conn grow failed! node_id: {:?}", id),
                BackendError::IO(e) => e.fmt(f),
                BackendError::Mysql(e) => e.fmt(f),
            }
        }
}
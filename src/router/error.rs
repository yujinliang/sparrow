#![allow(dead_code)] 
#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Clone)]
pub enum RouterError{
    ShardSchemaParameterILL(String),
    ShardSchemaIntegerRangeILL(String),
    LookupErrShardValueEmpty,
    LookupErrClusterPairsEmpty,
    LookupErrShardValueILL(String),
    LookupErrNotInIntegerRange(String),
    LookupErrTableNotExist,
    LookupErrDBNotExist,
    LookupErrSchemaNotExit,
}

/*impl std::convert::From<NoneError> for ShardRouterError {

    fn from(e: NoneError) -> Self {
        ShardRouterError::ShardSchemaParameterNoneErr(e)
    }
}*/

impl std::fmt::Display for RouterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        
        match self {
            RouterError::LookupErrTableNotExist => write!(f, "RouterError::LookupErrTableNotExist"),
            RouterError::ShardSchemaParameterILL(s) => write!(f, "RouterError::ShardSchemaParameterILL: {}", s),
            RouterError::LookupErrDBNotExist => write!(f, "RouterError::LookupErrDBNotExist"),
            RouterError::LookupErrSchemaNotExit => write!(f, "RouterError::LookupErrSchemaNotExit"),
            RouterError::ShardSchemaIntegerRangeILL(s) => write!(f, "RouterError::ShardSchemaIntegerRangeILL: {}", s),
            RouterError::LookupErrShardValueEmpty => write!(f, "RouterError::LookupErrShardValueEmpty"),
            RouterError::LookupErrClusterPairsEmpty => write!(f, "RouterError::LookupErrClusterPairsEmpty"),
            RouterError::LookupErrShardValueILL(s) => write!(f, "RouterError::LookupErrShardValueILL: {}", s),
            RouterError::LookupErrNotInIntegerRange(s) => write!(f, "RouterError::LookupErrNotInIntegerRange: {}", s),
            //ShardRouterError::Other(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for RouterError {

    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RouterError::LookupErrSchemaNotExit => None,
            RouterError::LookupErrDBNotExist => None,
            RouterError::LookupErrTableNotExist => None,
            RouterError::ShardSchemaParameterILL(..) => None,
            RouterError::ShardSchemaIntegerRangeILL(..) => None,
            RouterError::LookupErrShardValueEmpty => None,
            RouterError::LookupErrClusterPairsEmpty => None,
            RouterError::LookupErrShardValueILL(..) => None,
            RouterError::LookupErrNotInIntegerRange(_) => None,
            //ShardRouterError::Other(e) => e.source(),
        }
    }

}
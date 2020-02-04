#![allow(dead_code)] 
#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Clone)]
pub enum RouterError{
    NoShardSchemaConfig,
    NoShardSchemaDBSectionConfig,
    ShardSchemaParameterILL(String),
    NoClusterConfig(String),
    NoNodeConfig(String),
    ShardSchemaIntegerRangeILL(String),
    LookupErrShardValueEmpty,
    LookupErrClusterPairsEmpty,
    LookupErrShardValueILL(String),
    LookupErrNotInIntegerRange(String),
}

/*impl std::convert::From<NoneError> for ShardRouterError {

    fn from(e: NoneError) -> Self {
        ShardRouterError::ShardSchemaParameterNoneErr(e)
    }
}*/

impl std::fmt::Display for RouterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        
        match self {
            RouterError::NoShardSchemaConfig =>  write!(f, "There is no database shard schema config!"),
            RouterError::NoShardSchemaDBSectionConfig =>  write!(f, "There is no db section in shard schema!"),
            RouterError::NoClusterConfig(s) => write!(f, "RouterError::NoClusterConfig: {}", s),
            RouterError::ShardSchemaParameterILL(s) => write!(f, "RouterError::ShardSchemaParameterILL: {}", s),
            RouterError::NoNodeConfig(s) => write!(f, "RouterError::NoNodeConfig: {}", s),
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
            RouterError::NoShardSchemaConfig =>  None,
            RouterError::NoShardSchemaDBSectionConfig =>  None,
            RouterError::NoClusterConfig(..) => None,
            RouterError::NoNodeConfig(..) => None,
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
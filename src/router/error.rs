
#[derive(Debug)]
pub enum ShardRouterError{

    NoConfig,
    NoShardSchemaConfig,
    ShardSchemaParameterILL(String),
    NoClusterConfig(String),
    NoNodeConfig(String),
    Other(Box<dyn std::error::Error+Send+Sync>),
}

/*impl std::convert::From<NoneError> for ShardRouterError {

    fn from(e: NoneError) -> Self {
        ShardRouterError::ShardSchemaParameterNoneErr(e)
    }
}*/

impl std::fmt::Display for ShardRouterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        
        match self {

            ShardRouterError::NoConfig => write!(f, "There is no config::Config to pass on!"),
            ShardRouterError::NoShardSchemaConfig =>  write!(f, "There is no database shard schema config!"),
            ShardRouterError::NoClusterConfig(s) => write!(f, "ShardRouterError::NoClusterConfig: {}", s),
            ShardRouterError::ShardSchemaParameterILL(s) => write!(f, "ShardRouterError::ShardSchemaParameterILL: {}", s),
            ShardRouterError::NoNodeConfig(s) => write!(f, "ShardRouterError::NoNodeConfig: {}", s),
            ShardRouterError::Other(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for ShardRouterError {

    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ShardRouterError::NoConfig => None,
            ShardRouterError::NoShardSchemaConfig =>  None,
            ShardRouterError::NoClusterConfig(..) => None,
            ShardRouterError::NoNodeConfig(..) => None,
            ShardRouterError::ShardSchemaParameterILL(..) => None,
            ShardRouterError::Other(e) => e.source(),

        }
    }

}
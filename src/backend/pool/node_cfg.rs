#[derive(Debug)]
pub struct NodeCfg {
        pub mysql_user: String,
        pub mysql_pwd:String,
        pub mysql_addr:String,
        pub cluster_id:String,
        pub node_id:String,
        pub max_conns_limit:u64,
        pub min_conns_limit:u16,
        pub grow_count: u16,
        pub shrink_count:u16,
        pub idle_time_to_shrink:u64,
        pub time_to_check_interval:u64,
        pub ping_retry_count: u8 ,
        pub ping_retry_interval: u64 , //time unit: second 
        pub reconnect_retry_count: u8 ,
        pub reconnect_retry_interval:u64 ,//time unit: second 
}
use serde::{ Deserialize};
use std::fs::File;
use std::io::prelude::*;
use std::error::Error;

/// This is what we're going to decode into. Each field is optional, meaning
/// that it doesn't have to be present in TOML.
#[derive(Debug, Deserialize)]
pub struct Config {
        global: Option<GlobalConfig>,
        proxy: Option<ProxyConfig>,
        web:Option<WebConfig>,
        db_node_list:Option<Vec<DBNodeConfig>>,
        db_cluster_list:Option<Vec<DBClusterConfig>>,
     pub    db_shard_schema_list:Option<Vec<DBShardSchemaConfig>>,
}

impl Config {
    pub fn get_db_cluster(&self, id : &str) -> Option<&DBClusterConfig> {

        for x in self.db_cluster_list.as_ref().unwrap().iter() {
            if x.id.as_ref().unwrap() == id {
                return Some(x);
            }
        }
        None
    }

    pub fn get_db_node(&self, id: &str) -> Option<&DBNodeConfig> {

        for x in self.db_node_list.as_ref().unwrap().iter() {
            if x.id.as_ref().unwrap() == id {
                return Some(x);
            }
        }
        None
    }
}

#[derive(Debug, Deserialize)]
struct GlobalConfig {
    log_path: Option<String>,
    log_level: Option<String>,
    log_slow_query_time: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct ProxyConfig {
    listen_addr: Option<String>,
    charset: Option<String>,
    proxy_users: Option<Vec<ProxyUser>>,
}

#[derive(Debug, Deserialize)]
struct ProxyUser {
    user: Option<String>,
    pwd: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WebConfig {
    listen_addr: Option<String>,
    web_user: Option<String>,
    web_pwd: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DBNodeConfig {
    id: Option<String>,
    listen_addr: Option<String>,
    user: Option<String>,
    pwd: Option<String>,
    time_to_no_alive: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct DBClusterConfig {
 pub    id: Option<String>,
 pub   master_node_id: Option<String>,
 pub   slave_node_id_list: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct DBShardSchemaConfig {
      pub  owner: Option<String>,
      pub  db: Option<String>,
      pub  table: Option<String>,
      pub  shard_key: Option<String>,
      pub  db_cluster_id_list: Option<Vec<String>>,
      pub  default_write_to: Option<String>,
      pub  shard_type:Option<String>,
      pub each_cluster_table_split_count: Option<Vec<u16>>,
        day_range:Option<Vec<String>>,
}

//fn definition start here.
pub fn load_config() -> Result<Config, Box<dyn Error>> {

    //1.find and read the config file.
    let config_path = std::env::args().nth(1).expect("Please at least give me the config file path.");
    let mut f = File::open(config_path).unwrap();
    let mut contents = String::new();
    f.read_to_string(&mut contents).unwrap();
  
    //2. parse the toml 
    let cfg: Config = toml::de::from_str(&contents).unwrap();
    Ok(cfg)

}


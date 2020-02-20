#![allow(dead_code)] 
use serde::{ Deserialize};
use std::fs::File;
use std::io::prelude::*;
use std::error::Error;
use super::schema::DBShardSchemaConfig;
use std::collections::HashMap;

pub mod constants {
    //min unit : second
    pub static DB_NODE_TIME_TO_NO_ALIVE: u64 = 1800; //10 minutes
}
/// This is what we're going to decode into. Each field is optional, meaning
/// that it doesn't have to be present in TOML.
#[derive(Debug, Deserialize)]
pub struct Config {
     pub   global: Option<GlobalConfig>,
     pub   proxy: ProxyConfig,
     pub   web:Option<WebConfig>,
     pub  node:Vec<DBNodeConfig>,
     pub  cluster:Vec<DBClusterConfig>,
     pub  schema:Vec<DBShardSchemaConfig>,
}

#[derive(Debug, Deserialize)]
pub struct GlobalConfig {
    log_path: Option<String>,
    log_level: Option<String>,
    log_slow_query_time: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct ProxyConfig {
    listen_addr: String,
    charset: Option<String>,
    users: Vec<ProxyUser>,
    time_to_no_alive: Option<u64>, //none or zero value is for unlimited.
}

#[derive(Debug, Deserialize)]
pub struct ProxyUser {
    user: String,
    pwd: String,
}

#[derive(Debug, Deserialize)]
pub struct WebConfig {
    listen_addr: String,
    web_user: String,
    web_pwd: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DBNodeConfig {
    id: String,
    listen_addr: String,
    user: String,
    pwd: String,
    max_conns_limit: Option<u64>, //none or zero value is for unlimited.
}

#[derive(Debug, Deserialize,Clone)]
pub struct DBClusterConfig {
     id: String,
    master_node_id: String,
    slave_node_ids: Option<Vec<String>>,
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

impl Config {

        #[inline]
        pub fn query_log_path(&self) -> Option<&str> {
                self.global.as_ref()?.log_path.as_deref()
        }
       // error/warn/info/debug/trace
        #[inline]
        pub fn query_log_level(&self) -> Option<log::LevelFilter> {
            self.global.as_ref()?.log_level.as_deref().map(|level| {
                match level.trim() {
                    "error" => log::LevelFilter::Error,
                    "warn" => log::LevelFilter::Warn,
                    "info" => log::LevelFilter::Info,
                    "debug" => log::LevelFilter::Debug,
                    _ => log::LevelFilter::Trace,
                }
            })
        }
        #[inline]
        pub fn query_proxy_listen_addr(&self) -> &str {
            &self.proxy.listen_addr
        }
        #[inline]
        pub fn load_proxy_user_list(&self) -> HashMap<String, String> {
               let user_map : HashMap<String, String> 
                                            = self.proxy
                                            .users
                                            .iter()
                                            .map(|pu| {
                                                let user = pu.user.trim().to_string();
                                                let pwd = pu.pwd.trim().to_string();
                                                (user, pwd)
                                            } )
                                            .collect();
                user_map
        }

        #[inline]
        pub fn load_db_cluster_config(&self) -> HashMap<String, DBClusterConfig> {
                 let cluster_map : HashMap<String, DBClusterConfig> 
                 = self.cluster
                .iter()
                .map(|cc| {
                     (cc.id.clone(), cc.clone())
                })
                .collect();
                cluster_map
        }
        #[inline]
        pub fn load_db_node_config(&self) -> HashMap<String, DBNodeConfig> {
                let node_map : HashMap<String, DBNodeConfig> 
                                              = self.node
                                              .iter()
                                              .map(|nc| {
                                                    (nc.id.clone(), nc.clone())
                                              })
                                              .collect();
                node_map
        }
             
} //end of impl Config


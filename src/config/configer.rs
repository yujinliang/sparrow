#![allow(dead_code)] 
use serde::{ Deserialize};
use std::fs::File;
use std::io::prelude::*;
use std::error::Error;
use super::schema::DBShardSchemaConfig;
use std::collections::HashMap;
/// This is what we're going to decode into. Each field is optional, meaning
/// that it doesn't have to be present in TOML.
#[derive(Debug, Deserialize)]
pub struct Config {
     pub   global: Option<GlobalConfig>,
     pub   proxy: Option<ProxyConfig>,
     pub   web:Option<WebConfig>,
     pub  node:Option<Vec<DBNodeConfig>>,
     pub  cluster:Option<Vec<DBClusterConfig>>,
     pub  schema:Option<Vec<DBShardSchemaConfig>>,
}

#[derive(Debug, Deserialize)]
pub struct GlobalConfig {
    log_path: Option<String>,
    log_level: Option<String>,
    log_slow_query_time: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct ProxyConfig {
    listen_addr: Option<String>,
    charset: Option<String>,
    users: Option<Vec<ProxyUser>>,
}

#[derive(Debug, Deserialize)]
pub struct ProxyUser {
    user: Option<String>,
    pwd: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct WebConfig {
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

#[derive(Debug, Deserialize,Clone)]
pub struct DBClusterConfig {
     id: Option<String>,
    master_node_id: Option<String>,
    slave_node_ids: Option<Vec<String>>,
}

pub fn empty() -> Config {
    Config {
        global: None,
        proxy: None,
        web:None,
        node:None,
        cluster:None,
        schema:None,
    }
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
        pub fn query_proxy_listen_addr(&self) -> Option<&str> {
            self.proxy.as_ref()?.listen_addr.as_deref()
        }
        #[inline]
        pub fn load_proxy_user_list(&self) -> Option<HashMap<String, String>> {
               let user_map : HashMap<String, String> 
                                            = self.proxy
                                            .as_ref()?
                                            .users
                                            .as_ref() ?
                                            .iter()
                                            .map(|pu| {
                                                let user = pu.user.clone().unwrap_or_default().trim().to_string();
                                                let pwd = pu.pwd.clone().unwrap_or_default().trim().to_string();
                                                (user, pwd)
                                            } )
                                            .collect();
                Some(user_map)
        }

        #[inline]
        pub fn load_db_cluster_config(&self) -> Option< HashMap<String, DBClusterConfig>>{
                 let cluster_map : HashMap<String, DBClusterConfig> 
                 = self.cluster
                 .as_ref()?
                .iter()
                .map(|cc| {
                    let id = cc.id.as_ref().unwrap().to_string();
                     (id, cc.clone())
                })
                .collect();
                Some(cluster_map)
        }
        #[inline]
        pub fn load_db_node_config(&self) -> Option< HashMap<String, DBNodeConfig>> {
                let node_map : HashMap<String, DBNodeConfig> 
                                              = self.node
                                              .as_ref()?
                                              .iter()
                                              .map(|nc| {
                                                    let id = nc.id.as_ref().unwrap().to_string();
                                                    (id, nc.clone())
                                              })
                                              .collect();
                Some(node_map)
        }
             
} //end of impl Config


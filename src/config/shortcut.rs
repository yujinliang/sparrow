#![allow(dead_code)] 
use super::configer::{DBClusterConfig, DBNodeConfig};
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct ConfigShortcut {
    //should use hashmap to replace vec for efficiency!
    proxy_user_list: HashMap<String, String>,
    node_list: HashMap<String, DBNodeConfig>,
    cluster_list: HashMap<String, DBClusterConfig>,
}

pub fn build_config_shortcut() -> Result<ConfigShortcut, Box<dyn Error>> {
    let csc = ConfigShortcut {
        proxy_user_list: crate::GLOBAL_CONFIG.load_proxy_user_list(),
        node_list: crate::GLOBAL_CONFIG.load_db_node_config(),
        cluster_list: crate::GLOBAL_CONFIG.load_db_cluster_config(),
    };
    Ok(csc)
}
impl ConfigShortcut {
    #[inline]
    pub fn check_proxy_user_exists(&self, user: &str) -> Option<(&String, &String)> {
            self.proxy_user_list.get_key_value(user)
    }

    #[inline]
    pub fn get_db_cluster_config(&self, id : &str) -> &DBClusterConfig {
        self.cluster_list.get(id).unwrap()
    }
    #[inline]
    pub fn get_db_node_config(&self, id: &str) -> &DBNodeConfig {
        self.node_list.get(id).unwrap()
    }
}
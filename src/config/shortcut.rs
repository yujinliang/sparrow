#![allow(dead_code)] 
use super::configer::{DBClusterConfig, DBNodeConfig};
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct ConfigShortcut {
    //should use hashmap to replace vec for efficiency!
    proxy_user_list: Option<HashMap<String, String>>,

}
pub fn empty() -> ConfigShortcut {
    ConfigShortcut {
        proxy_user_list: None,
    }
}
pub fn build_config_shortcut() -> Result<ConfigShortcut, Box<dyn Error>> {
    unimplemented!();
}
impl ConfigShortcut {
    #[inline]
    pub fn check_proxy_user_exists(&self, user: &str) -> Option<(String, String)> {
            //self.proxy_user_list.as_ref()?.iter().find(|(u, _p)| {
           //     u == user
            //}).cloned()
            unimplemented!();
    }

    #[inline]
    pub fn get_db_cluster_config(&self, id : &str) -> Option<&DBClusterConfig> {
        unimplemented!();
    }
    #[inline]
    pub fn get_db_node_config(&self, id: &str) -> Option<&DBNodeConfig> {
        unimplemented!();
    }
}
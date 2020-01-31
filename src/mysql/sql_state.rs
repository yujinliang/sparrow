use std::collections::HashMap;
use super::errcode::*;

pub const DEFAULT_MYSQL_STATE:&str = "HY000";

lazy_static::lazy_static! {
   pub  static ref MY_SQLSTATE: HashMap<u16, &'static str> = {
        let mut m = HashMap::new();
        m.insert(ER_HANDSHAKE_ERROR, "08S01");
        m
    };
}
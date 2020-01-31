use std::collections::HashMap;
use super::errcode::*;

lazy_static::lazy_static! {
    pub static ref MY_SQLERR_NAME: HashMap<u16, &'static str> = {
        let mut m = HashMap::new();
        m.insert(ER_HANDSHAKE_ERROR, "Bad handshake");
        m
    };
}
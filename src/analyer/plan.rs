#![allow(dead_code)]
use std::collections::HashMap;

pub struct Plan {
    db : String,
    force_master:bool,
    cluster_id_vs_sql: HashMap<String, String>, //sql :  generated for each cluster.
}


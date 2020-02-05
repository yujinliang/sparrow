use serde::{ Deserialize};

#[derive(Debug, Deserialize)]
pub struct DBShardSchemaConfig {
      pub  owner: String,
      pub db: Vec<DBSectionConfig>,
}

#[derive(Debug, Deserialize)]
pub struct DBSectionConfig {
    pub  db: String,
    pub  cluster_ids: Vec<String>,
    pub table:Vec<TableSectionConfig>,
}

#[derive(Debug, Deserialize)]
pub struct TableSectionConfig {
    pub  table: String,
    pub  shard_key: String,
    pub  shard_type:String,
    pub  each_cluster_table_split_count: Vec<u16>,
    pub  integer_range:Option<Vec<String>>,
}


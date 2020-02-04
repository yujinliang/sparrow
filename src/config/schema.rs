use serde::{ Deserialize};

#[derive(Debug, Deserialize)]
pub struct DBShardSchemaConfig {
      pub  owner: Option<String>,
      pub db: Option<Vec<DBSectionConfig>>,
}

#[derive(Debug, Deserialize)]
pub struct DBSectionConfig {
    pub  db: Option<String>,
    pub  cluster_ids: Option<Vec<String>>,
    pub table:Option<Vec<TableSectionConfig>>,
}

#[derive(Debug, Deserialize)]
pub struct TableSectionConfig {
    pub  table: Option<String>,
    pub  shard_key: Option<String>,
    pub  shard_type:Option<String>,
    pub  each_cluster_table_split_count: Option<Vec<u16>>,
    pub  integer_range:Option<Vec<String>>,
}


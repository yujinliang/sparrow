#![allow(dead_code)] 
#![allow(unused_variables)]
use std::result::Result;
use async_std::sync::Arc;
use super::error::RouterError;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::ops::Range;

#[derive(Debug)]
struct SchemaEntry<'a> {
    owner: String,
    //key :db name 
    db_entries: HashMap<&'a str, DBSectionEntry<'a>>,
}

#[derive(Debug, Clone)]
pub struct DBSectionEntry<'a> {
    pub  db: String,
    pub  cluster_ids: Vec<String>,
    pub tables:HashMap<&'a str,TableSectionEntry>,
}
#[derive(Debug, Clone)]
pub struct TableSectionEntry {
    pub  table: String,
    pub  shard_key: String,
    pub  shard_type : ShardType,
    pub  cluster_pairs: Vec<(String,u16)>, //Vec<(cluster_id , table_split_count)>
    pub  integer_range:Vec<Range<u128>>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ShardType {
    IntegerRange,
    Integer,
    Hash,
}

#[derive(Debug)]
pub struct Router<'a> {
    //key: proxy user , value: schema of the proxy.
    schema_map: HashMap<&'a str, SchemaEntry<'a>>, 
}
impl<'a> Router<'a> {
    //proxy user , db name
   pub fn lookup_db(&self, user:&str, db:&str) ->  Result<&'a DBSectionEntry, RouterError> {
        unimplemented!();
   }
}
impl<'a> DBSectionEntry<'a> {
    pub fn load_cluster_ids(&self) -> Result<&Vec<&str>, RouterError> {
        unimplemented!();
    }
    //the result: (cluster_id, table_name)
    pub fn lookup_table(&self, table:&str) -> Result<&TableSectionEntry, RouterError> {
        unimplemented!();
    } 
}
impl TableSectionEntry {
    //the result: (cluster_id, table_name ) list.
    pub fn load_all_path(&self) -> Result<Vec<(&str, String)>, RouterError> {
        unimplemented!();
    }
    //the result: (cluster_id, table_name)
    pub fn lookup_one_path(&self,  shard_val:&str) -> Result<(&str, String), RouterError> {
        if shard_val.trim().is_empty() {
            return Err(RouterError::LookupErrShardValueEmpty);
        }
        match self.shard_type {
            ShardType::Hash => {
                let cluster_sum  = self.cluster_pairs.len() as u64 ;
                if cluster_sum > 0 {
                    let mut s = DefaultHasher::new();
                    shard_val.hash(&mut s);
                    let shard_hash = s.finish(); //u64
                    let cluster_idx = (shard_hash % cluster_sum) as usize;
                   //table split count
                    let table_idx = shard_hash % self.cluster_pairs[cluster_idx].1 as u64;
                    let table_final_name = format!("{}_{}", self.table, table_idx);
                     return Ok((&self.cluster_pairs[cluster_idx].0, table_final_name));   
                } 
                Err(RouterError::LookupErrClusterPairsEmpty)
            },
            ShardType::Integer => {
                let cluster_sum  = self.cluster_pairs.len() as u128 ;
                if cluster_sum > 0 {
                    let shard_u128 = u128::from_str_radix( shard_val, 10).map_err(|e| {
                        RouterError::LookupErrShardValueILL(format!("illegal integer: {:?}", e))
                    })?;
                    let cluster_idx = (shard_u128 % cluster_sum) as usize;
                    let table_idx = shard_u128 % self.cluster_pairs[cluster_idx].1 as u128;
                    let table_final_name = format!("{}_{}", self.table, table_idx);
                    return Ok((&self.cluster_pairs[cluster_idx].0, table_final_name));   
                }
                Err(RouterError::LookupErrClusterPairsEmpty)
            },
            ShardType::IntegerRange => {  
                let cluster_sum  = self.cluster_pairs.len() as u128 ;
                if cluster_sum > 0 {
                        let shard_u128 = u128::from_str_radix( shard_val, 10).map_err(|e| {
                        RouterError::LookupErrShardValueILL(format!("illegal integer: {:?}", e))
                    })?;
                    for (pos, r) in self.integer_range.iter().enumerate() {
                        if r.contains(&shard_u128) {
                            let table_idx = shard_u128 % self.cluster_pairs[pos].1 as u128;
                            let table_final_name = format!("{}_{}", self.table, table_idx);
                            return Ok((&self.cluster_pairs[pos].0, table_final_name));   
                        }
                    }
                    return Err(RouterError::LookupErrNotInIntegerRange(format!("{:?} not in integer range", shard_val)));
                }
                Err(RouterError::LookupErrClusterPairsEmpty)
            },       
        }
    } 
}
//not allow panic, just return Error
pub fn build_router<'a>( ) -> Result<Arc<Router<'a>> , RouterError> {
         let schema_map 
         = crate::GLOBAL_CONFIG
          .schema
         .as_ref()
         .ok_or(RouterError::NoShardSchemaConfig)
         .and_then(|schema_list|{
                    let mut schema_map: HashMap<&'a str, SchemaEntry<'a>> = HashMap::new();
                    for schema in schema_list.iter() {
                            let db_entries 
                            = schema.db
                            .as_ref()
                            .ok_or(RouterError::NoShardSchemaDBSectionConfig)
                            .and_then(|db_list| {
                                        let mut db_entries: HashMap<&'a str, DBSectionEntry<'a>> = HashMap::new();
                                        for db in db_list.iter() {
                                                let db_name
                                                 = db.db
                                                .as_ref()
                                                .ok_or_else(|| RouterError::ShardSchemaParameterILL("db name empty in DBSectionConfig".to_string()))
                                                .and_then(|s| {
                                                    if s.trim().is_empty() {
                                                        return Err(RouterError::ShardSchemaParameterILL("db name empty in DBSectionConfig".to_string()));
                                                    } 
                                                    Ok(s)   
                                                })?;
                                                //--
                                                let cluster_ids 
                                                = db.cluster_ids
                                                    .as_ref()
                                                    .ok_or_else(|| RouterError::ShardSchemaParameterILL("no cluster id list  in DBSectionConfig".to_string()))
                                                    .and_then(|cluster_ids| {
                                                        if cluster_ids.is_empty() {
                                                            return Err(RouterError::ShardSchemaParameterILL("zero len cluster id list in DBSectionConfig".to_string()));
                                                        }
                                                        Ok(cluster_ids)
                                                    })?;
                                                    //--
                                                    let table_map
                                                    = db.table
                                                        .as_ref()
                                                        .ok_or_else(|| RouterError::ShardSchemaParameterILL("no table  in DBSectionConfig".to_string()))
                                                        .and_then(|table_list| {
                                                            if table_list.is_empty() {
                                                                return Err(RouterError::ShardSchemaParameterILL("zero len table list  in TableSectionConfig".to_string()));
                                                            }
                                                            let mut table_map:HashMap<&'a str,TableSectionEntry> = HashMap::new();
                                                            for table_sec in table_list.iter() {
                                                                //Todo:1. create TableSectionEntry
                                                               let table_name = if let Some(name) = &table_sec.table {
                                                                        if name.trim().is_empty() {
                                                                            return Err(RouterError::ShardSchemaParameterILL("table name is empty in TableSectionConfig".to_string()));
                                                                        }
                                                                        name
                                                                } else {
                                                                    return Err(RouterError::ShardSchemaParameterILL("table name is noting in TableSectionConfig".to_string()));
                                                                };
                                                                let shard_key = if let Some(key) = &table_sec.shard_key {
                                                                        if key.trim().is_empty() {
                                                                            return Err(RouterError::ShardSchemaParameterILL("shard key is empty in TableSectionConfig".to_string()));
                                                                        }
                                                                        key
                                                                } else {
                                                                    return Err(RouterError::ShardSchemaParameterILL("shard key is noting in TableSectionConfig".to_string()));
                                                                };
                                                                let shard_type:ShardType
                                                                                                  = table_sec
                                                                                                  .shard_type
                                                                                                  .as_ref()
                                                                                                  .ok_or_else(|| RouterError::ShardSchemaParameterILL("no shard_type  in DBSectionConfig".to_string()))
                                                                                                  .and_then(|s_type| {
                                                                                                        match s_type.trim() {
                                                                                                            "hash" => Ok(ShardType::Hash),
                                                                                                            "integer_range" => Ok(ShardType::IntegerRange),
                                                                                                            "integer" => Ok(ShardType::Integer),
                                                                                                            _ => { 
                                                                                                                Err(RouterError::ShardSchemaParameterILL("wrong shard type in TableSectionConfig".to_string()))
                                                                                                            },
                                                                                                        }
                                                                                                  })?;
                                                                let integer_range:Vec<Range<u128>> = if shard_type == ShardType::IntegerRange {
                                                                        table_sec
                                                                        .integer_range
                                                                        .as_ref()
                                                                        .ok_or_else(|| RouterError::ShardSchemaParameterILL("no integer_range  in TableSectionConfig".to_string()))
                                                                        .and_then(|i_range| {
                                                                                if i_range.is_empty() {
                                                                                    return Err(RouterError::ShardSchemaParameterILL("no integer_range  in TableSectionConfig".to_string()));
                                                                                }
                                                                                let mut v : Vec<Range<u128>> = Vec::new();                         
                                                                                let range_sum = i_range.len() / 2;
                                                                                let cluster_sum = cluster_ids.len();
                                                                                if range_sum != cluster_sum {
                                                                                    return Err(RouterError::ShardSchemaParameterILL("(integer_range.len() / 2) != cluster_ids.len()  in TableSectionConfig".to_string()));
                                                                                }
                                                                                let mut parsed_fail = false;
                                                                                for pair in i_range.chunks(2) {
                                                                                        let start = u128::from_str_radix( &pair[0], 10).unwrap_or_else(|_|{
                                                                                            parsed_fail = true;
                                                                                            0
                                                                                        });
                                                                                        let end = u128::from_str_radix( &pair[1], 10).unwrap_or_else(|_|{
                                                                                            parsed_fail = true;
                                                                                            0
                                                                                        });
                                                                                        if parsed_fail {
                                                                                            return Err(RouterError::ShardSchemaParameterILL("Illegal integer in integer_range in TableSectionConfig".to_string()));
                                                                                        }
                                                                                        if start > end {
                                                                                            return Err(RouterError::ShardSchemaParameterILL("Wrong case : start > end in integer_range".to_string()));
                                                                                        }
                                                                                        v.push(Range{
                                                                                                 start,
                                                                                                 end,
                                                                                        } );
                                                                                 }
                                                                                Ok(v)
                                                                        })?
                                                                } else {
                                                                    Vec::new()
                                                                };
                                                                let cluster_pairs: Vec<(String,u16)>
                                                                                                     = table_sec.each_cluster_table_split_count 
                                                                                                     .as_ref()
                                                                                                     .ok_or_else(|| RouterError::ShardSchemaParameterILL("no each_cluster_table_split_count in TableSectionConfig".to_string()))
                                                                                                     .and_then(|tsc| {
                                                                                                            if tsc.len() != cluster_ids.len() {
                                                                                                                return Err(RouterError::ShardSchemaParameterILL("Wrong case : each_cluster_table_split_count.len() != cluster_ids.len() in TableSectionConfig".to_string()));
                                                                                                            }
                                                                                                            let mut cparis : Vec<(String,u16)> = Vec::new();
                                                                                                            for (pos, val) in tsc.iter().enumerate() {
                                                                                                                    cparis.push((cluster_ids[pos].to_string(), *val));
                                                                                                            }
                                                                                                            Ok(cparis)
                                                                                                     })?;        
                                                                //2. insert it into tables hashmap
                                                                table_map
                                                                .entry(table_name)
                                                                .or_insert(TableSectionEntry{
                                                                    table: table_name.to_string(),
                                                                    shard_key: shard_key.to_string(),
                                                                    shard_type,
                                                                    cluster_pairs,
                                                                    integer_range,
                                                                });
                                                            }
                                                            Ok(table_map)
                                                        })?;
                                                        db_entries
                                                        .entry(db_name)
                                                        .or_insert(DBSectionEntry{db:db_name.to_string(), cluster_ids: cluster_ids.to_vec(), tables:table_map});
                                        }
                                        Ok(db_entries)
                                })?; 
                                 schema_map
                                .entry(&schema.owner.as_ref().unwrap())
                                .or_insert(SchemaEntry{owner: schema.owner.as_ref().unwrap().to_string(), db_entries});
                                  
                        }
                        Ok(schema_map)
         })?;
         Ok(Arc::new(Router{schema_map}))
      
}


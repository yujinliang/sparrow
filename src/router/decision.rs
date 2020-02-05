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
     db: String,
    cluster_ids: Vec<String>,
    tables:HashMap<&'a str,TableSectionEntry>,
}
#[derive(Debug, Clone)]
pub struct TableSectionEntry {
    table: String,
    shard_key: String,
    shard_type : ShardType,
    cluster_pairs: Vec<(String,u16)>, //Vec<(cluster_id , table_split_count)>
    integer_range:Vec<Range<u128>>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum ShardType {
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
    #[inline]
   pub fn lookup_db(&self, user:&str, db:&str) ->  Result<&'a DBSectionEntry, RouterError> {
       self.schema_map
              .get(user)
              .ok_or_else(|| { RouterError::LookupErrSchemaNotExit })
              .and_then(|schema| {
                  schema
                  .db_entries
                  .get(db)
                  .ok_or_else(|| {RouterError::LookupErrDBNotExist})
              })
   }
}
impl<'a> DBSectionEntry<'a> {
    #[inline]
    pub fn load_cluster_ids(&self) -> &Vec<String> {
        &self.cluster_ids
    }
    //the result: (cluster_id, table_name)
    #[inline]
    pub fn lookup_table(&self, table:&str) -> Result<&TableSectionEntry, RouterError> {
            self.tables.get(table).ok_or_else(||{ RouterError::LookupErrTableNotExist})
    } 
}
impl TableSectionEntry {
    #[inline]
    pub fn get_shard_key(&self) -> &str {
        &self.shard_key
    }
    //the result: (cluster_id, table_name ) list.
    #[inline]
    pub fn load_all_path(&self) -> Result<Vec<(&str, String)>, RouterError> {
        if self.cluster_pairs.is_empty() {
            return Err(RouterError::LookupErrClusterPairsEmpty);
        }
        let mut  v: Vec<(&str, String)> = Vec::new();
        for pair in self.cluster_pairs.iter() {
            if pair.1 > 1 {
                for pos in 0..pair.1 {
                    let table_final_name = format!("{}_{}", self.table, pos);
                    v.push((&pair.0, table_final_name));
                } 
            } else {
                v.push((&pair.0, self.table.clone()));
            }
        }
        Ok(v)
    }
    //the result: (cluster_id, table_name)
    #[inline]
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
                   let tsc = self.cluster_pairs[cluster_idx].1 as u64;
                   if tsc > 1 {
                        let table_idx = shard_hash % tsc;
                        let table_final_name = format!("{}_{}", self.table, table_idx);
                        return Ok((&self.cluster_pairs[cluster_idx].0, table_final_name));   
                   } else {
                       return Ok((&self.cluster_pairs[cluster_idx].0, self.table.clone()));
                   }
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
                    let tsc = self.cluster_pairs[cluster_idx].1 as u128;
                    if tsc > 1 {
                        let table_idx = shard_u128 % tsc;
                        let table_final_name = format!("{}_{}", self.table, table_idx);
                        return Ok((&self.cluster_pairs[cluster_idx].0, table_final_name));   
                    } else {
                        return Ok((&self.cluster_pairs[cluster_idx].0, self.table.clone()));
                    }
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
                            let tsc = self.cluster_pairs[pos].1 as u128;
                            if tsc > 1 {
                                let table_idx = shard_u128 % tsc;
                                let table_final_name = format!("{}_{}", self.table, table_idx);
                                return Ok((&self.cluster_pairs[pos].0, table_final_name));   
                            } else {
                                return Ok((&self.cluster_pairs[pos].0, self.table.clone()));
                            }
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
                    let mut schema_map: HashMap<&'a str, SchemaEntry<'a>> = HashMap::new();
                    for schema in crate::GLOBAL_CONFIG.schema.iter() {
                                        let mut db_entries: HashMap<&'a str, DBSectionEntry<'a>> = HashMap::new();
                                        for db in schema.db.iter() {
                                                let db_name
                                                = if !db.db.trim().is_empty() {
                                                    db.db.trim()
                                                    } else {
                                                        return Err(RouterError::ShardSchemaParameterILL("db name empty in DBSectionConfig".to_string()));
                                                    };
                                                //--
                                                let cluster_ids = if !db.cluster_ids.is_empty() {
                                                            &db.cluster_ids
                                                        } else {
                                                            return Err(RouterError::ShardSchemaParameterILL("cluster id list empty in DBSectionConfig".to_string()));
                                                        };
                                                    //--
                                                     if db.table.is_empty() {
                                                        return Err(RouterError::ShardSchemaParameterILL("zero len table list  in TableSectionConfig".to_string()));
                                                    }
                                                    let mut table_map:HashMap<&'a str,TableSectionEntry> = HashMap::new();
                                                    for table_sec in db.table.iter() {
                                                        //1. create TableSectionEntry
                                                               let table_name =  if !table_sec.table.trim().is_empty() {
                                                                        table_sec.table.trim()
                                                                } else {
                                                                    return Err(RouterError::ShardSchemaParameterILL("table name is empty in TableSectionConfig".to_string()));
                                                                };
                                                                //---
                                                                let shard_key =  if !table_sec.shard_key.trim().is_empty() {
                                                                             table_sec.shard_key.trim()
                                                                } else {
                                                                        return Err(RouterError::ShardSchemaParameterILL("shard key is empty in TableSectionConfig".to_string()));
                                                                 };
                                                                  
                                                                let shard_type:ShardType 
                                                                                                = match table_sec.shard_type.trim() {
                                                                                                            "hash" => ShardType::Hash,
                                                                                                            "integer_range" => ShardType::IntegerRange,
                                                                                                            "integer" => ShardType::Integer,
                                                                                                            _ => { 
                                                                                                              return   Err(RouterError::ShardSchemaParameterILL("wrong shard type in TableSectionConfig".to_string()))
                                                                                                            },
                                                                    };
                                                                                                  
                                                                let integer_range:Vec<Range<u128>> = if shard_type == ShardType::IntegerRange {
                                                                                let ir = table_sec.integer_range.as_ref().ok_or_else(|| {
                                                                                    RouterError::ShardSchemaParameterILL("no integer_range  in TableSectionConfig".to_string())
                                                                                })?;
                                                                                if  ir.is_empty() {
                                                                                    return Err(RouterError::ShardSchemaParameterILL(" integer_range is empty in TableSectionConfig".to_string()));
                                                                                }
                                                                                let mut v : Vec<Range<u128>> = Vec::new();                         
                                                                                let range_sum =  ir.len() / 2;
                                                                                let cluster_sum = cluster_ids.len();
                                                                                if range_sum != cluster_sum {
                                                                                    return Err(RouterError::ShardSchemaParameterILL("(integer_range.len() / 2) != cluster_ids.len()  in TableSectionConfig".to_string()));
                                                                                }
                                                                                let mut parsed_fail = false;
                                                                                for pair in  ir.chunks(2) {
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
                                                                                 v
                                                                } else {
                                                                    Vec::new()
                                                                };
                                                                let cluster_pairs: Vec<(String,u16)>
                                                                =  {
                                                                        if table_sec.each_cluster_table_split_count .len() != cluster_ids.len() {
                                                                                return Err(RouterError::ShardSchemaParameterILL("Wrong case : each_cluster_table_split_count.len() != cluster_ids.len() in TableSectionConfig".to_string()));
                                                                        }
                                                                        let mut cparis : Vec<(String,u16)> = Vec::new();
                                                                        for (pos, val) in table_sec.each_cluster_table_split_count .iter().enumerate() {
                                                                                cparis.push((cluster_ids[pos].to_string(), *val));
                                                                        }
                                                                        cparis
                                                                 };        
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
                                                db_entries
                                                .entry(db_name)
                                                .or_insert(DBSectionEntry{db:db_name.to_string(), cluster_ids: cluster_ids.to_vec(), tables:table_map});
                                        }
                                 schema_map
                                .entry(&schema.owner)
                                .or_insert(SchemaEntry{owner: schema.owner.clone(), db_entries});                   
                        }
         Ok(Arc::new(Router{schema_map}))     
}


#![allow(dead_code)] 
use crate::config::Config;
use crate::config::DBNodeConfig;
use std::result::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::ops::Range;
use super::error::ShardRouterError;

#[derive(Debug)]
pub struct Router {

    table : Box<RouterTable>,
}

impl Router {

    //key: proxy user + '-' + db + '-' + 'table' 
    pub fn find_routing_path(&self, proxy_user:&str, db:&str, table:&str, shard_key:&str) ->  Option< ( &DBCluster, String ) > {

        let key  = format!("{}-{}-{}",proxy_user, db, table );
        self.table.table_entry.get(&key).and_then(| entry | { entry.do_find_routing_path(shard_key)})
    }

    #[allow(dead_code)]
    pub fn find_all_routing_path(&self,proxy_user:&str, db:&str, table:&str) -> Option< Vec<( &DBCluster, String )> >  {

        let key  = format!("{}-{}-{}",proxy_user, db, table );
        self.table.table_entry.get(&key).and_then(| entry | { entry.do_get_all_routing_path()})
    }

}
//Attention: just allow to be called in main start flow, should panic!
pub   fn load_shard_router( config: & Config) -> Result<Arc<Router> ,ShardRouterError > {
  build_router(config)
}
//not allow panic, just return Error
pub fn build_router( cfg: & Config) -> Result<Arc<Router> , ShardRouterError> {

        if let Some(ref schema_vec) = cfg.db_shard_schema_list.as_ref() {
            let mut router_table = Box::new(RouterTable{table_entry:HashMap::new()});
            //-------------------------------------------------------------------------------------------------------------------
            for schema in schema_vec.iter() {
            //--
            let owner = schema.owner.as_ref()
                                                                  .ok_or_else( || ShardRouterError::ShardSchemaParameterILL("There is no owner in shard schema config!".to_string()))
                                                                 .and_then(|s|{ 
                                                                            if s.trim().is_empty() {
                                                                                     return Err(ShardRouterError::ShardSchemaParameterILL("There is a zero length of the owner in shard schema config!".to_string()));
                                                                             }
                                                                            Ok(s)
                                                                 })?;
            //--
            let db = schema.db.as_ref()
                                                    .ok_or_else( || ShardRouterError::ShardSchemaParameterILL("There is no db in shard schema config!".to_string()))
                                                    .and_then(|s|{ 
                                                            if s.trim().is_empty() {
                                                                    return Err(ShardRouterError::ShardSchemaParameterILL("There is a zero length of the db in shard schema config!".to_string()));
                                                            }
                                                         Ok(s)
                                                    })?;
            //---
            let table = schema.table.as_ref()
                                                             .ok_or_else( || ShardRouterError::ShardSchemaParameterILL("There is no table in shard schema config!".to_string()))
                                                            .and_then(|s|{ 
                                                                   if s.trim().is_empty() {
                                                                            return Err(ShardRouterError::ShardSchemaParameterILL("There is a zero length of the table in shard schema config!".to_string()));
                                                                     }
                                                                    Ok(s)
                                                            })?;
            //---
            let shard_key = schema.shard_key.as_ref()
                                                                                .ok_or_else( || ShardRouterError::ShardSchemaParameterILL("There is no shard_key in shard schema config!".to_string()))
                                                                                .and_then(|s|{ 
                                                                                             if s.trim().is_empty() {
                                                                                                    return Err(ShardRouterError::ShardSchemaParameterILL("There is a zero length of the shard_key in shard schema config!".to_string()));
                                                                                             }
                                                                                            Ok(s)
                                                                                 })?;     
                                                                                 
            //----
            let shard_type = schema.shard_type.as_ref()
                                                                                        .ok_or_else( || ShardRouterError::ShardSchemaParameterILL("There is no shard_type in shard schema config!".to_string()))
                                                                                        .and_then(|s|{ 
                                                                                                    if s.trim().is_empty() {
                                                                                                                return Err(ShardRouterError::ShardSchemaParameterILL("There is a zero length of the shard_type in shard schema config!".to_string()));
                                                                                                     }
                                                                                                     Ok(s)
                                                                                        })
                                                                                        .and_then(|s|{
                                                                                             match s.as_str().to_lowercase().trim() {
                                                                                                "hash" => Ok(ShardType::Hash),
                                                                                                "integer_range" => Ok(ShardType::IntegerRange),
                                                                                                "integer" => Ok(ShardType::Integer),
                                                                                                _ => {
                                                                                                            Err(ShardRouterError::ShardSchemaParameterILL("There is a wrong shard_type string in shard schema config!".to_string()))
                                                                                                        },
                                                                                            } 
                                                                                        })?;
            //-----
            let cluster_list = schema.db_cluster_id_list.as_ref()
                                                                                                    .ok_or_else(|| ShardRouterError::ShardSchemaParameterILL("There is no db cluster id list in shard schema config!".to_string()))
                                                                                                    .and_then(|cluster_id_vec|{
                                                                                                        let mut vec: Vec<DBCluster> = Vec::new();
                                                                                                        for (pos , s) in cluster_id_vec.iter().enumerate() {
                                                                                                            let ccfg = cfg.get_db_cluster(s).ok_or_else(|| ShardRouterError::NoClusterConfig(s.clone()))?;
                                                                                                            //---
                                                                                                            let table_split_count = schema.each_cluster_table_split_count.as_ref().map_or( 0, | tsc | {
                                                                                                                tsc[pos]
                                                                                                            });
                                                                                                            //---
                                                                                                            let mut slave_n_vec: Vec<DBNode> = Vec::new();
                                                                                                            ccfg.slave_node_id_list
                                                                                                                    .as_ref()
                                                                                                                    .ok_or_else(|| { ShardRouterError::NoNodeConfig("slave node list is empty!".to_string())})
                                                                                                                    .and_then(|slave_id_vec|{
                                                                                                                            for n in slave_id_vec.iter() {
                                                                                                                                cfg.get_db_node(n)
                                                                                                                                      .ok_or_else(|| ShardRouterError::NoNodeConfig(n.clone()))
                                                                                                                                      .and_then(|node_cfg|{
                                                                                                                                            slave_n_vec.push(DBNode{node_cfg: node_cfg.clone(),});
                                                                                                                                            Ok(())
                                                                                                                                      })?;
                                                                                                                            }
                                                                                                                          Ok(())
                                                                                                                    })?;
                                                                                                            //---
                                                                                                            let master_n = ccfg.master_node_id
                                                                                                                                                    .as_ref()
                                                                                                                                                    .ok_or_else(||{ShardRouterError::NoNodeConfig("master node id is emtpy".to_string())})
                                                                                                                                                    .and_then(|s|{
                                                                                                                                                        cfg.get_db_node(&s)
                                                                                                                                                               .ok_or_else(||{
                                                                                                                                                                    ShardRouterError::NoNodeConfig(s.clone())
                                                                                                                                                               })
                                                                                                                                                    })?;
                                                                                                            //--
                                                                                                            vec.push(DBCluster{
                                                                                                                id: s.clone(),
                                                                                                                cluster_table_split_count: table_split_count,
                                                                                                                slave_node_list: slave_n_vec,
                                                                                                                master_node: DBNode{ node_cfg:  master_n.clone()},
                                                                                                            });
                                                                                                        } 
                                                                                                        Ok(vec)
                                                                                                    })?;
                //---
                let key = format!("{}-{}-{}", owner, db, table);   
                let int_range = schema.integer_range
                                 .as_ref()
                                 .and_then(|i|{
                                                let mut v : Vec<Range<u128>> = Vec::new();                         
                                                let range_sum = i.len() / 2;
                                                let cluster_sum = schema.db_cluster_id_list.as_ref().map_or(0, |l|{l.len()});
                                                 if range_sum != cluster_sum {
                                                     return None;
                                                 }               

                                                 let mut parsed_fail = false;
                                                for pair in i.chunks(2) {
                                                        let start = u128::from_str_radix( &pair[0], 10).unwrap_or_else(|_|{
                                                            parsed_fail = true;
                                                            0
                                                        });
                                                        let end = u128::from_str_radix( &pair[1], 10).unwrap_or_else(|_|{
                                                            parsed_fail = true;
                                                            0
                                                        });
                                                        if parsed_fail {
                                                            return None;
                                                        }
                                                        v.push(Range{
                                                                 start,
                                                                 end,
                                                        } );
                                                 }
                                                 Some(v)                                                          
                                 });
                    if  shard_type == ShardType::IntegerRange &&  int_range.is_none() {
                        return Err(ShardRouterError::ShardSchemaIntegerRangeILL(key));
                    }
                    //-----
                    router_table.table_entry.insert(key, RouterTableEntry{
                        owner: owner.clone(),
                        db: db.clone(),
                        table: table.clone(),
                        shard_key: shard_key.clone(),
                        shard_type,        
                        cluster_list,
                        integer_range: int_range,
                     });
            }
            //------------------------------------------------------------------------------------------------------------------
            return Ok(Arc::new(Router{table: router_table}));
        }
        return Err(ShardRouterError::NoShardSchemaConfig);   

    }


#[derive(Debug)]
struct RouterTable {
    //key : proxy user +db + table
    table_entry: HashMap<String, RouterTableEntry>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum ShardType {
    IntegerRange,
    Integer,
    Hash,
}

#[derive(Debug)]
pub struct DBCluster{
    id: String,
    master_node: DBNode,
    slave_node_list: Vec<DBNode>,
    cluster_table_split_count: u16,
}
#[derive(Debug)]
pub struct DBNode {
    node_cfg: DBNodeConfig,
    //for extending for later.
}

#[derive(Debug)]
pub struct RouterTableEntry {
    owner: String,
    db: String,
    table: String,
    shard_key: String,
    shard_type:ShardType,
    cluster_list: Vec<DBCluster>,
    integer_range:Option<Vec<Range<u128>>>,
}

impl RouterTableEntry {

    #[inline]
    #[allow(dead_code)]
    fn do_get_all_routing_path(&self) -> Option< Vec<( &DBCluster, String )> >  {

      if self.cluster_list.is_empty() {
          return None;
      }
      let mut v : Vec<( &DBCluster, String )> = Vec::new();
      for c in self.cluster_list.iter() {   
        if c.cluster_table_split_count > 1 {
            for i in 0..c.cluster_table_split_count {
                let table_shard_name = format!("{}_{}", self.table, i);
                v.push((c, table_shard_name));
            }
        } 
        else {
             v.push((c, self.table.clone()));
        }

      }
      Some(v)
    }
    
    fn do_find_routing_path(&self, shard_key : &str) -> Option< ( &DBCluster, String ) > {

        match self.shard_type {

            ShardType::Hash => {

                let cluster_sum  = self.cluster_list.len() as u64 ;
                if cluster_sum > 0 {
                    let mut s = DefaultHasher::new();
                    shard_key.hash(&mut s);
                    let shard_hash = s.finish(); //u64
                    let cluster_idx = shard_hash % cluster_sum;
                    let cluster : &DBCluster = &self.cluster_list[cluster_idx as usize];
                    if cluster.cluster_table_split_count > 1 {
                        let table_idx = shard_hash % cluster.cluster_table_split_count as u64;
                        let table_final_name = format!("{}_{}", self.table, table_idx);
                        return Some(( cluster, table_final_name));
                    }
                    return Some(( cluster, self.table.clone()));
                } 
                None
            },
            ShardType::Integer => {

                let cluster_sum  = self.cluster_list.len() as u128 ;
                if cluster_sum > 0 {
                    let shard_u128 = u128::from_str_radix( shard_key, 10).unwrap_or_default();
                    let cluster_idx = shard_u128 % cluster_sum;
                    let cluster : &DBCluster = &self.cluster_list[cluster_idx as usize];
                    if cluster.cluster_table_split_count > 1 {
                        let table_idx = shard_u128 % cluster.cluster_table_split_count as u128;
                        let table_final_name = format!("{}_{}", self.table, table_idx);
                        return Some(( cluster, table_final_name));
                    }
                    return Some(( cluster, self.table.clone()));
                } 
                None
            },
            ShardType::IntegerRange => {
            
                if !self.cluster_list.is_empty(){ 
                    if let Some(v) = self.integer_range.as_ref() {
                            let shard_u128 = u128::from_str_radix( shard_key, 10).unwrap_or_default();
                            for (idx, r )in v.iter().enumerate() {
                                 if r.contains(&shard_u128) {
                                    let cluster : &DBCluster = &self.cluster_list[idx];
                                    if cluster.cluster_table_split_count > 1 {
                                        let table_idx = shard_u128 % cluster.cluster_table_split_count as u128;
                                        let table_final_name = format!("{}_{}", self.table, table_idx);
                                        return Some(( cluster, table_final_name));
                                    }
                                    else {
                                        return Some(( cluster, self.table.clone()));
                                    }
                                 }
                            }   
                    }
                    //not in range.
                    let cluster : &DBCluster = &self.cluster_list[0];
                     if cluster.cluster_table_split_count > 1 {
                            let table_final_name = format!("{}_{}", self.table, 0);
                            return Some(( cluster, table_final_name));
                    } 
                    return Some(( cluster, self.table.clone()));
                }
                None
            },
            
        }

    }
}


use crate::config::Config;
use crate::config::DBNodeConfig;
use std::result::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

#[derive(Debug)]
pub struct Router <'a>{

    cfg : &'a Config,
    table : Box<RouterTable>,
}

impl<'a> Router<'a> {

    //key: proxy user + '-' + db + '-' + 'table' 
    pub fn get_table_entry(&self, key : &str) -> Option<&RouterTableEntry> {

        self.table.table_entry.get(key)
    }
}
//Attention: just allow to be called in main start flow.
  pub   fn init_shard_router( c: Option<& Config>) -> Result<Arc<Router> ,String > {

      c.map(|cfg| {

            let mut table = Box::new(RouterTable{table_entry:HashMap::new()});
            for x in cfg.db_shard_schema_list.as_ref().unwrap().iter() {

                let key = format!("{}-{}-{}",x.owner.as_ref().unwrap() , x.db.as_ref().unwrap() ,x.table.as_ref().unwrap());
                
                table.table_entry.insert(key, RouterTableEntry{

                        owner: x.owner.as_ref().unwrap().clone(),
                        db: x.db.as_ref().unwrap().clone(),
                        table: x.table.as_ref().unwrap().clone(),
                        shard_key: x.shard_key.as_ref().unwrap().clone(),
                        shard_type: x.shard_type.as_ref().map(|s|{
                            match s.as_str().to_lowercase().trim() {
                                "hash" => ShardType::Hash,
                                "date_day" => ShardType::DateDay,
                                "ordinal_number" => ShardType::OrdinalNumber,
                                _ => ShardType::Unknown,
                            } 
                        }).unwrap(),
                        default_write_to:x.default_write_to.as_ref().unwrap().clone(),
                        cluster_list: x.db_cluster_id_list.as_ref().map(| cc |{
                            let mut vec: Vec<DBCluster> = Vec::new();
                            for (pos , s) in cc.iter().enumerate() {
                                let ccfg = cfg.get_db_cluster(s);

                                let table_split_count = x.each_cluster_table_split_count.as_ref().map_or( 0, | x | {
                                    x[pos]
                                });

                                let mut slave_n_vec: Vec<DBNode> = Vec::new();
                                for n in ccfg.as_ref().unwrap().slave_node_id_list.as_ref().unwrap().iter() {
                                    slave_n_vec.push(DBNode{node_cfg: cfg.get_db_node(n).unwrap().clone(),});
                                }
                                vec.push(DBCluster{
                                    id: s.clone(),
                                    cluster_table_split_count: table_split_count,
                                    slave_node_list: slave_n_vec,
                                    master_node: DBNode{ node_cfg:  cfg.get_db_node(ccfg.as_ref().unwrap().master_node_id.as_ref().unwrap()).unwrap().clone()},
                                });
                            } 
                            vec
                        }).unwrap(),
                });
            }

            Arc::new(Router{cfg: cfg, table: table})
        }).ok_or("build router table failed!".to_string())

    }


#[derive(Debug)]
struct RouterTable {
    //key : proxy user +db + table
    table_entry: HashMap<String, RouterTableEntry>,
}

#[derive(Debug, Copy, Clone)]
enum ShardType {
    DateDay,
    OrdinalNumber,
    Hash,
    Unknown,
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
    default_write_to: String,
    //day_range:Vec<String>,
}

impl RouterTableEntry {

pub fn get_default_cluster(&self) -> Option<&DBCluster> {

            for x in self.cluster_list.iter() {
                    if self.default_write_to == x.id {
                        return Some(x)
                    }
            }

            None
    }

    #[inline]
    pub fn get_all_cluster(&self) -> &Vec<DBCluster>  {

        &self.cluster_list
    }

    pub fn find_router_path(&self, shard_key : &str) -> Option< ( &DBCluster, String ) > {

        match self.shard_type {

            ShardType::Hash => {

                let mut s = DefaultHasher::new();
                shard_key.hash(&mut s);
                let shard_hash = s.finish(); //u64
                let cluster_sum  = self.cluster_list.len() as u64 ;
                if cluster_sum > 0 {
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
            ShardType::OrdinalNumber => {

                let shard_u128 = u128::from_str_radix( shard_key, 10).unwrap();
             
                let cluster_sum  = self.cluster_list.len() as u128 ;
                if cluster_sum > 0 {
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
            ShardType::DateDay => {
                None
            }
            _ =>  None,
        }

    }
}


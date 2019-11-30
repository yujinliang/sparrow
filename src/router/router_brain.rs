use crate::config::Config;
use crate::config::DBNodeConfig;
use std::result::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::ops::Range;

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
                                "integer_range" => ShardType::IntegerRange,
                                "integer" => ShardType::Integer,
                                _ => ShardType::Unknown,
                            } 
                        }).unwrap(),
                        
                        cluster_list: x.db_cluster_id_list.as_ref().map(| cc |{
                            let mut vec: Vec<DBCluster> = Vec::new();
                            for (pos , s) in cc.iter().enumerate() {
                                let ccfg = cfg.get_db_cluster(s);

                                let table_split_count = x.each_cluster_table_split_count.as_ref().map_or( 0, | tsc | {
                                    tsc[pos]
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

                        integer_range: x.integer_range.as_ref().map(|i|{
                            let mut v : Vec<Range<u128>> = Vec::new();
                            let range_sum = i.len() / 2;
                            x.db_cluster_id_list.as_ref().and_then(move |l| {
                                if range_sum != l.len() {
                                     return None
                                }
                                Some(range_sum)
                            }).unwrap();

                            for pair in i.chunks(2) {
                                let start = u128::from_str_radix( &pair[0], 10).unwrap();
                                let end = u128::from_str_radix( &pair[1], 10).unwrap();
                                v.push(Range{
                                     start: start,
                                     end: end,
                                } );
                            }
                            v
                        }),

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
    IntegerRange,
    Integer,
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
    integer_range:Option<Vec<Range<u128>>>,
}

impl RouterTableEntry {

    #[inline]
    pub fn get_default_cluster(&self) -> &DBCluster {

        &self.cluster_list[0]
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
            ShardType::Integer => {

                let shard_u128 = u128::from_str_radix( shard_key, 10).unwrap_or_default();
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
            ShardType::IntegerRange => {
            
                if self.cluster_list.len() > 0 {
                    let shard_u128 = u128::from_str_radix( shard_key, 10).unwrap_or_default();
                    if let Some(v) = self.integer_range.as_ref() {
                            for (idx, r )in v.iter().enumerate() {
                                 if r.contains(&shard_u128) {
                                         return Some((&self.cluster_list[idx as usize], self.table.clone()));
                                 }
                            }
                            return Some(( self.get_default_cluster(), self.table.clone()));
                    }
                }
                None
            },
            _ =>  None,
        }

    }
}


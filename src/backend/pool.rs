#![allow(dead_code)]
use super::conn::P2MConn;
use std::collections::{HashMap, LinkedList};
use super::error::{BackendResult, BackendError};
use async_std::sync::Arc;
pub mod inner;

struct P2MConnPool {
    //static const  relationship data
        node_conns:HashMap<String, Arc<inner::NodePipeLine>>,
        cluster_id_node_ids: HashMap<String, Vec<String>>, //Attention: the index:0 is always master node id forever!
}

impl P2MConnPool {
    pub async fn build_pool() -> BackendResult<P2MConnPool> {
        unimplemented!();
    }
    pub async fn get_conns(&self, cluster_ids: &[String], force_master:bool) -> BackendResult<LinkedList<P2MConn>> {
       /* let mut v : LinkedList<P2MConn> = LinkedList::new();
        let mut rng = thread_rng();
        for c_id in cluster_ids.iter() {
            let nodes = self.cluster_id_node_ids.get(c_id).ok_or_else(|| { BackendError::PoolErrClusterIdNotFound})?;
            let mut node_line = if force_master {   
                self.node_conns.get(&nodes[0]).ok_or_else(|| { BackendError::PoolErrNodeLineNotFound})?.lock().await
            } else if nodes.len() > 1 {
               let n_id =  nodes[1..].choose(&mut rng).ok_or_else(|| { BackendError::PoolErrNodeIdNotFound})?;
               let n_line = self.node_conns.get(n_id).ok_or_else(|| { BackendError::PoolErrNodeLineNotFound})?.lock().await;
               if n_line.offline {
                self.node_conns.get(&nodes[0]).ok_or_else(|| { BackendError::PoolErrNodeLineNotFound})?.lock().await
               } else {
                   n_line
               }
            } else {
                self.node_conns.get(&nodes[0]).ok_or_else(|| { BackendError::PoolErrNodeLineNotFound})?.lock().await
            };
            if node_line.offline {
                return Err(BackendError::InnerErrNodeOffline);
            }
            v.push_back(node_line.get_conn().await?);
        }
        Ok(v)*/
        unimplemented!();
    }
    //如果conn在使用时报错：网络中断，则不必归还，就地close就好。
    pub async fn recycle(&self, conn:P2MConn) {
        unimplemented!();
    }
    //the fn should be called by moniter mod.
    pub async fn reonline(&self, node_ids:&[String]) -> BackendResult<()> {
        /*
        * 1. get each node config
        * 2. create conn for each node, 成功者则标定node_offline为false
        * 完全成功则返回Ok, 只要有一个失败， 则返回Err, 错误信息中包含failed node id list; 当然记日志.
        */
        unimplemented!();
    }
    //the fn should be called by moniter mod.
    pub async fn force_offline(&self, node_ids:&[String]) -> BackendResult<()> {
        /*
        * 1. 标定node_id offline true， 
        * 2. 释放清楚此node_id下的所有conns
        */
        unimplemented!();
    }
    /*
    * 1.  shrink conns of the each node
    * 2. 
    */
   
}

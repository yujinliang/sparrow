#![allow(dead_code)]
use super::conn::P2MConn;
use std::collections::{HashMap, LinkedList};
use super::error::{BackendResult, BackendError};
use async_std::sync::Arc;
pub mod inner;
pub mod node;
pub mod node_cfg;

struct P2MConnPool {
    //static const  relationship data
        node_conns:HashMap<String, Arc<node::NodePipeLine>>,
        cluster_id_node_ids: HashMap<String, Vec<String>>, //Attention: the index:0 is always master node id forever!
}

impl P2MConnPool {
    pub async fn build_pool() -> BackendResult<P2MConnPool> {
        unimplemented!();
    }
    pub async fn get_conns(&self, cluster_ids: &[String], force_master:bool) -> BackendResult<LinkedList<P2MConn>> {
       let mut v : LinkedList<P2MConn> = LinkedList::new();
        for c_id in cluster_ids.iter() {
            let nodes = self.cluster_id_node_ids.get(c_id).ok_or_else(|| { BackendError::PoolErrClusterIdNotFound(c_id.to_string())})?;
            let node_line = if force_master {   
                self.node_conns.get(&nodes[0]).ok_or_else(|| { BackendError::PoolErrNodeNotFound(nodes[0].to_string())})?
            } else if nodes.len() > 1 {
               let n_id = &nodes[0];//rand to choose one in nodes.
               let n_line = self.node_conns.get(n_id).ok_or_else(|| { BackendError::PoolErrNodeNotFound(n_id.to_string())})?;
               if n_line.is_offline().await {
                self.node_conns.get(&nodes[0]).ok_or_else(|| { BackendError::PoolErrNodeNotFound(nodes[0].to_string())})?
               } else {
                   n_line
               }
            } else {
                self.node_conns.get(&nodes[0]).ok_or_else(|| { BackendError::PoolErrNodeNotFound(nodes[0].to_string())})?
            };
            if node_line.is_offline().await {
                return Err(BackendError::InnerErrOfflineOrQuit);
            }
            v.push_back(node_line.get_conn().await?);
        }
        Ok(v)     
    }
    pub async fn recycle(&self, conn:P2MConn) {
        if let Some(n) = self.node_conns.get(&conn.node_id) {
            n.recycle(conn).await;
        }
    }
    pub async fn reonline_node(&self, node_id:&str) -> BackendResult<()> {
        self.node_conns.get(node_id).ok_or_else(|| { BackendError::PoolErrNodeNotFound(node_id.to_string())})?.reonline().await?;    
        Ok(())
    }
    pub async fn offline_node(&self, node_id:&str) -> BackendResult<usize> {
        self.node_conns.get(node_id).ok_or_else(|| { BackendError::PoolErrNodeNotFound(node_id.to_string())})?.offline().await  
    }
    #[allow(unused_must_use)]
    pub async fn quit(&self) {
        for (_,n) in self.node_conns.iter() {
            n.quit().await;
        }
    }
   
}

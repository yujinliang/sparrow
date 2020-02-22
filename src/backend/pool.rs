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
       let mut v : LinkedList<P2MConn> = LinkedList::new();
        for c_id in cluster_ids.iter() {
            let nodes = self.cluster_id_node_ids.get(c_id).ok_or_else(|| { BackendError::PoolErrClusterIdNotFound})?;
            let node_line = if force_master {   
                self.node_conns.get(&nodes[0]).ok_or_else(|| { BackendError::PoolErrNodeLineNotFound})?
            } else if nodes.len() > 1 {
               let n_id = &nodes[0];//rand to choose one in nodes.
               let n_line = self.node_conns.get(n_id).ok_or_else(|| { BackendError::PoolErrNodeLineNotFound})?;
               if n_line.is_offline().await {
                self.node_conns.get(&nodes[0]).ok_or_else(|| { BackendError::PoolErrNodeLineNotFound})?
               } else {
                   n_line
               }
            } else {
                self.node_conns.get(&nodes[0]).ok_or_else(|| { BackendError::PoolErrNodeLineNotFound})?
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
    pub async fn reonline_node(&self, node_ids:&[String]) -> BackendResult<()> {
        for n_id in node_ids.iter() {
            self.node_conns.get(n_id).ok_or_else(|| { BackendError::PoolErrNodeLineNotFound})?.reonline().await;
        }
        Ok(())
    }
    pub async fn offline_node(&self, node_ids:&[String]) -> BackendResult<()> {
        for n_id in node_ids.iter() {
            self.node_conns.get(n_id).ok_or_else(|| { BackendError::PoolErrNodeLineNotFound})?.offline().await;
        }
        Ok(())
    }
    pub async fn quit(&self) {
        for (_,n) in self.node_conns.iter() {
            n.quit().await;
        }
    }
   
}

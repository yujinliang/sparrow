use log::info;
use async_std::task;
use std::time::Duration;
use async_std::net::TcpStream;
use async_std::task::JoinHandle;
use async_std::sync::Arc;
use std::collections::LinkedList;
use super::node::NodePipeLine;
use crate::backend::conn::P2MConn;
use super::{BackendResult};

pub async fn loop_check(receiver: &Arc<NodePipeLine>) { 
    let self_shared = receiver.clone();  
    task::spawn(async move {
        loop {
                if self_shared.is_quit().await {
                    return;
                }
                shrink_check(&self_shared).await ;
                health_check(&self_shared).await;
                task::sleep(Duration::from_secs(self_shared.cfg.time_to_check_interval)).await;
        }
    });
}
#[allow(unused_must_use)]
async fn health_check(receiver: &Arc<NodePipeLine>) {
        let self_shared = receiver.clone();  
        task::spawn(async move {
                //1. ping
                if let Ok(c) = self_shared.get_conn().await {
                        let mut ping_tick:u8 = 0;
                        while ping_tick < self_shared.cfg.ping_retry_count {
                            ping_tick += 1;
                            let p_r = c.ping().await;
                            if p_r.is_ok() {
                                   let rc = self_shared.reonline().await;
                                   info!("health_check, ping, reonline: {:?}", rc);
                                   self_shared.recycle(c).await;
                                   return;
                            }
                            task::sleep(Duration::from_secs(self_shared.cfg.ping_retry_interval)).await;
                        }
                        self_shared.discard(c).await;
                 } 
                 //2. reconnect
                let mut reconnect_tick:u8 = 0;
                while reconnect_tick < self_shared.cfg.reconnect_retry_count {
                        reconnect_tick += 1;
                        if let Ok(c) = create_conn(
                                &self_shared.cfg.mysql_user,
                                &self_shared.cfg.mysql_pwd,
                                 &self_shared.cfg.mysql_addr, 
                                &self_shared.cfg.cluster_id, 
                                &self_shared.cfg.node_id).await {
                                     let rc = self_shared.reonline().await;
                                     info!("health_check, reconnect, reonline: {:?}", rc); 
                                     self_shared.takeup(c).await;
                                    return;
                         } 
                        task::sleep(Duration::from_secs(self_shared.cfg.reconnect_retry_interval)).await;
                }
                 //Attention: if total conn count  == max conn limit , then give up  offline!!!
                 //coz: can not get/create conn to  ping and reconnect to test peer node `s alive!!!
                self_shared.offline_where(self_shared.cfg.max_conns_limit).await;
        });
}
#[allow(unused_must_use)]
async fn shrink_check(receiver: &Arc<NodePipeLine>) {
    receiver.inner.lock().await.shrink(receiver.cfg.idle_time_to_shrink, receiver.cfg.min_conns_limit, receiver.cfg.shrink_count).await;
}
//Must be No Lock!!!
async fn create_conn( user:&str,pwd:&str,addr:&str,c_id:&str,n_id:&str) -> BackendResult<P2MConn> {
    //1. tcp::connect to peer mysql . 
    let tcp = TcpStream::connect(addr).await?;         
    //2. create P2MConn
    let mut con_wrap:P2MConn = P2MConn::build_conn(
        tcp,
        user.to_string(),
        pwd.to_string(),
        addr.to_string(),
       c_id.to_string(),
       n_id.to_string()
    ).await?;
    //3. mysql handshake
    con_wrap.handshake().await?;
    //return conn or error
    Ok(con_wrap)
}
//Must be No Lock!!!
pub async fn grow(receiver: &Arc<NodePipeLine>, size:u16)  ->  LinkedList<P2MConn> {   
    let mut conns: LinkedList<P2MConn> = LinkedList::new();
    let mut tasks: Vec<JoinHandle<BackendResult<P2MConn>>> = Vec::new();
    for _ in 0..size {
       let user =  receiver.cfg.mysql_user.clone();
       let pwd =  receiver.cfg.mysql_pwd.clone();
       let addr = receiver.cfg.mysql_addr.clone();
       let c_id = receiver.cfg.cluster_id.clone();
       let n_id =  receiver.cfg.node_id.clone();
        tasks.push(task::spawn(async move {
            create_conn(&user, &pwd, &addr, &c_id, &n_id).await
        }));
    }
   for t in  tasks {
       match t.await {
           Ok(c) => conns.push_back(c),
           e => info!("create new mysql conn failed: {:?}", e),
       }
   }
   conns
}


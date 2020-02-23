use async_std::task;
use std::time::Duration;
use log::info;
use async_std::net::TcpStream;
use async_std::task::JoinHandle;
use async_std::sync::{Mutex, Arc};
use super::inner::InnerLine;
use super::node_cfg::NodeCfg;
use crate::backend::conn::P2MConn;
use super::{BackendError, BackendResult};
use std::collections::LinkedList;

#[derive(Debug)]
pub struct NodePipeLine {
        //dynamic data
        inner: Mutex<InnerLine>,
        //static config data 
        pub cfg: NodeCfg,
}

impl NodePipeLine {
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub async fn new( cfg: NodeCfg) -> Arc<Self> {
        Arc::new(NodePipeLine{
            inner: Mutex::new(InnerLine::new().await),
            cfg,
        })
    }
    pub async fn init(self: &Arc<Self>)  {
        let self_shared = self.clone();
        task::spawn(async move {
            let mut conn_list = self_shared.grow(self_shared.cfg.min_conns_limit).await;
             self_shared.inner.lock().await.takeup_batch(&mut conn_list).await;
        });
        self.loop_check().await;
    }
    pub async fn get_conn(self: &Arc<Self>) -> BackendResult<P2MConn> {
       let mut l = self.inner.lock().await;
       if l.is_offline().await { 
           return Err(BackendError::InnerErrOfflineOrQuit);
       }
       l.update_time_stamp().await;
       if l.get_cache_size().await > self.cfg.min_conns_limit.into() {
            l.lend_conn().await
       } else if l.total_conn_count < self.cfg.max_conns_limit {
                let grow_c = if (l.total_conn_count + self.cfg.grow_count as u64) <= self.cfg.max_conns_limit {
                    self.cfg.grow_count
                } else {
                    (l.total_conn_count + self.cfg.grow_count as u64 - self.cfg.max_conns_limit) as u16
                };
                 let mut conn_list = self.grow(grow_c).await;
                 l.takeup_batch(&mut conn_list).await;
                 l.lend_conn().await
        } else  {
                 Err(BackendError::InnerErrPipeEmpty)
                 //fast fail!
            }
    }
    #[allow(unused_must_use)]
    pub async fn recycle(self: &Arc<Self>, conn:P2MConn) {
        let mut l = self.inner.lock().await;
        if l.is_offline().await {
            l.discard(conn).await;
            return;
        }
        l.send_back(conn).await;
        if l.get_cache_size().await > self.cfg.max_conns_limit {
            l.eliminate(1).await;
        }
    }
    async fn discard(self: &Arc<Self>, conn:P2MConn) {
        self.inner.lock().await.discard(conn).await;
    }
    async fn takeup(self: &Arc<Self>, conn:P2MConn) {
        let mut l = self.inner.lock().await;
        if !l.is_offline().await {
            l.takeup(conn).await;
            return;
        } else {
            conn.quit();
        }
    }
    pub async fn offline(self: &Arc<Self>) {
        self.inner.lock().await.offline().await
  
    }
    pub async fn reonline(self: &Arc<Self>) -> BackendResult<usize>{
        let mut conn_list = self.grow(self.cfg.min_conns_limit).await;
        let l_size = conn_list.len();
        if l_size == 0 {
            return Err(BackendError::PoolErrConnGrowFailed(self.cfg.node_id.clone()));
        }
        self.inner.lock().await.reonline_with(&mut conn_list).await
   }
   pub async fn is_offline(self: &Arc<Self>) -> bool {
        self.inner.lock().await.is_offline().await
   }
   pub async fn is_quit(self: &Arc<Self>) -> bool {
    self.inner.lock().await.is_quit().await
}
   pub async fn quit(self: &Arc<Self>) {
        self.inner.lock().await.quit().await
    }
    async fn grow(self: &Arc<Self>, size:u16)  ->  LinkedList<P2MConn> {   
            let mut conns: LinkedList<P2MConn> = LinkedList::new();
            let mut tasks: Vec<JoinHandle<BackendResult<P2MConn>>> = Vec::new();
            for _ in 0..size {
               let user =  self.cfg.mysql_user.clone();
               let pwd =  self.cfg.mysql_pwd.clone();
               let addr = self.cfg.mysql_addr.clone();
               let c_id = self.cfg.cluster_id.clone();
               let n_id =  self.cfg.node_id.clone();
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
async fn loop_check(self: &Arc<Self>) { 
        let self_shared = self.clone();  
        task::spawn(async move {
            loop {
                     if !shrink_or_quit_check(&self_shared).await {
                             health_check(&self_shared).await;
                     } else {
                         return;
                     }
                    task::sleep(Duration::from_secs(self_shared.cfg.time_to_check_interval)).await;
            }
        });
    }
}
async fn health_check(receiver: &Arc<NodePipeLine>) {
        let self_shared = receiver.clone();  
        task::spawn(async move {
                //0. check if node is quited, if yes , then give up run.
                if self_shared.is_quit().await {
                    return; 
                }
                //1. ping
                if let Ok(c) = self_shared.get_conn().await {
                        let mut ping_tick:u8 = 0;
                        while ping_tick < self_shared.cfg.ping_retry_count {
                            ping_tick += 1;
                            let p_r = c.ping().await;
                            if p_r.is_ok() {
                                self_shared.recycle(c).await;
                                if self_shared.is_offline().await {
                                   let rc = self_shared.reonline().await;
                                   info!("health_check, ping, reonline: {:?}", rc);
                                }
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
                                   self_shared.takeup(c).await;
                                    if self_shared.is_offline().await {
                                        let rc = self_shared.reonline().await;
                                        info!("health_check, reconnect, reonline: {:?}", rc);
                                     }
                                    return;
                         } 
                        task::sleep(Duration::from_secs(self_shared.cfg.reconnect_retry_interval)).await;
                }
                self_shared.offline().await;
        });
}
#[allow(unused_must_use)]
async fn shrink_or_quit_check(receiver: &Arc<NodePipeLine>) -> bool {
    let mut l = receiver.inner.lock().await;
    let decision = l.whether_to_start_shrink(receiver.cfg.idle_time_to_shrink, receiver.cfg.min_conns_limit, receiver.cfg.shrink_count).await;
    if !decision.0 {
        return l.quit;
    }
    l.eliminate(decision.1).await;  
    l.quit
}
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
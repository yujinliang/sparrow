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
             self_shared.inner.lock().await.takeup_new_conn(&mut conn_list).await;
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
                 l.takeup_new_conn(&mut conn_list).await;
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
            l.discard_one(conn).await;
            return;
        }
        l.send_back(conn).await;
        if l.get_cache_size().await > self.cfg.max_conns_limit {
            l.eliminate(1).await;
        }
    }
    pub async fn offline(self: &Arc<Self>) {
        self.inner.lock().await.offline().await
  
    }
    pub async fn reonline(self: &Arc<Self>) -> BackendResult<u16>{
        let mut conn_list = self.grow(self.cfg.min_conns_limit).await;
        let l_size = conn_list.len();
        if l_size == 0 {
            return Err(BackendError::PoolErrConnGrowFailed(self.cfg.node_id.clone()));
        }
        self.inner.lock().await.reonline_with(&mut conn_list).await;
        Ok(l_size as u16)
   }
   pub async fn is_offline(self: &Arc<Self>) -> bool {
        self.inner.lock().await.is_offline().await
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
                //1. tcp::connect to peer mysql . 
                let tcp = TcpStream::connect(addr.clone()).await?;         
                //2. create P2MConn
                let mut con_wrap:P2MConn = P2MConn::build_conn(
                    tcp,
                    user,
                    pwd,
                    addr,
                    c_id,
                    n_id
                ).await?;
                //3. mysql handshake
                con_wrap.handshake().await?;
                //return conn or error
                Ok(con_wrap)
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
        info!("{:?}", self_shared);
                /*3. ping one conn, if failed then shutdown and discard it 
            * or to spawn a task to ping and reconnected mysql node , if failed then node offline
            * if 同时ping所有conn, 代价太大，　所以只挑选一个conn  to check;　需要关闭mysql wait_time 或设置尽可能长，避免conn频繁失效。
            * 如果ping 失败，　则　reconnect, 　间隔reconnect 3次，　认定node offline , discard all conns !
            *  但是loop_check会一直运行知道退出！　当cache为空时，　其新建一个conn, 用于探测peer node 是否重新上线。
            * if 重新上线，　则自动新建min_conns_limit个conn补充进cache!
            * ping retry,  reconnect retry, sleep some time , finally, to be considered node offline.
            */ 
        });
}
#[allow(unused_must_use)]
async fn shrink_or_quit_check(receiver: &Arc<NodePipeLine>) -> bool {
    let mut l = receiver.inner.lock().await;
    if l.quit {
        return l.quit;
    }
    let decision = l.whether_to_start_shrink(receiver.cfg.idle_time_to_shrink, receiver.cfg.min_conns_limit, receiver.cfg.shrink_count).await;
    if !decision.0 {
        return l.quit;
    }
    l.eliminate(decision.1).await;  
    l.quit
}
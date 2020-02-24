use async_std::task;
use async_std::sync::{Mutex, Arc};
use super::inner::InnerLine;
use super::node_cfg::NodeCfg;
use crate::backend::conn::P2MConn;
use super::{BackendError, BackendResult};
use std::sync::atomic::{AtomicBool, Ordering}; //should async???
use super::checker::*;

#[derive(Debug)]
pub struct NodePipeLine {
        //dynamic data
        pub inner: Mutex<InnerLine>,
        offline:AtomicBool, 
        quit:AtomicBool,
        //static config data 
        pub cfg: NodeCfg,
}

impl NodePipeLine {
    #[inline]
    pub async fn new( cfg: NodeCfg) -> Arc<Self> {
        Arc::new(NodePipeLine{
            inner: Mutex::new(InnerLine::new().await),
            cfg,
            offline: AtomicBool::new(false),
            quit: AtomicBool::new(false),
        })
    }
    #[inline]
    pub async fn init(self: &Arc<Self>)  {
        let self_shared = self.clone();
        task::spawn(async move {
            let mut conn_list = grow(&self_shared, self_shared.cfg.min_conns_limit).await;
             self_shared.inner.lock().await.takeup_batch(&mut conn_list).await;
        });
        loop_check(&self).await;
    }
    pub async fn get_conn(self: &Arc<Self>) -> BackendResult<P2MConn> {
       if self.is_offline().await { 
           return Err(BackendError::InnerErrOfflineOrQuit);
       }
       let mut l = self.inner.lock().await;
       l.update_time_stamp().await;
       if l.get_cache_size().await > self.cfg.min_conns_limit.into() {
            l.lend_conn().await
       } else if l.total_conn_count < self.cfg.max_conns_limit {
                let grow_c = if (l.total_conn_count + self.cfg.grow_count as u64) <= self.cfg.max_conns_limit {
                    self.cfg.grow_count
                } else {
                    (l.total_conn_count + self.cfg.grow_count as u64 - self.cfg.max_conns_limit) as u16
                };
                 let mut conn_list = grow(&self, grow_c).await;
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
        if self.is_offline().await {
            l.discard(conn).await;
            return;
        }
        l.send_back(conn).await;
        if l.get_cache_size().await > self.cfg.max_conns_limit {
            l.eliminate(1).await;
        }
    }
    pub async fn reonline(self: &Arc<Self>) -> BackendResult<usize>{
        if self.is_quit().await {
            return Err(BackendError::InnerErrOfflineOrQuit);
        }
        let mut l_size: usize = 0;
        if self.is_offline().await {
            let mut conn_list = grow(&self, self.cfg.min_conns_limit).await;
            l_size = conn_list.len();
            if l_size == 0 {
                return Err(BackendError::PoolErrConnGrowFailed(self.cfg.node_id.clone()));
            }
            self.inner.lock().await.reonline_with(&mut conn_list).await;
            self.offline.store(false, Ordering::Relaxed);
        }
        Ok(l_size)
   }
   pub async fn offline(self: &Arc<Self>) -> BackendResult<usize> {
    self.offline.store(true, Ordering::Relaxed);
    self.inner.lock().await.eliminate_all().await
}
   pub async fn is_offline(self: &Arc<Self>) -> bool {
         self.offline.load(Ordering::Relaxed) | self.quit.load(Ordering::Relaxed)
   }
   pub async fn is_quit(self: &Arc<Self>) -> bool {
        self.quit.load(Ordering::Relaxed)
}
   pub async fn quit(self: &Arc<Self>) -> BackendResult<usize> {
        self.quit.store(true, Ordering::Relaxed);
        self.offline.store(true, Ordering::Relaxed);
        self.inner.lock().await.eliminate_all().await
    }

}

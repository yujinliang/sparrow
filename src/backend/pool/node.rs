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
            offline: AtomicBool::new(true),
            quit: AtomicBool::new(false),
        })
    }
    #[inline]
    #[allow(unused_must_use)]
    pub async fn init(self: &Arc<Self>)  {
        let self_shared = self.clone();
        task::spawn(async move {
                self_shared.reonline().await;
        });
        loop_check(&self).await;
    }
    //fast fail!!!
    pub async fn get_conn(self: &Arc<Self>) -> BackendResult<P2MConn> {
       if self.is_offline().await { 
           return Err(BackendError::InnerErrOfflineOrQuit);
       }
       let rc = self.inner.lock().await.lend_conn().await;
       if rc.is_ok() {
            rc
       } else if self.inner.lock().await.get_total_count().await < self.cfg.max_conns_limit {
                 let mut conn_list = grow(&self, self.cfg.grow_count).await;
                 self.inner.lock().await.takeup_batch(&mut conn_list).await;
                 self.inner.lock().await.lend_conn().await
        } else  {
                 Err(BackendError::InnerErrPipeEmpty)
        }
    }
    #[allow(unused_must_use)]
    pub async fn recycle(self: &Arc<Self>, conn:P2MConn) {
        if self.is_offline().await {
            self.inner.lock().await.discard(conn).await;
            return;
        }
        self.inner.lock().await.send_back(conn).await;
        let total_c = self.inner.lock().await.get_total_count().await;
        if  total_c > self.cfg.max_conns_limit {
            self.inner.lock().await.eliminate(total_c - self.cfg.max_conns_limit).await;
        }
    }
    pub async fn reonline(self: &Arc<Self>) -> BackendResult<()>{
        if self.is_quit().await {
            return Err(BackendError::InnerErrOfflineOrQuit);
        }
        if self.is_offline().await {
                let mut conns = grow(&self, self.cfg.min_conns_limit).await;
                if conns.is_empty() {
                    return Err(BackendError::PoolErrConnGrowFailed(self.cfg.node_id.clone()));
                }
                self.inner.lock().await.update_time_stamp().await;
                self.inner.lock().await.takeup_batch(&mut conns).await;
                self.offline.store(false, Ordering::Relaxed); 
         }  
        Ok(())
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

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
        self.inner.lock().await.lend_with(self.cfg.max_conns_limit, grow(&self, self.cfg.grow_count)).await
    }
    #[allow(unused_must_use)]
    pub async fn recycle(self: &Arc<Self>, conn:P2MConn) {
        if self.is_offline().await {
            self.inner.lock().await.discard(conn).await;
            return;
        }
        self.inner.lock().await.recycle(self.cfg.max_conns_limit,conn).await;
    }
    pub async fn reonline(self: &Arc<Self>) -> BackendResult<()> {
        if self.is_quit().await {
            return Err(BackendError::InnerErrOfflineOrQuit);
        }
        if self.is_offline().await {
            self.inner.lock().await.grow_with(&self.cfg.node_id, grow(&self, self.cfg.grow_count)).await?;
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

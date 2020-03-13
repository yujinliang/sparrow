use async_std::task;
use async_std::sync::{Mutex, Arc};
use super::inner::InnerLine;
use crate::backend::pool::node_cfg::NodeCfg;
use crate::backend::conn::P2MConn;
use crate::backend::error::BackendResult;
use super::checker::{grow, loop_check};

#[derive(Debug)]
pub struct NodePipeLine {
        //dynamic data
        pub inner: Mutex<InnerLine>,
        //static config data 
        pub cfg: NodeCfg,
}

impl NodePipeLine {
    #[inline]
    pub async fn new( cfg: NodeCfg) -> Arc<Self> {
        Arc::new(NodePipeLine{
            inner: Mutex::new(InnerLine::new().await),
            cfg,
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
    #[inline]
    pub async fn get_conn(self: &Arc<Self>) -> BackendResult<P2MConn> {
        self.inner.lock().await.lend_with(self.cfg.max_conns_limit, grow(&self, self.cfg.grow_count)).await
    }
    #[inline]
    #[allow(unused_must_use)]
    pub async fn recycle(self: &Arc<Self>, conn:P2MConn) {
        self.inner.lock().await.recycle(self.cfg.max_conns_limit,conn).await;
    }
    #[inline]
    pub async fn reonline(self: &Arc<Self>) -> BackendResult<()> {
        self.inner.lock().await.reonline_with(&self.cfg.node_id, grow(&self, self.cfg.grow_count)).await
   }
   #[inline]
   pub async fn offline(self: &Arc<Self>) -> BackendResult<usize> {
        self.inner.lock().await.offline().await
    }
    #[inline]
    pub async fn offline_where(self: &Arc<Self>, max: u64) -> BackendResult<usize> {
         self.inner.lock().await.offline_where(max).await
     }
    #[inline]
   pub async fn is_offline(self: &Arc<Self>) -> bool {
        self.inner.lock().await.is_offline().await
   }
   #[inline]
   pub async fn is_quit(self: &Arc<Self>) -> bool {
        self.inner.lock().await.is_quit().await
    }
    #[inline]
   pub async fn quit(self: &Arc<Self>) -> BackendResult<usize> {
        self.inner.lock().await.quit().await
    }
    #[inline]
    pub async fn discard(self: &Arc<Self>, conn:P2MConn) {
        self.inner.lock().await.discard(conn).await
    }
    #[inline]
    pub async fn takeup(self: &Arc<Self>, conn:P2MConn) {
        self.inner.lock().await.takeup(self.cfg.max_conns_limit, conn).await; 
    }
}

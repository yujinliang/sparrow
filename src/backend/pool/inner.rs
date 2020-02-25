#![allow(dead_code)]
use crate::backend::conn::P2MConn;
use super::{BackendError, BackendResult};
use std::collections::LinkedList;
use async_std::prelude::*;

#[derive(Debug)]
pub struct InnerLine {
    cache:  LinkedList<P2MConn>, //pop/push
    recent_request_time:u64,
    total_conn_count: u64,
    offline:bool, 
    quit:bool,
}
impl InnerLine {
    #[inline]
    pub async fn new() -> InnerLine {
        InnerLine{
            cache:  LinkedList::new(), 
            recent_request_time:0,
            total_conn_count: 0,
            offline:false,
            quit:false,
        }
    }
    #[inline]
    pub async fn update_time_stamp(&mut self) {
        self.recent_request_time = 0;//now
    }
    #[inline]
    pub async fn is_offline(&self) -> bool {
       self.offline
    }
    #[inline]
    pub async fn is_quit(&self) -> bool {
        self.quit
    }
    #[inline]
    pub async fn offline(&mut self) -> BackendResult<usize> {
        self.offline = true;
        self.clean_cache().await
    }
    #[inline]
    pub async fn offline_where(&mut self, max:u64) -> BackendResult<usize> {
        if self.total_conn_count >= max {
            return Err(BackendError::InnerErrGreaterThenMaxConnCount);
        }
        self.offline = true;
        self.clean_cache().await
    }
    #[inline]
    pub async fn quit(&mut self) -> BackendResult<usize> {
        self.quit = true;
        self.offline = true;
        self.clean_cache().await
    }
    #[inline]
    pub async fn lend_with<F>(&mut self, max:u64, grow:F) -> BackendResult<P2MConn>
                                where F: Future<Output = LinkedList<P2MConn>> {  
        if self.is_offline().await | self.is_quit().await { 
                return Err(BackendError::InnerErrOfflineOrQuit);
        }
        self.update_time_stamp().await;
        let rc = self.cache.pop_front().ok_or_else(|| { BackendError::InnerErrPipeEmpty});
        if rc.is_ok() {
             rc
        } else if self.total_conn_count < max {
                  let mut conn_list = grow.await;
                  self.takeup_batch(&mut conn_list).await;
                  self.cache.pop_front().ok_or_else(|| { BackendError::InnerErrPipeEmpty})
         } else  {
                  Err(BackendError::InnerErrPipeEmpty)
         }
    }
    #[inline]
    #[allow(unused_must_use)]
    pub async fn reonline_with<F>(&mut self, node_id:&str, grow: F) -> BackendResult<()> 
                                where F: Future<Output = LinkedList<P2MConn>> {
                if self.is_quit().await {
                        return Err(BackendError::InnerErrOfflineOrQuit);
                }
               if !self.is_offline().await {
                    return Err(BackendError::PoolErrConnGrowGiveup(node_id.to_string()));
               }
                let mut conns = grow.await;
                 if conns.is_empty() {
                        return Err(BackendError::PoolErrConnGrowFailed(node_id.to_string()));
                }
                self.update_time_stamp().await;
                self.clean_cache().await;
                self.takeup_batch(&mut conns).await;
                Ok(())
    }
    #[inline]
    #[allow(unused_must_use)]
    pub async fn shrink(&mut self, _idle_time:u64, min:u16, shrink_c:u16) {
            if self.total_conn_count <= min as u64 {
                return;
            }
            //Todo: design a shrink algorithm
             self. eliminate(shrink_c as u64).await; 
    }
 
    #[inline]
    #[allow(unused_must_use)]
    pub async fn clean_cache(&mut self) -> BackendResult<usize> {
        let c_size = self.cache.len();
        for _ in 0..c_size {
            self.cache.pop_front().ok_or_else(|| { BackendError::InnerErrPipeEmpty})?.quit();
            self.total_conn_count -= 1;
        }
        Ok(c_size)
    }
    #[inline]
    #[allow(unused_must_use)]
    pub async fn discard(&mut self, conn:P2MConn) {
        conn.quit();
        self.total_conn_count -= 1;
    }
    #[inline]
    #[allow(unused_must_use)]
    pub async fn recycle(&mut self,max: u64,  conn:P2MConn) {
        if self.is_offline().await | self.is_quit().await {
            self.discard(conn).await;
            return;
        }
        self.cache.push_back(conn);
        if  self.total_conn_count > max {
            self.eliminate( self.total_conn_count - max).await;
        }
    }
    #[inline]
    #[allow(unused_must_use)]
    pub async fn takeup(&mut self, max:u64, conn:P2MConn) {
        if self.is_offline().await | self.is_quit().await {
            conn.quit();
            return;
        }
        self.total_conn_count += 1;
        self.cache.push_back(conn);
        if  self.total_conn_count > max {
            self.eliminate( self.total_conn_count - max).await;
        }
    }
    #[inline]
    async fn takeup_batch(&mut self, conns:&mut LinkedList<P2MConn> ) {
        self.total_conn_count += conns.len() as u64;
        self.cache.append(conns);
    }
    #[inline]
    #[allow(unused_must_use)]
     async fn eliminate(&mut self, count:u64) -> BackendResult<u64> {
        for _ in 0..count {
            self.cache.pop_front().ok_or_else(|| { BackendError::InnerErrPipeEmpty})?.quit();
            self.total_conn_count -= 1;
        }
        Ok(count)
    }
}

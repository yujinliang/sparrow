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
}
impl InnerLine {
    #[inline]
    pub async fn new() -> InnerLine {
        InnerLine{
            cache:  LinkedList::new(), 
            recent_request_time:0,
            total_conn_count: 0,
        }
    }
    pub async fn update_time_stamp(&mut self) {
        self.recent_request_time = 0;//now
    }
    #[inline]
    pub async fn lend_with<F>(&mut self, max:u64, grow:F) -> BackendResult<P2MConn>
                                where F: Future<Output = LinkedList<P2MConn>> {  
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
    pub async fn grow_with<F>(&mut self, node_id:&str, grow: F) -> BackendResult<()> 
                                where F: Future<Output = LinkedList<P2MConn>> {
                    let mut conns = grow.await;
                    if conns.is_empty() {
                            return Err(BackendError::PoolErrConnGrowFailed(node_id.to_string()));
                    }
                    self.update_time_stamp().await;
                    self.takeup_batch(&mut conns).await;
                    Ok(())
    }
    #[inline]
    #[allow(unused_must_use)]
    pub async fn eliminate(&mut self, count:u64) -> BackendResult<u64> {
        for _ in 0..count {
            self.cache.pop_front().ok_or_else(|| { BackendError::InnerErrPipeEmpty})?.quit();
            self.total_conn_count -= 1;
        }
        Ok(count)
    }
    #[inline]
    #[allow(unused_must_use)]
    pub async fn eliminate_all(&mut self) -> BackendResult<usize> {
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
    pub async fn takeup(&mut self, conn:P2MConn) {
        self.total_conn_count += 1;
        self.cache.push_back(conn)
    }
    pub async fn whether_to_start_shrink(&self,  _time_to_shrink: u64, min_conns_limit:u16, shrink_count:u16) -> (bool, u16 ){
        if self.total_conn_count <= min_conns_limit as u64 {
            return (false, 0);
        }
        let shrink_count = if (self.total_conn_count - shrink_count as u64 ) <= min_conns_limit as u64{
            min_conns_limit - self.total_conn_count as u16 + shrink_count 
        } else  {
            shrink_count 
        };
        (false, shrink_count)
    }

}

#![allow(dead_code)]
use crate::backend::conn::P2MConn;
use super::{BackendError, BackendResult};
use std::collections::LinkedList;

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
    #[inline]
    pub async fn get_cache_size(&self) -> u64 {
        self. cache.len() as u64
    }
    #[inline]
    pub async fn get_total_conn_count(&self) -> u64 {
        self. total_conn_count
    }
    pub async fn update_time_stamp(&mut self) {
        self.recent_request_time = 0;//now
    }
    #[inline]
    pub async fn lend_conn(&mut self) -> BackendResult<P2MConn> {  
        self.cache.pop_front().ok_or_else(|| { BackendError::InnerErrPipeEmpty})
    }
    #[inline]
    #[allow(unused_must_use)]
    pub async fn eliminate(&mut self, count:u16) -> BackendResult<u16> {
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
    pub async fn send_back(&mut self, conn:P2MConn) {
        self.cache.push_back(conn);
    }
    #[inline]
    pub async fn takeup_batch(&mut self, conns:&mut LinkedList<P2MConn> ) {
        self.total_conn_count += conns.len() as u64;
        self.cache.append(conns);
    }
    #[inline]
    pub async fn takeup(&mut self, conn:P2MConn) {
        self.total_conn_count += 1;
        self.cache.push_back(conn)
    }
    pub async fn whether_to_start_shrink(&self,  _time_to_shrink: u64, min_conns_limit:u16, shrink_count:u16) -> (bool, u16 ){
        let cache_size = self.get_cache_size().await;
        if cache_size <= min_conns_limit as u64 {
            return (false, 0);
        }
        let shrink_count = if (cache_size - shrink_count as u64 ) <= min_conns_limit as u64{
            min_conns_limit - cache_size as u16 + shrink_count 
        } else  {
            shrink_count 
        };
        (false, shrink_count)
    }

}

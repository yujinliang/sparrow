#![allow(dead_code)]
use crate::backend::conn::P2MConn;
use super::{BackendError, BackendResult};
use std::collections::LinkedList;

#[derive(Debug)]
pub struct InnerLine {
    cache:  LinkedList<P2MConn>, //pop/push
    recent_request_time:u64,
    pub total_conn_count: u64,
    lend_conn_count:u64,
    offline:bool, 
    pub quit:bool,
}
impl InnerLine {
    #[inline]
    pub async fn new() -> InnerLine {
        InnerLine{
            cache:  LinkedList::new(), 
            recent_request_time:0,
            total_conn_count: 0,
            lend_conn_count:0,
            offline:false, 
            quit:false,
        }
    }
    #[inline]
    pub async fn get_cache_size(&self) -> u64 {
        self. cache.len() as u64
    }
    pub async fn update_time_stamp(&mut self) {
        self.recent_request_time = 0;//now
    }
    #[inline]
    pub async fn lend_conn(&mut self) -> BackendResult<P2MConn> {  
       let rc =  self.cache.pop_front().ok_or_else(|| { BackendError::InnerErrPipeEmpty});
       if rc.is_ok() {
            self.lend_conn_count += 1;
       }
       rc
    }
    #[inline]
    #[allow(unused_must_use)]
    pub async fn eliminate(&mut self, count:u16) -> BackendResult<u16> {
        for _ in 0..count {
            self.cache.pop_front().ok_or_else(|| { BackendError::InnerErrPipeEmpty})?.quit();
            self.total_conn_count -= 1;
            self.lend_conn_count -= 1;
        }
        Ok(count)
    }
    #[inline]
    #[allow(unused_must_use)]
    async fn eliminate_all(&mut self) -> BackendResult<()> {
        for _ in 0..self.cache.len() {
            self.cache.pop_front().ok_or_else(|| { BackendError::InnerErrPipeEmpty})?.quit();
            self.total_conn_count -= 1;
            self.lend_conn_count -= 1;
        }
        Ok(())
    }
    #[inline]
    #[allow(unused_must_use)]
    pub async fn discard_one(&mut self, conn:P2MConn) {
        conn.quit();
        self.total_conn_count -= 1;
        self.lend_conn_count -= 1;
    }
    #[inline]
    pub async fn send_back(&mut self, conn:P2MConn) {
        self.cache.push_back(conn);
        self.lend_conn_count -= 1;
    }
    #[inline]
    pub async fn takeup_new_conn(&mut self, conns:&mut LinkedList<P2MConn> ) {
        self.total_conn_count += conns.len() as u64;
        self.cache.append(conns);
    }
    #[inline]
    pub async fn reonline_with(&mut self, conns:&mut LinkedList<P2MConn> ) -> BackendResult<usize> {
        if self.quit {
            return Err(BackendError::InnerErrOfflineOrQuit);
        }
        let c_size =  conns.len();
        self.recent_request_time = 0;//now
        self.total_conn_count = 0;
        self.lend_conn_count = 0;
        self.total_conn_count += c_size as u64;
        self.cache.append(conns);
        self.offline = false;
        Ok(c_size)
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
    #[inline]
    pub async fn is_offline(&self) -> bool {
         self.offline || self.quit
    }
    #[inline]
    #[allow(unused_must_use)]
    pub async fn offline(&mut self) {
        self.offline = true;
        self.eliminate_all().await;
    }
    #[inline]
    #[allow(unused_must_use)]
     pub async fn quit(&mut self) {
        self.quit = true;
        self.offline = true;  
        self.eliminate_all().await;
     }
}

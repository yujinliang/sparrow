#![allow(dead_code)]
use crate::backend::conn::P2MConn;
use super::{BackendError, BackendResult};
use std::collections::LinkedList;
use async_std::net::TcpStream;
use async_std::task::JoinHandle;
use async_std::sync::{Mutex, Arc};
use async_std::task;
use std::time::Duration;
use log::info;

#[derive(Debug)]
struct InnerLine {
    cache:  LinkedList<P2MConn>, //pop/push
    recent_request_time:u64,
    total_conn_count: u64,
    lend_conn_count:u64,
    offline:bool, 
    quit:bool,
}
impl InnerLine {
    #[inline]
    async fn new() -> InnerLine {
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
    async fn get_cache_size(&self) -> u64 {
        self. cache.len() as u64
    }
    async fn update_time_stamp(&mut self) {
        self.recent_request_time = 0;
    }
    #[inline]
    async fn lend_conn(&mut self) -> BackendResult<P2MConn> {  
       let rc =  self.cache.pop_front().ok_or_else(|| { BackendError::InnerErrPipeEmpty});
       if rc.is_ok() {
            self.lend_conn_count += 1;
       }
       rc
    }
    #[inline]
    #[allow(unused_must_use)]
    async fn eliminate(&mut self, count:u16) -> BackendResult<u16> {
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
    async fn discard_one(&mut self, conn:P2MConn) {
        conn.quit();
        self.total_conn_count -= 1;
        self.lend_conn_count -= 1;
    }
    #[inline]
    async fn send_back(&mut self, conn:P2MConn) {
        self.cache.push_back(conn);
        self.lend_conn_count -= 1;
    }
    #[inline]
    async fn takeup_new_conn(&mut self, conns:&mut LinkedList<P2MConn> ) {
        self.cache.append(conns);
        self.total_conn_count += conns.len() as u64;
    }
    #[inline]
    async fn reonline_with(&mut self, conns:&mut LinkedList<P2MConn> ) {
        self.recent_request_time = 0;
        self.total_conn_count = 0;
        self.lend_conn_count = 0;
        self.cache.append(conns);
        self.total_conn_count += conns.len() as u64;
        self.quit = false;
        self.offline = false;
    }
    async fn whether_to_start_shrink(&self,  _time_to_shrink: u64, min_conns_limit:u16, shrink_count:u16) -> (bool, u16 ){
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
    async fn is_offline(&self) -> bool {
         self.offline || self.quit
    }
    #[inline]
    #[allow(unused_must_use)]
    async fn offline(&mut self) {
        self.offline = true;
        self.eliminate_all().await;
    }
    #[inline]
    #[allow(unused_must_use)]
     async fn quit(&mut self) {
        self.quit = true;
        self.offline = true;  
        self.eliminate_all().await;
     }
}
#[derive(Debug)]
pub struct NodePipeLine {
        //dynamic data
        inner: Mutex<InnerLine>,
        //static data 
        mysql_user: String,
        mysql_pwd:String,
        mysql_addr:String,
        pub cluster_id:String,
        pub node_id:String,
        max_conns_limit:u64,
        min_conns_limit:u16,
        grow_count: u16,
        shrink_count:u16,
        idle_time_to_shrink:u64,
        time_to_check_interval:u64,
        ping_retry_count: u8 ,
        ping_retry_min_interval: u16 , //time unit: second 
        reconnect_retry_count: u8 ,
        reconnect_retry_min_interval:u16 ,//time unit: second 
}

impl NodePipeLine {
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub async fn new(       
        mysql_user: String,
        mysql_pwd:String,
        mysql_addr:String,
        cluster_id:String,
        node_id:String,
        max_conns_limit:u64,
        min_conns_limit:u16,
        grow_count: u16,
        shrink_count:u16,
        idle_time_to_shrink:u64,
        time_to_check_interval:u64,
        ping_retry_count: u8 ,
        ping_retry_min_interval: u16 ,
        reconnect_retry_count: u8 ,
        reconnect_retry_min_interval:u16 
        ) -> Arc<Self> {
        Arc::new(NodePipeLine{
            inner: Mutex::new(InnerLine::new().await),
            mysql_user,
            mysql_pwd,
            mysql_addr,
            cluster_id,
            node_id,
            max_conns_limit,
            min_conns_limit,
            grow_count,
            shrink_count,
            idle_time_to_shrink,
            time_to_check_interval,
            ping_retry_count,
            ping_retry_min_interval,
            reconnect_retry_count,
            reconnect_retry_min_interval,
        })
    }
    pub async fn init(self: &Arc<Self>)  {
        let self_shared = self.clone();
        task::spawn(async move {
            let mut conn_list = self_shared.grow(self_shared.min_conns_limit).await;
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
       if l.get_cache_size().await > self.min_conns_limit.into() {
            l.lend_conn().await
       } else if l.total_conn_count < self.max_conns_limit {
                let grow_c = if (l.total_conn_count + self.grow_count as u64) <= self.max_conns_limit {
                    self.grow_count
                } else {
                    (l.total_conn_count + self.grow_count as u64 - self.max_conns_limit) as u16
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
        if l.get_cache_size().await > self.max_conns_limit {
            l.eliminate(1).await;
        }
    }
    pub async fn offline(self: &Arc<Self>) {
        self.inner.lock().await.offline().await
  
    }
    pub async fn reonline(self: &Arc<Self>) -> BackendResult<u16>{
        let mut conn_list = self.grow(self.min_conns_limit).await;
        let l_size = conn_list.len();
        if l_size == 0 {
            return Err(BackendError::PoolErrConnGrowFailed(self.node_id.clone()));
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
               let user =  self.mysql_user.clone();
               let pwd =  self.mysql_pwd.clone();
               let addr = self.mysql_addr.clone();
               let c_id = self.cluster_id.clone();
               let n_id =  self.node_id.clone();
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
                    task::sleep(Duration::from_secs(self_shared.time_to_check_interval)).await;
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
    let decision = l.whether_to_start_shrink(receiver.idle_time_to_shrink, receiver.min_conns_limit, receiver.shrink_count).await;
    if !decision.0 {
        return l.quit;
    }
    l.eliminate(decision.1).await;  
    l.quit
}
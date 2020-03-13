use std::collections::LinkedList;
//use async_std::prelude::*;
//use async_std::sync::channel;
use async_std::sync::{Arc};
use crate::backend::conn::P2MConn;
use crate::backend::error::{BackendResult, BackendError};
use crate::backend::pool::node_cfg::NodeCfg;


#[derive(Debug)]
pub struct NodePipeLine {
        //dynamic data
        cache:  LinkedList<P2MConn>, //pop/push
        // a mpmc channel PIPE
        recent_request_time:u64,
        total_conn_count: u64,
        offline:bool, 
        quit:bool,
        //static config data 
        pub cfg: NodeCfg,
}

impl NodePipeLine {
    #[inline]
    pub async fn get_conn(self: &Arc<Self>) -> BackendResult<P2MConn> {
        //update recent_request_time with now timestamp
        //block read on channel PIPE
        unimplemented!();
    }
    #[inline]
    pub async fn recycle(self: &Arc<Self>) -> BackendResult<P2MConn> {
        //async write to channel PIPE
        unimplemented!();
    }
    #[inline]
    pub async fn discard(self: &Arc<Self>) -> BackendResult<P2MConn> {
        //1. close conn
        //2. update total_conn_count by atomic
        unimplemented!();
    }
    pub async fn pump_station(self: &Arc<Self>) {
        /*loop {
            select {
                case1 read channel PIPE {
                    //sync way
                    push the conn to cache
                }
                case2 can write to channel PIPE {
                    //sync way
                    if cache is not empty {
                        pop a conn from the cache
                        send the conn to channel PIPE
                    } else if total_conn_count < max_conn_limit {
                        create some new conn
                        push them into cache
                        send one into channel PIPE
                    }
                }
                case3 a interval task{
                    //sync way
                    shrink check
                    if recent_request_time - now() > shrink time threshold  {
                        if channel PIPE len() + cache len() > min conn limit && reach on 2/3 of the total_conn_count {
                            start to shrink shrink_count_limit conn
                            
                        }
                    }
                }
                case4 a interval task {
                    //async way
                    node health check , online/offline
                    first pingpong
                    second reconnect
                    create new conn
                    retry some times
                    finally to be considered offline
                }
                
            }
        }*/
        unimplemented!();
    }
}

#![allow(dead_code)] 
use crate::router;
use async_std::prelude::*;
//use async_std::io;
use async_std::task;
use async_std::net::{TcpListener, TcpStream};
use async_std::sync::Arc;
use super::errors::{ProxyResult, ProxyError};
use log::info;
use crate::frontend;
use std::sync::atomic::{AtomicU32, Ordering};
use crate::mysql::{packet, errcode};

#[derive(Debug)]
pub struct ProxyServer{}

impl ProxyServer {

    pub fn new( ) -> ProxyServer {
        ProxyServer{}
    }

   pub fn run(&self) -> ProxyResult<()> {
    task::block_on(async {
          //init shard router
        let shard_r = router::build_router()?;
        info!("shard router module init ok! {:?}", &shard_r);

        let listen_addr = crate::GLOBAL_CONFIG.query_proxy_listen_addr().unwrap_or_else(|| "127.0.0.1:9696");
        let ipv4_listener = TcpListener::bind(listen_addr).await?;
        let mut  incoming = ipv4_listener.incoming();
       // println!("global config: {:?}", crate::GLOBAL_CONFIG.query_log_path()); 
        while let Some(stream) = incoming.next().await {
            let stream = stream?;
            let client_router = shard_r.clone();
            task::spawn(async move {    
                    let rc = process(stream, generate_id(), client_router).await;
                    info!("process result: {:?}", rc);
                    rc
             });
         }
        Ok(())
        })
    }
} // impl end

async fn process<'a>( stream: TcpStream, id : u32, r : Arc<router::Router<'a>>) ->  ProxyResult<()>  {
    info!("Accepted from: {}, mysql thread id: {}", stream.peer_addr()?, id);
    let mut c2p = frontend::conn::C2PConn::build_c2p_conn(stream, id, r).await?;
    if let Err(e) = c2p.s2c_handshake().await {
        let err_p = packet::ErrPacket::new(errcode::ER_HANDSHAKE_ERROR, format!("{:?}", e));
       return  c2p.write_err(err_p).await.map_err(|e|{
           ProxyError::Other(Box::new(e))
       });
    }
    c2p.run_loop().await;
    Ok(())
}

 fn generate_id() -> u32 {
    static COUNTER: AtomicU32 = AtomicU32::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}
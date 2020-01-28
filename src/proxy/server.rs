#![allow(dead_code)] 
use crate::config::Config;
use crate::router::Router;
use async_std::prelude::*;
//use async_std::io;
use async_std::task;
use async_std::net::{TcpListener, TcpStream};
use super::errors::{ProxyResult};
use log::info;
use crate::frontend;
use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Debug)]
pub struct ProxyServer <'a>{

    cfg : &'a Config,
    router: &'a Router,
}

impl<'a> ProxyServer<'a> {

    pub fn new( c : &'a Config, r:&'a Router ) -> ProxyServer <'a> {
        ProxyServer{cfg : c, router: r}
    }

   pub fn run(&self) -> ProxyResult<()> {
    task::block_on(async {
        let listen_addr = self.cfg.query_proxy_listen_addr().unwrap_or_else(|| "127.0.0.1:9696");
        let ipv4_listener = TcpListener::bind(listen_addr).await?;
        let mut  incoming = ipv4_listener.incoming();
        while let Some(stream) = incoming.next().await {
            let stream = stream?;
            task::spawn(async move {    
                    let rc = process(stream, generate_id()).await;
                    info!("process result: {:?}", rc);
                    rc
             });
         }
        Ok(())
        })
    }
} // impl end

async fn process( stream: TcpStream, id : u32) ->  ProxyResult<()>  {
    info!("Accepted from: {}, mysql thread id: {}", stream.peer_addr()?, id);
    //let (reader, writer) = &mut (&stream, &stream);
    //io::copy(reader, writer).await?;
    let mut c2p_conn = frontend::conn::C2PConn::build_c2p_conn(stream, id).await?;
    c2p_conn.s2c_handshake().await?;
    c2p_conn.run_loop().await;
    Ok(())
}

 fn generate_id() -> u32 {
    static COUNTER: AtomicU32 = AtomicU32::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}
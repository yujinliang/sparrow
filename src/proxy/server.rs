#![allow(dead_code)] 
use crate::config::Config;
use crate::router::Router;
use async_std::prelude::*;
use async_std::io;
use async_std::task;
use async_std::net::{TcpListener, TcpStream};
use super::errors::{ProxyResult};

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
                    process(stream).await
             });
         }
        Ok(())
        })
    }
} // impl end

async fn process( stream: TcpStream) ->  ProxyResult<()>  {
    println!("Accepted from: {}", stream.peer_addr()?);

    let (reader, writer) = &mut (&stream, &stream);
    io::copy(reader, writer).await?;

    Ok(())
}
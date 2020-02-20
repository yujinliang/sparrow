#![allow(dead_code)]
use crate::mysql::{packetio, constants, utils};
use super::error::{BackendResult};
use async_std::net::TcpStream;

#[derive(Debug)]
pub struct P2MConn {
    pkg: packetio::PacketIO,
    conn_id: u32,
    capability: constants::CapabilityFlags,
    salt : Vec<u8>, //8 or 20 bytes.
    collation_id: u8,
    status: constants::StatusFlags,
//---
    mysql_user: String,
    mysql_pwd:String,
    mysql_addr:String,
    cluster_id:String,
    pub node_id:String,
    db:String,
//--
    quited:bool,
}

impl P2MConn {
    pub async fn  build_conn (
        tcp: TcpStream,
        mysql_user: String,
        mysql_pwd:String,
        mysql_addr:String,
        cluster_id:String,
        node_id:String) -> BackendResult<P2MConn> {
        let pkg = packetio::PacketIO::new(tcp);
        let conn_id: u32 = 0;
        let capability = constants::get_default_capability_flags();
        let salt:Vec<u8> = utils::random_salt(20)?;
        let collation_id: u8 = constants::UTF8MB4_GENERAL_CI;
        let status = constants::StatusFlags::SERVER_STATUS_AUTOCOMMIT;
        let db:String = String::new();
        Ok(P2MConn{
            pkg,
            conn_id,
            capability,
            salt,
            collation_id,
            status,
            mysql_user,
            mysql_pwd,
            mysql_addr,
            node_id,
            cluster_id,
            db,
            quited:false,
        })
}
    pub async fn ping(&self) -> BackendResult<()> {
        //1. send mysql ping command 
        //2. wait read result
        unimplemented!();
    }
    pub async fn close(&self) {
        //pool.recycle(this);
        unimplemented!();
    }
    //真正的关闭网络链接。
    #[allow(unused_must_use)]
    pub fn quit(&self) {
        if !self.quited {
            self.pkg.quit();
        }
    }

    pub async fn handshake(& mut self) ->  BackendResult<()> {
        unimplemented!();
    }
}

impl std::ops::Drop for P2MConn {
    fn drop(&mut self) {
        if !self.quited {
            self.quited = true;
            self.quit();
        }
    }
}
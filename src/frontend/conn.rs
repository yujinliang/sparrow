#![allow(dead_code)] 
use crate::mysql::{packetio, constants::*};
use async_std::net::{TcpStream};
use super::errors::{FrontendResult, FrontendError};
use byteorder::{LittleEndian as LE, WriteBytesExt};

//client to proxy conn abstraction
pub struct C2PConn {
    pkg: packetio::PacketIO,
    c:TcpStream,
    conn_id: u32,
    capability: CapabilityFlags,
    salt : Vec<u8>, //8 or 20 bytes.
    collation_id: u8,
    status: u16,
//---
    proxy_user: String,
    db:String,
}

impl C2PConn {

    fn get_proxy_flags(&self) -> CapabilityFlags {
        let proxy_flags = CapabilityFlags::CLIENT_PROTOCOL_41
            | CapabilityFlags::CLIENT_SECURE_CONNECTION
            | CapabilityFlags::CLIENT_LONG_PASSWORD
            | CapabilityFlags::CLIENT_TRANSACTIONS
            | CapabilityFlags::CLIENT_PLUGIN_AUTH
            | CapabilityFlags::CLIENT_LONG_FLAG;
            return proxy_flags;
    }

    //https://dev.mysql.com/doc/dev/mysql-server/latest/page_protocol_connection_phase_packets_protocol_handshake_v10.html
    //http://hutaow.com/blog/2013/11/06/mysql-protocol-analysis/#41
    async fn write_initial_handshake(&mut self) ->  FrontendResult<()>{
         let mut data = vec![0u8; 4]; //default 4 bytes for header.
        data.push(MIN_PROTOCOL_VERSION);
        data.extend_from_slice(SERVER_VERSION.as_bytes());
        data.push(0u8);
        let mut conn_id_v = Vec::new();
        conn_id_v.write_u32::<LE>(self.conn_id)?;
        data.append(&mut conn_id_v);
        data.extend_from_slice(&self.salt[0..8]);
        data.push(0u8);
        //the lower 2 bytes of the capability
        data.push(self.capability.bits() as u8);
        data.push((self.capability.bits() >> 8) as u8);
        //charset, utf-8 default
        data.push(self.collation_id);
        data.push(self.status as u8);
        data.push((self.status >> 8)as u8);
        //capability flag upper 2 bytes, using default capability here
        data.push((self.capability.bits() >> 16) as u8);
        data.push((self.capability.bits() >> 24) as u8);
        if self.capability.contains(CapabilityFlags::CLIENT_PLUGIN_AUTH) {
            data.push(0x15);
        }
        else {
            data.push(0u8);
        }
        let reserved_u8 = [0u8; 10];
        data.extend_from_slice(&reserved_u8);
        //auth-plugin-data-part-2
        data.extend_from_slice(&self.salt[8..]);
        data.push(0u8);
        self.pkg.write_packet(&mut data).await.map_err(|e| { FrontendError::ErrMysqlWrite(e)})
    }

}
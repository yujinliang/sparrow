#![allow(dead_code)] 
use crate::mysql::{packetio, constants::*, utils};
use async_std::net::{TcpStream};
use super::errors::{FrontendResult, FrontendError};
use byteorder::{LittleEndian as LE, WriteBytesExt};

//client to proxy conn abstraction
pub struct C2PConn {
    pkg: packetio::PacketIO,
    conn_id: u32,
    capability: CapabilityFlags,
    salt : Vec<u8>, //8 or 20 bytes.
    collation_id: u8,
    status: StatusFlags,
//---
    proxy_user: String,
    db:String,
}

impl C2PConn {

    //https://dev.mysql.com/doc/dev/mysql-server/latest/page_protocol_connection_phase_packets_protocol_handshake_v10.html
    //http://hutaow.com/blog/2013/11/06/mysql-protocol-analysis/#41
    async fn write_initial_handshake(&mut self) ->  FrontendResult<()>{
         //reserved 4 bytes for header.
         let mut data = vec![0u8; 4];
         //min version 10
        data.push(MIN_PROTOCOL_VERSION);
        	//server version[00]
        data.extend_from_slice(SERVER_VERSION.as_bytes());
        data.push(0u8);
        //connection id
        let mut conn_id_v = Vec::new();
        conn_id_v.write_u32::<LE>(self.conn_id)?;
        data.append(&mut conn_id_v);
        //auth-plugin-data-part-1
        data.extend_from_slice(&self.salt[0..8]);
        //filter [00]
        data.push(0u8);
        //the lower 2 bytes of the capability
        data.push(self.capability.bits() as u8);
        data.push((self.capability.bits() >> 8) as u8);
        //charset, utf-8 default
        data.push(self.collation_id);
        //status
        data.push(self.status.bits() as u8);
        data.push((self.status .bits() >> 8)as u8);
        //capability flag upper 2 bytes, using default capability here
        data.push((self.capability.bits() >> 16) as u8);
        data.push((self.capability.bits() >> 24) as u8); 
        //filter [0x15], for wireshark dump, value is 0x15
         data.push(0x15); //????
         //reserved 10 [00]
        let reserved_u8 = [0u8; 10];
        data.extend_from_slice(&reserved_u8);
        //auth-plugin-data-part-2
        data.extend_from_slice(&self.salt[8..]);
        data.push(0u8);
        //server send first auth packet to client by tcp stream
        self.pkg.write_packet(&mut data).await.map_err(|e| { FrontendError::ErrMysqlWrite(e)})
    }

    pub async fn  build_c2p_conn(tcp: TcpStream) -> FrontendResult<C2PConn> {
            let pkg = packetio::PacketIO::new(tcp);
            let conn_id: u32 = 0;
            let capability = C2PConn::get_proxy_flags();
            let salt:Vec<u8> = utils::random_salt(20)?;
            let collation_id: u8 = UTF8_GENERAL_CI;
            let status = StatusFlags::SERVER_STATUS_AUTOCOMMIT;
            let proxy_user: String = String::new();
            let db:String = String::new();
            Ok(C2PConn{
                pkg,
                conn_id,
                capability,
                salt,
                collation_id,
                status,
                proxy_user,
                db,
            })
    }
    fn get_proxy_flags() -> CapabilityFlags {
        let proxy_flags = CapabilityFlags::CLIENT_PROTOCOL_41
            | CapabilityFlags::CLIENT_SECURE_CONNECTION
            | CapabilityFlags::CLIENT_LONG_PASSWORD
            | CapabilityFlags::CLIENT_TRANSACTIONS
            | CapabilityFlags::CLIENT_PLUGIN_AUTH
            | CapabilityFlags::CLIENT_LONG_FLAG;
            return proxy_flags;
    }
}


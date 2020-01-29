#![allow(dead_code)] 
#[allow(unused_assignments)]
#[allow(unused_variables)]
use crate::mysql::{packetio, constants, utils, errors};
use async_std::net::{TcpStream};
use async_std::io;
use async_std::sync;
use super::errors::{FrontendResult, FrontendError};
use byteorder::{LittleEndian as LE, WriteBytesExt};
use log::info;
use std::io::Cursor;
use crate::router;

//client to proxy conn abstraction
#[derive(Debug)]
pub struct C2PConn {
    pkg: packetio::PacketIO,
    conn_id: u32,
    capability: constants::CapabilityFlags,
    salt : Vec<u8>, //8 or 20 bytes.
    collation_id: u8,
    status: constants::StatusFlags,
//---
    proxy_user: String,
    db:String,
//--
    r : sync::Arc<router::Router>,
}

impl C2PConn {

    //https://dev.mysql.com/doc/dev/mysql-server/latest/page_protocol_connection_phase_packets_protocol_handshake_v10.html
    //http://hutaow.com/blog/2013/11/06/mysql-protocol-analysis/#41
    async fn write_initial_handshake(&mut self) ->  FrontendResult<()>{
         //reserved 4 bytes for header.
         let mut data = vec![0u8; 4];
         //min version 10
        data.push(constants::MIN_PROTOCOL_VERSION);
        	//server version[00]
        data.extend_from_slice(constants::SERVER_VERSION.as_bytes());
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
   // http://hutaow.com/blog/2013/11/06/mysql-protocol-analysis/#41
   //https://dev.mysql.com/doc/internals/en/connection-phase-packets.html#packet-Protocol::HandshakeResponse
   //https://dev.mysql.com/doc/dev/mysql-server/latest/page_protocol_connection_phase_packets_protocol_handshake_response.html
    async fn read_client_handshake(&mut self) ->  FrontendResult<()>{
        let data = self.pkg.read_packet().await?;
        info!("read_client_handshake raw data: {:?}", &data);

        let mut pos: usize = 0;
        //capability
        let mut rdr = Cursor::new(&data[..4]);
        self.capability &= constants::CapabilityFlags::from_bits_truncate( byteorder::ReadBytesExt::read_u32::<LE>(& mut rdr).unwrap_or(0));
        pos += 4;
        //skip max packet size
        pos += 4;
        //charset, skip, if you want to use another charset, use set names
	    //c.collation = CollationId(data[pos])
       pos += 1;
       	//skip reserved 23[00]
	    pos += 23;
    	//user name
       self.proxy_user = data[pos..].iter().position(|&x| { x == 0}).and_then(|offset| {
           Some(String::from_utf8_lossy( &data[pos.. (pos+offset)]))
       }).unwrap_or_default().to_string();
       pos += self.proxy_user.len() + 1;
    //auth length and auth
	let auth_len = data[pos] as usize;
	pos +=1;
    let auth = data[pos .. (pos+auth_len)].to_vec();
    info!("client auth user: {}, auth:{:?}", &self.proxy_user, &auth);
    //check proxy user exists?
    let user_pair = crate::SHOTCUT_GLOBAL_CONFIG.check_proxy_user_exists(&self.proxy_user);
    if user_pair.is_none() {
        info!("proxy user do not exist: {}, auth:{:?}", &self.proxy_user, &auth);
        return Err(FrontendError::ProxyAuthDenied);
    }
    //check user password?
    let scramble = utils::scramble_password(&self.salt, user_pair.unwrap_or_default().1).unwrap_or_default();
	if !(scramble == auth) {
        info!("proxy user pwd check failed: {}, auth:{:?}", &self.proxy_user, &auth);
		return Err(FrontendError::ProxyAuthDenied);
    }
    pos += auth_len;
    
    //init with db
    let mut init_with_db = String::new();
    if self.capability.contains(constants::CapabilityFlags::CLIENT_CONNECT_WITH_DB ) {
            let may_be_db = &data[pos..];
            if may_be_db.len() > 0 {
                init_with_db = data[pos..].iter().position(|&x| { x == 0}).and_then(|offset| {
                    Some(String::from_utf8_lossy( &data[pos.. (pos+offset)]))
                }).unwrap_or_default().to_string();
                pos += init_with_db.len() + 1;
            }
    }
    self.db = init_with_db;
    info!("init_with_db: {:?}, final pos: {}", &self, pos);
    Ok(())
    }

    //mysql server to client handshake 
    pub async fn s2c_handshake(&self) ->  FrontendResult<()> {
        info!("handshake : {:?}", &self);
        Ok(())
    }
    #[allow(unused_assignments)]
    pub async fn  run_loop(&mut self)  {
        info!("run_loop : {:?}", &self);
        //println!("global config: {:?}", *crate::GLOBAL_CONFIG); 

        let mut exit_flag = false;
        loop {
                match self.pkg.read_packet().await {
                            Err(e) => {
                                  match e {
                                        errors::MysqlError::Io(o) if o.kind() == io::ErrorKind::TimedOut  => exit_flag = true,
                                        errors::MysqlError::Io(o) if o.kind() == io::ErrorKind::ConnectionReset  => exit_flag = true,
                                        errors::MysqlError::Io(o) if o.kind() == io::ErrorKind::ConnectionAborted  => exit_flag = true,
                                        errors::MysqlError::Io(o) if o.kind() == io::ErrorKind::ConnectionRefused  => exit_flag = true,
                                        errors::MysqlError::Io(o) if o.kind() == io::ErrorKind::NotConnected  => exit_flag = true,
                                        errors::MysqlError::Io(o) if o.kind() == io::ErrorKind::BrokenPipe  => exit_flag = true,
                                        _ => continue,
                                  } 
                                  if exit_flag { return; }
                            }
                            Ok(ref mut data) => {
                                    info!("mysql client packet data: {:?}", data);
                                    if let Err(e) = self.dispatch_mysql_cmd(data).await {
                                            info!("dispatch_mysql_cmd err: {:?}",e);
                                            return;
                                    }
                            }
                }
        }
    }
    pub async fn dispatch_mysql_cmd(&mut self, data: &mut [u8]) -> FrontendResult<()> {
        info!("dispatch_mysql_cmd data: {:?}", data);
        Ok(())
    }
    pub async fn  build_c2p_conn(tcp: TcpStream, id: u32,  r : sync::Arc<router::Router>) -> FrontendResult<C2PConn> {
            let pkg = packetio::PacketIO::new(tcp);
            let conn_id: u32 = id;
            let capability = constants::get_default_capability_flags();
            let salt:Vec<u8> = utils::random_salt(20)?;
            let collation_id: u8 = constants::UTF8_GENERAL_CI;
            let status = constants::StatusFlags::SERVER_STATUS_AUTOCOMMIT;
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
                r,
            })
    }
}


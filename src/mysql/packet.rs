#![allow(dead_code)]
use super::errors::{MysqlError, MysqlResult};
use super::{constants, errcode, sql_state};
use super::utils;
use byteorder::{ByteOrder, LittleEndian as LE};

#[derive(Debug, Clone)]
pub struct OkPacket {
    //0x00 as header mark
    affected_rows:u64,
    last_insert_id:u64,
    status: constants::StatusFlags,
    warnings:u16,
}

impl OkPacket {

    pub fn empty(status:constants::StatusFlags) -> OkPacket {
        OkPacket{
            status,
            last_insert_id:0,
            affected_rows:0,
            warnings:0,
        }
    }
    pub fn new(affected_rows:u64,
        last_insert_id:u64,
        status: constants::StatusFlags,
        warnings:u16, ) ->OkPacket {
       OkPacket{affected_rows, last_insert_id, status, warnings}
    }
    //http://hutaow.com/blog/2013/11/06/mysql-protocol-analysis/#41
    //https://dev.mysql.com/doc/internals/en/packet-OK_Packet.html
    //https://github.com/blackbeam/rust_mysql_common/blob/master/src/packets.rs
    #[allow(unused_assignments)]
    pub fn parse(data: &[u8]) -> MysqlResult<OkPacket> {
        if data.len() < 7 {return Err(MysqlError::OkPacketWrongSize);}
        let mut pos: usize = 0;
        if data[pos] == constants::OK_PACKET_HEADER_MARK {
            pos +=1;
            let affected_rows = utils::read_length_encoded_int(&data[pos..]);
            pos += affected_rows.0;
            let last_insert_id = utils::read_length_encoded_int(&data[pos..]);
            pos += last_insert_id.0;
            // // We assume that CLIENT_PROTOCOL_41 was set
            let status =  LE::read_u16(&data[pos..]);
            pos +=2;
            let warnings =  LE::read_u16(&data[pos..]);
            pos += 2;
            let status_flags = constants::StatusFlags::from_bits_truncate(status);
            return Ok(OkPacket{affected_rows: affected_rows.1, last_insert_id: last_insert_id.1, status: status_flags, warnings});     
        }
        Err(MysqlError::OkPacketILL)
    }
    pub fn to_bits(&self) -> Vec<u8> {
        let mut data:Vec<u8> = Vec::new();
        data.push(constants::OK_PACKET_HEADER_MARK);
        data.extend_from_slice(&utils::write_length_encoded_int(self.affected_rows));
        data.extend_from_slice(&utils::write_length_encoded_int(self.last_insert_id));
        let status_bits = self.status.bits();
        data.extend_from_slice(vec![status_bits as u8, (status_bits >> 8) as u8].as_slice());
        data.extend_from_slice(vec![self.warnings as u8, (self.warnings >> 8) as u8].as_slice());
        data
    }
    
}
//--------------------------
//https://dev.mysql.com/doc/internals/en/packet-ERR_Packet.html
//https://dev.mysql.com/doc/dev/mysql-server/latest/page_protocol_basic_err_packet.html
//http://hutaow.com/blog/2013/11/06/mysql-protocol-analysis/#41
#[derive(Debug, Clone)]
pub struct ErrPacket {
    //0xFF as header mark
    err_code:u16,
    sql_state: Vec<u8>, //mark as '#', fixed length: 5bytes.
    err_msg:Vec<u8>, //apeend 0x00 as EOF string
}

impl ErrPacket {

    pub fn new(err_code:u16,  err_msg:String) ->ErrPacket {
            let  sql_state:&str = sql_state::MY_SQLSTATE.get(&errcode::ER_HANDSHAKE_ERROR).unwrap_or_else(||{
                     &sql_state::DEFAULT_MYSQL_STATE
             });
            ErrPacket{err_code,sql_state:sql_state.as_bytes().to_vec(),  err_msg:err_msg.as_bytes().to_vec()}
    }
    #[allow(unused_assignments)]
    pub fn parse(data: &[u8]) -> MysqlResult<ErrPacket> {
        if data.len() < 3 {return Err(MysqlError::ErrPacketWrongSize);}
        let mut pos: usize = 0;
        if data[pos] == constants::ERR_PACKET_HEADER_MARK {
            pos +=1;
            let err_code =  LE::read_u16(&data[pos..]);
            pos += 2;
            //we always assume capability CLIENT_PROTOCOL_41
            pos +=1; //ignore '#' mark.
            let sql_state  = data[pos..(pos+5)].to_vec();
            pos += 5;
            let err_msg = data[pos..].to_vec();
            return Ok(ErrPacket{err_code, sql_state, err_msg});
        }
        Err(MysqlError::ErrPacketILL)
    }
    pub fn to_bits(&self) -> Vec<u8> {
        let mut data:Vec<u8> = Vec::new();
        data.push(constants::ERR_PACKET_HEADER_MARK);
        data.extend_from_slice(vec![self.err_code as u8, (self.err_code >> 8) as u8].as_slice());
        data.push(b'#');
        data.extend_from_slice(&self.sql_state);
        data.extend_from_slice(&self.err_msg);
        data
    }
}
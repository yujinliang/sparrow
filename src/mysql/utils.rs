#![allow(dead_code)] 
use super::errors::{MysqlResult};
use rand::{thread_rng, Rng};
use byteorder::{ByteOrder, LittleEndian as LE};

pub fn random_salt( size : usize) -> MysqlResult<Vec<u8>> {
        let mut buf : Vec<u8> = Vec::new();
        let mut rng = thread_rng();
        for _i in 0..size {
                buf.push(rng.gen_range(30u8, 127u8));
        }
        Ok(buf)
}

//https://github.com/go-sql-driver/mysql/blob/master/auth.go
//defaultAuthPlugin: mysql_native_password
//golang drive code: authResp := scramblePassword(authData[:20], mc.cfg.Passwd)
// Hash password using 4.1+ method (SHA1)
pub fn  scramble_password(scramble: &[u8], password:String) -> Option<Vec<u8>> {
        if password.is_empty() {
                return None;
        }
        //first 
        let mut m = sha1::Sha1::new();
        m.update(password.as_bytes());
        let stage1 = m.digest().bytes();

        //second
        m.reset();
        m.update(&stage1);
        let hash = m.digest().bytes();

        // outer Hash
        m.reset();
        m.update(scramble);
        m.update(&hash);
        let conbined_scramble = m.digest().bytes();

        // token = scrambleHash XOR stage1Hash
	//for (pos, _e) in  conbined_scramble.iter().enumerate() {
	//	conbined_scramble[pos] ^= stage1[pos];
        //}
       let final_scramel : Vec<u8> =  conbined_scramble.iter().enumerate() .map(|(pos, e)|{
                e ^ stage1[pos]
        }).collect();
	return Some(final_scramel)
}
// Writes MySql's length-encoded integer.
pub fn write_length_encoded_int( x: u64) -> Vec<u8> {
        let mut encoded_i : Vec<u8> = Vec::new();
        if x < 251 {
                        encoded_i.push(x as u8);
        } else if x < 65_536 {
                        encoded_i.push(0xfc);
                        encoded_i.push(x as u8);
                        encoded_i.push((x >> 8) as u8);
        } else if x < 16_777_216 {
                        encoded_i.extend_from_slice( &[0xfd, x as u8, (x >> 8) as u8, (x >> 16) as u8]);
        } else {
                        encoded_i.append(&mut vec![0xfe, (x) as u8, (x >> 8) as u8, (x >> 16) as u8, (x >> 24) as u8,
			(x >> 32) as u8, (x >> 40) as u8, (x >> 48) as u8, (x >> 56) as u8]);
        }
        encoded_i
}
// Reads MySql's length-encoded integer.
#[allow(unused_assignments)]
pub fn read_length_encoded_int(data: &[u8]) -> (usize ,u64) {
        if data.len() < 1 {
                return (0, 0);
        }
        let mut byte_c = 0;
        match data[0] {
            x if x < 0xfc => return (1, x.into()),
            0xfc if data.len() >= 3 => byte_c = 2,
            0xfd if data.len() >=4 => byte_c = 3,
            0xfe if data.len() >=9 => byte_c = 8,
            _ => byte_c = 0,
        };
        if byte_c > 0 {
                return (byte_c+1,LE::read_uint(&data[1..], byte_c));
        } else {
                return (0,0);
        }
}
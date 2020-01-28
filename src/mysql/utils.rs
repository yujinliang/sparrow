#![allow(dead_code)] 
use super::errors::{MysqlResult};
use rand::{thread_rng, Rng};

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

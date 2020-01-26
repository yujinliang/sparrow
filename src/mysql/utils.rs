use super::errors::{MysqlResult};
use rand::{thread_rng, Rng};

pub fn random_salt( size : usize) -> MysqlResult<Vec<u8>> {
        let mut buf = vec![0u8;size];
        let mut rng = thread_rng();
        for _i in 0..size {
                buf.push(rng.gen_range(30u8, 127u8));
        }
        Ok(buf)
}
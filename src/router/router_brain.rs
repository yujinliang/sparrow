use crate::config::Config;
use std::result::Result;

pub struct Router <'a>{

    cfg : &'a Config,
}

impl<'a> Router<'a> {

   pub  fn new( cfg: &Config) -> Router {
       Router{cfg}
    }

  pub   fn gen_routing_table(&mut self) -> Result<String ,String > {

        println!("{:?}" , self.cfg);
        Ok("test".to_string())
    }
}
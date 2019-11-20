use crate::config::Config;
use std::result::Result;
use std::collections::HashMap;

pub struct Router <'a>{

    cfg : &'a Config,
}

impl<'a> Router<'a> {

   pub  fn new( cfg: &Config) -> Router {
       Router{cfg}
    }

  pub   fn gen_routing_table(&mut self) -> Result<String ,String > {

        let r_table = RouterTable{
            table_entry:HashMap::new(),
        };
        let s = format!("{:?}",  r_table);
        Ok(s)
    }
}

#[derive(Debug)]
struct RouterTable<'a> {
    table_entry: HashMap<&'a str, RouterTableEntry>,
}

#[derive(Debug)]
struct RouterTableEntry {
    id : String, //proxy user + db + table.
}
use crate::config::Config;
use std::result::Result;
use std::collections::HashMap;

pub struct Router <'a>{

    cfg : Option<&'a Config>,
    r_table : Option< RouterTable<'a>>,
}

impl<'a> Router<'a> {

   pub  fn new( cfg: &Config) -> Router {
       Router{cfg : Some(cfg), r_table : None}
    }

  pub   fn gen_routing_table(&mut self) -> Result<String ,String > {

        self.r_table = Some(RouterTable{
            table_entry:HashMap::new(),
        });
        let s = format!("{:?}",  self.r_table);
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
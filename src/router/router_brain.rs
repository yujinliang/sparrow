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

        let mut r_table = RouterTable{
            table_entry:HashMap::new(),
        };

        r_table.table_entry.insert("1", RouterTableEntry{id:"test1".to_string()});
        r_table.table_entry.insert("2", RouterTableEntry{id:"test2".to_string()});
        r_table.table_entry.insert("3", RouterTableEntry{id:"test3".to_string()});

        let s = format!("{:?}",  r_table);
        self.r_table = Some(r_table);
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
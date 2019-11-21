use crate::config::Config;
use std::result::Result;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug)]
pub struct Router <'a>{

    cfg : Option<&'a Config>,
    r_table : Option< RouterTable>,
}

impl<'a> Router<'a> {

    fn new( ) -> Router <'a> {
       Router{cfg : None, r_table : None}
    }
}
//Attention: just allow to be called in main start flow.
  pub   fn init_shard_router( c: Option<& Config>) -> Result<Arc<Router> ,String > {

        //save config.
        let mut r  = Router::new();
        r.cfg = c;

        //build router table entry.
        let mut r_table = RouterTable{
            table_entry:HashMap::new(),
        };

        r_table.table_entry.insert("1".to_string(), RouterTableEntry{id:"test1".to_string()});
        r_table.table_entry.insert("2".to_string(), RouterTableEntry{id:"test2".to_string()});
        r_table.table_entry.insert("3".to_string(), RouterTableEntry{id:"test3".to_string()});


        r.r_table = Some(r_table);
        Ok(Arc::new(r))
    }


#[derive(Debug)]
struct RouterTable {
    table_entry: HashMap<String, RouterTableEntry>,
}

#[derive(Debug)]
struct RouterTableEntry {
    id : String, //proxy user + db + table.
}
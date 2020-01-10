include!(concat!(env!("OUT_DIR"), "/commit_id.rs"));
mod config;
mod router;
mod proxy;
mod mysql;

#[tokio::main]
async fn main() {
    
    println!("--Sparrow mysql/mariadb proxy running!--");
    println!("commit_id: {}compile_time: {}", COMMIT_ID, COMPILE_TIME);
    println!("------------------------------------------------------");

    //1. config.
    let cfg = config::load_config().unwrap();
    //println!("{:#?}", cfg );

    //2. init shard router
    let shard_r = router::load_shard_router(Some(&cfg)).unwrap();
    //test start
   // println!("{:#?}",shard_r );
    /*shard_r.get_table_entry("root-ordinal_db-integer_table").and_then(| entry |{
        let road = entry.find_routing_path("1234567");
        print!("----------------------------------------------------------------\n" );
        println!("{:#?}", road);
        Some(())
    }).unwrap_or_default();
    //------------
    shard_r.get_table_entry("root-test_db-hash_table").and_then(| entry |{
        let road = entry.find_routing_path("hello4569233");
        print!("----------------------------------------------------------------\n" );
        println!("{:#?}", road);
        Some(())
    }).unwrap_or_default();*/
    //------------------
    //shard_r.get_table_entry("root-sparrow-ordinal_range_table").and_then(| entry |{
        shard_r.find_routing_path("root", "sparrow", "ordinal_range_table", "20201102").and_then(| path| {
        println!("----------------------------------------------------------------\n" );
        println!("{:#?}", path);
        Some(())
    });
    //test end.
    //3. run proxy server
    proxy::ProxyServer::new(Some(&cfg));
    //4. run web server.
}

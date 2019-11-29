include!(concat!(env!("OUT_DIR"), "/commit_id.rs"));
mod config;
mod router;
mod proxy;

fn main() {
    
    println!("--Sparrow mysql/mariadb proxy running!--");
    println!("commit_id: {}compile_time: {}", COMMIT_ID, COMPILE_TIME);
    println!("------------------------------------------------------");

    //1. config.
    let cfg = config::load_config().unwrap();
    //println!("{:#?}", cfg );

    //2. init shard router
    let shard_r = router::init_shard_router(Some(&cfg)).unwrap();
    //test start
    println!("{:#?}",shard_r );
    shard_r.get_table_entry("root-ordinal_db-integer_table").and_then(| entry |{
        let road = entry.find_router_path("1234567");
        print!("----------------------------------------------------------------\n" );
        println!("{:#?}", road);
        Some(())
    }).unwrap();
    //------------
    shard_r.get_table_entry("root-test_db-hash_table").and_then(| entry |{
        let road = entry.find_router_path("hello4569233");
        print!("----------------------------------------------------------------\n" );
        println!("{:#?}", road);
        Some(())
    }).unwrap();
    //------------------
    shard_r.get_table_entry("root-sparrow-ordinal_range_table").and_then(| entry |{
        let road = entry.find_router_path("20201102");
        print!("----------------------------------------------------------------\n" );
        println!("{:#?}", road);
        Some(())
    }).unwrap();
    //test end.
    //3. run proxy server
    proxy::ProxyServer::new(Some(&cfg));
    //4. run web server.
}

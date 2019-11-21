include!(concat!(env!("OUT_DIR"), "/commit_id.rs"));
mod config;
mod router;

fn main() {
    
    println!("--Sparrow mysql/mariadb proxy running!--");
    println!("commit_id: {}compile_time: {}", COMMIT_ID, COMPILE_TIME);
    println!("------------------------------------------------------");

    //1. config.
    let cfg = config::load_config().unwrap();
    println!("{:#?}", cfg );

    //2. init shard router
    let shard_r = router::init_shard_router(Some(&cfg));
    println!("{:#?}",shard_r );
    //3. run proxy server

    //4. run web server.
}

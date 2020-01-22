include!(concat!(env!("OUT_DIR"), "/commit_id.rs"));
use log::{error, warn , info, debug, trace};
mod config;
mod router;
mod proxy;
mod mysql;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    println!("--Sparrow mysql/mariadb proxy running!--");
    println!("commit_id: {}compile_time: {}", COMMIT_ID, COMPILE_TIME);
    println!("------------------------------------------------------");

    /*Attention: here we use unwrap() directly! , that means all errors in every module  do not allow to propagte to main level !
       every module log  own errors,  so if  we hit a error in here , then should panic! coz , this is a server ,  need long time to live !  
       do best to run , no complaint!*/
    //1. config.
    let cfg = config::load_config().unwrap();
      
    //2 init log module
 

    //3. init shard router
    let shard_r = router::load_shard_router(&cfg).unwrap();
 
    //4. run proxy server
    let proxy = proxy::ProxyServer::new(&cfg, &shard_r);
    proxy.run();

    Ok(())
}


fn convert_log_level( level : &str) -> log::LevelFilter  {
        match level {
            "error" => log::LevelFilter::Error,
            "warn" => log::LevelFilter::Warn,
            "info" => log::LevelFilter::Info,
            "debug" => log::LevelFilter::Debug,
            "trace" => log::LevelFilter::Trace,
            _ => log::LevelFilter::Error,
        }
}


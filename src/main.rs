include!(concat!(env!("OUT_DIR"), "/commit_id.rs"));
mod config;
mod router;
mod proxy;
mod mysql;
mod frontend;
use log::info;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    println!("--Sparrow mysql/mariadb proxy running!--");
    println!("commit_id: {}compile_time: {}", COMMIT_ID, COMPILE_TIME);
    println!("------------------------------------------------------");

    /*Attention: here we use unwrap() directly! , that means all errors in every module  do not allow to propagte to main level !
       every module log  own errors,  so if  we hit a error in here , then should panic! coz , this is a server ,  need long time to live !  
       do best to run , no complaint!*/
    //1. config.
    let cfg = config::load_config().unwrap();
    info!("Sparrow run commit_id: {}compile_time: {}", COMMIT_ID, COMPILE_TIME);
      
    //2 init log module
    setup_logger(&cfg);
    info!("log module init ok!");
    
    //3. init shard router
    let shard_r = router::load_shard_router(&cfg).unwrap();
    info!("shard router module init ok!");
    //4. run proxy server
    info!("start to run proxy server!");
    let proxy = proxy::ProxyServer::new(&cfg, &shard_r);
    proxy.run()?;

    Ok(())
}

fn setup_logger(cfg : &config::Config) {
    let logger = femme::pretty::Logger::new();
    async_log::Logger::wrap(logger, || /* get the task id here */ 0)
        .start(cfg.query_log_level().unwrap_or(log::LevelFilter::Trace))
        .unwrap();
}


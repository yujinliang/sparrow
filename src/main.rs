include!(concat!(env!("OUT_DIR"), "/commit_id.rs"));
mod config;
mod router;
mod proxy;
mod mysql;
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
      
    //2 init log module
    setup_logger();
    info!("log module init ok!");
    //3. init shard router
    let shard_r = router::load_shard_router(&cfg).unwrap();
 
    //4. run proxy server
    let proxy = proxy::ProxyServer::new(&cfg, &shard_r);
    proxy.run()?;

    Ok(())
}

fn setup_logger() {
    let logger = femme::pretty::Logger::new();
    async_log::Logger::wrap(logger, || /* get the task id here */ 0)
        .start(log::LevelFilter::Trace)
        .unwrap();
}


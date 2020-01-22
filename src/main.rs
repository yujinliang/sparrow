include!(concat!(env!("OUT_DIR"), "/commit_id.rs"));
use tracing::Level;
use tracing_subscriber;
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
    tracing_subscriber::FmtSubscriber::builder()
    // all spans/events with a level higher than TRACE (e.g, info, warn, etc.)
    // will be written to stdout.
    .with_max_level(Level::TRACE)
    // sets this to be the default, global subscriber for this application.
    .init();

    //tracing::error!("log error");
    //tracing::warn!("log warn");
    //tracing::info!("log info");
    //tracing::debug!("log debug");
    tracing::trace!("log trace");

    //3. init shard router
    let shard_r = router::load_shard_router(&cfg).unwrap();
 
    //4. run proxy server
    let proxy = proxy::ProxyServer::new(&cfg, &shard_r);
    proxy.run()?;

    Ok(())
}



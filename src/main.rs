include!(concat!(env!("OUT_DIR"), "/commit_id.rs"));
mod config;
mod router;
mod proxy;
mod mysql;
mod frontend;
use log::info;

lazy_static::lazy_static! {
        //1 init global config
        static ref  GLOBAL_CONFIG: config::Config = {
            config::load_config().unwrap_or_else(|_|{ config::empty()})
        };

        //global config shortcut
        static ref SHOTCUT_GLOBAL_CONFIG: config::ConfigShortcut = {
            config::ConfigShortcut{
                proxy_user_list: GLOBAL_CONFIG.load_proxy_user_list(),
            }
        };
}

fn main() -> Result<(), Box<dyn std::error::Error>> {

    println!("--Sparrow mysql/mariadb proxy running!--");
    println!("commit_id: {}compile_time: {}", COMMIT_ID, COMPILE_TIME);
    println!("------------------------------------------------------");
    //println!("global config: {:?}", *GLOBAL_CONFIG); 
    //2 init log module
    setup_logger();
    info!("log module init ok!");
    info!("Sparrow run commit_id: {}compile_time: {}", COMMIT_ID, COMPILE_TIME);
    //3. run proxy server
    info!("start to run proxy server!");
    proxy::ProxyServer::new().run()?;

    Ok(())
}

fn setup_logger() {
    let logger = femme::pretty::Logger::new();
    async_log::Logger::wrap(logger, || /* get the task id here */ 0)
        .start(GLOBAL_CONFIG.query_log_level().unwrap_or(log::LevelFilter::Trace))
        .unwrap();
}


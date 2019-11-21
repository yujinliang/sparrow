use crate::config::Config;

#[derive(Debug)]
pub struct ProxyServer <'a>{

    cfg : Option<&'a Config>,
}

impl<'a> ProxyServer<'a> {

    pub fn new( c : Option<&'a Config> ) -> ProxyServer <'a> {
        ProxyServer{cfg : c}
    }


}
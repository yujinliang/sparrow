#[derive(Debug, Clone)]
pub struct ConfigShortcut {
    pub  proxy_user_list: Option<Vec<(String, String)>>,
}
impl ConfigShortcut {
    pub fn check_proxy_user_exists(&self, user: &str) -> Option<(String, String)> {
            self.proxy_user_list.as_ref()?.iter().find(|(u, _p)| {
                u == user
            }).cloned()
    }

}
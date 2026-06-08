use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TunnelType {
    Port,
    VirtualDomain,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TunnelConfig {
    pub id: String,
    pub name: String,
    pub local_addr: String,
    pub host_header: Option<String>,
    pub tunnel_type: TunnelType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub worker_url: String,
    pub auth_token: String,
    pub tunnels: Vec<TunnelConfig>,
    #[serde(default = "default_max_logs")]
    pub max_logs: u32,
}

fn default_max_logs() -> u32 {
    200
}

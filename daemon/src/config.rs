use std::env;
use std::fs;
use std::path::PathBuf;

use serde::Deserialize;

#[derive(Deserialize)]
struct ConfigFile {
    #[serde(default)]
    daemon: DaemonConfig,
}

#[derive(Deserialize)]
#[serde(default)]
pub struct DaemonConfig {
    pub server_url: String,
    pub poll_interval: f64,
    pub hostname: String,
    pub verify_ssl: bool,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            server_url: "https://localhost:8443".to_string(),
            poll_interval: 1.0,
            hostname: String::new(),
            verify_ssl: false,
        }
    }
}

pub fn load_daemon_config() -> DaemonConfig {
    let path = config_file_path();
    let mut config = fs::read_to_string(&path)
        .ok()
        .and_then(|text| toml::from_str::<ConfigFile>(&text).ok())
        .map(|f| f.daemon)
        .unwrap_or_default();

    if config.hostname.is_empty() {
        config.hostname = env::var("HOSTNAME").unwrap_or_else(|_| {
            hostname::get()
                .map(|h| h.to_string_lossy().into_owned())
                .unwrap_or_else(|_| "unknown".to_string())
        });
    }

    config
}

fn config_file_path() -> PathBuf {
    let home = env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".config/clipshare/config.toml")
}

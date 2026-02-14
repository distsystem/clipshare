use std::env;
use std::fs;
use std::path::PathBuf;

use serde::Deserialize;

#[derive(Deserialize)]
struct ConfigFile {
    daemon: Option<DaemonToml>,
}

#[derive(Deserialize)]
struct DaemonToml {
    server_url: Option<String>,
    poll_interval: Option<f64>,
    hostname: Option<String>,
    verify_ssl: Option<bool>,
}

pub struct DaemonConfig {
    pub server_url: String,
    pub poll_interval: f64,
    pub hostname: String,
    pub verify_ssl: bool,
}

pub fn load_daemon_config() -> DaemonConfig {
    let mut config = DaemonConfig {
        server_url: "https://localhost:8443".to_string(),
        poll_interval: 1.0,
        hostname: String::new(),
        verify_ssl: false,
    };

    let path = config_file_path();
    if let Ok(text) = fs::read_to_string(&path) {
        if let Ok(file) = toml::from_str::<ConfigFile>(&text) {
            if let Some(d) = file.daemon {
                if let Some(v) = d.server_url {
                    config.server_url = v;
                }
                if let Some(v) = d.poll_interval {
                    config.poll_interval = v;
                }
                if let Some(v) = d.hostname {
                    config.hostname = v;
                }
                if let Some(v) = d.verify_ssl {
                    config.verify_ssl = v;
                }
            }
        }
    }

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

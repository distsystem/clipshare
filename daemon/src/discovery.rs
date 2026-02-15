use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use mdns_sd::{ServiceDaemon, ServiceEvent};

const SERVICE_TYPE: &str = "_clipshare._tcp.local.";

/// Maps mDNS fullname → server URL (one URL per service instance).
pub type ServerRegistry = Arc<Mutex<HashMap<String, String>>>;

pub fn spawn_listener() -> ServerRegistry {
    let registry: ServerRegistry = Arc::new(Mutex::new(HashMap::new()));
    let reg = registry.clone();

    std::thread::spawn(move || {
        let mdns = match ServiceDaemon::new() {
            Ok(d) => d,
            Err(e) => {
                log::warn!("mDNS daemon failed to start: {e}, running without auto-discovery");
                return;
            }
        };

        let receiver = match mdns.browse(SERVICE_TYPE) {
            Ok(r) => r,
            Err(e) => {
                log::warn!("mDNS browse failed: {e}");
                return;
            }
        };

        log::info!("mDNS browse started for {SERVICE_TYPE}");

        while let Ok(event) = receiver.recv() {
            match event {
                ServiceEvent::ServiceResolved(info) => {
                    let port = info.get_port();
                    let protocol = info
                        .get_property_val_str("protocol")
                        .unwrap_or("https");
                    let fullname = info.get_fullname().to_string();

                    if let Some(addr) = info.get_addresses_v4().into_iter().next() {
                        let url = format!("{protocol}://{addr}:{port}");
                        log::info!("Discovered server: {url}");
                        reg.lock().unwrap().insert(fullname, url);
                    }
                }
                ServiceEvent::ServiceRemoved(_stype, fullname) => {
                    if let Some(url) = reg.lock().unwrap().remove(&fullname) {
                        log::info!("Server removed: {url}");
                    }
                }
                _ => {}
            }
        }
    });

    registry
}

pub fn active_servers(registry: &ServerRegistry) -> Vec<String> {
    registry.lock().unwrap().values().cloned().collect()
}

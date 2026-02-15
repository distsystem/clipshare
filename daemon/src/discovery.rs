use std::collections::HashMap;
use std::net::{Ipv4Addr, UdpSocket};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const MULTICAST_GROUP: Ipv4Addr = Ipv4Addr::new(239, 255, 42, 1);
const MULTICAST_PORT: u16 = 4243;
const EXPIRY: Duration = Duration::from_secs(30);

pub type ServerRegistry = Arc<Mutex<HashMap<String, Instant>>>;

pub fn spawn_listener() -> ServerRegistry {
    let registry: ServerRegistry = Arc::new(Mutex::new(HashMap::new()));
    let reg = registry.clone();

    std::thread::spawn(move || {
        let socket = match UdpSocket::bind(("0.0.0.0", MULTICAST_PORT)) {
            Ok(s) => s,
            Err(e) => {
                log::warn!("Discovery bind failed: {e}, running without auto-discovery");
                return;
            }
        };

        if let Err(e) = socket.join_multicast_v4(&MULTICAST_GROUP, &Ipv4Addr::UNSPECIFIED) {
            log::warn!("Join multicast failed: {e}");
            return;
        }

        socket.set_read_timeout(Some(Duration::from_secs(5))).ok();
        let mut buf = [0u8; 1024];

        loop {
            match socket.recv_from(&mut buf) {
                Ok((len, src)) => {
                    if let Ok(text) = std::str::from_utf8(&buf[..len]) {
                        if let Ok(ann) = serde_json::from_str::<serde_json::Value>(text) {
                            if ann.get("service").and_then(|v| v.as_str()) == Some("clipshare") {
                                let port =
                                    ann.get("port").and_then(|v| v.as_u64()).unwrap_or(8443);
                                let protocol = ann
                                    .get("protocol")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("https");
                                let url = format!("{protocol}://{}:{port}", src.ip());
                                log::debug!("Discovered server: {url}");
                                reg.lock().unwrap().insert(url, Instant::now());
                            }
                        }
                    }
                }
                Err(ref e)
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::TimedOut => {}
                Err(e) => {
                    log::debug!("Discovery recv error: {e}");
                }
            }

            reg.lock().unwrap().retain(|_, ts| ts.elapsed() < EXPIRY);
        }
    });

    registry
}

pub fn active_servers(registry: &ServerRegistry) -> Vec<String> {
    registry
        .lock()
        .unwrap()
        .iter()
        .filter(|(_, ts)| ts.elapsed() < EXPIRY)
        .map(|(url, _)| url.clone())
        .collect()
}

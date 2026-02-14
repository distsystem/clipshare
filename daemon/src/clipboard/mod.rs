mod wayland;
mod x11;

use std::process::Command;

use crate::api::MimeContent;

pub const SUPPORTED_TYPES: &[&str] = &["text/plain", "text/html", "image/png"];

pub trait ClipboardReader {
    fn name(&self) -> &str;
    fn available(&self) -> bool;
    fn read(&self) -> Vec<MimeContent>;
}

pub fn get_reader() -> Box<dyn ClipboardReader> {
    let readers: Vec<Box<dyn ClipboardReader>> = vec![
        Box::new(wayland::WaylandReader),
        Box::new(x11::X11Reader),
    ];

    for reader in readers {
        if reader.available() {
            log::info!("Clipboard backend: {}", reader.name());
            return reader;
        }
    }

    log::error!("No supported clipboard backend found");
    std::process::exit(1);
}

pub fn cmd_exists(name: &str) -> bool {
    Command::new("which")
        .arg(name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

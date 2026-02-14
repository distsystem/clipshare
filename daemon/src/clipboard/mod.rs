use std::collections::HashSet;
use std::process::Command;

use base64::Engine;

use crate::api::MimeContent;

pub const SUPPORTED_TYPES: &[&str] = &["text/plain", "text/html", "image/png"];

struct CmdBackend {
    name: &'static str,
    env_var: &'static str,
    cmd: &'static str,
    list_args: &'static [&'static str],
    read_prefix: &'static [&'static str],
}

const WAYLAND: CmdBackend = CmdBackend {
    name: "Wayland",
    env_var: "WAYLAND_DISPLAY",
    cmd: "wl-paste",
    list_args: &["--list-types"],
    read_prefix: &["--no-newline", "--type"],
};

const X11: CmdBackend = CmdBackend {
    name: "X11",
    env_var: "DISPLAY",
    cmd: "xclip",
    list_args: &["-selection", "clipboard", "-o", "-target", "TARGETS"],
    read_prefix: &["-selection", "clipboard", "-o", "-target"],
};

impl CmdBackend {
    fn available(&self) -> bool {
        std::env::var(self.env_var).is_ok() && cmd_exists(self.cmd)
    }

    fn list_targets(&self) -> HashSet<String> {
        Command::new(self.cmd)
            .args(self.list_args)
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| {
                String::from_utf8_lossy(&o.stdout)
                    .lines()
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default()
    }

    fn read_mime(&self, mime: &str) -> Option<MimeContent> {
        let output = Command::new(self.cmd)
            .args(self.read_prefix)
            .arg(mime)
            .output()
            .ok()?;

        if !output.status.success() || output.stdout.is_empty() {
            return None;
        }

        let data = if mime.starts_with("text/") {
            String::from_utf8_lossy(&output.stdout).into_owned()
        } else {
            base64::engine::general_purpose::STANDARD.encode(&output.stdout)
        };

        Some(MimeContent {
            mime_type: mime.to_string(),
            data,
        })
    }

    fn read(&self) -> Vec<MimeContent> {
        let targets = self.list_targets();
        SUPPORTED_TYPES
            .iter()
            .filter(|&&mime| targets.contains(mime))
            .filter_map(|&mime| self.read_mime(mime))
            .collect()
    }
}

pub trait ClipboardReader {
    fn read(&self) -> Vec<MimeContent>;
}

pub fn get_reader() -> Box<dyn ClipboardReader> {
    let backends: &[&CmdBackend] = &[&WAYLAND, &X11];

    for &backend in backends {
        if backend.available() {
            log::info!("Clipboard backend: {}", backend.name);
            return Box::new(StaticBackend(backend));
        }
    }

    log::error!("No supported clipboard backend found");
    std::process::exit(1);
}

// Wrapper to own a &'static CmdBackend for boxing
struct StaticBackend(&'static CmdBackend);

impl ClipboardReader for StaticBackend {
    fn read(&self) -> Vec<MimeContent> {
        self.0.read()
    }
}

fn cmd_exists(name: &str) -> bool {
    Command::new("which")
        .arg(name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

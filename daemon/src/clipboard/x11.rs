use std::collections::HashSet;
use std::process::Command;

use base64::Engine;

use crate::api::MimeContent;
use super::{ClipboardReader, SUPPORTED_TYPES, cmd_exists};

pub struct X11Reader;

impl ClipboardReader for X11Reader {
    fn name(&self) -> &str {
        "X11"
    }

    fn available(&self) -> bool {
        std::env::var("DISPLAY").is_ok() && cmd_exists("xclip")
    }

    fn read(&self) -> Vec<MimeContent> {
        let targets = list_targets();
        let mut results = Vec::new();
        for &mime in SUPPORTED_TYPES {
            if !targets.contains(mime) {
                continue;
            }
            if let Some(c) = read_mime(mime) {
                results.push(c);
            }
        }
        results
    }
}

fn list_targets() -> HashSet<String> {
    Command::new("xclip")
        .args(["-selection", "clipboard", "-o", "-target", "TARGETS"])
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

fn read_mime(mime: &str) -> Option<MimeContent> {
    let output = Command::new("xclip")
        .args(["-selection", "clipboard", "-o", "-target", mime])
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

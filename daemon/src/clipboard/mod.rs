use base64::Engine;
use png::EncodingError;

use crate::api::MimeContent;

pub struct ClipboardReader {
    inner: arboard::Clipboard,
}

impl ClipboardReader {
    pub fn new() -> Self {
        match arboard::Clipboard::new() {
            Ok(clipboard) => {
                log::info!("Clipboard backend: arboard");
                Self { inner: clipboard }
            }
            Err(e) => {
                log::error!("Failed to initialize clipboard: {e}");
                std::process::exit(1);
            }
        }
    }

    pub fn read(&mut self) -> Vec<MimeContent> {
        let mut contents = Vec::new();

        if let Ok(text) = self.inner.get_text() {
            if !text.is_empty() {
                contents.push(MimeContent {
                    mime_type: "text/plain".to_string(),
                    data: text,
                });
            }
        }

        if let Ok(html) = self.inner.get().html() {
            if !html.is_empty() {
                contents.push(MimeContent {
                    mime_type: "text/html".to_string(),
                    data: html,
                });
            }
        }

        if let Ok(img) = self.inner.get_image() {
            match encode_rgba_to_png(img.width, img.height, &img.bytes) {
                Ok(png_bytes) => {
                    contents.push(MimeContent {
                        mime_type: "image/png".to_string(),
                        data: base64::engine::general_purpose::STANDARD.encode(&png_bytes),
                    });
                }
                Err(e) => log::warn!("Failed to encode clipboard image as PNG: {e}"),
            }
        }

        contents
    }
}

fn encode_rgba_to_png(
    width: usize,
    height: usize,
    rgba: &[u8],
) -> Result<Vec<u8>, EncodingError> {
    let mut buf = Vec::new();
    let mut encoder = png::Encoder::new(&mut buf, width as u32, height as u32);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(rgba)?;
    writer.finish()?;
    Ok(buf)
}

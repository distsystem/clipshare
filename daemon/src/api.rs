use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct MimeContent {
    pub mime_type: String,
    pub data: String,
}

#[derive(Serialize)]
struct CreateEntryRequest<'a> {
    source_host: &'a str,
    contents: &'a [MimeContent],
}

pub fn push_entry(
    agent: &ureq::Agent,
    server_url: &str,
    hostname: &str,
    contents: &[MimeContent],
) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("{}/api/entries", server_url.trim_end_matches('/'));
    let body = CreateEntryRequest {
        source_host: hostname,
        contents,
    };

    let resp = agent
        .post(&url)
        .set("Content-Type", "application/json")
        .send_string(&serde_json::to_string(&body)?)?;

    log::debug!("Upload response: {}", resp.status());
    Ok(())
}

mod api;
mod clipboard;
mod config;

use std::sync::Arc;
use std::time::Duration;

use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{DigitallySignedStruct, Error, SignatureScheme};
use sha2::{Digest, Sha256};

fn main() {
    let verbose = std::env::args().any(|a| a == "-v" || a == "--verbose");

    env_logger::Builder::from_env(
        env_logger::Env::default()
            .default_filter_or(if verbose { "debug" } else { "info" }),
    )
    .init();

    let config = config::load_daemon_config();
    log::info!(
        "Server: {}, host: {}, interval: {:.1}s",
        config.server_url,
        config.hostname,
        config.poll_interval
    );

    let reader = clipboard::get_reader();
    let agent = build_agent(config.verify_ssl);

    let mut last_hash = String::new();
    loop {
        std::thread::sleep(Duration::from_secs_f64(config.poll_interval));

        let contents = reader.read();
        if contents.is_empty() {
            continue;
        }

        let current_hash = hash_contents(&contents);
        if current_hash == last_hash {
            continue;
        }

        last_hash = current_hash;
        log::info!(
            "Clipboard changed, uploading ({} content(s))",
            contents.len()
        );

        if let Err(e) = api::push_entry(&agent, &config, &contents) {
            log::warn!("Upload failed: {}", e);
        }
    }
}

fn hash_contents(contents: &[api::MimeContent]) -> String {
    let mut hasher = Sha256::new();
    for c in contents {
        hasher.update(c.mime_type.as_bytes());
        hasher.update(c.data.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}

fn build_agent(verify_ssl: bool) -> ureq::Agent {
    if verify_ssl {
        return ureq::agent();
    }

    let tls_config = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(NoVerifier))
        .with_no_client_auth();

    ureq::AgentBuilder::new()
        .tls_config(Arc::new(tls_config))
        .build()
}

// Accept any server certificate (for self-signed certs).
#[derive(Debug)]
struct NoVerifier;

impl ServerCertVerifier for NoVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        vec![
            SignatureScheme::RSA_PKCS1_SHA256,
            SignatureScheme::RSA_PKCS1_SHA384,
            SignatureScheme::RSA_PKCS1_SHA512,
            SignatureScheme::ECDSA_NISTP256_SHA256,
            SignatureScheme::ECDSA_NISTP384_SHA384,
            SignatureScheme::ED25519,
            SignatureScheme::RSA_PSS_SHA256,
            SignatureScheme::RSA_PSS_SHA384,
            SignatureScheme::RSA_PSS_SHA512,
        ]
    }
}

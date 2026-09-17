//! Shared HTTPS configuration for API and media clients.
use reqwest::ClientBuilder;

pub fn client_builder() -> ClientBuilder {
    // reqwest's rustls-no-provider requires registration before Client::build.
    // An existing process-wide provider remains unchanged on repeated calls.
    let _ = rustls::crypto::ring::default_provider().install_default();
    let builder = reqwest::Client::builder();
    #[cfg(target_os = "android")]
    let builder =
        builder.tls_certs_only(webpki_root_certs::TLS_SERVER_ROOT_CERTS.iter().map(|cert| {
            // Use bundled roots without requiring Android's verifier JNI setup.
            // Merging roots would still invoke the platform verifier.
            reqwest::Certificate::from_der(cert.as_ref())
                .expect("bundled Mozilla root must be valid DER")
        }));
    builder
}

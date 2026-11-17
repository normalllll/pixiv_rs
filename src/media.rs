//! Streaming media transport shared by application image providers.
use futures_core::Stream;
use futures_util::StreamExt;
use reqwest::{
    Client, Url,
    header::{HeaderMap, HeaderName, HeaderValue},
};
use std::{
    collections::HashMap,
    pin::Pin,
    sync::{Mutex, OnceLock},
    time::Duration,
};

pub struct MediaChunk {
    pub bytes: Vec<u8>,
    pub received: u64,
    pub total: Option<u64>,
}

pub type MediaStream = Pin<Box<dyn Stream<Item = Result<MediaChunk, String>> + Send>>;

// Retain the current transport pool without retaining request cookies or headers.
type MediaClientCache = Mutex<Option<(Option<String>, Client)>>;
static CLIENT: OnceLock<MediaClientCache> = OnceLock::new();

fn client(proxy: Option<String>) -> Result<Client, String> {
    let proxy = proxy.filter(|value| !value.trim().is_empty());
    let mut cached = CLIENT
        .get_or_init(|| Mutex::new(None))
        .lock()
        .map_err(|_| "Media client unavailable")?;
    if let Some((previous, client)) = &*cached
        && previous == &proxy
    {
        return Ok(client.clone());
    }
    let mut builder = crate::http::client_builder()
        .user_agent("Mozilla/5.0")
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(Duration::from_secs(15))
        .read_timeout(Duration::from_secs(45))
        // Each picture gets its own connection. Over HTTP/2 every picture
        // in flight shares one TCP connection, whose throughput a slow or
        // proxied route caps well below what parallel connections reach.
        .http1_only()
        .pool_max_idle_per_host(16);
    if let Some(proxy) = &proxy {
        builder = builder.proxy(reqwest::Proxy::all(proxy).map_err(|_| "Invalid media proxy")?);
    }
    let client = builder
        .build()
        .map_err(|_| "Could not create media client")?;
    *cached = Some((proxy, client.clone()));
    Ok(client)
}

/// Dropping the stream cancels the request. Diagnostics omit URLs and headers.
pub fn stream_media(
    url: String,
    proxy: Option<String>,
    headers: HashMap<String, String>,
) -> Result<MediaStream, String> {
    let url = Url::parse(&url).map_err(|_| "Invalid media URL")?;
    if url.scheme() != "https"
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err("Media requires an HTTPS URL without credentials".into());
    }
    let mut parsed_headers = HeaderMap::new();
    for (name, value) in headers {
        let name =
            HeaderName::from_bytes(name.as_bytes()).map_err(|_| "Invalid media header name")?;
        if name == reqwest::header::COOKIE {
            let host = url.host_str().unwrap_or_default();
            if !(host == "fanbox.cc" || host.ends_with(".fanbox.cc"))
                || url.port_or_known_default() != Some(443)
            {
                return Err("FANBOX cookies require a FANBOX media host".into());
            }
        }
        let mut value = HeaderValue::from_str(&value).map_err(|_| "Invalid media header value")?;
        if name == reqwest::header::COOKIE || name == reqwest::header::AUTHORIZATION {
            value.set_sensitive(true);
        }
        parsed_headers.insert(name, value);
    }
    let client = client(proxy)?;
    Ok(Box::pin(async_stream::try_stream! {
        let response = client.get(url).headers(parsed_headers).send().await.map_err(|_| "Media request failed")?;
        let status = response.status();
        if !status.is_success() { Err(format!("Media HTTP {}", status.as_u16()))?; }
        let content_type = response.headers().get(reqwest::header::CONTENT_TYPE).and_then(|v| v.to_str().ok()).unwrap_or_default();
        if content_type.starts_with("text/") || content_type.starts_with("application/json") { Err("Media response is not binary content")?; }
        const MAX_IMAGE_BYTES: u64 = 256 * 1024 * 1024;
        let total = response.content_length();
        if total.is_some_and(|size| size > MAX_IMAGE_BYTES) { Err("Media exceeds image memory limit")?; }
        let mut received = 0_u64;
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let bytes = chunk.map_err(|_| "Media stream interrupted")?;
            received += bytes.len() as u64;
            if received > MAX_IMAGE_BYTES { Err("Media exceeds image memory limit")?; }
            yield MediaChunk { bytes: bytes.to_vec(), received, total };
        }
        if received == 0 { Err("Empty media response")?; }
        if total.is_some_and(|expected| received != expected) { Err("Incomplete media response")?; }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncReadExt;

    /// The TLS ClientHello of a media request, read by a local listener that
    /// never answers it.
    async fn media_client_hello() -> Vec<u8> {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let hello = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut bytes = vec![0; 4096];
            let read = socket.read(&mut bytes).await.unwrap();
            bytes.truncate(read);
            bytes
        });
        let mut stream = stream_media(
            format!("https://127.0.0.1:{port}/picture.jpg"),
            None,
            HashMap::new(),
        )
        .unwrap();
        // The request is sent when the stream is first polled; it fails once
        // the listener hangs up.
        let (hello, response) = tokio::join!(hello, stream.next());
        assert!(response.unwrap().is_err());
        hello.unwrap()
    }

    fn contains(haystack: &[u8], needle: &[u8]) -> bool {
        haystack
            .windows(needle.len())
            .any(|window| window == needle)
    }

    #[tokio::test]
    async fn media_requests_offer_only_http1() {
        let hello = media_client_hello().await;
        // ALPN names are length-prefixed: 8 "http/1.1", 2 "h2".
        assert!(contains(&hello, b"\x08http/1.1"));
        assert!(!contains(&hello, b"\x02h2"));
    }
}

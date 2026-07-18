use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixivErrorKind {
    HttpClient,
    Json,
    Url,
    InvalidHeader,
    InvalidEndpoint,
    HttpStatus,
    MissingAccount,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PixivError {
    pub kind: PixivErrorKind,
    pub message: String,
    pub status: Option<u16>,
    pub body: Option<String>,
    pub url: Option<String>,
}

impl PixivError {
    pub fn new(kind: PixivErrorKind, message: String) -> Self {
        Self {
            kind,
            message,
            status: None,
            body: None,
            url: None,
        }
    }

    pub fn http_status(status: u16, body: String) -> Self {
        Self {
            kind: PixivErrorKind::HttpStatus,
            message: format_status(status),
            status: Some(status),
            body: Some(body),
            url: None,
        }
    }

    pub(crate) fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    pub fn missing_account() -> Self {
        Self::new(
            PixivErrorKind::MissingAccount,
            "no account is available for authenticated request".to_owned(),
        )
    }
}

impl fmt::Display for PixivError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.status, self.body.as_deref()) {
            (Some(status), Some(body)) => write!(f, "{}: {status}: {body}", self.message),
            _ => f.write_str(&self.message),
        }
    }
}

impl Error for PixivError {}

impl From<reqwest::Error> for PixivError {
    fn from(error: reqwest::Error) -> Self {
        let status = error.status().map(|status| status.as_u16());
        let url = error.url().map(|url| url.to_string());
        let error = error.without_url();
        let mut pixiv_error = Self::new(PixivErrorKind::HttpClient, raw_reqwest_error(&error));
        pixiv_error.status = status;
        pixiv_error.url = url;
        pixiv_error
    }
}

impl From<serde_json::Error> for PixivError {
    fn from(error: serde_json::Error) -> Self {
        Self::new(PixivErrorKind::Json, error.to_string())
    }
}

impl From<url::ParseError> for PixivError {
    fn from(error: url::ParseError) -> Self {
        Self::new(PixivErrorKind::Url, error.to_string())
    }
}

impl From<reqwest::header::InvalidHeaderValue> for PixivError {
    fn from(error: reqwest::header::InvalidHeaderValue) -> Self {
        Self::new(PixivErrorKind::InvalidHeader, error.to_string())
    }
}

fn format_status(status: u16) -> String {
    match reqwest::StatusCode::from_u16(status) {
        Ok(status_code) => format!("status: {status_code}"),
        Err(_) => format!("status: {status}"),
    }
}

fn raw_reqwest_error(error: &reqwest::Error) -> String {
    let mut lines = Vec::new();

    lines.push(format!("reqwest display: {error}"));
    lines.push(format!("reqwest debug: {error:?}"));
    if let Some(status) = error.status() {
        lines.push(format!("reqwest status: {status}"));
    }

    let mut source = error.source();
    let mut index = 0;

    while let Some(error) = source {
        lines.push(format!("source[{index}]: {error}"));
        source = error.source();
        index += 1;
    }

    lines.join("\n")
}

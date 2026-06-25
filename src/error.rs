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
}

impl PixivError {
    pub fn new(kind: PixivErrorKind, message: String) -> Self {
        Self {
            kind,
            message,
            status: None,
            body: None,
        }
    }

    pub fn http_status(status: u16, body: String) -> Self {
        Self {
            kind: PixivErrorKind::HttpStatus,
            message: format!("request failed with status {status}"),
            status: Some(status),
            body: Some(body),
        }
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
        Self::new(PixivErrorKind::HttpClient, error.to_string())
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

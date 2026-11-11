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
    Html,
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

    /// Pixiv's message for one request field (such as `profile_image`), read
    /// from the error body, when the body carries one.
    pub fn field_message(&self, field: &str) -> Option<String> {
        let message = serde_json::from_str::<crate::ErrorMessage>(self.body.as_deref()?).ok()?;
        message.error.user_message_details?.remove(field)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_message_reads_the_named_field() {
        let error = PixivError::http_status(
            400,
            r#"{"error":{"user_message_details":{"profile_image":"bad","x":["a","b"]}}}"#.into(),
        );
        assert_eq!(error.field_message("profile_image").as_deref(), Some("bad"));
        assert_eq!(error.field_message("x").as_deref(), Some("a\nb"));
        assert_eq!(error.field_message("missing"), None);
    }

    #[test]
    fn field_message_tolerates_other_bodies() {
        let html = PixivError::http_status(502, "<html>".into());
        assert_eq!(html.field_message("profile_image"), None);
        assert_eq!(PixivError::missing_account().field_message("x"), None);
    }
}

use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::random_range;
use reqwest::header::{ACCEPT_LANGUAGE, HeaderMap, HeaderValue, USER_AGENT};
use reqwest::{Client, Method, Url};
use sha2::{Digest, Sha256};

use crate::error::{PixivError, PixivErrorKind};
use crate::responses::UserAccountResult;

const OAUTH_HOST: &str = "oauth.secure.pixiv.net";
const AUTH_USER_AGENT: &str = "PixivAndroidApp/6.184.0 (Android 15.0; {device})";
const CLIENT_ID: &str = "MOBrBDS8blbauoSck0ZfDbtuzpyT";
const CLIENT_SECRET: &str = "lsACyCD94FhDUtGTXi3QzcFE2uU1hqtDaKeqrdwj";
const RANDOM_KEY_SET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PixivAuthConfig {
    pub target_ip: String,
    pub language: String,
    pub device_name: String,
    pub accept_invalid_certs: bool,
}

impl PixivAuthConfig {
    pub fn new(target_ip: String, language: String, device_name: String) -> Self {
        Self {
            target_ip,
            language,
            device_name,
            accept_invalid_certs: false,
        }
    }

    pub fn with_accept_invalid_certs(mut self, accept_invalid_certs: bool) -> Self {
        self.accept_invalid_certs = accept_invalid_certs;
        self
    }
}

#[derive(Clone)]
pub struct PixivAuth {
    config: PixivAuthConfig,
    code_verifier: String,
    code_challenge: String,
}

impl PixivAuth {
    pub fn new(config: PixivAuthConfig) -> Self {
        let code_verifier = generate_code_verifier();
        let code_challenge = build_code_challenge(&code_verifier);
        Self {
            config,
            code_verifier,
            code_challenge,
        }
    }

    pub fn from_parts(target_ip: String, language: String, device_name: String) -> Self {
        Self::new(PixivAuthConfig::new(target_ip, language, device_name))
    }

    pub fn code_verifier(&self) -> String {
        self.code_verifier.clone()
    }

    pub fn code_challenge(&self) -> String {
        self.code_challenge.clone()
    }

    pub async fn refresh_auth_token(
        &self,
        refresh_token: String,
    ) -> Result<UserAccountResult, PixivError> {
        self.post_token(&[
            ("client_id".to_owned(), CLIENT_ID.to_owned()),
            ("client_secret".to_owned(), CLIENT_SECRET.to_owned()),
            ("include_policy".to_owned(), "true".to_owned()),
            ("grant_type".to_owned(), "refresh_token".to_owned()),
            ("refresh_token".to_owned(), refresh_token),
        ])
        .await
    }

    pub async fn init_account_auth_token(
        &self,
        code: String,
    ) -> Result<UserAccountResult, PixivError> {
        self.post_token(&[
            ("client_id".to_owned(), CLIENT_ID.to_owned()),
            ("client_secret".to_owned(), CLIENT_SECRET.to_owned()),
            ("include_policy".to_owned(), "true".to_owned()),
            ("grant_type".to_owned(), "authorization_code".to_owned()),
            ("code_verifier".to_owned(), self.code_verifier.clone()),
            ("code".to_owned(), code),
            (
                "redirect_uri".to_owned(),
                "https://app-api.pixiv.net/web/v1/users/auth/pixiv/callback".to_owned(),
            ),
        ])
        .await
    }

    pub fn generate_login_url(&self) -> String {
        format!(
            "https://app-api.pixiv.net/web/v1/login?code_challenge={}&code_challenge_method=S256&client=pixiv-android",
            &self.code_challenge
        )
    }

    async fn post_token(&self, form: &[(String, String)]) -> Result<UserAccountResult, PixivError> {
        let client = self.client()?;
        let url = Url::parse(&format!("https://{OAUTH_HOST}/auth/token"))?;
        let mut last_error = None;

        for attempt in 0..=2 {
            let response = client
                .request(Method::POST, url.clone())
                .headers(self.headers()?)
                .form(form)
                .send()
                .await;

            match response {
                Ok(response) => {
                    let status = response.status();
                    let body = response.text().await?;
                    if status.is_success() {
                        return Ok(serde_json::from_str(&body)?);
                    }
                    return Err(PixivError::http_status(status.as_u16(), body));
                }
                Err(error) if is_retryable_error(&error) && attempt < 2 => {
                    last_error = Some(error);
                }
                Err(error) => return Err(error.into()),
            }
        }

        Err(last_error.map(PixivError::from).unwrap_or_else(|| {
            PixivError::new(PixivErrorKind::HttpClient, "request failed".to_owned())
        }))
    }

    fn client(&self) -> Result<Client, PixivError> {
        let mut builder = Client::builder()
            .connect_timeout(Duration::from_secs(6))
            .timeout(Duration::from_secs(6))
            .danger_accept_invalid_certs(self.config.accept_invalid_certs);

        if let Ok(ip) = self.config.target_ip.parse::<IpAddr>() {
            builder = builder.resolve(OAUTH_HOST, SocketAddr::new(ip, 443));
        }

        Ok(builder.build()?)
    }

    fn headers(&self) -> Result<HeaderMap, PixivError> {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_str(&AUTH_USER_AGENT.replace("{device}", &self.config.device_name))?,
        );
        headers.insert("App-OS", HeaderValue::from_static("android"));
        headers.insert("App-OS-Version", HeaderValue::from_static("11.0"));
        headers.insert("App-Version", HeaderValue::from_static("6.21.1"));
        headers.insert(
            ACCEPT_LANGUAGE,
            HeaderValue::from_str(&self.config.language)?,
        );
        Ok(headers)
    }
}

fn generate_code_verifier() -> String {
    (0..128)
        .map(|_| RANDOM_KEY_SET[random_range(0..RANDOM_KEY_SET.len())] as char)
        .collect()
}

fn build_code_challenge(code_verifier: &str) -> String {
    let hash = Sha256::digest(code_verifier.as_bytes());
    URL_SAFE_NO_PAD.encode(hash)
}

fn is_retryable_error(error: &reqwest::Error) -> bool {
    if error.is_timeout() {
        return true;
    }

    let message = error.to_string();
    message.contains("Connection closed before full header was received")
        || message.contains("Connection terminated during handshake")
        || message.contains("timed out")
}

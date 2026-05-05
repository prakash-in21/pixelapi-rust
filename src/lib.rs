use serde::{Deserialize, Serialize};
use std::time::Duration;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct Result {
    pub output_url: String,
    #[serde(default)]
    pub credits_used: f64,
    #[serde(default)]
    pub job_id: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid API key")]
    Auth,
    #[error("insufficient credits")]
    Credits,
    #[error("rate limit exceeded")]
    RateLimit,
    #[error("PixelAPI error {0}: {1}")]
    Api(u16, String),
    #[error(transparent)]
    Http(#[from] reqwest::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub struct Client {
    api_key: String,
    base_url: String,
    http: reqwest::blocking::Client,
}

impl Client {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: "https://api.pixelapi.dev".into(),
            http: reqwest::blocking::Client::builder()
                .timeout(Duration::from_secs(120))
                .build()
                .unwrap(),
        }
    }

    fn post(&self, endpoint: &str, form: &[(&str, &str)]) -> std::result::Result<Result, Error> {
        let resp = self
            .http
            .post(format!("{}{}", self.base_url, endpoint))
            .header("X-API-Key", &self.api_key)
            .form(form)
            .send()?;
        let status = resp.status().as_u16();
        let text = resp.text()?;
        match status {
            401 => Err(Error::Auth),
            402 => Err(Error::Credits),
            429 => Err(Error::RateLimit),
            400..=599 => Err(Error::Api(status, text)),
            _ => Ok(serde_json::from_str(&text).map_err(|e| Error::Api(500, e.to_string()))?),
        }
    }

    pub fn generate(&self, prompt: &str) -> std::result::Result<Result, Error> {
        self.post("/v1/image/generate", &[("prompt", prompt)])
    }

    pub fn remove_background(&self, image_url: &str) -> std::result::Result<Result, Error> {
        self.post("/v1/image/remove-background", &[("image_url", image_url)])
    }

    pub fn upscale(&self, image_url: &str, scale: u8) -> std::result::Result<Result, Error> {
        let s = scale.to_string();
        self.post("/v1/image/upscale", &[("image_url", image_url), ("scale", &s)])
    }

    pub fn save(&self, result: &Result, path: &str) -> std::result::Result<(), Error> {
        let bytes = self.http.get(&result.output_url).send()?.bytes()?;
        if let Some(parent) = Path::new(path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, bytes)?;
        Ok(())
    }
}

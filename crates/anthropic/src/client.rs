use color_eyre::eyre::{self, Context, Result};
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client as ReqwestClient, Response as ReqwestResponse};
use reqwest_eventsource::EventSource;
use std::time::Duration;

#[derive(Clone, Debug, Default)]
pub struct Client {
    reqwest: ReqwestClient,
    base_url: String,
    headers: HeaderMap,
}

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com";

fn create_headers(api_key: String) -> Result<HeaderMap> {
    let mut headers = HeaderMap::new();

    let authorization =
        HeaderValue::from_str(api_key.as_str()).context("can't create authorization header")?;
    let content_type =
        HeaderValue::from_str("application/json").context("can't create content-type header")?;
    let version =
        HeaderValue::from_str("2023-06-01").context("can't create anthropic-version header")?;

    headers.insert("anthropic-version", version);
    headers.insert("X-API-Key", authorization);
    headers.insert("Content-Type", content_type);

    Ok(headers)
}

impl Client {
    /// Creates a new client with the given API key.
    pub fn new(api_key: String) -> Result<Self> {
        let reqwest = ReqwestClient::builder()
            .timeout(Duration::from_secs(300))
            .build()
            .context("can't create reqwest client")?;

        tracing::event!(tracing::Level::INFO, "Creating API client headers...");
        let headers = create_headers(api_key).context("can't create headers")?;

        Ok(Self {
            reqwest,
            base_url: ANTHROPIC_API_URL.to_string(),
            headers,
        })
    }

    /// Changes the client base url.
    pub fn set_base_url(&mut self, base_url: String) -> &mut Self {
        self.base_url = base_url;
        self
    }

    /// Change the Anthropic API key.
    pub fn set_api_key(&mut self, api_key: String) -> Result<&mut Self> {
        self.headers = create_headers(api_key).context("can't create headers")?;
        Ok(self)
    }

    /// Makes a GET request to the Anthropic API.
    pub async fn get(&self, endpoint: &str) -> Result<ReqwestResponse> {
        let mut url = self.base_url.clone();
        url.push_str(endpoint);

        tracing::event!(tracing::Level::INFO, "GET {}", url);

        self.reqwest
            .get(url)
            .headers(self.headers.clone())
            .send()
            .await
            .context("can't send reqwest request")
    }

    /// Makes a POST request to the Anthropic API.
    pub async fn post(&self, endpoint: &str, body: String) -> Result<ReqwestResponse> {
        let mut url = self.base_url.clone();
        url.push_str(endpoint);

        tracing::event!(tracing::Level::INFO, "POST {}", url);

        self.reqwest
            .post(url)
            .headers(self.headers.clone())
            .body(body)
            .send()
            .await
            .context("can't send reqwest request")
    }

    /// Makes a POST request to the OpenAi API that returns a SSE stream.
    pub async fn post_stream(&self, endpoint: &str, body: String) -> Result<EventSource> {
        let mut url = self.base_url.clone();
        url.push_str(endpoint);

        tracing::event!(tracing::Level::INFO, "POST {}", url);

        let builder = self
            .reqwest
            .post(url)
            .headers(self.headers.clone())
            .body(body);

        match EventSource::new(builder) {
            Ok(x) => Ok(x),
            Err(e) => Err(eyre::eyre!("can't create event source: {}", e)),
        }
    }
}

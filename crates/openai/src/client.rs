use std::time::Duration;

use log;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client as ReqwestClient, Response as ReqwestResponse};

use crate::error;

#[derive(Clone, Debug, Default)]
pub struct Client {
    reqwest: ReqwestClient,
    base_url: String,
    headers: HeaderMap,
}

const OPEN_API_URL: &str = "https://api.openai.com/v1";

fn create_headers(api_key: String) -> Result<HeaderMap, error::OpenAi> {
    let mut auth = String::from("Bearer ");
    auth.push_str(&api_key);

    let mut headers = HeaderMap::new();
    let authorization = match HeaderValue::from_str(auth.as_str()) {
        Ok(x) => x,
        Err(e) => {
            return Err(error::OpenAi::RequestError {
                body: e.to_string(),
            })
        }
    };
    let content_type = match HeaderValue::from_str("application/json") {
        Ok(x) => x,
        Err(e) => {
            return Err(error::OpenAi::RequestError {
                body: e.to_string(),
            })
        }
    };

    headers.insert("Authorization", authorization);
    headers.insert("Content-Type", content_type);

    Ok(headers)
}

impl Client {
    /// Creates a new client.
    pub fn new(api_key: String) -> Result<Self, error::OpenAi> {
        let reqwest = match ReqwestClient::builder()
            .timeout(Duration::from_secs(90))
            .build()
        {
            Ok(x) => x,
            Err(e) => {
                return Err(error::OpenAi::RequestError {
                    body: e.to_string(),
                });
            }
        };

        log::debug!("Created reqwest client");

        let headers = match create_headers(api_key) {
            Ok(x) => x,
            Err(e) => {
                return Err(error::OpenAi::RequestError {
                    body: e.to_string(),
                })
            }
        };

        log::debug!("Created headers");

        Ok(Client {
            reqwest,
            headers,
            base_url: OPEN_API_URL.to_string(),
        })
    }

    /// Changes the client's base_url
    pub fn set_base_url(&mut self, base_url: String) -> &mut Self {
        self.base_url = base_url;
        self
    }

    /// Change the OpenAi API key
    pub fn set_api_key(&mut self, api_key: String) -> Result<&mut Self, error::OpenAi> {
        let headers = match create_headers(api_key) {
            Ok(x) => x,
            Err(e) => {
                return Err(error::OpenAi::RequestError {
                    body: e.to_string(),
                })
            }
        };

        self.headers = headers;
        Ok(self)
    }

    /// Makes a GET request to the OpenAi API.
    pub async fn get(&self, endpoint: &str) -> Result<ReqwestResponse, error::OpenAi> {
        let mut url = self.base_url.clone();
        url.push_str(endpoint);

        log::debug!("GET: {}", url);

        let request = self.reqwest.get(url).headers(self.headers.clone());

        match request.send().await {
            Ok(x) => Ok(x),
            Err(e) => {
                log::error!("Error: {}", e);
                Err(error::OpenAi::RequestError {
                    body: e.to_string(),
                })
            }
        }
    }

    /// Makes a POST request to the OpenAi API.
    pub async fn post(
        &self,
        endpoint: &str,
        body: String,
    ) -> Result<ReqwestResponse, error::OpenAi> {
        let mut url = self.base_url.clone();
        url.push_str(endpoint);

        log::debug!("POST: {}1", url);

        match self
            .reqwest
            .post(url)
            .headers(self.headers.clone())
            .body(body)
            .send()
            .await
        {
            Ok(x) => Ok(x),
            Err(e) => Err(error::OpenAi::RequestError {
                body: e.to_string(),
            }),
        }
    }
}

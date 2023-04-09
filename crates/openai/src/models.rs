use serde::{Deserialize, Serialize};

use crate::client::Client;
use crate::error;
use log;

/// ModelsAPI struct.
#[derive(Debug, Default)]
pub struct ModelsApi {
    client: Client,
}

/// OpenAi Completions Model.
#[derive(Serialize, Deserialize, Debug)]
pub struct Model {
    pub id: String,
    pub object: String,
    pub owned_by: String,
    pub created: i64,
    pub permission: Vec<ModelPermission>,
    pub root: String,
    pub parent: Option<String>,
}

/// OpenAi Model permissions.
#[derive(Serialize, Deserialize, Debug)]
pub struct ModelPermission {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub allow_create_engine: bool,
    pub allow_sampling: bool,
    pub allow_logprobs: bool,
    pub allow_search_indices: bool,
    pub allow_view: bool,
    pub allow_fine_tuning: bool,
    pub organization: String,
    pub group: Option<String>,
    pub is_blocking: bool,
}

/// OpenAi Models Request Body
#[derive(Serialize, Deserialize, Debug)]
pub struct ModelsRequestBody {
    pub data: Vec<Model>,
    pub object: String,
}

impl ModelsApi {
    pub fn new(client: Client) -> Self {
        let client = Self {
            client: client.clone(),
        };

        log::debug!("Created ModelsApi");

        client
    }

    pub async fn list(&self) -> Result<Vec<Model>, error::OpenAi> {
        let body = match self.client.get("/models").await {
            Ok(response) => match response.text().await {
                Ok(text) => text,
                Err(e) => {
                    return Err(error::OpenAi::RequestError {
                        body: e.to_string(),
                    })
                }
            },
            Err(e) => {
                return Err(error::OpenAi::RequestError {
                    body: e.to_string(),
                })
            }
        };

        let body: ModelsRequestBody = match serde_json::from_str(&body) {
            Ok(body) => body,
            Err(e) => {
                return Err(error::OpenAi::RequestError {
                    body: e.to_string(),
                })
            }
        };

        Ok(body.data)
    }
}

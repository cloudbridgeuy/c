use log::{debug, info, warn, error};
use std::time::Duration;
use reqwest::{
    blocking::Client,
    header::{HeaderMap, HeaderValue},
};
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use crate::util::get_current_date;
use std::fmt;

#[derive(Debug, Clone)]
struct PromptTooLongError;

impl fmt::Display for PromptTooLongError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "prompt is too long")
    }
}

impl Error for PromptTooLongError {}

type BoxResult<T> = Result<T, Box<dyn Error>>;

#[derive(Serialize, Deserialize, Debug)]
struct Prompt {
    model: String,
    temperature: f32,
    top_p: f32,
    n: u32,
    presence_penalty: f32,
    frequency_penalty: f32,
    messages: Vec<ChatMessage>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug)]
pub struct GPTClient {
    api_key: String,
    last_request_path: String,
    prompt: Prompt,
    url: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ChatResponseMessage {
    content: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ChatResponse {
    message: ChatResponseMessage
}

#[derive(Serialize, Deserialize, Debug)]
struct ChatError {
    message: String
}

#[derive(Serialize, Deserialize, Debug)]
struct Response {
    error: Option<ChatError>,
    choices: Option<Vec<ChatResponse>>,
}

const OPEN_API_URL: &str = "https://api.openai.com/v1/chat/completions";
const MODEL: &str = "gpt-3.5-turbo";
const TEMPERATURE: f32 = 0.0;
const TOP_P: f32 = 0.8;
const N: u32 = 1;
const PRESENCE_PENALTY: f32 = 0.0;
const FREQUENCY_PENALTY: f32 = 0.0;

fn retry_make_request<F>(client: &mut GPTClient, make_request: F) -> BoxResult<String>
where
    F: Fn(&mut GPTClient) -> BoxResult<String>,
{
    let mut retries = 0;
    loop {
        match make_request(client) {
            Ok(response) => return Ok(response),
            Err(err) if err.is::<PromptTooLongError>() => {
                retries += 1;

                if retries > 5 {
                    error!("Failed to make request after 5 retries");
                    std::process::exit(1);
                }

                if client.prompt.messages.len() < 3 {
                    error!("Prompt is to big. Reduce the prompt size or delete the {} file", client.last_request_path);
                    std::process::exit(1);
                }
                info!("Removing oldest chat interaction");
                client.prompt.messages.remove(1);
                client.prompt.messages.remove(1);
                debug!("Retrying request [{}]", retries);
            },
            Err(e) => {
                error!("Uncaught error: {:#?}", e);
                std::process::exit(2);
            }
        }
    }
}

fn make_api_request(client: &mut GPTClient) -> BoxResult<String> {
    info!("Calculating estimated_tokens");
    let estimated_tokens = client.prompt.messages.iter()
        .map(|s| s.content.chars().count())
        .sum::<usize>() as f32 / 4.0;
    info!("estimated_tokens = {}", estimated_tokens);

    if estimated_tokens > crate::MAX_TOKENS as f32 / 2.0 {
        info!("Estimated tokens is bigger than {}. Reducing the prompt context and retrying", crate::MAX_TOKENS as f32 / 2.0);
        client.prompt.messages.remove(1);
        client.prompt.messages.remove(1);
        return make_api_request(client);
    }
    info!("Estimated tokens are less than {}.", crate::MAX_TOKENS as f32 / 2.0);

    info!("Creating auth string from OPEN_AI_KEY");
    let mut auth = String::from("Bearer ");
    auth.push_str(&client.api_key);

    info!("Creating request headers");
    let mut headers = HeaderMap::new();
    headers.insert("Authorization", HeaderValue::from_str(auth.as_str())?);
    headers.insert("Content-Type", HeaderValue::from_str("application/json")?);

    info!("Serializing request body");
    let body = serde_json::to_string(&client.prompt)?;
    debug!("body: {:#?}", body);

    info!("Making request");
    let http = Client::builder().timeout(Duration::from_secs(90)).build()?;
    let mut response_body = String::new();
    let mut response = http.post(&client.url).headers(headers).body(body).send()?;
    info!("Reading response body");
    response.read_to_string(&mut response_body)?;
    debug!("response_body: {:#?}", response_body);

    return process_json_object(&response_body);
}

fn process_json_object(json_str: &str) -> BoxResult<String> {
    info!("Deserializing response body");
    let object: Response = serde_json::from_str(json_str)?;

    info!("Checking if an error was returned");
    if object.error.is_some() {
        warn!("TODO: Check the error message to see if the issue is related to the prompt size");
        error!("Response error: {:#?}", object.error);
        return Err(Box::new(PromptTooLongError {}));
    }

    info!("Getting response content");
    match object.choices {
        Some(choices) => Ok(choices[0].message.content.to_string()),
        None => {
            error!("No content could be found: {:#?}", object);
            std::process::exit(2);
        }
    }
}

impl GPTClient {
    pub fn new(api_key: String) -> Self {
        info!("Creating GPTClient");
        GPTClient {
            api_key,
            url: String::from(OPEN_API_URL),
            last_request_path: String::from(crate::CONFIG_DIRECTORY_PATH) + "/" + &String::from(crate::LAST_REQUEST_FILE),
            prompt: Prompt {
                model: String::from(MODEL),
                temperature: TEMPERATURE,
                top_p: TOP_P,
                n: N,
                presence_penalty: PRESENCE_PENALTY,
                frequency_penalty: FREQUENCY_PENALTY,
                messages: Vec::new()
            }
        }
    }

    fn serialize_and_store(&self) -> std::io::Result<()> {
        info!("Serializing prompt messages");
        let serialized = serde_json::to_string(&self.prompt.messages)?;
        info!("Opening/creating storage file {}", &self.last_request_path);
        let mut file = File::create(&self.last_request_path)?;
        info!("Writing prompt messages to {}", &self.last_request_path);
        file.write_all(serialized.as_bytes())?;
        Ok(())
    }

    fn read_and_deserialize(&self) -> BoxResult<Vec<ChatMessage>> {
        info!("Opening/creating storage config directory {}", crate::CONFIG_DIRECTORY_PATH);
        match crate::util::create_dir_if_not_exist(crate::CONFIG_DIRECTORY_PATH) {
            Ok(_) => (),
            Err(e) => {
                error!("failed to create config directory: {}", e);
                std::process::exit(1);
            }
        }

        info!("Opening storage last request file {}", &self.last_request_path);
        let mut file = File::open(&self.last_request_path)?;
        info!("Reading storage last request file {}", &self.last_request_path);
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        debug!("contents: {}", contents);

        info!("Serializing content");
        let chat_messages: Vec<ChatMessage> = serde_json::from_str(&contents)?;
        debug!("chat_messages: {:#?}", chat_messages);

        Ok(chat_messages)
    }

        pub fn prompt(&mut self, prompt: String) -> BoxResult<String> {
        info!("Creating system message prompt");
        self.prompt.messages.push(ChatMessage {
            role: String::from("system"),
            content: String::from(format!("You are an intelligent language model designed to create programming code in any language. All prompts will include the language to use as the first word. OUTPUT MUST BE CODE. NEVER ADD ANY TEXT THAT IS NOT CODE. DO NOT INCLUDE THE PROGRAMMING LANGUAGE. DO NOT EXPLAIN THE CODE OR ADD ADDITIONAL CONTEXT. DON'T MENTION THE PROGRAMMING LANGUAGE AND ALWAYS USE TWO SPACES INSTEAD OF TABS. Current date: {{ {} }}", get_current_date()))});
        debug!("system message: {:#?}", self.prompt.messages[0]);

        info!("Loading last request file");
        let last_request: Vec<ChatMessage> = match self.read_and_deserialize() {
            Ok(v) => v,
            Err(_) => Vec::new()
        };

        info!("Adding last requests to the message prompts");
        for vector in last_request {
            self.prompt.messages.push(vector);
        }

        info!("Adding current prompt");
        let message = ChatMessage {
            role: String::from("user"),
            content: String::from(&prompt)
        };

        self.prompt.messages.push(message);

        info!("Running request in retry mode");
        let content = retry_make_request(self, make_api_request)?;

        debug!("content = {}", content);

        info!("Storing resoonse in last request file");
        self.prompt.messages.drain(..1);
        self.prompt.messages.push(ChatMessage {
            role: String::from("assistant"),
            content: content.to_string()
        });
        self.serialize_and_store()?;

        return Ok(String::from(content));
    }
}

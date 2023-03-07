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
const TEMPERATURE: f32 = 0.2;
const TOP_P: f32 = 1.0;
const N: u32 = 1;
const MAX_TOKENS: u32 = 4096;
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
                    panic!("Failed to make request after 5 retries");
                }

                if client.prompt.messages.len() < 3 {
                    panic!("Prompt is to big. Reduce the prompt size or delete the {} file", client.last_request_path);
                }
                client.prompt.messages.remove(1);
                client.prompt.messages.remove(1);

                println!("Prompt is too long. Retrying with less history");
            },
            Err(e) => panic!("{:#?}", e),
        }
    }
}

fn make_api_request(client: &mut GPTClient) -> BoxResult<String> {
    let mut auth = String::from("Bearer ");
    auth.push_str(&client.api_key);

    let mut headers = HeaderMap::new();
    headers.insert("Authorization", HeaderValue::from_str(auth.as_str())?);
    headers.insert("Content-Type", HeaderValue::from_str("application/json")?);
    let body = serde_json::to_string(&client.prompt)?;

    let http = Client::new();
    let mut response_body = String::new();
    let mut response = http.post(&client.url).headers(headers).body(body).send()?;
    response.read_to_string(&mut response_body)?;

    return process_json_object(&response_body);
}

fn process_json_object(json_str: &str) -> BoxResult<String> {
    let object: Response = serde_json::from_str(json_str)?;

    if object.error.is_some() {
        return Err(Box::new(PromptTooLongError {}));
    }

    match object.choices {
        Some(choices) => Ok(choices[0].message.content.to_string()),
        None => panic!("No choices found"),
    }
}


impl GPTClient {
    pub fn new(api_key: String) -> Self {
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
        let serialized = serde_json::to_string(&self.prompt.messages)?;
        let mut file = File::create(&self.last_request_path)?;
        file.write_all(serialized.as_bytes())?;
        Ok(())
    }

    fn read_and_deserialize(&self) -> BoxResult<Vec<ChatMessage>> {
        match crate::util::create_dir_if_not_exist(crate::CONFIG_DIRECTORY_PATH) {
            Ok(_) => (),
            Err(e) => {
                println!("failed to create config directory: {}", e);
                std::process::exit(1);
            }
        }

        let mut file = File::open(&self.last_request_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let chat_messages: Vec<ChatMessage> = serde_json::from_str(&contents)?;

        Ok(chat_messages)
    }


    pub fn prompt(&mut self, prompt: String) -> BoxResult<String> {
        let prompt_length = prompt.len() as u32;
        if prompt_length >= MAX_TOKENS {
            return Err(format!("Prompt cannot exeed length of {} characters", MAX_TOKENS - 1).into());
        }
        self.prompt.messages.push(ChatMessage {
            role: String::from("system"),
            content: String::from(format!(
r#"You are a senior software engineer with years of experience working with multiple programming languages.
I'm going to ask you a series of questions regarding software engineering and I want you to answer them
by returning only code. The first word of each prompt represents the language you should use. All lines
that are not code should be represented as code comments. Always use two spaces for tabs.
Current date: {{ {} }}"#, get_current_date()))});

        let prev_prompt: Vec<ChatMessage> = match self.read_and_deserialize() {
            Ok(v) => v,
            Err(_) => Vec::new()
        };

        for vector in prev_prompt {
            self.prompt.messages.push(vector);
        }

        let message = ChatMessage {
            role: String::from("user"),
            content: String::from(&prompt)
        };

        self.prompt.messages.push(message);

        let max_tokens = self.prompt.messages.iter()
                .map(|s| s.content.split_whitespace().count())
                .sum::<usize>() as f32 * 0.75;

        if max_tokens > MAX_TOKENS as f32 / 2.0 {
            self.prompt.messages.remove(1);
            self.prompt.messages.remove(1);
        }

        let content = retry_make_request(self, make_api_request)?;

        self.prompt.messages.drain(..1);
        self.prompt.messages.push(ChatMessage {
            role: String::from("assistant"),
            content: content.to_string()
        });
        self.serialize_and_store()?;

        return Ok(String::from(content));
    }
}

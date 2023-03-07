use reqwest::{
    blocking::Client,
    header::{HeaderMap, HeaderValue},
};
use serde::{Deserialize, Serialize};
use serde_json::{from_str, Value};
use std::io::Read;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;

const OPEN_API_URL: &str = "https://api.openai.com/v1/chat/completions";
const MODEL: &str = "gpt-3.5-turbo";
const TEMPERATURE: f32 = 0.2;
const TOP_P: f32 = 1.0;
const N: u32 = 1;
const MAX_TOKENS: u32 = 3096;
const PRESENCE_PENALTY: f32 = 0.0;
const FREQUENCY_PENALTY: f32 = 0.0;
const FILE_PATH: &str = "/tmp/a_previous_prompt.json";

type BoxResult<T> = Result<T, Box<dyn Error>>;

#[derive(Serialize, Deserialize, Debug)]
struct Prompt {
    model: String,
    temperature: f32,
    top_p: f32,
    n: u32,
    max_tokens: u32,
    presence_penalty: f32,
    frequency_penalty: f32,
    messages: Vec<ChatMessage>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ChatMessage {
    role: String,
    content: String,
}

pub struct GPTClient {
    api_key: String,
    url: String,
    prev_prompt_path: String,
}

impl GPTClient {
    pub fn new(api_key: String) -> Self {
        GPTClient {
            api_key,
            url: String::from(OPEN_API_URL),
            prev_prompt_path: String::from(FILE_PATH),
        }
    }

    fn serialize_and_store(&self, messages: Vec<ChatMessage>) -> std::io::Result<()> {
        let serialized = serde_json::to_string(&messages)?;
        let mut file = File::create(&self.prev_prompt_path)?;
        file.write_all(serialized.as_bytes())?;
        Ok(())
    }

    fn read_and_deserialize(&self) -> BoxResult<Vec<ChatMessage>> {
        let mut file = File::open(&self.prev_prompt_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let chat_messages: Vec<ChatMessage> = serde_json::from_str(&contents)?;

        Ok(chat_messages)
    }


    pub fn prompt(&self, prompt: String) -> BoxResult<String> {
        let prompt_length = prompt.len() as u32;
        if prompt_length >= MAX_TOKENS {
            return Err(format!("Prompt cannot exeed length of {} characters", MAX_TOKENS - 1).into());
        }
        let mut p = Prompt {
            model: String::from(MODEL),
            temperature: TEMPERATURE,
            top_p: TOP_P,
            n: N,
            max_tokens: MAX_TOKENS,
            presence_penalty: PRESENCE_PENALTY,
            frequency_penalty: FREQUENCY_PENALTY,
            messages: vec![
                ChatMessage {
                    role: String::from("system"),
                    content: String::from(r#"
You are a senior software engineer with years of experience working with multiple programming languages.
I'm going to ask you a series of questions regarding software engineering and I want you to answer them
by returning code examples related to the given problem. The first word of each prompt represents the
language you should use to give your answer.

Every part of your answer that is not part of the code example should be written as a code comment.

All your lines that are not code related should have a lenght of less than 90 characters.

For example, if I give you the following prompt:

"""
bash script that can start a recording from the cli and then store it as an mp3 file
"""

You should return something like this:

"""
# Record an mp3 audio file

echo Enter the recording file path
read file_path
rec -c 1 -r 16000 -b 16 -e signed-integer -t raw - |
    sox -t raw -r 16000 -b 16 -e signed-integer - -t mp3 "$file_path"

# This code assumes that you have `rec` and `sox` available on your system
# and that you are running this from macOS.
"""
"#)
                },
            ],
        };

        let prev_prompt: Vec<ChatMessage> = match self.read_and_deserialize() {
            Ok(v) => v,
            Err(_) => Vec::new()
        };

        for vector in prev_prompt {
            p.messages.push(vector);
        }

        let message = ChatMessage {
            role: String::from("user"),
            content: String::from(&prompt)
        };

        p.messages.push(message);

        let mut auth = String::from("Bearer ");
        auth.push_str(&self.api_key);

        let mut headers = HeaderMap::new();
        headers.insert("Authorization", HeaderValue::from_str(auth.as_str())?);
        headers.insert("Content-Type", HeaderValue::from_str("application/json")?);

        let body = serde_json::to_string(&p)?;

        let client = Client::new();
        let mut res = client.post(&self.url)
            .body(body)
            .headers(headers)
            .send()?;

        let mut response_body = String::new();
        res.read_to_string(&mut response_body)?;
        let json_object: Value = from_str(&response_body)?;
        let body = json_object["choices"][0]["message"]["content"].as_str();

        let answer = match body {
            Some(a) => {
                self.serialize_and_store(vec![ChatMessage {
                    role: String::from("user"), content: prompt
                }, ChatMessage {
                    role: String::from("assistant"), content: a.to_string()
                }])?;
                Ok(String::from(a))
            }
            None => Err(format!("JSON parse error: {response_body}").into()),
        };

        return answer;
    }
}

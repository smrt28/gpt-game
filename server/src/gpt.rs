

use anyhow::{anyhow, Context, Result};
use log::{error, info};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{config, string_enum};

string_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Model {
        Gpt5 => "gpt-5",
        Gpt5Mini => "gpt-5-mini",
        Gpt5Nano => "gpt-5-nano",
    }
}

pub struct Answer {
    response: Response,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum ContentPart {
    #[serde(rename = "output_text")]
    OutputText { text: String },
    #[serde(other)]
    Other,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum OutputItem {
    #[serde(rename = "message")]
    Message {
        #[serde(default)]
        content: Vec<ContentPart>
    },
    #[serde(other)]
    Other,
}


#[derive(Deserialize)]
pub struct Response {
    #[serde(default)]
    output: Vec<OutputItem>,
}

impl Response {
    pub fn first_output_text_typed(&self) -> Option<&str> {
        for item in &self.output {
            if let OutputItem::Message { content } = item {
                for part in content {
                    if let ContentPart::OutputText { text } = part {
                        return Some(text.as_str());
                    }
                }
            }
        }
        None
    }
}


impl Answer {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let response = serde_json::from_slice(bytes).context("JSON parse failed")?;
        Ok(Self { response })
    }

    pub fn to_string(&self) -> Option<String> {
        self.response.first_output_text_typed().map(ToString::to_string)
    }

}

pub struct GptClient {
    client: reqwest::Client,
    key: String,
}

string_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Verbosity {
        Low => "low",
        Medium => "medium",
        High => "high",
    }
}

pub struct QuestionParams {
    verbosity: Verbosity,
    model: Model,
    instructions: Option<String>,
    max_output_tokens: Option<i32>,
    temperature: Option<f32>,
}

impl Default for QuestionParams {
    fn default() -> Self {
        Self {
            verbosity: Verbosity::Medium,
            model: Model::Gpt5Nano,
            instructions: None,
            max_output_tokens: None,
            temperature: None,
        }
    }
}

impl QuestionParams {
    pub fn set_instructions<S: AsRef<str>>(&mut self, instructions: S) {
        let s = instructions.as_ref().trim();
        if s.len() > 1 {
            self.instructions = Some(s.to_owned());
        }
    }
}

#[derive(Serialize)]
struct RequestBody<'a> {
    model: String,
    input: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    instructions: Option<&'a str>,
    text: serde_json::Value,
    max_output_tokens: Option<i32>,
    temperature: Option<f32>,
}

impl GptClient {
    pub fn new(config: &config::Gpt) -> Self {
        Self {
            client: reqwest::Client::new(),
            key: config.gpt_key.clone(),
        }
    }

    fn get_key(&self) -> &str {
        &self.key
    }

    pub async fn ask(&self, question: &str, params: &QuestionParams) -> Result<Answer> {
        info!("Asking...");
        let body = RequestBody {
            model: params.model.to_string(),
            input: question,
            temperature: params.temperature,
            instructions: params.instructions.as_deref(),
            max_output_tokens: params.max_output_tokens,
            text: json!({ "verbosity": params.verbosity.to_string() }),
        };
        let body = serde_json::to_value(&body)?;
        
        let resp = self.client
            .post("https://api.openai.com/v1/responses")
            .header(CONTENT_TYPE, "application/json")
            .header(AUTHORIZATION, format!("Bearer {}", self.get_key()))
            .json(&body)
            .send()
            .await
            .map_err(|err| {
                error!("OpenAI error: {}", err);
                anyhow!("OpenAI error: {}", err)
            })?;

        let status = resp.status();
        let bytes = resp.bytes().await.context("Reading body failed")?;
        
        if !status.is_success() {
            let text = String::from_utf8_lossy(&bytes);
            error!("OpenAI error {}: {}", status, text);
            anyhow::bail!("OpenAI error {}: {}", status, text);
        }

        Answer::from_bytes(&bytes)
    }
}




#![allow(unused_attributes)]
#![allow(unused_imports)]
#![allow(dead_code)]

//use crate::fs::TryLockError::Error;
use crate::{config, string_enum};
use anyhow::{anyhow, Context, Result};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use log::{error, info};
use serde::{Deserialize, Serialize};

string_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Model {
        //Gpt4o => "gpt-4o-mini",
        Gpt5  => "gpt-5",
        Gpt5Mini => "gpt-5-mini",
        Gpt5Nano => "gpt-5-nano",
    }
}

pub struct Answer {
    response: Response,
    json: Value,
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
    #[serde(other)] // ignore "reasoning" or anything else
    Other,
}


#[derive(Deserialize)]
pub struct Response {
    #[serde(default)]
    output: Vec<OutputItem>,


    //output_text: Option<String>, // sometimes provided by API
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
        Ok(Self {
            json: serde_json::from_slice(&bytes).context("JSON parse failed")?,
            response: serde_json::from_slice(&bytes)?,
        })
    }

    pub fn to_string(&self) -> Option<String> {
        self.response.first_output_text_typed().map(|s| s.to_string())
    }

    pub fn dump(&self) {
        if let Ok(s) = serde_json::to_string_pretty(&self.json) {
            println!("{}", s);
        }
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

impl QuestionParams {
    pub fn default() -> Self {
        Self {
            verbosity: Verbosity::Medium,
            model: Model::Gpt5Nano,
            instructions: None,
            max_output_tokens: None,
            temperature: None,
        }
    }

    #[allow(dead_code)]
    pub fn set_model(&mut self, model: Model) {
        self.model = model;
    }

    #[allow(dead_code)]
    pub fn set_instructions<S: AsRef<str>>(&mut self, instructions: S) {
        let s = instructions.as_ref().trim();
        if s.len() <= 1 { return };
        self.instructions = Some(s.to_owned());
    }

    #[allow(dead_code)]
    pub fn set_max_output_tokens(&mut self, max_output_tokens: i32) {
        self.max_output_tokens = Some(max_output_tokens);
    }

    #[allow(dead_code)]
    pub fn set_temperature(&mut self, temperature: f32) {
        self.temperature = Some(temperature);
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
            key: config.gpt_key.clone()
        }
    }

    fn get_key(&self) -> anyhow::Result<&str> {
        Ok(&self.key)
    }

    pub async fn ask(&self, question: &str, params: &QuestionParams) -> Result<Answer, anyhow::Error> {
        info!("asking...");
        let body = RequestBody {
            model: params.model.to_string(),
            input: question,
            temperature: params.temperature,
            instructions: params.instructions.as_deref(),
            max_output_tokens: params.max_output_tokens,
            text: json!({ "verbosity": params.verbosity.to_string() }),
        };
        info!("asking...1");
        let body = serde_json::to_value(&body)?;



        info!("asking...2");
        let resp = self.client
            .post("https://api.openai.com/v1/responses")
            .header(CONTENT_TYPE, "application/json")
            .header(AUTHORIZATION, format!("Bearer {}", self.get_key()?))
            .json(&body)
            .send()
            .await;


        let resp = match resp {
            Ok(resp) => resp,
            Err(err) => {
                error!("OpenAI error: {}", err.to_string());
                return Err(anyhow!("OpenAI error: {}", err.to_string()));
            }
        };

        info!("asking...3");
        let status = resp.status();
        info!("asking...4");
        let bytes = resp.bytes().await.context("reading body failed")?;
        info!("asking...5");
        if !status.is_success() {
            let text = String::from_utf8_lossy(&bytes);
            anyhow::bail!("OpenAI error {}: {}", status, text);
        }

        info!("got responseX");
        Ok(Answer::from_bytes(&bytes)?)
    }
}


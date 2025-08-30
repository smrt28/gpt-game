#![allow(dead_code, unused_imports)]
use serde::{Serialize, Deserialize};
use serde_json::json;
use serde_with::skip_serializing_none;
use log::{error, info};
use serde::de::DeserializeOwned;
/*
trait MessageId {
    fn get_message_id(&self) -> &str;
}
*/
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Verdict {
    #[serde(rename = "yes")]
    Yes,
    #[serde(rename = "no")]
    No,
    #[serde(rename = "unable")]
    Unable,
    #[serde(rename = "not_set")]
    NotSet
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Question {
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Answer {
    pub verdict: Option<Verdict>,
    pub comment: Option<String>,
    timestamp: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Record {
    pub questions: Question,
    pub answers: Option<Answer>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum GameError {
    #[serde(rename = "error")]
    GPTError(String),
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[derive(Default)]
pub struct GameState {
    pub subject: Option<String>,
    pub records: Vec<Record>,
    pub pending_question: Option<Question>,
    pub error: Option<GameError>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Status {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "ok")]
    Ok,
    #[serde(rename = "error")]
    Error,
}


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ServerResponse<Content: Serialize> {
    pub status: Status,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Content>,
}


impl <Content: DeserializeOwned + Serialize> ServerResponse::<Content> {
    pub fn from_response(response: &str) -> Result<Self, anyhow::Error> {
        Ok(serde_json::from_str::<ServerResponse<Content>>(response)?)
    }

    pub fn to_response(&self) -> Result<String, anyhow::Error> {
        Ok(serde_json::to_string(self)?)
    }

    pub fn from_status(status: Status) -> Self {
        Self {
            status: status,
            content: None,
        }
    }
}


pub fn response_to_content<Content>(res: &str) -> anyhow::Result<Content>
where
    Content: DeserializeOwned + Serialize,
{
    let r: ServerResponse<Content> = serde_json::from_str(res)?;
    r.content.ok_or_else(|| anyhow::anyhow!("missing content"))
}

pub fn content_to_response<Content: Serialize>(status: Status, content: Content) -> String {
    let rv = ServerResponse::<Content> {
        status: status.clone(),
        content: Some(content)
    };

    if let Ok(rv) = serde_json::to_string(&rv) {
        return rv;
    };

    error!("failed to serialize response");
    json!({
            "status": "error",
    }).to_string()
}


pub fn status_response(status: Status) -> String {
    json!({
            "status": status
        }).to_string()
}

impl Answer {
    pub fn new() -> Self {
        let now = chrono::Utc::now();
        Self {
            verdict: None,
            comment: None,
            timestamp: now.to_rfc3339(),
        }
    }
}

impl Record {
    pub fn new(question: String) -> Self {
        Record {
            questions: Question {text: question},
            answers: None,
        }
    }

    pub fn set_answer(&mut self, answer: &Answer) {
        self.answers = Some(answer.clone());
    }
}


impl GameState {
    pub fn add_record(&mut self, record: Record) {
        self.records.push(record);
    }
}
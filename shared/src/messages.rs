#![allow(dead_code, unused_imports)]
use serde::{Serialize, Deserialize};
use serde_json::json;
use serde_with::skip_serializing_none;
use log::{error, info};
use serde::de::DeserializeOwned;
use time::OffsetDateTime;
//use serde_with::{serde_as, TimestampMilliSeconds}; // or use Rfc3339
/*
trait MessageId {
    fn get_message_id(&self) -> &str;
}
*/
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Yes,
    No,
    Unable,
    NotSet,
    Pending
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Question {
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Answer {
    #[serde(default)]
    pub verdict: Option<Verdict>,
    #[serde(default)]
    pub comment: Option<String>,

    timestamp: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Record {
    pub questions: Question,
    #[serde(default)]
    pub answers: Option<Answer>,
}


#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[derive(Default)]
pub struct GameState {
    #[serde(default)]
    pub subject: Option<String>,
    #[serde(default)]
    pub records: Vec<Record>,
    #[serde(default)]
    pub pending_question: Option<Question>,
    #[serde(default)]
    pub error: Option<GameError>,

    #[serde(skip_serializing)]
    pub target: Option<String>,
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum GameError {
    #[serde(rename = "error")]
    GPTError(String),
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

    pub fn from_content(status :Status, content: Content) -> ServerResponse::<Content> {
        Self {
            status: status,
            content: Some(content),
        }
    }

    pub fn from_status(status: Status) -> Self {
        Self {
            status,
            content: None,
        }
    }
}

pub fn status_response(status: Status) -> String {
    json!({
            "status": status
        }).to_string()
}

impl Answer {
    pub fn new() -> Self {
        Self {
            verdict: None,
            comment: None,
            timestamp: time::OffsetDateTime::now_utc().unix_timestamp(),
        }
    }

    pub fn new_pending() -> Self {
        Self {
            verdict: Some(Verdict::Pending),
            comment: None,
            timestamp: 0,
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
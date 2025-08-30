
use serde::{Serialize, Deserialize};
use serde_json::json;
use serde_with::skip_serializing_none;

trait MessageId {
    fn get_message_id(&self) -> &str;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Debug, )]
pub struct Question {
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Answer {
    pub verdict: Option<Verdict>,
    pub comment: Option<String>,
    timestamp: String,
}

#[derive(Serialize, Deserialize, Debug, )]
pub struct Record {
    pub questions: Question,
    pub answers: Option<Answer>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum GameError {
    #[serde(rename = "error")]
    GPTError(String),
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, )]
#[derive(Default)]
pub struct GameState {
    pub subject: Option<String>,
    pub records: Vec<Record>,
    pub pending_question: Option<Question>,
    pub error: Option<GameError>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Status {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "ok")]
    Ok,
    #[serde(rename = "error")]
    Error,
}


pub fn status_response(status: Status) -> String {
    json!({
            "status": status
        }).to_string()
}

pub fn content_response<Content: Serialize>(status: Status, content: &Content) -> String {
    json!({
            "status": status,
            "content": content
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
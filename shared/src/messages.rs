use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::json;
use serde_with::skip_serializing_none;
use time::OffsetDateTime;


use crate::locale::Language;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Yes,
    No,
    Behave,
    Unable,
    Final,
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
    pub game_ended: bool,

    pub lang: Language,
}


impl Default for GameState {
    fn default() -> Self {
        Self {
            subject: None,
            records: vec![],
            pending_question: None,
            error: None,
            target: None,
            game_ended: false,
            lang: Language::English,
        }
    }
}

impl GameState {
    pub fn clear_comments(&mut self) {
        for record in &mut self.records {
            if let Some(answer) = &mut record.answers {
                answer.comment = None;
            }
        }
    }
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

    #[serde(skip_serializing_if = "Option::is_none")]
    pub invalid_token: Option<bool>,
}

impl<Content: DeserializeOwned + Serialize> ServerResponse<Content> {
    pub fn from_response(response: &str) -> Result<Self, anyhow::Error> {
        Ok(serde_json::from_str::<ServerResponse<Content>>(response)?)
    }

    pub fn to_response(&self) -> Result<String, anyhow::Error> {
        Ok(serde_json::to_string(self)?)
    }

    pub fn from_content(status: Status, content: Content) -> Self {
        Self {
            status,
            content: Some(content),
            invalid_token: None,
        }
    }

    pub fn from_status(status: Status) -> Self {
        Self {
            status,
            content: None,
            invalid_token: None,
        }
    }

    pub fn need_new_token(&self) -> bool {
        self.invalid_token.unwrap_or(true)
    }
}

pub fn status_response(status: Status) -> String {
    json!({ "status": status }).to_string()
}

pub fn parse_reply(s: &str) -> Option<(&str, &str)> {
    s.split_once(';')
        .map(|(tok, cmt)| (tok.trim(), cmt.trim()))
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

    pub fn get_final_answer(result: &str) -> Self {
        Self {
            verdict: Some(Verdict::Final),
            comment: Some(format!("{}", result)),
            timestamp: OffsetDateTime::now_utc().unix_timestamp(),
        }
    }

    pub fn parse_from_string(input: &str) -> Self {
        if let Some((token, comment)) = parse_reply(input) {
            let verdict = match token {
                "YES" => Verdict::Yes,
                "NO" => Verdict::No,
                "UNABLE" => Verdict::Unable,
                "FINAL" => Verdict::Final,
                "BEHAVE" => Verdict::Behave,
                _ => Verdict::NotSet,
            };
            Self {
                verdict: Some(verdict),
                comment: Some(comment.to_string()),
                timestamp: OffsetDateTime::now_utc().unix_timestamp(),
            }
        } else {
            Self {
                verdict: Some(Verdict::NotSet),
                comment: Some("Weird question, skip...".to_string()),
                timestamp: OffsetDateTime::now_utc().unix_timestamp(),
            }
        }
    }
}

impl Record {
    pub fn new(question: String) -> Self {
        Self {
            questions: Question { text: question },
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
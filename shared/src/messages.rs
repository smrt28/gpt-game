
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
    pending_question: Option<Question>,
    error: Option<GameError>,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GGGResponse<Content: Serialize> {
    status: Status,
    content: Content,
}


pub fn serialize_response_to_string<Content: Serialize>(status: Status, content: &Content) -> String {
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
    fn touch(&mut self) {

    }

    pub fn is_pending(&self) -> bool {
        self.pending_question.is_some()
    }

    pub fn get_pending_question(&self) -> Option<&Question> {
        self.pending_question.as_ref()
    }

    pub fn set_pending_question(&mut self, question: &String) -> bool {
        if self.pending_question.is_some() {
            return false;
        }
        self.pending_question = Some(Question{text: question.clone()});
        self.touch();
        true
    }

    pub fn add_record(&mut self, record: Record) {
        self.records.push(record);
        self.touch();
    }

    pub fn to_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self)
    }

    pub fn answer_pending_question(&mut self, answer: &Answer) -> bool {
        if self.pending_question.is_none() {
            return false;
        }
        let mut record = Record::new(self.pending_question.take().unwrap().text);
        record.set_answer(answer);
        self.add_record(record);
        self.touch();
        true
    }

    pub fn handle_error_response(&mut self, error: GameError) {
        self.error = Some(error);
        self.pending_question = None;
        self.touch();
    }
}
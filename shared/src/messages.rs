use serde::{Serialize, Deserialize};
use serde_with::skip_serializing_none;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Verdict {
    Yes, No, Unable
}

#[derive(Serialize, Deserialize, Debug, )]
pub struct Question {
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Answer {
    pub verdict: Verdict,
    pub comment: String,
}

#[derive(Serialize, Deserialize, Debug, )]
pub struct Record {
    pub questions: Question,
    pub answers: Option<Answer>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, )]
#[derive(Default)]
pub struct GameState {
    pub subject: String,
    pub records: Vec<Record>,
    pending_question: Option<Question>,
    pub versions: u32,
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
        self.versions += 1;
    }

    pub fn get_version(&self) -> u32 {
        self.versions
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
}
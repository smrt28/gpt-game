use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, Debug, )]
pub enum Verdict {
    Yes, No, Unable
}

#[derive(Serialize, Deserialize, Debug, )]
pub struct Question {
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug, )]
pub struct Answer {
    pub verdict: Verdict,
    pub comment: String,
}

#[derive(Serialize, Deserialize, Debug, )]
pub struct Record {
    pub questions: Question,
    pub answers: Option<Answer>,
}

#[derive(Serialize, Deserialize, Debug, )]
#[derive(Default)]
pub struct GameState {
    pub subject: String,
    pub records: Vec<Record>,
    pub pending_question: Option<Question>,
    pub versions: u32,
}

impl Record {
    pub fn new(question: String) -> Self {
        Record {
            questions: Question {text: question},
            answers: None,
        }
    }

    pub fn answer(&mut self, verdict: Verdict, comment: String) {
        self.answers = Some(Answer{ verdict, comment});
    }
}

impl GameState {
    fn touch(&mut self) {
        self.versions += 1;
    }

    pub fn get_version(&self) -> u32 {
        self.versions
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
}
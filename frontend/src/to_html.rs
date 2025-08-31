use yew::{html, Html};
use shared::messages::{Answer, GameState, Question, Record, Verdict};

#[derive(Default)]
pub struct ToHtmlExArgs {}

pub trait ToHtmlEx {
    fn to_html_with_args(&self, args: &ToHtmlExArgs) -> Html;
    fn to_html(&self) -> Html {
        self.to_html_with_args(&ToHtmlExArgs::default())
    }
}

impl ToHtmlEx for Question {
    fn to_html_with_args(&self, args: &ToHtmlExArgs) -> Html {
        html! {
            <div class="question">
                { self.text.to_string() }
            </div>
        }
    }
}

impl ToHtmlEx for Verdict {
    fn to_html_with_args(&self, args: &ToHtmlExArgs) -> Html {
        let (label, class) = match self {
            Verdict::Yes    => ("Yes",    "badge badge--yes"),
            Verdict::No     => ("No",     "badge badge--no"),
            Verdict::Unable => ("Unable", "badge badge--unable"),
            Verdict::NotSet => ("N/A",    "badge badge--na"),
        };

        html! {
            <div class={class}>
                { label }
            </div>
        }
    }
}

impl ToHtmlEx for Answer {
    fn to_html_with_args(&self, args: &ToHtmlExArgs) -> Html {
        html! {
            <div class="comment">
                { self.comment.as_deref().unwrap_or(&"".to_string()) }
            </div>
        }
    }
}

fn get_verdict_from_record(record: &Record) -> Verdict {
    if let Some(answer) = &record.answers {
        answer.verdict.clone().unwrap_or(Verdict::NotSet)
    } else {
        Verdict::NotSet
    }
}

impl ToHtmlEx for Record {
    fn to_html_with_args(&self, args: &ToHtmlExArgs) -> Html {
        html! {
            <div class="record record--two-col">
                { get_verdict_from_record(self).to_html() }
                <div class="qa">
                {self.questions.to_html() }
                {self.answers.as_ref().map(|a| a.to_html())}
                </div>
            </div>
        }
    }
}

fn create_pending_question_record(question: &Question) -> Record {

    Record {
        questions: question.clone(),
        answers: None,
    }
}

impl ToHtmlEx for GameState {
    fn to_html_with_args(&self, args: &ToHtmlExArgs) -> Html {
        html! {
            <div class="game">
                { for self.records.iter().map(|r| r.to_html()) }
                { if let Some(pendig_question) = &self.pending_question {

                    html! { create_pending_question_record(&pendig_question).to_html() }

                } else { html! {} }}

            </div>
        }
    }
}
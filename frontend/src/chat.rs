#![allow(dead_code)]
use yew::prelude::*;

/// Verdict shown as the leading badge.
#[derive(Clone, PartialEq)]
pub enum Verdict {
    Yes,
    No,
    Unable,
    Other(String), // e.g. "Maybe", "N/A", "Hmm"
}

impl Verdict {
    fn label(&self) -> &str {
        match self {
            Verdict::Yes => "Yes",
            Verdict::No => "No",
            Verdict::Unable => "Unable",
            Verdict::Other(s) => s.as_str(),
        }
    }
    /// CSS class controlling the badge colors.
    fn class(&self) -> &'static str {
        match self {
            Verdict::Yes => "badge yes",
            Verdict::No => "badge no",
            Verdict::Unable => "badge wrong",
            Verdict::Other(_) => "badge other",
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct QaItem {
    pub question: AttrValue,
    pub verdict: Verdict,
    pub explanation: AttrValue, // keep it short; paragraph-ish
}

#[derive(Properties, PartialEq)]
pub struct QaListProps {
    pub items: Vec<QaItem>,
    /// Fired when user submits the ask box; payload is the raw text.
    #[prop_or_default]
    pub on_submit: Callback<String>,

    #[prop_or(AttrValue::from("Guesses"))]
    pub heading: AttrValue,
}

#[function_component(QaList)]
pub fn qa_list(props: &QaListProps) -> Html {
    let text = use_state(|| String::new());
    let on_change = {
        let text = text.clone();
        Callback::from(move |e: InputEvent| {
            let target = e.target_unchecked_into::<web_sys::HtmlTextAreaElement>();
            text.set(target.value());
        })
    };
    let on_submit_click = {
        let text = text.clone();
        let cb = props.on_submit.clone();
        Callback::from(move |_| {
            let val = (*text).trim().to_owned();
            if !val.is_empty() {
                cb.emit(val.clone());
                text.set(String::new());
            }
        })
    };

    html! {
        <div class="qa-wrap">
            <style>{r#"
                .qa-wrap {
                    font-family: system-ui, -apple-system, Segoe UI, Roboto, Helvetica, Arial, sans-serif;
                    color: #222;
                }
                .qa-heading { margin: 24px 0 12px; font-size: 1.5rem; font-weight: 650; }
                .qa-list { display: flex; flex-direction: column; gap: 16px; margin-bottom: 24px; }

                .qa-item {
                  display: flex;
                  align-items: flex-start;
                  background: #fff;
                  border: 1px solid #ddd;
                  border-radius: 6px;
                  box-shadow: 0 1px 3px rgba(0,0,0,0.05);
                  margin-bottom: 12px;
                }

                .badge {
                  flex: 0 0 70px;
                  display: flex;
                  align-items: center;
                  justify-content: center;
                  font-weight: 700;
                  text-transform: uppercase;
                  padding: 12px;
                  border-right: 1px solid #ddd;
                }

                .badge.yes   { color: #000; }
                .badge.no    { color: #000; }
                .badge.wrong { color: #555; font-style: italic; }

                .qa-body {
                  flex: 1;
                  padding: 12px;
                }

                .question {
                  font-weight: 600;
                  margin-bottom: 6px;
                }

                .explanation {
                  color: #555;
                }

                .explanation { color: #555; }
                .ask-box { background:#fff; border:1px solid #ddd; border-radius:8px; padding:16px; }
                .ask-label { display:block; margin-bottom:8px; font-weight:600; }
                .ask-input { width:100%; padding:10px; border:1px solid #ccc; border-radius:4px; font:inherit; resize: vertical; }
                .ask-btn { margin-top:10px; padding:10px 16px; border:none; border-radius:4px; background:#0077cc; color:white; cursor:pointer; font:inherit; }
                .ask-btn:hover { background:#005fa3; }
            "#}</style>

            <h1 class="qa-heading">{ props.heading.clone() }</h1>

            <div class="qa-list">
                { for props.items.iter().map(|it| html!{

                   <div class="qa-item">
                     <div class={it.verdict.class()}>{ it.verdict.label() }</div>
                     <div class="qa-body">
                       <div class="question">{ it.question.clone() }</div>
                       <div class="explanation">{ it.explanation.clone() }</div>
                     </div>
                   </div>

                })}
            </div>

            <div class="ask-box">
                <label class="ask-label" for="ask-input">{ "Ask a new question:" }</label>
                <textarea
                    id="ask-input"
                    class="ask-input"
                    rows={3}
                    placeholder="Type your question here…"
                    value={(*text).clone()}
                    oninput={on_change}
                />
                <br/>
                <button type="button" class="ask-btn" onclick={on_submit_click}>{ "Submit" }</button>
            </div>
        </div>
    }
}

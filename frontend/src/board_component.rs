
use gloo_storage::{LocalStorage, Storage};
use log::info;
use yew::{function_component, html, use_effect_with, use_state, Callback, Html, Properties};
use crate::{use_navigator_expect, Route};
use crate::com::{fetch_pending, fetch_text, send_question};
use crate::ask_prompt_component::*;


#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    #[prop_or_default]
    pub pending: Option<bool>,
}

#[function_component]
pub fn Board(props: &Props) -> Html {
    html! {
        <div>
        <b>{"Debug:"}</b>
        <pre>{"pending: "} {props.pending}</pre>
        </div>
    }
}
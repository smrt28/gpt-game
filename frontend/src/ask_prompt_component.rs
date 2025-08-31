use yew::{function_component, html, use_node_ref, Callback, Html, Properties};

#[derive(Properties, PartialEq)]
pub struct Props {
    #[prop_or_default]
    pub prompt: Option<String>,
    #[prop_or(100)]
    pub max_len: usize,
    #[prop_or_default]
    pub on_send: Callback<String>,
    #[prop_or(false)]
    pub disabled: bool,
}

#[function_component(AskPrompt)]
pub fn ask_prompt(props: &Props) -> Html {
    let on_send = props.on_send.clone();
    let textarea_ref = use_node_ref();
    let onclick = {
        let textarea_ref = textarea_ref.clone();
        Callback::from(move |_| {
            if let Some(el) = textarea_ref.cast::<web_sys::HtmlTextAreaElement>() {
                on_send.emit(el.value());
            }
        })
    };

    html! {
        <div class="ask-prompt">
            if let Some(p) = &props.prompt {
                <label for="ask-text">{ p }</label>
            }
            <textarea
                id="ask-text"
                ref={textarea_ref}
                disabled={props.disabled}
                maxlength={props.max_len.to_string()}
            />
            <button {onclick} disabled={props.disabled}>{ "Send" }</button>
        </div>
    }
}
use yew::{function_component, html, use_effect_with, use_node_ref, Callback, Html, Properties};
use web_sys::KeyboardEvent;

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
    #[prop_or_default]
    pub token: Option<String>,
}

#[function_component(AskPrompt)]
pub fn ask_prompt(props: &Props) -> Html {
    let on_send = props.on_send.clone();
    let textarea_ref = use_node_ref();
    let send_message = {
        let textarea_ref = textarea_ref.clone();
        let on_send = on_send.clone();
        move || {
            if let Some(el) = textarea_ref.cast::<web_sys::HtmlTextAreaElement>() {
                let value = el.value();
                on_send.emit(value);
                el.set_value(""); // clear it
                let _ = el.focus(); // keep focus
            }
        }
    };

    let onclick = {
        let send_message = send_message.clone();
        Callback::from(move |_| {
            send_message();
        })
    };

    let onkeydown = {
        let send_message = send_message.clone();
        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Enter" {
                e.prevent_default();
                if !e.shift_key() && !e.ctrl_key() {
                    send_message();
                }
            }
        })
    };

    // Focus textarea when it becomes enabled
    use_effect_with(props.disabled, {
        let textarea_ref = textarea_ref.clone();
        move |disabled| {
            if !disabled {
                if let Some(el) = textarea_ref.cast::<web_sys::HtmlTextAreaElement>() {
                    let _ = el.focus();
                }
            }
        }
    });
   
    html! {
        <div class="ask-prompt">
            if let Some(token) = &props.token {
                <div class="game-id">{ format!("Game Id: {}", token) }</div>
            }
            if let Some(p) = &props.prompt {
                <label for="ask-text">{ p }</label>
            }
            <textarea
                rows="1"
                id="ask-text"
                ref={textarea_ref}
                disabled={props.disabled}
                maxlength={props.max_len.to_string()}
                wrap="off"
                {onkeydown}
            />
            <button {onclick} disabled={props.disabled}>{ "Send" }</button>
        </div>
    }
}
use yew::html::IntoHtmlResult;
use yew::HtmlResult;

#[derive(Debug, thiserror::Error)]
pub enum WwwError {

    #[error(transparent)]
    Any(#[from] anyhow::Error),
}


use gloo::net::http::Request;
use log::info;
use shared::messages::GameTemplate;
use crate::locale::get_current_language;

pub async fn fetch_text(path: &str) -> anyhow::Result<String> {
    info!("fetching: {}", path);
    let res = Request::get(path).send().await?;
    if res.status() != 200 {
        return Err(anyhow::anyhow!("status[{}]: {}", path, res.status()));
    }
    let text = res.text().await?;
    Ok(text)
}


pub async fn send_question(token: &str, text: &str) -> anyhow::Result<String> {
    info!("asking : {}: {}", token, text);
    let path = format!("/api/game/{token}/ask");
    let request = Request::post(path.as_str()).body(text.to_string())?;
    let res = request.send().await?;
    if res.status() != 200 {
        return Err(anyhow::anyhow!("ask: server error: {}", res.status()));
    }
    info!("asked");
    Ok(res.text().await?)
}

pub async fn fetch_new_game_token() -> anyhow::Result<String> {
    let code = get_current_language().to_code();
    fetch_text(&format!("/api/game/new?lang={}", code)).await
}


pub async fn create_game_template(game_template: &GameTemplate) -> anyhow::Result<String> {
    let request = Request::post("/api/template/new")
            .body(serde_json::to_string(game_template)?)?;
    let res = request.send().await?;
    if res.status() != 200 {
        return Err(anyhow::anyhow!("create_custom_game: server error: {}", res.status()));
    }
    let text = res.text().await?;
    info!("created game template: {}", &text);
    Ok(text)
}
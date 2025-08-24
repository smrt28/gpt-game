use std::time::Duration;
use gloo::net::http::Request;
use log::info;

pub async fn fetch_text(path: &str) -> anyhow::Result<String> {
    let res = Request::get(path).send().await?;
    if res.status() != 200 {
        return Err(anyhow::anyhow!("status[{}]: {}", path, res.status()));
    }
    let text = res.text().await?;
    info!("res: {:?} {:?}", text, res.status());
    Ok(text)
}

pub async fn send_question(token: &str, text: &str) -> anyhow::Result<String> {
    let path = format!("/api/game/{token}/ask");
    let request = Request::post(path.as_str()).body(text.to_string())?;
    let res = request.send().await?;
    if res.status() != 200 {
        return Err(anyhow::anyhow!("ask: server error: {}", res.status()));
    }
    //async_std::task::sleep(Duration::new(1, 0)).await;
    info!("{}: asking: {}", token, text);
    Ok(res.text().await?)
}
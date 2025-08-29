use std::time::Duration;
use gloo::net::http::Request;
use log::info;
use serde_json::{Result, Value};

pub async fn fetch_text(path: &str) -> anyhow::Result<String> {
    let res = Request::get(path).send().await?;
    if res.status() != 200 {
        return Err(anyhow::anyhow!("status[{}]: {}", path, res.status()));
    }
    let text = res.text().await?;
    //info!("res: {:?} {:?}", text, res.status());
    Ok(text)
}

pub async fn fetch_pending(token: &str, wait: bool) -> anyhow::Result<bool> {
    let path = if wait {
        format!("/api/game/{token}/version?wait=1")
    } else {
        format!("/api/game/{token}/version")
    };

    let text = fetch_text(path.as_str()).await?;
    info!("FETCH PENDING: {}: {}", token, text);
    let v: Value = serde_json::from_str(text.as_str())?;
    if let Some(res) = v["pending"].as_bool() {
        info!("{}: pending: {}", token, res);
        Ok(res)
    } else {
        info!("{}: pending: invalid response: {:?}", token, v);
        Err(anyhow::anyhow!("pending: invalid response: {:?}", v))
    }
}



pub async fn send_question(token: &str, text: &str) -> anyhow::Result<String> {
    let path = format!("/api/game/{token}/ask");
    let request = Request::post(path.as_str()).body(text.to_string())?;
    let res = request.send().await?;
    if res.status() != 200 {
        return Err(anyhow::anyhow!("ask: server error: {}", res.status()));
    }
    //async_std::task::sleep(Duration::new(1, 0)).await;
    info!("{} text:{}", path, text);
    Ok(res.text().await?)
}



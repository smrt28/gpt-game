use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Status {
    #[serde(rename = "status")]
    pub status: String,
}



impl Status {
    pub fn should_repeat(&self) -> bool {
        self.status == "pending" || self.status == "timeout"
    }
}
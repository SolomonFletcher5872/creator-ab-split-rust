use reqwest::{Client, StatusCode};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::Value;
use std::time::Duration;

#[derive(Debug)]
pub struct InfraiError { pub code: String, pub message: String, pub status: u16 }
impl std::fmt::Display for InfraiError { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}: {}", self.code, self.message) } }
impl std::error::Error for InfraiError {}

impl InfraiError {
    fn is_not_found(&self) -> bool { self.status == StatusCode::NOT_FOUND.as_u16() }
}

#[derive(Deserialize)] struct Envelope<T> { ok: bool, data: Option<T>, error: Option<Value> }

pub struct InfraiClient { http: Client, base_url: String, key: String }
impl InfraiClient {
    pub fn new(base_url: String, key: String) -> Result<Self, reqwest::Error> { Ok(Self { http: Client::builder().timeout(Duration::from_secs(20)).build()?, base_url, key }) }
    async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, InfraiError> {
        let mut delay = Duration::from_millis(200);
        for attempt in 0..4 {
            let response = self.http.request(reqwest::Method::GET, format!("{}{}", self.base_url, path)).header("Authorization", format!("Bearer {}", self.key)).send().await.map_err(|e| InfraiError { code: "TRANSPORT".into(), message: e.to_string(), status: 0 })?;
            let status = response.status();
            let retry_after = response.headers().get("retry-after").and_then(|v| v.to_str().ok()).and_then(|v| v.parse::<u64>().ok());
            let envelope: Envelope<T> = response.json().await.map_err(|e| InfraiError { code: "DECODE".into(), message: e.to_string(), status: status.as_u16() })?;
            if envelope.ok { return envelope.data.ok_or_else(|| InfraiError { code: "EMPTY_DATA".into(), message: "successful response had no data".into(), status: status.as_u16() }); }
            if status == StatusCode::TOO_MANY_REQUESTS && attempt < 3 { tokio::time::sleep(Duration::from_secs(retry_after.unwrap_or(delay.as_secs().max(1)))).await; delay *= 2; continue; }
            let error = envelope.error.unwrap_or(Value::Null);
            let code = error.get("code").and_then(Value::as_str).unwrap_or("REQUEST_REJECTED").to_owned();
            return Err(InfraiError { code, message: error.to_string(), status: status.as_u16() });
        }
        Err(InfraiError { code: "RETRY_EXHAUSTED".into(), message: "request retries exhausted".into(), status: 429 })
    }
    pub async fn flag_value(&self, flag: &str, _subject: &str) -> Result<Option<String>, InfraiError> {
        match self.get(&format!("/v1/flags/get_value/{}", flag)).await {
            Ok(value) => Ok(value),
            Err(error) if error.is_not_found() => Ok(None),
            Err(error) => Err(error),
        }
    }
    pub async fn user_get(&self, user_id: &str) -> Result<Option<Value>, InfraiError> { self.get(&format!("/v1/auth/user/get/{}", user_id)).await }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_missing_resource_response() {
        let error = InfraiError { code: "NOT_FOUND".into(), message: "missing".into(), status: 404 };
        assert!(error.is_not_found());
    }
}

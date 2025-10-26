use std::time::Duration;

use reqwest::Client;

use crate::gemini::generate::{Content, GenerateContentRequest, Part};

pub async fn is_api_key_valid(token: &str) -> Result<bool, anyhow::Error> {
    let client = Client::builder()
        .connect_timeout(Duration::from_secs(120))
        .build()?;
    let url =
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent";

    let request_body = GenerateContentRequest {
        contents: vec![Content {
            parts: vec![Part {
                text: "Hello".to_string(),
            }],
        }],
    };

    match client
        .post(url)
        .header("Content-Type", "application/json")
        .header("x-goog-api-key", token)
        .json(&request_body)
        .send()
        .await
    {
        Ok(response) => Ok(response.status().is_success()),
        Err(_) => Ok(false),
    }
}

use serde::{Deserialize, Serialize};

use crate::cli_config::CliConfig;
use crate::gemini::http_client::create_client_with_proxy;
use std::process::Command;

#[derive(Serialize)]
pub struct Part {
    pub text: String,
}

#[derive(Serialize)]
pub struct Content {
    pub parts: Vec<Part>,
}

#[derive(Serialize)]
pub struct GenerateContentRequest {
    pub contents: Vec<Content>,
}

#[derive(Deserialize, Debug)]
struct PartResponse {
    pub text: String,
}

#[derive(Deserialize, Debug)]
struct ContentResponse {
    pub parts: Vec<PartResponse>,
}

#[derive(Deserialize, Debug)]
struct Candidate {
    pub content: ContentResponse,
}

#[derive(Deserialize, Debug)]
struct GenerateContentResponse {
    pub candidates: Option<Vec<Candidate>>,
}

const PROMPT_STR: &str = "На основе следующего git diff, сгенерируй краткое и содержательное сообщение коммита на английском языке, используя стандарт Conventional Commits (например, feat:, fix:, docs:), не включая никаких пояснительных слов, только само сообщение: \n\n";
const MAX_DIFF_SIZE: usize = 20000;

fn truncate_diff(diff: &str) -> String {
    if diff.len() <= MAX_DIFF_SIZE {
        return diff.to_string();
    }

    println!(
        "⚠️ Diff слишком большой ({} байт). Обрезаем до {} байт для экономии токенов.",
        diff.len(),
        MAX_DIFF_SIZE
    );

    let truncated = diff.chars().take(MAX_DIFF_SIZE).collect::<String>();
    format!(
        "{}\n\n... (diff усечен из-за большого размера) ...",
        truncated
    )
}

pub async fn handle_generate_command(config: CliConfig) -> Result<(), anyhow::Error> {
    let api_key = match config.gemini_api_key {
        Some(key) => key,
        None => {
            return Err(anyhow::anyhow!("Отсутствует Gemini API Key"));
        }
    };
    println!("🔑 API Key найден. Начинаем генерацию коммита...");

    let git_diff_output = Command::new("git").arg("diff").arg("--staged").output()?;
    let git_diff_str = String::from_utf8(git_diff_output.stdout)?;
    let git_diff_str = truncate_diff(&git_diff_str);

    let git_log_output = Command::new("git")
        .arg("log")
        .arg("-5")
        .arg("--pretty=format:--Commit--%nSubject: %s%nBody:%n%b")
        .output()?;
    let git_log_str = String::from_utf8(git_log_output.stdout)?;

    let mut prompt_str = PROMPT_STR.to_string();

    prompt_str.push_str("\n\n--- Контекст: Последние 5 коммитов ---\n");
    prompt_str.push_str(git_log_str.trim());
    prompt_str.push_str("\n-------------------------------------\n\n");

    prompt_str.push_str("Вот изменения, которые нужно проанализировать для нового сообщения коммита (git diff --staged):\n");
    prompt_str.push_str(git_diff_str.as_str());

    let generated_commit_message = generate_commit_message(prompt_str, api_key).await?;

    println!("\n\n\n {generated_commit_message}");

    Ok(())
}

async fn generate_commit_message(
    prompt_str: String,
    api_key: String,
) -> Result<String, anyhow::Error> {
    let client = create_client_with_proxy()?;
    let url =
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent";

    let request_body = GenerateContentRequest {
        contents: vec![Content {
            parts: vec![Part { text: prompt_str }],
        }],
    };

    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("x-goog-api-key", &api_key)
        .json(&request_body)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!("Gemini API Error ({}): {}", status, body));
    }

    let api_response: GenerateContentResponse = response.json().await?;

    let commit_message = api_response
        .candidates
        .and_then(|c| c.into_iter().next())
        .and_then(|c| c.content.parts.into_iter().next())
        .map(|p| p.text)
        .ok_or_else(|| anyhow::anyhow!("Gemini не вернул сгенерированный текст. Проверьте ответ на наличие блокировок или ошибок."))?;

    Ok(commit_message.trim().to_string())
}

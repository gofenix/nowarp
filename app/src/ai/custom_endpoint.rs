use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Configuration for a custom OpenAI-compatible AI endpoint.
/// Captured before spawning async requests so the closure doesn't need AppContext.
#[derive(Clone, Debug)]
pub struct CustomEndpointConfig {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
}

#[derive(Debug, thiserror::Error)]
pub enum CustomEndpointError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    #[error("No content in response")]
    EmptyResponse,
    #[error("Response parsing failed: {0}")]
    ParseFailed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ChoiceMessage,
}

#[derive(Debug, Deserialize)]
struct ChoiceMessage {
    content: Option<String>,
}

/// Sends a chat completion request to a custom OpenAI-compatible endpoint.
/// Returns the suggestion text on success.
pub async fn request_suggestion(
    config: CustomEndpointConfig,
    command: String,
    output: String,
    exit_code: i32,
    pwd: Option<String>,
) -> Result<String, CustomEndpointError> {
    let url = format!("{}/chat/completions", config.base_url.trim_end_matches('/'));

    let messages = build_prompt(&command, &output, exit_code, pwd.as_deref());

    let request = ChatCompletionRequest {
        model: config.model,
        messages,
        temperature: Some(0.3),
        max_tokens: Some(500),
    };

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(CustomEndpointError::RequestFailed)?;

    let mut request_builder = client
        .post(&url)
        .header("Content-Type", "application/json");

    if !config.api_key.is_empty() {
        request_builder = request_builder.bearer_auth(&config.api_key);
    }

    let response = request_builder
        .json(&request)
        .send()
        .await
        .map_err(CustomEndpointError::RequestFailed)?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(CustomEndpointError::ParseFailed(format!(
            "HTTP {status}: {body}"
        )));
    }

    let completion: ChatCompletionResponse = response
        .json()
        .await
        .map_err(CustomEndpointError::RequestFailed)?;

    let content = completion
        .choices
        .into_iter()
        .next()
        .and_then(|c| c.message.content)
        .filter(|s| !s.trim().is_empty())
        .ok_or(CustomEndpointError::EmptyResponse)?;

    Ok(content.trim().to_string())
}

fn build_prompt(
    command: &str,
    output: &str,
    exit_code: i32,
    pwd: Option<&str>,
) -> Vec<ChatMessage> {
    let pwd_line = pwd
        .map(|p| format!("\nWorking Directory: {p}"))
        .unwrap_or_default();

    let system = "You are a terminal command assistant. Analyze the command output and provide a brief, actionable suggestion. Keep your response to 1-2 sentences. Do not use markdown formatting.";

    let user = if exit_code != 0 {
        format!(
            "A command failed with exit code {exit_code}. Analyze the error and suggest how to fix it.\n\nCommand: {command}\nExit Code: {exit_code}{pwd_line}\nOutput:\n{output}"
        )
    } else {
        format!(
            "Suggest a useful next action based on the command that was just executed.\n\nCommand: {command}{pwd_line}\nOutput:\n{output}"
        )
    };

    vec![
        ChatMessage {
            role: "system".into(),
            content: system.into(),
        },
        ChatMessage {
            role: "user".into(),
            content: user,
        },
    ]
}

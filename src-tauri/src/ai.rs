use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct AiRequest {
    pub provider: String,
    pub model: String,
    pub api_key: String,
    pub message: String,
    pub instructions: Option<String>,
    pub context_documents: Vec<String>,
}

#[derive(Debug, Serialize)]
struct OpenRouterRequest {
    model: String,
    messages: Vec<ChatMessage>,
}

#[derive(Debug, Serialize)]
struct OpenAiRequest {
    model: String,
    messages: Vec<ChatMessage>,
}

#[derive(Debug, Serialize)]
struct GoogleRequest {
    contents: Vec<GoogleContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<GoogleContent>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GoogleContent {
    parts: Vec<GooglePart>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GooglePart {
    text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ChatMessage,
}

#[derive(Debug, Deserialize)]
struct GoogleResponse {
    candidates: Vec<GoogleCandidate>,
}

#[derive(Debug, Deserialize)]
struct GoogleCandidate {
    content: GoogleContent,
}

fn build_system_message(instructions: Option<&str>, docs: &[String]) -> String {
    let mut system = String::from("You are an AI interview assistant. Provide helpful, concise, and relevant answers during interviews.");

    if let Some(inst) = instructions {
        system.push_str("\n\nCustom instructions:\n");
        system.push_str(inst);
    }

    if !docs.is_empty() {
        system.push_str("\n\nContext documents:\n");
        for doc in docs {
            system.push_str(doc);
            system.push_str("\n---\n");
        }
    }

    system
}

async fn call_openrouter(request: &AiRequest) -> Result<String> {
    let client = reqwest::Client::new();
    let system_msg = build_system_message(request.instructions.as_deref(), &request.context_documents);

    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: system_msg,
        },
        ChatMessage {
            role: "user".to_string(),
            content: request.message.clone(),
        },
    ];

    let body = OpenRouterRequest {
        model: request.model.clone(),
        messages,
    };

    let response = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", request.api_key))
        .header("Content-Type", "application/json")
        .header("HTTP-Referer", "https://interviewcracker.app")
        .json(&body)
        .send()
        .await?;

    let chat_response: ChatResponse = response.json().await?;
    Ok(chat_response
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .unwrap_or_else(|| "No response".to_string()))
}

async fn call_openai(request: &AiRequest) -> Result<String> {
    let client = reqwest::Client::new();
    let system_msg = build_system_message(request.instructions.as_deref(), &request.context_documents);

    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: system_msg,
        },
        ChatMessage {
            role: "user".to_string(),
            content: request.message.clone(),
        },
    ];

    let body = OpenAiRequest {
        model: request.model.clone(),
        messages,
    };

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", request.api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    let chat_response: ChatResponse = response.json().await?;
    Ok(chat_response
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .unwrap_or_else(|| "No response".to_string()))
}

async fn call_google(request: &AiRequest) -> Result<String> {
    let client = reqwest::Client::new();
    let system_msg = build_system_message(request.instructions.as_deref(), &request.context_documents);

    let body = GoogleRequest {
        contents: vec![GoogleContent {
            parts: vec![GooglePart {
                text: request.message.clone(),
            }],
        }],
        system_instruction: Some(GoogleContent {
            parts: vec![GooglePart {
                text: system_msg,
            }],
        }),
    };

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        request.model, request.api_key
    );

    let response = client.post(&url).json(&body).send().await?;

    let google_response: GoogleResponse = response.json().await?;
    Ok(google_response
        .candidates
        .first()
        .and_then(|c| c.content.parts.first())
        .map(|p| p.text.clone())
        .unwrap_or_else(|| "No response".to_string()))
}

async fn call_nvidia(request: &AiRequest) -> Result<String> {
    let client = reqwest::Client::new();
    let system_msg = build_system_message(request.instructions.as_deref(), &request.context_documents);

    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: system_msg,
        },
        ChatMessage {
            role: "user".to_string(),
            content: request.message.clone(),
        },
    ];

    let body = OpenAiRequest {
        model: request.model.clone(),
        messages,
    };

    let response = client
        .post("https://integrate.api.nvidia.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", request.api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    let chat_response: ChatResponse = response.json().await?;
    Ok(chat_response
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .unwrap_or_else(|| "No response".to_string()))
}

async fn call_deepseek(request: &AiRequest) -> Result<String> {
    let client = reqwest::Client::new();
    let system_msg = build_system_message(request.instructions.as_deref(), &request.context_documents);

    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: system_msg,
        },
        ChatMessage {
            role: "user".to_string(),
            content: request.message.clone(),
        },
    ];

    let body = OpenAiRequest {
        model: request.model.clone(),
        messages,
    };

    let response = client
        .post("https://api.deepseek.com/chat/completions")
        .header("Authorization", format!("Bearer {}", request.api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    let chat_response: ChatResponse = response.json().await?;
    Ok(chat_response
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .unwrap_or_else(|| "No response".to_string()))
}

async fn call_xai(request: &AiRequest) -> Result<String> {
    let client = reqwest::Client::new();
    let system_msg = build_system_message(request.instructions.as_deref(), &request.context_documents);

    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: system_msg,
        },
        ChatMessage {
            role: "user".to_string(),
            content: request.message.clone(),
        },
    ];

    let body = OpenAiRequest {
        model: request.model.clone(),
        messages,
    };

    let response = client
        .post("https://api.x.ai/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", request.api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    let chat_response: ChatResponse = response.json().await?;
    Ok(chat_response
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .unwrap_or_else(|| "No response".to_string()))
}

pub async fn ask(request: AiRequest) -> Result<String> {
    match request.provider.as_str() {
        "openrouter" | "openrouter-free" => call_openrouter(&request).await,
        "openai" => call_openai(&request).await,
        "google" => call_google(&request).await,
        "nvidia" => call_nvidia(&request).await,
        "deepseek" => call_deepseek(&request).await,
        "xai" => call_xai(&request).await,
        _ => Err(anyhow::anyhow!("Unknown provider: {}", request.provider)),
    }
}

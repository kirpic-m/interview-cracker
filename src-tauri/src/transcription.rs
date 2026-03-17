use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct WhisperRequest {
    model: String,
    language: Option<String>,
    prompt: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WhisperResponse {
    text: String,
    #[serde(default)]
    segments: Option<Vec<WhisperSegment>>,
}

#[derive(Debug, Deserialize)]
struct WhisperSegment {
    id: u32,
    start: f32,
    end: f32,
    text: String,
}

/// Transcribe audio using OpenAI Whisper API
pub async fn transcribe_whisper_api(
    api_key: &str,
    audio_wav: Vec<u8>,
    language: Option<&str>,
) -> Result<String> {
    let client = reqwest::Client::new();

    let form = reqwest::multipart::Form::new()
        .text("model", "whisper-1")
        .part(
            "file",
            reqwest::multipart::Part::bytes(audio_wav)
                .file_name("audio.wav")
                .mime_str("audio/wav")?,
        );

    let form = if let Some(lang) = language {
        form.text("language", lang.to_string())
    } else {
        form
    };

    let response = client
        .post("https://api.openai.com/v1/audio/transcriptions")
        .header("Authorization", format!("Bearer {}", api_key))
        .multipart(form)
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(anyhow::anyhow!("Whisper API error: {}", error_text));
    }

    let result: WhisperResponse = response.json().await?;
    Ok(result.text)
}

/// Transcribe audio using OpenRouter (if available)
pub async fn transcribe_openrouter(
    api_key: &str,
    audio_wav: Vec<u8>,
    language: Option<&str>,
) -> Result<String> {
    let client = reqwest::Client::new();

    let form = reqwest::multipart::Form::new()
        .text("model", "openai/whisper-large-v3")
        .part(
            "file",
            reqwest::multipart::Part::bytes(audio_wav)
                .file_name("audio.wav")
                .mime_str("audio/wav")?,
        );

    let form = if let Some(lang) = language {
        form.text("language", lang.to_string())
    } else {
        form
    };

    let response = client
        .post("https://openrouter.ai/api/v1/audio/transcriptions")
        .header("Authorization", format!("Bearer {}", api_key))
        .multipart(form)
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(anyhow::anyhow!("OpenRouter Whisper error: {}", error_text));
    }

    let result: WhisperResponse = response.json().await?;
    Ok(result.text)
}

/// Transcribe using Google Cloud Speech-to-Text (alternative)
pub async fn transcribe_google(
    api_key: &str,
    audio_wav: Vec<u8>,
    language: Option<&str>,
) -> Result<String> {
    use base64::Engine;

    let client = reqwest::Client::new();
    let encoded = base64::engine::general_purpose::STANDARD.encode(&audio_wav);

    let lang_code = language.unwrap_or("en-US");

    let body = serde_json::json!({
        "config": {
            "encoding": "LINEAR16",
            "sampleRateHertz": 16000,
            "languageCode": lang_code,
            "enableAutomaticPunctuation": true,
        },
        "audio": {
            "content": encoded,
        }
    });

    let url = format!(
        "https://speech.googleapis.com/v1/speech:recognize?key={}",
        api_key
    );

    let response = client.post(&url).json(&body).send().await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(anyhow::anyhow!("Google Speech API error: {}", error_text));
    }

    let result: serde_json::Value = response.json().await?;

    let text = result["results"]
        .as_array()
        .and_then(|results| results.first())
        .and_then(|r| r["alternatives"].as_array())
        .and_then(|alts| alts.first())
        .and_then(|alt| alt["transcript"].as_str())
        .unwrap_or("")
        .to_string();

    Ok(text)
}

/// Supported transcription providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TranscriptionProvider {
    OpenAI,
    OpenRouter,
    Google,
}

impl TranscriptionProvider {
    pub fn from_str(s: &str) -> Self {
        match s {
            "openai" => TranscriptionProvider::OpenAI,
            "openrouter" => TranscriptionProvider::OpenRouter,
            "google" => TranscriptionProvider::Google,
            _ => TranscriptionProvider::OpenAI,
        }
    }
}

/// Main transcription function that routes to the appropriate provider
pub async fn transcribe(
    provider: &TranscriptionProvider,
    api_key: &str,
    audio_wav: Vec<u8>,
    language: Option<&str>,
) -> Result<String> {
    match provider {
        TranscriptionProvider::OpenAI => {
            transcribe_whisper_api(api_key, audio_wav, language).await
        }
        TranscriptionProvider::OpenRouter => {
            transcribe_openrouter(api_key, audio_wav, language).await
        }
        TranscriptionProvider::Google => {
            transcribe_google(api_key, audio_wav, language).await
        }
    }
}

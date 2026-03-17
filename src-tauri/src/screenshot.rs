use anyhow::Result;
use base64::Engine;
use std::process::Command;

/// Take a screenshot and return as base64-encoded PNG
#[cfg(target_os = "linux")]
pub async fn capture_screen() -> Result<Vec<u8>> {
    // Use gnome-screenshot, scrot, or import (ImageMagick)
    let output = Command::new("gnome-screenshot")
        .arg("-f")
        .arg("/tmp/interview_hunter_screenshot.png")
        .output();

    match output {
        Ok(_) => {
            let data = std::fs::read("/tmp/interview_hunter_screenshot.png")?;
            let _ = std::fs::remove_file("/tmp/interview_hunter_screenshot.png");
            Ok(data)
        }
        Err(_) => {
            // Fallback to scrot
            let output = Command::new("scrot")
                .arg("/tmp/interview_hunter_screenshot.png")
                .output();

            match output {
                Ok(_) => {
                    let data = std::fs::read("/tmp/interview_hunter_screenshot.png")?;
                    let _ = std::fs::remove_file("/tmp/interview_hunter_screenshot.png");
                    Ok(data)
                }
                Err(_) => Err(anyhow::anyhow!(
                    "No screenshot tool found. Install gnome-screenshot or scrot."
                )),
            }
        }
    }
}

#[cfg(target_os = "windows")]
pub async fn capture_screen() -> Result<Vec<u8>> {
    // Use PowerShell for screenshot on Windows
    let script = r#"
        Add-Type -AssemblyName System.Windows.Forms
        Add-Type -AssemblyName System.Drawing
        $screen = [System.Windows.Forms.SystemInformation]::VirtualScreen
        $bitmap = New-Object System.Drawing.Bitmap $screen.Width, $screen.Height
        $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
        $graphics.CopyFromScreen($screen.Left, $screen.Top, 0, 0, $bitmap.Size)
        $bitmap.Save("$env:TEMP\interview_hunter_screenshot.png", [System.Drawing.Imaging.ImageFormat]::Png)
        $graphics.Dispose()
        $bitmap.Dispose()
    "#;

    let output = Command::new("powershell")
        .arg("-Command")
        .arg(script)
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("PowerShell screenshot failed"));
    }

    let temp_path = format!(
        "{}\\interview_hunter_screenshot.png",
        std::env::var("TEMP").unwrap_or_else(|_| ".".to_string())
    );

    let data = std::fs::read(&temp_path)?;
    let _ = std::fs::remove_file(&temp_path);
    Ok(data)
}

#[cfg(target_os = "macos")]
pub async fn capture_screen() -> Result<Vec<u8>> {
    let output = Command::new("screencapture")
        .arg("-x")
        .arg("/tmp/interview_hunter_screenshot.png")
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("screencapture failed"));
    }

    let data = std::fs::read("/tmp/interview_hunter_screenshot.png")?;
    let _ = std::fs::remove_file("/tmp/interview_hunter_screenshot.png");
    Ok(data)
}

/// Convert screenshot to base64 string
pub fn to_base64(png_data: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(png_data)
}

/// Analyze screenshot with AI vision model
pub async fn analyze_screenshot(
    provider: &str,
    model: &str,
    api_key: &str,
    image_base64: &str,
    prompt: &str,
) -> Result<String> {
    let client = reqwest::Client::new();

    match provider {
        "openai" => {
            // Use GPT-4o vision
            let body = serde_json::json!({
                "model": model,
                "messages": [{
                    "role": "user",
                    "content": [
                        {
                            "type": "text",
                            "text": prompt
                        },
                        {
                            "type": "image_url",
                            "image_url": {
                                "url": format!("data:image/png;base64,{}", image_base64)
                            }
                        }
                    ]
                }],
                "max_tokens": 1024
            });

            let response = client
                .post("https://api.openai.com/v1/chat/completions")
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await?;

            let result: serde_json::Value = response.json().await?;
            Ok(result["choices"][0]["message"]["content"]
                .as_str()
                .unwrap_or("No response")
                .to_string())
        }
        "openrouter" => {
            // Use OpenRouter vision models
            let body = serde_json::json!({
                "model": model,
                "messages": [{
                    "role": "user",
                    "content": [
                        {
                            "type": "text",
                            "text": prompt
                        },
                        {
                            "type": "image_url",
                            "image_url": {
                                "url": format!("data:image/png;base64,{}", image_base64)
                            }
                        }
                    ]
                }],
                "max_tokens": 1024
            });

            let response = client
                .post("https://openrouter.ai/api/v1/chat/completions")
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .header("HTTP-Referer", "https://interviewcracker.app")
                .json(&body)
                .send()
                .await?;

            let result: serde_json::Value = response.json().await?;
            Ok(result["choices"][0]["message"]["content"]
                .as_str()
                .unwrap_or("No response")
                .to_string())
        }
        "google" => {
            // Use Gemini vision
            let body = serde_json::json!({
                "contents": [{
                    "parts": [
                        {
                            "text": prompt
                        },
                        {
                            "inline_data": {
                                "mime_type": "image/png",
                                "data": image_base64
                            }
                        }
                    ]
                }],
                "generationConfig": {
                    "maxOutputTokens": 1024
                }
            });

            let url = format!(
                "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
                model, api_key
            );

            let response = client.post(&url).json(&body).send().await?;

            let result: serde_json::Value = response.json().await?;
            Ok(result["candidates"][0]["content"]["parts"][0]["text"]
                .as_str()
                .unwrap_or("No response")
                .to_string())
        }
        _ => Err(anyhow::anyhow!("Vision not supported for provider: {}", provider)),
    }
}

/// Get vision-capable models for a provider
pub fn get_vision_models(provider: &str) -> Vec<String> {
    match provider {
        "openai" => vec![
            "gpt-5.4".to_string(),
            "gpt-5.4-pro".to_string(),
            "gpt-5.3-chat".to_string(),
            "gpt-5.2-chat".to_string(),
            "o3".to_string(),
            "o4-mini".to_string(),
        ],
        "openrouter" => vec![
            "openai/gpt-5.4".to_string(),
            "openai/gpt-5.3-chat".to_string(),
            "openai/o3".to_string(),
            "anthropic/claude-opus-4.6".to_string(),
            "anthropic/claude-sonnet-4.6".to_string(),
            "google/gemini-3.1-pro-preview".to_string(),
            "google/gemini-2.5-flash".to_string(),
            "openrouter/healer-alpha".to_string(),
        ],
        "google" => vec![
            "gemini-3.1-pro-preview".to_string(),
            "gemini-3.1-flash-lite-preview".to_string(),
            "gemini-2.5-flash".to_string(),
            "gemini-2.5-pro".to_string(),
        ],
        _ => vec![],
    }
}

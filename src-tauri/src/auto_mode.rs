use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::{interval, Instant};

/// Configuration for auto mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoModeConfig {
    /// Interval in seconds between transcription attempts
    pub transcription_interval: f32,
    /// Minimum audio duration (seconds) before transcribing
    pub min_audio_duration: f32,
    /// Maximum silence duration (seconds) before considering speech ended
    pub silence_threshold: f32,
    /// Whether to always respond or only to questions
    pub respond_to_all: bool,
    /// Keywords that trigger a response (if respond_to_all is false)
    pub trigger_keywords: Vec<String>,
}

impl Default for AutoModeConfig {
    fn default() -> Self {
        Self {
            transcription_interval: 3.0,
            min_audio_duration: 2.0,
            silence_threshold: 1.5,
            respond_to_all: false,
            trigger_keywords: vec![
                "?".to_string(),
                "what".to_string(),
                "how".to_string(),
                "why".to_string(),
                "when".to_string(),
                "where".to_string(),
                "who".to_string(),
                "which".to_string(),
                "can you".to_string(),
                "could you".to_string(),
                "tell me".to_string(),
                "explain".to_string(),
                "describe".to_string(),
                "як".to_string(),    // Ukrainian
                "що".to_string(),
                "чому".to_string(),
                "коли".to_string(),
                "де".to_string(),
                "хто".to_string(),
                "який".to_string(),
                "расскажи".to_string(),  // Russian
                "объясни".to_string(),
                "что".to_string(),
                "как".to_string(),
                "почему".to_string(),
            ],
        }
    }
}

/// Auto mode state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoModeState {
    pub is_active: bool,
    pub last_transcription: Option<String>,
    pub last_response: Option<String>,
    pub questions_detected: u32,
    pub responses_given: u32,
}

impl Default for AutoModeState {
    fn default() -> Self {
        Self {
            is_active: false,
            last_transcription: None,
            last_response: None,
            questions_detected: 0,
            responses_given: 0,
        }
    }
}

/// Check if text contains a question
pub fn is_question(text: &str, config: &AutoModeConfig) -> bool {
    if config.respond_to_all {
        return !text.trim().is_empty();
    }

    let text_lower_owned = text.to_lowercase();
    let text_lower = text_lower_owned.trim();

    // Check for question mark
    if text_lower.contains('?') {
        return true;
    }

    // Check for trigger keywords at the beginning of sentences
    for keyword in &config.trigger_keywords {
        if text_lower.starts_with(keyword) || text_lower.contains(&format!(" {}", keyword)) {
            return true;
        }
    }

    false
}

/// Extract questions from text
pub fn extract_questions(text: &str) -> Vec<String> {
    let mut questions = Vec::new();

    // Split by sentence-ending punctuation
    let sentences: Vec<&str> = text
        .split(|c: char| c == '.' || c == '!' || c == '?')
        .filter(|s| !s.trim().is_empty())
        .collect();

    for sentence in sentences {
        let trimmed = sentence.trim();
        // Check if it looks like a question
        if trimmed.ends_with('?') || is_question_start(trimmed) {
            questions.push(format!("{}?", trimmed.trim_end_matches('?')));
        }
    }

    // If no specific questions found but text is non-empty, return all text
    if questions.is_empty() && !text.trim().is_empty() {
        questions.push(text.trim().to_string());
    }

    questions
}

fn is_question_start(text: &str) -> bool {
    let lower = text.to_lowercase();
    let question_starters = [
        "what", "how", "why", "when", "where", "who", "which", "can", "could",
        "would", "should", "is", "are", "do", "does", "did", "will", "have",
        "has", "як", "що", "чому", "коли", "де", "хто", "який", "чи",
        "расскажи", "объясни", "что", "как", "почему", "можешь", "можно",
    ];

    for starter in &question_starters {
        if lower.starts_with(starter) {
            return true;
        }
    }

    false
}

/// Format the AI response for auto mode
pub fn format_auto_response(question: &str, answer: &str) -> String {
    format!("Q: {}\n\nA: {}", question, answer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_question() {
        let config = AutoModeConfig::default();

        assert!(is_question("What is your experience?", &config));
        assert!(is_question("How do you handle conflicts?", &config));
        assert!(is_question("Tell me about yourself", &config));
        assert!(!is_question("I have 5 years of experience.", &config));
    }

    #[test]
    fn test_extract_questions() {
        let text = "What is your experience? I worked at Google. How did you handle stress?";
        let questions = extract_questions(text);
        assert_eq!(questions.len(), 2);
    }
}

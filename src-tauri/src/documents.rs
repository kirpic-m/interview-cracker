use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub name: String,
    pub doc_type: DocumentType,
    pub content: String,
    pub file_path: Option<String>,
    pub uploaded_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DocumentType {
    Resume,
    JobDescription,
    TechnicalDoc,
    Other,
}

impl DocumentType {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "resume" | "cv" => DocumentType::Resume,
            "job" | "job_description" | "vacancy" => DocumentType::JobDescription,
            "tech" | "technical" | "doc" => DocumentType::TechnicalDoc,
            _ => DocumentType::Other,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            DocumentType::Resume => "Resume",
            DocumentType::JobDescription => "Job Description",
            DocumentType::TechnicalDoc => "Technical Document",
            DocumentType::Other => "Other",
        }
    }
}

/// Supported file types
pub fn is_supported_file(path: &str) -> bool {
    let ext = PathBuf::from(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    matches!(ext.as_str(), "txt" | "md" | "pdf" | "doc" | "docx" | "json")
}

/// Read document content from file
pub async fn read_document(file_path: &str, doc_type: DocumentType) -> Result<Document> {
    let path = PathBuf::from(file_path);
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown")
        .to_string();

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let content = match ext.as_str() {
        "txt" | "md" | "json" => read_text_file(file_path).await?,
        "pdf" => read_pdf_file(file_path).await?,
        "doc" | "docx" => read_doc_file(file_path).await?,
        _ => return Err(anyhow::anyhow!("Unsupported file type: {}", ext)),
    };

    let id = uuid::Uuid::new_v4().to_string();

    Ok(Document {
        id,
        name,
        doc_type,
        content,
        file_path: Some(file_path.to_string()),
        uploaded_at: chrono::Utc::now().to_rfc3339(),
    })
}

/// Read plain text file
async fn read_text_file(path: &str) -> Result<String> {
    let content = tokio::fs::read_to_string(path).await?;
    Ok(content)
}

/// Read PDF file (simplified - uses pdftotext if available)
async fn read_pdf_file(path: &str) -> Result<String> {
    // Try pdftotext first (poppler-utils)
    let output = tokio::process::Command::new("pdftotext")
        .arg(path)
        .arg("-")
        .output()
        .await;

    match output {
        Ok(out) if out.status.success() => {
            Ok(String::from_utf8_lossy(&out.stdout).to_string())
        }
        _ => {
            // Fallback: read raw and try to extract text
            // This is a simplified approach
            Err(anyhow::anyhow!(
                "PDF reading requires pdftotext (poppler-utils). Install with:\n\
                 Ubuntu/Debian: sudo apt install poppler-utils\n\
                 macOS: brew install poppler\n\
                 Or convert PDF to TXT manually."
            ))
        }
    }
}

/// Read DOC/DOCX file (simplified - uses antiword or textutil)
async fn read_doc_file(path: &str) -> Result<String> {
    #[cfg(target_os = "macos")]
    {
        let output = tokio::process::Command::new("textutil")
            .arg("-convert")
            .arg("txt")
            .arg("-stdout")
            .arg(path)
            .output()
            .await?;

        if output.status.success() {
            return Ok(String::from_utf8_lossy(&output.stdout).to_string());
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        let output = tokio::process::Command::new("antiword")
            .arg(path)
            .output()
            .await;

        if let Ok(out) = output {
            if out.status.success() {
                return Ok(String::from_utf8_lossy(&out.stdout).to_string());
            }
        }
    }

    Err(anyhow::anyhow!(
        "DOC/DOCX reading requires additional tools.\n\
         Linux: sudo apt install antiword\n\
         macOS: uses built-in textutil\n\
         Or convert to TXT manually."
    ))
}

/// Create document from pasted text
pub fn create_from_text(name: &str, doc_type: DocumentType, content: &str) -> Document {
    let id = uuid::Uuid::new_v4().to_string();

    Document {
        id,
        name: name.to_string(),
        doc_type,
        content: content.to_string(),
        file_path: None,
        uploaded_at: chrono::Utc::now().to_rfc3339(),
    }
}

/// Format documents as context for AI
pub fn format_as_context(documents: &[Document]) -> String {
    let mut context = String::new();

    for doc in documents {
        context.push_str(&format!("=== {} ({}) ===\n", doc.name, doc.doc_type.as_str()));
        context.push_str(&doc.content);
        context.push_str("\n\n");
    }

    context
}

/// Extract key information from resume
pub fn extract_resume_info(content: &str) -> ResumeInfo {
    let mut info = ResumeInfo::default();

    // Simple extraction (could be enhanced with NLP)
    let lines: Vec<&str> = content.lines().collect();

    for line in lines {
        let lower = line.to_lowercase();

        // Try to find name (first non-empty line)
        if info.name.is_empty() && !line.trim().is_empty() && line.len() < 50 {
            info.name = line.trim().to_string();
        }

        // Look for email
        if lower.contains("@") && info.email.is_empty() {
            for word in line.split_whitespace() {
                if word.contains("@") {
                    info.email = word.trim().to_string();
                }
            }
        }

        // Look for phone
        if (lower.contains("phone") || lower.contains("телефон") || line.contains("+"))
            && info.phone.is_empty()
        {
            info.phone = line.trim().to_string();
        }

        // Look for skills section
        if lower.contains("skill") || lower.contains("навички") || lower.contains("технології") {
            info.has_skills_section = true;
        }

        // Look for experience section
        if lower.contains("experience") || lower.contains("досвід") || lower.contains("робота") {
            info.has_experience_section = true;
        }
    }

    info
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ResumeInfo {
    pub name: String,
    pub email: String,
    pub phone: String,
    pub has_skills_section: bool,
    pub has_experience_section: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_supported_file() {
        assert!(is_supported_file("resume.pdf"));
        assert!(is_supported_file("job.txt"));
        assert!(is_supported_file("doc.md"));
        assert!(!is_supported_file("image.png"));
        assert!(!is_supported_file("video.mp4"));
    }

    #[test]
    fn test_create_from_text() {
        let doc = create_from_text("My Resume", DocumentType::Resume, "John Doe\nSoftware Engineer");
        assert_eq!(doc.name, "My Resume");
        assert_eq!(doc.doc_type, DocumentType::Resume);
        assert!(doc.content.contains("John Doe"));
    }

    #[test]
    fn test_format_as_context() {
        let docs = vec![
            create_from_text("Resume", DocumentType::Resume, "Skills: Rust, Python"),
            create_from_text("Job", DocumentType::JobDescription, "Looking for Rust developer"),
        ];
        let context = format_as_context(&docs);
        assert!(context.contains("Resume"));
        assert!(context.contains("Job"));
    }
}

use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub title: String,
    pub instructions: Option<String>,
    pub created_at: String,
    pub ended_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub id: i64,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub created_at: String,
}

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(path: &str) -> Result<Self> {
        let pool = SqlitePool::connect(&format!("sqlite://{}?mode=rwc", path)).await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                instructions TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                ended_at TEXT
            )
            "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (session_id) REFERENCES sessions(id)
            )
            "#,
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }

    pub async fn create_session(
        &self,
        id: &str,
        title: &str,
        instructions: Option<&str>,
    ) -> Result<()> {
        sqlx::query("INSERT INTO sessions (id, title, instructions) VALUES (?, ?, ?)")
            .bind(id)
            .bind(title)
            .bind(instructions)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn end_session(&self, id: &str) -> Result<()> {
        sqlx::query("UPDATE sessions SET ended_at = datetime('now') WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_sessions(&self) -> Result<Vec<Session>> {
        let rows = sqlx::query("SELECT id, title, instructions, created_at, ended_at FROM sessions ORDER BY created_at DESC")
            .fetch_all(&self.pool)
            .await?;

        Ok(rows
            .into_iter()
            .map(|row| Session {
                id: row.get("id"),
                title: row.get("title"),
                instructions: row.get("instructions"),
                created_at: row.get("created_at"),
                ended_at: row.get("ended_at"),
            })
            .collect())
    }

    pub async fn save_message(&self, session_id: &str, role: &str, content: &str) -> Result<()> {
        sqlx::query(
            "INSERT INTO messages (session_id, role, content) VALUES (?, ?, ?)",
        )
        .bind(session_id)
        .bind(role)
        .bind(content)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_session_messages(&self, session_id: &str) -> Result<Vec<Message>> {
        let rows = sqlx::query(
            "SELECT id, session_id, role, content, created_at FROM messages WHERE session_id = ? ORDER BY created_at ASC",
        )
        .bind(session_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| Message {
                id: row.get("id"),
                session_id: row.get("session_id"),
                role: row.get("role"),
                content: row.get("content"),
                created_at: row.get("created_at"),
            })
            .collect())
    }
}

use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, Serialize)]
#[serde(tag = "kind", content = "data")]
pub enum GitError {
    #[error("git failed (exit {exit_code}): {friendly}")]
    GitExit { exit_code: i32, friendly: String, stderr: String },
    #[error("spawn error: {0}")]
    Spawn(String),
    #[error("io error: {0}")]
    Io(String),
    #[error("parse error: {0}")]
    Parse(String),
}

pub type GitResult<T> = Result<T, GitError>;

impl GitError {
    pub fn spawn(msg: impl Into<String>) -> Self { Self::Spawn(msg.into()) }
    pub fn parse(msg: impl Into<String>) -> Self { Self::Parse(msg.into()) }

    pub fn from_stderr(exit_code: i32, stderr: &str) -> Self {
        let lower = stderr.to_lowercase();
        let friendly = if lower.contains("index.lock") {
            "Another git process is holding the index lock. Wait for it to finish or remove .git/index.lock.".into()
        } else if lower.contains("not a git repository") {
            "Not a git repository.".into()
        } else if lower.contains("non-fast-forward") {
            "Push rejected: remote has commits you don't have. Pull first or use force-with-lease.".into()
        } else if lower.contains("would be overwritten") {
            "Operation blocked: uncommitted changes would be overwritten.".into()
        } else {
            stderr.lines().next().unwrap_or("git failed").to_string()
        };
        Self::GitExit { exit_code, friendly, stderr: stderr.to_string() }
    }
}

impl From<std::io::Error> for GitError {
    fn from(e: std::io::Error) -> Self { Self::Io(e.to_string()) }
}

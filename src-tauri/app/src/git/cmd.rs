use crate::error::{GitError, GitResult};
use std::path::Path;
use tokio::process::Command;

#[derive(Debug)]
pub struct GitOutput { pub stdout: String, pub stderr: String }

pub async fn run_git(cwd: &Path, args: &[&str]) -> GitResult<GitOutput> {
    let output = Command::new("git").current_dir(cwd).args(args).output().await
        .map_err(|e| GitError::spawn(format!("spawn git failed: {e}")))?;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    if !output.status.success() {
        return Err(GitError::from_stderr(output.status.code().unwrap_or(-1), &stderr));
    }
    Ok(GitOutput { stdout, stderr })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn run_git_version_succeeds() {
        let dir = tempdir().unwrap();
        let out = run_git(dir.path(), &["--version"]).await.unwrap();
        assert!(out.stdout.starts_with("git version"));
    }

    #[tokio::test]
    async fn run_git_in_non_repo_errors() {
        let dir = tempdir().unwrap();
        let err = run_git(dir.path(), &["status"]).await.unwrap_err();
        assert!(format!("{err}").to_lowercase().contains("not a git repository"));
    }
}

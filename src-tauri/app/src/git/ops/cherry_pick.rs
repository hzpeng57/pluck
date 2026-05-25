use crate::error::{GitError, GitResult};
use std::path::Path;

/// Apply the given commits as new commits on top of HEAD.
/// `hashes` should already be ordered oldest→newest by the caller.
/// On conflict, CHERRY_PICK_HEAD remains; snapshot.inProgress will surface it.
pub async fn cherry_pick(repo: &Path, hashes: &[String]) -> GitResult<()> {
    if hashes.is_empty() {
        return Err(GitError::parse("cherry_pick: empty hash list"));
    }
    let mut args: Vec<&str> = vec!["cherry-pick"];
    for h in hashes {
        args.push(h.as_str());
    }
    let output = tokio::process::Command::new("git")
        .current_dir(repo)
        .args(&args)
        .output()
        .await
        .map_err(|e| GitError::spawn(e.to_string()))?;
    if output.status.success() {
        return Ok(());
    }
    if repo.join(".git/CHERRY_PICK_HEAD").exists() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(GitError::from_stderr(output.status.code().unwrap_or(-1), &stderr))
}

pub async fn cherry_pick_continue(repo: &Path) -> GitResult<()> {
    let output = tokio::process::Command::new("git")
        .current_dir(repo)
        .args(["cherry-pick", "--continue"])
        .output()
        .await
        .map_err(|e| GitError::spawn(e.to_string()))?;
    if output.status.success() {
        return Ok(());
    }
    if repo.join(".git/CHERRY_PICK_HEAD").exists() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(GitError::from_stderr(output.status.code().unwrap_or(-1), &stderr))
}

pub async fn cherry_pick_abort(repo: &Path) -> GitResult<()> {
    let output = tokio::process::Command::new("git")
        .current_dir(repo)
        .args(["cherry-pick", "--abort"])
        .output()
        .await
        .map_err(|e| GitError::spawn(e.to_string()))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(GitError::from_stderr(output.status.code().unwrap_or(-1), &stderr));
    }
    Ok(())
}

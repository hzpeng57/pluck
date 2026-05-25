use crate::error::{GitError, GitResult};
use crate::git::cmd::run_git;
use std::path::Path;

/// For each hash, generate a new commit that undoes its changes.
/// `hashes` should already be ordered (typically newest→oldest for revert so the
/// inverse diff applies cleanly, but git itself handles either order via the
/// `git revert <h1> <h2>...` form). Caller decides.
/// Refuses merge commits up-front to avoid the `-m` mainline ambiguity.
pub async fn revert(repo: &Path, hashes: &[String]) -> GitResult<()> {
    if hashes.is_empty() {
        return Err(GitError::parse("revert: empty hash list"));
    }
    // Pre-flight: reject merge commits (parent count >= 2). Match WebStorm behavior
    // of not silently picking a mainline.
    for h in hashes {
        let parents = run_git(repo, &["show", "-s", "--format=%P", h]).await?;
        let count = parents.stdout.trim().split_whitespace().count();
        if count >= 2 {
            return Err(GitError::parse(format!(
                "{}: refusing to revert merge commit (multiple parents). Revert merges via the CLI with --mainline.",
                &h[..h.len().min(7)]
            )));
        }
    }
    let mut args: Vec<&str> = vec!["revert", "--no-edit"];
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
    if repo.join(".git/REVERT_HEAD").exists() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(GitError::from_stderr(output.status.code().unwrap_or(-1), &stderr))
}

pub async fn revert_continue(repo: &Path) -> GitResult<()> {
    let output = tokio::process::Command::new("git")
        .current_dir(repo)
        .args(["revert", "--continue"])
        .output()
        .await
        .map_err(|e| GitError::spawn(e.to_string()))?;
    if output.status.success() {
        return Ok(());
    }
    if repo.join(".git/REVERT_HEAD").exists() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(GitError::from_stderr(output.status.code().unwrap_or(-1), &stderr))
}

pub async fn revert_abort(repo: &Path) -> GitResult<()> {
    let output = tokio::process::Command::new("git")
        .current_dir(repo)
        .args(["revert", "--abort"])
        .output()
        .await
        .map_err(|e| GitError::spawn(e.to_string()))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(GitError::from_stderr(output.status.code().unwrap_or(-1), &stderr));
    }
    Ok(())
}

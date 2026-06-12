use crate::error::{GitError, GitResult};
use crate::git::cmd::{git_command, run_git};
use crate::git::ops::merge::rebase_in_progress;
use std::path::Path;

/// WebStorm "Pull into <current> using rebase" semantics:
/// fetch `source` if it's a remote-tracking branch, then rebase current HEAD onto it.
/// Conflicts leave rebase state in place — caller refreshes snapshot and the
/// in-progress banner takes over.
pub async fn pull_into_rebase(repo: &Path, source: &str) -> GitResult<()> {
    if let Some((remote, branch)) = source.split_once('/') {
        let remotes = run_git(repo, &["remote"]).await?;
        let remote_exists = remotes.stdout.lines().any(|l| l.trim() == remote);
        if remote_exists {
            run_git(repo, &["fetch", remote, branch]).await?;
        }
    }
    let output = git_command(repo)
        .args(["rebase", source])
        .output()
        .await
        .map_err(|e| GitError::spawn(e.to_string()))?;
    if output.status.success() {
        return Ok(());
    }
    // Conflicts → leave state for InProgressBanner, do not toast as failure.
    if rebase_in_progress(repo) {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(GitError::from_stderr(output.status.code().unwrap_or(-1), &stderr))
}

pub async fn pull_rebase(repo: &Path, target_branch: &str) -> GitResult<()> {
    let head = run_git(repo, &["symbolic-ref", "--short", "HEAD"]).await?;
    let current = head.stdout.trim().to_string();
    if current == target_branch {
        let output = git_command(repo)
            .args(["pull", "--rebase"])
            .output()
            .await
            .map_err(|e| GitError::spawn(e.to_string()))?;
        if output.status.success() || rebase_in_progress(repo) {
            return Ok(());
        }
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(GitError::from_stderr(
            output.status.code().unwrap_or(-1),
            &stderr,
        ));
    }
    // Non-current target: fetch-only fast-forward via refspec.
    // Find upstream of target branch
    let upstream = run_git(repo, &["rev-parse", "--abbrev-ref", &format!("{target_branch}@{{upstream}}")]).await
        .map_err(|_| GitError::parse(format!("{target_branch} has no upstream configured")))?;
    let upstream = upstream.stdout.trim().to_string();
    let (remote, _rb) = upstream.split_once('/').ok_or_else(|| GitError::parse("upstream not in remote/branch form"))?;
    run_git(repo, &["fetch", remote, &format!("{target_branch}:{target_branch}")]).await
        .map_err(|e| GitError::parse(format!("Non-fast-forward — checkout {target_branch} first and pull manually. ({e})")))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use tempfile::tempdir;

    fn git(cwd: &Path, args: &[&str]) {
        let output = Command::new("git").current_dir(cwd).args(args).output().unwrap();
        assert!(
            output.status.success(),
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[tokio::test]
    async fn pull_rebase_conflict_leaves_rebase_state_without_error() {
        let tmp = tempdir().unwrap();
        let remote = tmp.path().join("remote.git");
        let seed = tmp.path().join("seed");
        let work = tmp.path().join("work");

        git(tmp.path(), &["init", "--bare", remote.to_str().unwrap()]);
        std::fs::create_dir(&seed).unwrap();
        git(&seed, &["init", "-b", "main"]);
        git(&seed, &["config", "user.email", "t@t.t"]);
        git(&seed, &["config", "user.name", "t"]);
        std::fs::write(seed.join("file.txt"), "base\n").unwrap();
        git(&seed, &["add", "file.txt"]);
        git(&seed, &["commit", "-m", "init"]);
        git(&seed, &["remote", "add", "origin", remote.to_str().unwrap()]);
        git(&seed, &["push", "-u", "origin", "main"]);

        git(tmp.path(), &["clone", remote.to_str().unwrap(), work.to_str().unwrap()]);
        git(&work, &["config", "user.email", "t@t.t"]);
        git(&work, &["config", "user.name", "t"]);
        std::fs::write(work.join("file.txt"), "local\n").unwrap();
        git(&work, &["commit", "-am", "local"]);

        std::fs::write(seed.join("file.txt"), "remote\n").unwrap();
        git(&seed, &["commit", "-am", "remote"]);
        git(&seed, &["push", "origin", "main"]);

        pull_rebase(&work, "main").await.unwrap();

        assert!(
            work.join(".git/rebase-merge").exists() || work.join(".git/rebase-apply").exists()
        );
    }
}

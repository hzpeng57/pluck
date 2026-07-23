use crate::error::{GitError, GitResult};
use crate::git::cmd::{git_command, run_git};
use crate::git::git_dir;
use crate::git::ops::conflict::ensure_no_unresolved_conflicts;
use std::path::Path;

pub async fn merge_into_current(repo: &Path, branch: &str) -> GitResult<()> {
    let output = git_command(repo)
        .args(["merge", "--no-edit", branch])
        .output()
        .await
        .map_err(|e| GitError::spawn(e.to_string()))?;
    if output.status.success() {
        return Ok(());
    }
    if merge_in_progress(repo) {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(GitError::from_stderr(output.status.code().unwrap_or(-1), &stderr))
}

pub fn merge_in_progress(repo: &Path) -> bool {
    git_dir(repo).join("MERGE_HEAD").exists()
}

pub fn rebase_in_progress(repo: &Path) -> bool {
    let git_dir = git_dir(repo);
    git_dir.join("rebase-merge").exists() || git_dir.join("rebase-apply").exists()
}

pub async fn merge_abort(repo: &Path) -> GitResult<()> {
    run_git(repo, &["merge", "--abort"]).await?;
    Ok(())
}

pub async fn merge_continue(repo: &Path) -> GitResult<()> {
    ensure_no_unresolved_conflicts(repo).await?;
    run_git(repo, &["merge", "--continue"]).await?;
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
    async fn merge_conflict_leaves_in_progress_state_without_error() {
        let repo = tempdir().unwrap();
        git(repo.path(), &["init", "-b", "main"]);
        git(repo.path(), &["config", "user.email", "t@t.t"]);
        git(repo.path(), &["config", "user.name", "t"]);
        std::fs::write(repo.path().join("file.txt"), "base\n").unwrap();
        git(repo.path(), &["add", "file.txt"]);
        git(repo.path(), &["commit", "-m", "init"]);

        git(repo.path(), &["switch", "-c", "feature"]);
        std::fs::write(repo.path().join("file.txt"), "feature\n").unwrap();
        git(repo.path(), &["commit", "-am", "feature"]);

        git(repo.path(), &["switch", "main"]);
        std::fs::write(repo.path().join("file.txt"), "main\n").unwrap();
        git(repo.path(), &["commit", "-am", "main"]);

        merge_into_current(repo.path(), "feature").await.unwrap();

        assert!(repo.path().join(".git/MERGE_HEAD").exists());
    }

    #[test]
    fn in_progress_detection_resolves_gitfile_worktrees() {
        let repo = tempdir().unwrap();
        let gitdir = repo.path().join("real-gitdir");
        std::fs::create_dir(&gitdir).unwrap();
        std::fs::write(
            repo.path().join(".git"),
            format!("gitdir: {}\n", gitdir.display()),
        )
        .unwrap();

        std::fs::write(gitdir.join("MERGE_HEAD"), "").unwrap();
        std::fs::create_dir(gitdir.join("rebase-merge")).unwrap();

        assert!(merge_in_progress(repo.path()));
        assert!(rebase_in_progress(repo.path()));
    }
}

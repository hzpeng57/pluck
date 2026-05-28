use crate::error::GitResult;
use crate::git::cmd::run_git;
use std::path::Path;

pub async fn checkout_branch(repo: &Path, name: &str) -> GitResult<()> {
    if ref_exists(repo, &format!("refs/heads/{name}")).await {
        run_git(repo, &["checkout", name]).await?;
        return Ok(());
    }

    if ref_exists(repo, &format!("refs/remotes/{name}")).await {
        if let Some((_, local)) = name.split_once('/') {
            if ref_exists(repo, &format!("refs/heads/{local}")).await {
                run_git(repo, &["checkout", local]).await?;
            } else {
                run_git(repo, &["checkout", "--track", name]).await?;
            }
            return Ok(());
        }
    }

    run_git(repo, &["checkout", name]).await?;
    Ok(())
}

async fn ref_exists(repo: &Path, refname: &str) -> bool {
    run_git(repo, &["show-ref", "--verify", "--quiet", refname])
        .await
        .is_ok()
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
    async fn remote_checkout_creates_local_tracking_branch() {
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
        git(&seed, &["switch", "-c", "feature"]);
        std::fs::write(seed.join("feature.txt"), "feature\n").unwrap();
        git(&seed, &["add", "feature.txt"]);
        git(&seed, &["commit", "-m", "feature"]);
        git(&seed, &["push", "-u", "origin", "feature"]);

        git(tmp.path(), &["clone", remote.to_str().unwrap(), work.to_str().unwrap()]);
        checkout_branch(&work, "origin/feature").await.unwrap();

        let head = Command::new("git")
            .current_dir(&work)
            .args(["symbolic-ref", "--short", "HEAD"])
            .output()
            .unwrap();
        assert_eq!(String::from_utf8_lossy(&head.stdout).trim(), "feature");

        let upstream = Command::new("git")
            .current_dir(&work)
            .args(["rev-parse", "--abbrev-ref", "feature@{upstream}"])
            .output()
            .unwrap();
        assert_eq!(String::from_utf8_lossy(&upstream.stdout).trim(), "origin/feature");
    }
}

use std::process::Command;
use tempfile::tempdir;
use git_lite_app_lib::git::ops::push::push;

fn git(p: &std::path::Path, args: &[&str]) { Command::new("git").current_dir(p).args(args).status().unwrap(); }

#[tokio::test]
async fn push_to_local_bare_remote() {
    let remote = tempdir().unwrap();
    Command::new("git").args(["init", "--bare", remote.path().to_str().unwrap()]).status().unwrap();

    let work = tempdir().unwrap(); let p = work.path();
    git(p, &["init", "-b", "main"]);
    git(p, &["config", "user.email", "t@t.t"]); git(p, &["config", "user.name", "t"]);
    git(p, &["commit", "--allow-empty", "-m", "c1"]);
    git(p, &["remote", "add", "origin", remote.path().to_str().unwrap()]);

    push(p, false).await.unwrap();

    let out = Command::new("git").args(["--git-dir", remote.path().to_str().unwrap(), "log", "--oneline"]).output().unwrap();
    assert!(String::from_utf8_lossy(&out.stdout).contains("c1"));
}

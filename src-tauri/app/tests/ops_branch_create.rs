use std::process::Command;
use tempfile::tempdir;
use git_lite_app_lib::git::ops::branch::create_branch;

fn git(p: &std::path::Path, args: &[&str]) { Command::new("git").current_dir(p).args(args).status().unwrap(); }

#[tokio::test]
async fn creates_branch_from_specified_base() {
    let dir = tempdir().unwrap(); let p = dir.path();
    git(p, &["init", "-b", "main"]);
    git(p, &["config", "user.email", "t@t.t"]); git(p, &["config", "user.name", "t"]);
    git(p, &["commit", "--allow-empty", "-m", "c1"]);

    create_branch(p, "feat/x", Some("main")).await.unwrap();

    let out = Command::new("git").current_dir(p).args(["symbolic-ref", "--short", "HEAD"]).output().unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "feat/x");
}

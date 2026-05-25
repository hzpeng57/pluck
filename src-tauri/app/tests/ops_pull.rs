use std::process::Command;
use tempfile::tempdir;
use pluck_app_lib::git::ops::pull::pull_rebase;

fn git(p: &std::path::Path, args: &[&str]) { Command::new("git").current_dir(p).args(args).status().unwrap(); }

#[tokio::test]
async fn pull_rebase_current_branch() {
    let remote = tempdir().unwrap();
    Command::new("git").args(["init", "--bare", remote.path().to_str().unwrap()]).status().unwrap();

    let a = tempdir().unwrap(); let pa = a.path();
    git(pa, &["init", "-b", "main"]);
    git(pa, &["config", "user.email", "t@t.t"]); git(pa, &["config", "user.name", "t"]);
    git(pa, &["commit", "--allow-empty", "-m", "c1"]);
    git(pa, &["remote", "add", "origin", remote.path().to_str().unwrap()]);
    git(pa, &["push", "-u", "origin", "main"]);

    let b = tempdir().unwrap(); let pb = b.path();
    Command::new("git").args(["clone", remote.path().to_str().unwrap(), pb.to_str().unwrap()]).status().unwrap();
    git(pb, &["config", "user.email", "t@t.t"]); git(pb, &["config", "user.name", "t"]);
    git(pb, &["commit", "--allow-empty", "-m", "remote-c2"]);
    git(pb, &["push"]);

    git(pa, &["commit", "--allow-empty", "-m", "local-c2"]);

    pull_rebase(pa, "main").await.unwrap();

    let out = Command::new("git").current_dir(pa).args(["log", "--oneline"]).output().unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("local-c2"));
    assert!(s.contains("remote-c2"));
}

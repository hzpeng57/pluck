use std::process::Command;
use tempfile::tempdir;
use pluck_app_lib::git::ops::merge::merge_into_current;

fn git(p: &std::path::Path, args: &[&str]) { Command::new("git").current_dir(p).args(args).status().unwrap(); }

#[tokio::test]
async fn fast_forward_merge() {
    let dir = tempdir().unwrap(); let p = dir.path();
    git(p, &["init", "-b", "main"]);
    git(p, &["config", "user.email", "t@t.t"]); git(p, &["config", "user.name", "t"]);
    git(p, &["commit", "--allow-empty", "-m", "c1"]);
    git(p, &["checkout", "-b", "feat"]);
    git(p, &["commit", "--allow-empty", "-m", "c2"]);
    git(p, &["checkout", "main"]);

    merge_into_current(p, "feat").await.unwrap();

    let out = Command::new("git").current_dir(p).args(["log", "--oneline"]).output().unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("c2"));
    assert!(s.contains("c1"));
}

use std::process::Command;
use tempfile::tempdir;
use git_lite_app_lib::git::ops::commit::commit_files;

fn git(p: &std::path::Path, args: &[&str]) { Command::new("git").current_dir(p).args(args).status().unwrap(); }

#[tokio::test]
async fn commit_only_selected_files() {
    let dir = tempdir().unwrap(); let p = dir.path();
    git(p, &["init", "-b", "main"]); git(p, &["config", "user.email", "t@t.t"]); git(p, &["config", "user.name", "t"]);
    git(p, &["commit", "--allow-empty", "-m", "init"]);

    std::fs::write(p.join("a.txt"), "1").unwrap();
    std::fs::write(p.join("b.txt"), "2").unwrap();

    commit_files(p, &["a.txt".to_string()], "add a", false).await.unwrap();

    let out = Command::new("git").current_dir(p).args(["status", "--porcelain"]).output().unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("b.txt"), "b should remain untracked: {s:?}");
    assert!(!s.contains("a.txt"), "a should be committed: {s:?}");
}

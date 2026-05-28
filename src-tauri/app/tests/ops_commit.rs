use pluck_app_lib::git::ops::commit::commit_files;
use std::process::Command;
use tempfile::tempdir;

fn git(p: &std::path::Path, args: &[&str]) {
    let output = Command::new("git").current_dir(p).args(args).output().unwrap();
    assert!(
        output.status.success(),
        "git {:?} failed: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[tokio::test]
async fn commit_only_selected_files() {
    let dir = tempdir().unwrap();
    let p = dir.path();
    git(p, &["init", "-b", "main"]);
    git(p, &["config", "user.email", "t@t.t"]);
    git(p, &["config", "user.name", "t"]);
    git(p, &["commit", "--allow-empty", "-m", "init"]);

    std::fs::write(p.join("a.txt"), "1").unwrap();
    std::fs::write(p.join("b.txt"), "2").unwrap();

    commit_files(p, &["a.txt".to_string()], "add a", false)
        .await
        .unwrap();

    let out = Command::new("git")
        .current_dir(p)
        .args(["status", "--porcelain"])
        .output()
        .unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("b.txt"), "b should remain untracked: {s:?}");
    assert!(!s.contains("a.txt"), "a should be committed: {s:?}");
}

#[tokio::test]
async fn commit_preserves_unrelated_staged_files() {
    let dir = tempdir().unwrap();
    let p = dir.path();
    git(p, &["init", "-b", "main"]);
    git(p, &["config", "user.email", "t@t.t"]);
    git(p, &["config", "user.name", "t"]);
    std::fs::write(p.join("a.txt"), "base a").unwrap();
    std::fs::write(p.join("b.txt"), "base b").unwrap();
    git(p, &["add", "a.txt", "b.txt"]);
    git(p, &["commit", "-m", "init"]);

    std::fs::write(p.join("a.txt"), "selected").unwrap();
    std::fs::write(p.join("b.txt"), "staged elsewhere").unwrap();
    git(p, &["add", "b.txt"]);

    commit_files(p, &["a.txt".to_string()], "commit a", false)
        .await
        .unwrap();

    let show_a = Command::new("git")
        .current_dir(p)
        .args(["show", "HEAD:a.txt"])
        .output()
        .unwrap();
    assert_eq!(String::from_utf8_lossy(&show_a.stdout), "selected");

    let show_b = Command::new("git")
        .current_dir(p)
        .args(["show", "HEAD:b.txt"])
        .output()
        .unwrap();
    assert_eq!(String::from_utf8_lossy(&show_b.stdout), "base b");

    let diff_cached = Command::new("git")
        .current_dir(p)
        .args(["diff", "--cached", "--name-only"])
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8_lossy(&diff_cached.stdout).trim(),
        "b.txt"
    );
}

use std::process::Command;
use tempfile::tempdir;
use git_lite_app_lib::git::snapshot::build_snapshot;

fn git(dir: &std::path::Path, args: &[&str]) {
    let st = Command::new("git").current_dir(dir).args(args).status().unwrap();
    assert!(st.success(), "git {:?} failed", args);
}

#[tokio::test]
async fn snapshot_reflects_repo_state() {
    let dir = tempdir().unwrap();
    let p = dir.path();
    git(p, &["init", "-b", "main"]);
    git(p, &["config", "user.email", "t@t.t"]);
    git(p, &["config", "user.name", "t"]);
    std::fs::write(p.join("a.txt"), "hello").unwrap();
    git(p, &["add", "a.txt"]);
    git(p, &["commit", "-m", "init"]);

    std::fs::write(p.join("b.txt"), "x").unwrap();
    std::fs::write(p.join("a.txt"), "world").unwrap();

    let snap = build_snapshot(p, None).await.unwrap();
    assert_eq!(snap.head.branch.as_deref(), Some("main"));
    assert_eq!(snap.files.len(), 2);
    assert_eq!(snap.branches.local.len(), 1);
    assert_eq!(snap.branches.local[0].name, "main");
    assert_eq!(snap.log.len(), 1);
    assert_eq!(snap.log[0].subject, "init");
    assert!(snap.in_progress.is_none());
}

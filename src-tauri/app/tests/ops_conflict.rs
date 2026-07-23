use std::path::{Path, PathBuf};
use std::process::Command;

use pluck_app_lib::git::ops::cherry_pick::{cherry_pick, cherry_pick_continue};
use pluck_app_lib::git::ops::conflict::resolve_conflict_text;
use pluck_app_lib::git::ops::merge::{merge_continue, merge_into_current};
use pluck_app_lib::git::ops::rebase::rebase_continue;
use pluck_app_lib::git::ops::revert::{revert, revert_continue};
use tempfile::{tempdir, TempDir};

fn git(repo: &Path, args: &[&str]) {
    let output = Command::new("git")
        .current_dir(repo)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn git_output(repo: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .current_dir(repo)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

fn write(repo: &Path, content: &str) {
    std::fs::write(repo.join("conflict.txt"), content).unwrap();
}

fn setup_repo() -> TempDir {
    let repo = tempdir().unwrap();
    git(repo.path(), &["init", "-b", "main"]);
    git(repo.path(), &["config", "user.email", "test@example.com"]);
    git(repo.path(), &["config", "user.name", "Test User"]);
    git(repo.path(), &["config", "core.editor", "true"]);
    write(repo.path(), "base\n");
    git(repo.path(), &["add", "conflict.txt"]);
    git(repo.path(), &["commit", "-m", "base"]);
    repo
}

fn assert_no_unmerged_paths(repo: &Path) {
    assert!(git_output(repo, &["diff", "--name-only", "--diff-filter=U"]).is_empty());
}

fn rebase_state_exists(repo: &Path) -> bool {
    repo.join(".git/rebase-merge").exists() || repo.join(".git/rebase-apply").exists()
}

fn divergent_branches(repo: &Path) {
    git(repo, &["switch", "-c", "feature"]);
    write(repo, "feature\n");
    git(repo, &["commit", "-am", "feature"]);

    git(repo, &["switch", "main"]);
    write(repo, "main\n");
    git(repo, &["commit", "-am", "main"]);
}

#[tokio::test]
async fn rebase_continue_rejects_unresolved_conflicts() {
    let repo = setup_repo();
    divergent_branches(repo.path());
    git(repo.path(), &["switch", "feature"]);

    let output = Command::new("git")
        .current_dir(repo.path())
        .args(["rebase", "main"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(rebase_state_exists(repo.path()));

    let error = rebase_continue(repo.path()).await.unwrap_err();
    assert!(format!("{error}").contains("1 unresolved conflict(s)"));
    assert!(rebase_state_exists(repo.path()));

    resolve_conflict_text(repo.path(), "conflict.txt", "resolved\n")
        .await
        .unwrap();
    rebase_continue(repo.path()).await.unwrap();
    assert_no_unmerged_paths(repo.path());
}

#[tokio::test]
async fn merge_continue_rejects_unresolved_conflicts() {
    let repo = setup_repo();
    divergent_branches(repo.path());

    merge_into_current(repo.path(), "feature").await.unwrap();
    assert!(repo.path().join(".git/MERGE_HEAD").exists());

    let error = merge_continue(repo.path()).await.unwrap_err();
    assert!(format!("{error}").contains("1 unresolved conflict(s)"));
    assert!(repo.path().join(".git/MERGE_HEAD").exists());

    resolve_conflict_text(repo.path(), "conflict.txt", "resolved\n")
        .await
        .unwrap();
    merge_continue(repo.path()).await.unwrap();
    assert_no_unmerged_paths(repo.path());
}

#[tokio::test]
async fn cherry_pick_continue_rejects_unresolved_conflicts() {
    let repo = setup_repo();
    git(repo.path(), &["switch", "-c", "source"]);
    write(repo.path(), "source\n");
    git(repo.path(), &["commit", "-am", "source"]);
    let source = git_output(repo.path(), &["rev-parse", "HEAD"]);

    git(repo.path(), &["switch", "main"]);
    write(repo.path(), "target\n");
    git(repo.path(), &["commit", "-am", "target"]);

    cherry_pick(repo.path(), &[source.trim().to_string()])
        .await
        .unwrap();
    assert!(repo.path().join(".git/CHERRY_PICK_HEAD").exists());

    let error = cherry_pick_continue(repo.path()).await.unwrap_err();
    assert!(format!("{error}").contains("1 unresolved conflict(s)"));
    assert!(repo.path().join(".git/CHERRY_PICK_HEAD").exists());

    resolve_conflict_text(repo.path(), "conflict.txt", "resolved\n")
        .await
        .unwrap();
    cherry_pick_continue(repo.path()).await.unwrap();
    assert_no_unmerged_paths(repo.path());
}

#[tokio::test]
async fn revert_continue_rejects_unresolved_conflicts() {
    let repo = setup_repo();
    write(repo.path(), "changed\n");
    git(repo.path(), &["commit", "-am", "change"]);
    let change = git_output(repo.path(), &["rev-parse", "HEAD"]);

    write(repo.path(), "current\n");
    git(repo.path(), &["commit", "-am", "current"]);

    revert(repo.path(), &[change.trim().to_string()]).await.unwrap();
    assert!(repo.path().join(".git/REVERT_HEAD").exists());

    let error = revert_continue(repo.path()).await.unwrap_err();
    assert!(format!("{error}").contains("1 unresolved conflict(s)"));
    assert!(repo.path().join(".git/REVERT_HEAD").exists());

    resolve_conflict_text(repo.path(), "conflict.txt", "resolved\n")
        .await
        .unwrap();
    revert_continue(repo.path()).await.unwrap();
    assert_no_unmerged_paths(repo.path());
}

#[tokio::test]
async fn cherry_pick_conflict_is_detected_in_linked_worktree() {
    let repo = setup_repo();
    git(repo.path(), &["switch", "-c", "source"]);
    write(repo.path(), "source\n");
    git(repo.path(), &["commit", "-am", "source"]);
    let source = git_output(repo.path(), &["rev-parse", "HEAD"]);

    git(repo.path(), &["switch", "main"]);
    write(repo.path(), "target\n");
    git(repo.path(), &["commit", "-am", "target"]);

    let worktree = tempdir().unwrap();
    let worktree_path = worktree.path().join("linked");
    git(
        repo.path(),
        &[
            "worktree",
            "add",
            "-b",
            "linked",
            worktree_path.to_str().unwrap(),
            "main",
        ],
    );

    cherry_pick(&worktree_path, &[source.trim().to_string()])
        .await
        .unwrap();
    assert!(std::fs::metadata(worktree_path.join(".git"))
        .unwrap()
        .is_file());
    let git_dir = PathBuf::from(
        git_output(&worktree_path, &["rev-parse", "--git-dir"]).trim(),
    );
    assert!(git_dir.join("CHERRY_PICK_HEAD").exists());
}

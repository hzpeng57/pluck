use pluck_app_lib::git::ops::branch::{
    delete_branch, delete_precheck, BranchDeleteKind,
};
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;

fn git(cwd: &Path, args: &[&str]) {
    let output = Command::new("git")
        .current_dir(cwd)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {:?} failed: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
}

fn git_output(cwd: &Path, args: &[&str]) -> std::process::Output {
    Command::new("git")
        .current_dir(cwd)
        .args(args)
        .output()
        .unwrap()
}

#[tokio::test]
async fn deletes_remote_branch_and_remote_tracking_ref() {
    let tmp = tempdir().unwrap();
    let remote = tmp.path().join("remote.git");
    let seed = tmp.path().join("seed");
    let work = tmp.path().join("work");

    git(tmp.path(), &["init", "--bare", remote.to_str().unwrap()]);
    std::fs::create_dir(&seed).unwrap();
    git(&seed, &["init", "-b", "main"]);
    git(&seed, &["config", "user.email", "t@t.t"]);
    git(&seed, &["config", "user.name", "t"]);
    git(&seed, &["commit", "--allow-empty", "-m", "init"]);
    git(&seed, &["remote", "add", "origin", remote.to_str().unwrap()]);
    git(&seed, &["push", "-u", "origin", "main"]);
    git(&seed, &["switch", "-c", "feature/remove-me"]);
    git(&seed, &["commit", "--allow-empty", "-m", "feature"]);
    git(&seed, &["push", "-u", "origin", "feature/remove-me"]);

    git(tmp.path(), &["clone", remote.to_str().unwrap(), work.to_str().unwrap()]);
    let precheck = delete_precheck(
        &work,
        "origin/feature/remove-me",
        BranchDeleteKind::Remote,
    )
    .await
    .unwrap();
    assert!(precheck.exists);
    assert!(!precheck.is_current);

    delete_branch(
        &work,
        "origin/feature/remove-me",
        BranchDeleteKind::Remote,
        false,
    )
    .await
    .unwrap();

    let remote_refs = git_output(
        tmp.path(),
        &[
            "ls-remote",
            "--heads",
            remote.to_str().unwrap(),
            "feature/remove-me",
        ],
    );
    assert!(remote_refs.status.success());
    assert_eq!(String::from_utf8_lossy(&remote_refs.stdout).trim(), "");

    let tracking_ref = git_output(
        &work,
        &[
            "show-ref",
            "--verify",
            "--quiet",
            "refs/remotes/origin/feature/remove-me",
        ],
    );
    assert!(!tracking_ref.status.success());
}

#[tokio::test]
async fn remote_delete_precheck_uses_exact_ref_name() {
    let tmp = tempdir().unwrap();
    let remote = tmp.path().join("remote.git");
    let seed = tmp.path().join("seed");
    let work = tmp.path().join("work");

    git(tmp.path(), &["init", "--bare", remote.to_str().unwrap()]);
    std::fs::create_dir(&seed).unwrap();
    git(&seed, &["init", "-b", "main"]);
    git(&seed, &["config", "user.email", "t@t.t"]);
    git(&seed, &["config", "user.name", "t"]);
    git(&seed, &["commit", "--allow-empty", "-m", "init"]);
    git(&seed, &["remote", "add", "origin", remote.to_str().unwrap()]);
    git(&seed, &["push", "-u", "origin", "main"]);
    git(&seed, &["switch", "-c", "feature/name/child"]);
    git(&seed, &["commit", "--allow-empty", "-m", "child"]);
    git(&seed, &["push", "-u", "origin", "feature/name/child"]);

    git(tmp.path(), &["clone", remote.to_str().unwrap(), work.to_str().unwrap()]);
    let precheck = delete_precheck(&work, "origin/feature/name", BranchDeleteKind::Remote)
        .await
        .unwrap();

    assert!(!precheck.exists);
}

#[tokio::test]
async fn remote_delete_rejects_remote_head_symbolic_ref() {
    let dir = tempdir().unwrap();
    git(dir.path(), &["init", "-b", "main"]);

    let err = delete_branch(dir.path(), "origin/HEAD", BranchDeleteKind::Remote, false)
        .await
        .unwrap_err();

    assert!(format!("{err}").contains("<remote>/<branch>"));
}

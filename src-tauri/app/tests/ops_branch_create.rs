use std::process::Command;
use tempfile::tempdir;
use pluck_app_lib::git::ops::branch::{create_branch, rename_branch};

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

#[tokio::test]
async fn creates_branch_from_remote_base_without_tracking() {
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

    git(tmp.path(), &["clone", remote.to_str().unwrap(), work.to_str().unwrap()]);
    create_branch(&work, "perf/no-track", Some("origin/main")).await.unwrap();

    let head = Command::new("git").current_dir(&work).args(["symbolic-ref", "--short", "HEAD"]).output().unwrap();
    assert_eq!(String::from_utf8_lossy(&head.stdout).trim(), "perf/no-track");

    let upstream = Command::new("git")
        .current_dir(&work)
        .args(["rev-parse", "--abbrev-ref", "perf/no-track@{upstream}"])
        .output()
        .unwrap();
    assert!(!upstream.status.success(), "new branch unexpectedly tracks an upstream");
}

#[tokio::test]
async fn renames_branch_and_can_unset_upstream() {
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

    git(tmp.path(), &["clone", remote.to_str().unwrap(), work.to_str().unwrap()]);
    rename_branch(&work, "main", "perf/renamed", true).await.unwrap();

    let head = Command::new("git").current_dir(&work).args(["symbolic-ref", "--short", "HEAD"]).output().unwrap();
    assert_eq!(String::from_utf8_lossy(&head.stdout).trim(), "perf/renamed");

    let upstream = Command::new("git")
        .current_dir(&work)
        .args(["rev-parse", "--abbrev-ref", "perf/renamed@{upstream}"])
        .output()
        .unwrap();
    assert!(!upstream.status.success(), "renamed branch unexpectedly kept upstream");
}

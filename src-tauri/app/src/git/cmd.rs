use crate::error::{GitError, GitResult};
use std::{
    env,
    ffi::OsString,
    path::{Path, PathBuf},
};
use tokio::process::Command;

#[derive(Debug)]
pub struct GitOutput {
    pub stdout: String,
    pub stderr: String,
}

pub async fn run_git(cwd: &Path, args: &[&str]) -> GitResult<GitOutput> {
    let output = git_command(cwd)
        .args(args)
        .output()
        .await
        .map_err(|e| GitError::spawn(format!("spawn git failed: {e}")))?;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    if !output.status.success() {
        return Err(GitError::from_stderr(output.status.code().unwrap_or(-1), &stderr));
    }
    Ok(GitOutput { stdout, stderr })
}

pub async fn run_git_allow_exit_codes(
    cwd: &Path,
    args: &[&str],
    allowed: &[i32],
) -> GitResult<(GitOutput, i32)> {
    let output = git_command(cwd)
        .args(args)
        .output()
        .await
        .map_err(|e| GitError::spawn(format!("spawn git failed: {e}")))?;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let code = output.status.code().unwrap_or(-1);
    if !output.status.success() && !allowed.contains(&code) {
        return Err(GitError::from_stderr(code, &stderr));
    }
    Ok((GitOutput { stdout, stderr }, code))
}

pub fn git_command(cwd: &Path) -> Command {
    let mut command = Command::new("git");
    command.current_dir(cwd);
    if let Some(path) = git_child_path_from(env::var_os("PATH")) {
        command.env("PATH", path);
    }
    command
}

fn git_child_path_from(current: Option<OsString>) -> Option<OsString> {
    let mut paths: Vec<PathBuf> = current
        .as_ref()
        .map(env::split_paths)
        .map(Iterator::collect)
        .unwrap_or_default();

    append_common_git_tool_paths(&mut paths);

    if paths.is_empty() {
        return current;
    }
    env::join_paths(paths).ok()
}

#[cfg(target_os = "macos")]
fn append_common_git_tool_paths(paths: &mut Vec<PathBuf>) {
    for path in [
        "/opt/homebrew/bin",
        "/opt/homebrew/sbin",
        "/usr/local/bin",
        "/usr/local/sbin",
        "/opt/local/bin",
        "/usr/bin",
        "/bin",
        "/usr/sbin",
        "/sbin",
    ] {
        push_path_if_missing(paths, PathBuf::from(path));
    }
}

#[cfg(not(target_os = "macos"))]
fn append_common_git_tool_paths(_paths: &mut Vec<PathBuf>) {}

#[cfg(target_os = "macos")]
fn push_path_if_missing(paths: &mut Vec<PathBuf>, path: PathBuf) {
    if !paths.iter().any(|existing| existing == &path) {
        paths.push(path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{env, ffi::OsString, path::PathBuf};
    use tempfile::tempdir;

    #[cfg(target_os = "macos")]
    #[test]
    fn git_child_path_adds_common_macos_tool_paths() {
        let path = git_child_path_from(Some(OsString::from(
            "/usr/bin:/bin:/usr/sbin:/sbin",
        )))
        .unwrap();
        let paths: Vec<PathBuf> = env::split_paths(&path).collect();

        assert!(paths.contains(&PathBuf::from("/opt/homebrew/bin")));
        assert!(paths.contains(&PathBuf::from("/usr/local/bin")));
        assert_eq!(
            paths
                .iter()
                .filter(|p| **p == PathBuf::from("/usr/bin"))
                .count(),
            1
        );
    }

    #[tokio::test]
    async fn run_git_version_succeeds() {
        let dir = tempdir().unwrap();
        let out = run_git(dir.path(), &["--version"]).await.unwrap();
        assert!(out.stdout.starts_with("git version"));
    }

    #[tokio::test]
    async fn run_git_in_non_repo_errors() {
        let dir = tempdir().unwrap();
        let err = run_git(dir.path(), &["status"]).await.unwrap_err();
        assert!(format!("{err}").to_lowercase().contains("not a git repository"));
    }

    #[tokio::test]
    async fn run_git_allow_exit_codes_returns_allowed_nonzero_exit() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("a.txt"), "one\n").unwrap();
        std::fs::write(dir.path().join("b.txt"), "two\n").unwrap();

        let (out, code) = run_git_allow_exit_codes(
            dir.path(),
            &["diff", "--no-index", "a.txt", "b.txt"],
            &[1],
        )
        .await
        .unwrap();

        assert_eq!(code, 1);
        assert!(out.stdout.contains("-one"));
        assert!(out.stdout.contains("+two"));
    }
}

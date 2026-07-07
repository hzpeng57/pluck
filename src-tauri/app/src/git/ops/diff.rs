use crate::error::{GitError, GitResult};
use crate::git::cmd::{run_git, run_git_allow_exit_codes};
use serde::Serialize;
use std::path::{Component, Path};

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum DiffKind {
    WorkingTree,
    Commit,
}

#[derive(Debug, Clone)]
pub struct DiffMeta {
    pub kind: DiffKind,
    pub path: String,
    pub old_path: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum DiffLineKind {
    Context,
    Added,
    Deleted,
    NoNewline,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DiffLine {
    pub kind: DiffLineKind,
    pub old_number: Option<u32>,
    pub new_number: Option<u32>,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DiffHunk {
    pub header: String,
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FileDiff {
    pub kind: DiffKind,
    pub path: String,
    pub old_path: Option<String>,
    pub status: String,
    pub binary: bool,
    pub too_large: bool,
    pub additions: u32,
    pub deletions: u32,
    pub hunks: Vec<DiffHunk>,
}

const MAX_DIFF_BYTES: usize = 1_500_000;

fn validate_repo_relative(path: &str) -> GitResult<()> {
    let p = Path::new(path);
    if path.is_empty()
        || p.is_absolute()
        || p.components().any(|c| {
            matches!(c, Component::ParentDir)
                || matches!(
                    c,
                    Component::Normal(name)
                        if name.to_string_lossy().eq_ignore_ascii_case(".git")
                )
        })
    {
        return Err(GitError::parse(format!("unsafe repository path: {path}")));
    }
    Ok(())
}

fn diff_paths<'a>(path: &'a str, old_path: Option<&'a str>) -> Vec<&'a str> {
    match old_path {
        Some(old) if old != path => vec![old, path],
        _ => vec![path],
    }
}

async fn ensure_untracked_worktree_file(repo: &Path, path: &str) -> GitResult<()> {
    let out = run_git(
        repo,
        &["ls-files", "-z", "--others", "--exclude-standard", "--", path],
    )
    .await?;
    let matches = out
        .stdout
        .split_terminator('\0')
        .any(|candidate| candidate == path);
    if !matches {
        return Err(GitError::parse(format!(
            "not an untracked worktree file: {path}"
        )));
    }
    Ok(())
}

pub async fn working_file_diff(
    repo: &Path,
    path: &str,
    old_path: Option<&str>,
    status: &str,
) -> GitResult<FileDiff> {
    validate_repo_relative(path)?;
    if let Some(old) = old_path {
        validate_repo_relative(old)?;
    }

    let raw = if status == "untracked" {
        ensure_untracked_worktree_file(repo, path).await?;
        let (out, _) = run_git_allow_exit_codes(
            repo,
            &[
                "diff",
                "--no-ext-diff",
                "--no-index",
                "--no-color",
                "-U3",
                "--",
                "/dev/null",
                path,
            ],
            &[1],
        )
        .await?;
        out.stdout
    } else {
        let mut args = vec![
            "diff",
            "--no-ext-diff",
            "--no-color",
            "--find-renames",
            "-U3",
            "HEAD",
            "--",
        ];
        let paths = diff_paths(path, old_path);
        for p in &paths {
            args.push(p);
        }
        run_git(repo, &args).await?.stdout
    };

    parse_unified_diff(
        &raw,
        DiffMeta {
            kind: DiffKind::WorkingTree,
            path: path.to_string(),
            old_path: old_path.map(str::to_string),
            status: status.to_string(),
        },
    )
}

pub async fn commit_file_diff(
    repo: &Path,
    hash: &str,
    path: &str,
    old_path: Option<&str>,
    status: &str,
) -> GitResult<FileDiff> {
    validate_repo_relative(path)?;
    if let Some(old) = old_path {
        validate_repo_relative(old)?;
    }

    let mut args = vec![
        "diff-tree",
        "--root",
        "--no-commit-id",
        "-r",
        "-p",
        "-m",
        "--no-ext-diff",
        "--first-parent",
        "--find-renames",
        "--no-color",
        "-U3",
        hash,
        "--",
    ];
    let paths = diff_paths(path, old_path);
    for p in &paths {
        args.push(p);
    }
    let raw = run_git(repo, &args).await?.stdout;

    parse_unified_diff(
        &raw,
        DiffMeta {
            kind: DiffKind::Commit,
            path: path.to_string(),
            old_path: old_path.map(str::to_string),
            status: status.to_string(),
        },
    )
}

pub fn parse_unified_diff(raw: &str, meta: DiffMeta) -> GitResult<FileDiff> {
    let too_large = raw.len() > MAX_DIFF_BYTES;
    let content = truncate_to_char_boundary(raw, MAX_DIFF_BYTES);
    let binary = content.contains("Binary files ") || content.contains("GIT binary patch");
    let mut hunks = Vec::new();
    let mut current: Option<DiffHunk> = None;
    let mut old_line = 0u32;
    let mut new_line = 0u32;
    let mut additions = 0u32;
    let mut deletions = 0u32;

    for line in content.lines() {
        if let Some((old_start, old_lines, new_start, new_lines)) = parse_hunk_header(line)? {
            if let Some(hunk) = current.take() {
                hunks.push(hunk);
            }
            old_line = old_start;
            new_line = new_start;
            current = Some(DiffHunk {
                header: line.to_string(),
                old_start,
                old_lines,
                new_start,
                new_lines,
                lines: Vec::new(),
            });
            continue;
        }

        let Some(hunk) = current.as_mut() else { continue };
        if line.starts_with("\\ No newline at end of file") {
            hunk.lines.push(DiffLine {
                kind: DiffLineKind::NoNewline,
                old_number: None,
                new_number: None,
                content: line.to_string(),
            });
        } else if let Some(content) = line.strip_prefix('+') {
            additions += 1;
            hunk.lines.push(DiffLine {
                kind: DiffLineKind::Added,
                old_number: None,
                new_number: Some(new_line),
                content: content.to_string(),
            });
            new_line += 1;
        } else if let Some(content) = line.strip_prefix('-') {
            deletions += 1;
            hunk.lines.push(DiffLine {
                kind: DiffLineKind::Deleted,
                old_number: Some(old_line),
                new_number: None,
                content: content.to_string(),
            });
            old_line += 1;
        } else {
            let content = line.strip_prefix(' ').unwrap_or(line);
            hunk.lines.push(DiffLine {
                kind: DiffLineKind::Context,
                old_number: Some(old_line),
                new_number: Some(new_line),
                content: content.to_string(),
            });
            old_line += 1;
            new_line += 1;
        }
    }

    if let Some(hunk) = current.take() {
        hunks.push(hunk);
    }

    Ok(FileDiff {
        kind: meta.kind,
        path: meta.path,
        old_path: meta.old_path,
        status: meta.status,
        binary,
        too_large,
        additions,
        deletions,
        hunks: if binary { Vec::new() } else { hunks },
    })
}

fn truncate_to_char_boundary(raw: &str, max_bytes: usize) -> &str {
    if raw.len() <= max_bytes {
        return raw;
    }

    let mut end = max_bytes;
    while !raw.is_char_boundary(end) {
        end -= 1;
    }
    &raw[..end]
}

fn parse_hunk_header(line: &str) -> GitResult<Option<(u32, u32, u32, u32)>> {
    if !line.starts_with("@@ ") {
        return Ok(None);
    }
    let end = line[3..]
        .find(" @@")
        .ok_or_else(|| GitError::parse(format!("bad hunk header: {line}")))?;
    let range = &line[3..3 + end];
    let mut parts = range.split_whitespace();
    let old = parts.next().ok_or_else(|| GitError::parse("missing old hunk range"))?;
    let new = parts.next().ok_or_else(|| GitError::parse("missing new hunk range"))?;
    let (old_start, old_lines) = parse_range(old.trim_start_matches('-'))?;
    let (new_start, new_lines) = parse_range(new.trim_start_matches('+'))?;
    Ok(Some((old_start, old_lines, new_start, new_lines)))
}

fn parse_range(raw: &str) -> GitResult<(u32, u32)> {
    let mut parts = raw.split(',');
    let start = parts
        .next()
        .and_then(|s| s.parse::<u32>().ok())
        .ok_or_else(|| GitError::parse(format!("bad hunk range: {raw}")))?;
    let lines = parts.next().and_then(|s| s.parse::<u32>().ok()).unwrap_or(1);
    Ok((start, lines))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meta() -> DiffMeta {
        DiffMeta {
            kind: DiffKind::WorkingTree,
            path: "src/app.ts".into(),
            old_path: None,
            status: "modified".into(),
        }
    }

    #[test]
    fn parses_unified_hunks_with_line_numbers_and_stats() {
        let raw = "\
diff --git a/src/app.ts b/src/app.ts
index 1111111..2222222 100644
--- a/src/app.ts
+++ b/src/app.ts
@@ -1,3 +1,4 @@
 const a = 1;
-const b = 2;
+const b = 3;
+const c = 4;
 const d = 5;
";
        let diff = parse_unified_diff(raw, meta()).unwrap();

        assert!(!diff.binary);
        assert!(!diff.too_large);
        assert_eq!(diff.additions, 2);
        assert_eq!(diff.deletions, 1);
        assert_eq!(diff.hunks.len(), 1);
        assert_eq!(diff.hunks[0].old_start, 1);
        assert_eq!(diff.hunks[0].new_start, 1);
        assert_eq!(diff.hunks[0].lines[1].kind, DiffLineKind::Deleted);
        assert_eq!(diff.hunks[0].lines[1].old_number, Some(2));
        assert_eq!(diff.hunks[0].lines[1].new_number, None);
        assert_eq!(diff.hunks[0].lines[2].kind, DiffLineKind::Added);
        assert_eq!(diff.hunks[0].lines[2].old_number, None);
        assert_eq!(diff.hunks[0].lines[2].new_number, Some(2));
    }

    #[test]
    fn detects_binary_patch_without_hunks() {
        let raw = "Binary files a/logo.png and b/logo.png differ\n";
        let diff = parse_unified_diff(raw, meta()).unwrap();

        assert!(diff.binary);
        assert_eq!(diff.hunks.len(), 0);
    }

    #[test]
    fn truncates_large_utf8_diff_on_char_boundary() {
        let prefix = "\
diff --git a/src/app.ts b/src/app.ts
index 1111111..2222222 100644
--- a/src/app.ts
+++ b/src/app.ts
@@ -1 +1 @@
+";
        let filler = "x".repeat(MAX_DIFF_BYTES - prefix.len() - 1);
        let raw = format!("{prefix}{filler}好\n");

        assert!(raw.len() > MAX_DIFF_BYTES);
        assert!(!raw.is_char_boundary(MAX_DIFF_BYTES));

        let diff = parse_unified_diff(&raw, meta()).unwrap();

        assert!(diff.too_large);
        assert_eq!(diff.hunks.len(), 1);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::path::Path;
    use std::process::Command;
    use tempfile::tempdir;

    fn git(repo: &Path, args: &[&str]) {
        let status = Command::new("git").current_dir(repo).args(args).status().unwrap();
        assert!(status.success(), "git {:?} failed", args);
    }

    fn repo() -> tempfile::TempDir {
        let dir = tempdir().unwrap();
        git(dir.path(), &["init", "-b", "main"]);
        git(dir.path(), &["config", "user.email", "t@t.t"]);
        git(dir.path(), &["config", "user.name", "t"]);
        std::fs::write(dir.path().join("a.txt"), "one\ntwo\n").unwrap();
        git(dir.path(), &["add", "a.txt"]);
        git(dir.path(), &["commit", "-m", "init"]);
        dir
    }

    #[tokio::test]
    async fn working_file_diff_includes_staged_and_unstaged_changes() {
        let dir = repo();
        std::fs::write(dir.path().join("a.txt"), "one\nthree\n").unwrap();
        git(dir.path(), &["add", "a.txt"]);
        std::fs::write(dir.path().join("a.txt"), "one\nthree\nfour\n").unwrap();

        let diff = working_file_diff(dir.path(), "a.txt", None, "modified").await.unwrap();

        assert_eq!(diff.path, "a.txt");
        assert_eq!(diff.additions, 2);
        assert_eq!(diff.deletions, 1);
        assert!(!diff.hunks.is_empty());
    }

    #[tokio::test]
    async fn untracked_file_diff_uses_no_index_exit_one_as_success() {
        let dir = repo();
        std::fs::write(dir.path().join("new.txt"), "hello\n").unwrap();

        let diff = working_file_diff(dir.path(), "new.txt", None, "untracked").await.unwrap();

        assert_eq!(diff.status, "untracked");
        assert_eq!(diff.additions, 1);
        assert_eq!(diff.deletions, 0);
    }

    #[tokio::test]
    async fn working_file_diff_rejects_absolute_and_parent_paths() {
        let dir = repo();
        let absolute = dir.path().join("a.txt");

        assert_parse_error(
            working_file_diff(
                dir.path(),
                absolute.to_str().unwrap(),
                None,
                "modified",
            )
            .await,
            "unsafe repository path",
        );
        assert_parse_error(
            working_file_diff(dir.path(), "../a.txt", None, "modified").await,
            "unsafe repository path",
        );
    }

    #[tokio::test]
    async fn untracked_file_diff_rejects_git_metadata_paths() {
        let dir = repo();

        assert_parse_error(
            working_file_diff(dir.path(), ".git/config", None, "untracked").await,
            "unsafe repository path",
        );
    }

    #[tokio::test]
    async fn untracked_file_diff_requires_an_actual_untracked_file() {
        let dir = repo();

        assert_parse_error(
            working_file_diff(dir.path(), "a.txt", None, "untracked").await,
            "not an untracked worktree file",
        );
    }

    #[tokio::test]
    async fn untracked_file_diff_rejects_ignored_paths() {
        let dir = repo();
        std::fs::write(dir.path().join(".gitignore"), "*.secret\n").unwrap();
        std::fs::write(dir.path().join("token.secret"), "hidden\n").unwrap();

        assert_parse_error(
            working_file_diff(dir.path(), "token.secret", None, "untracked").await,
            "not an untracked worktree file",
        );
    }

    #[tokio::test]
    async fn commit_file_diff_reads_selected_commit_patch() {
        let dir = repo();
        std::fs::write(dir.path().join("a.txt"), "one\nthree\n").unwrap();
        git(dir.path(), &["commit", "-am", "change a"]);
        let hash = Command::new("git")
            .current_dir(dir.path())
            .args(["rev-parse", "HEAD"])
            .output()
            .unwrap();
        let hash = String::from_utf8_lossy(&hash.stdout).trim().to_string();

        let diff = commit_file_diff(dir.path(), &hash, "a.txt", None, "modified").await.unwrap();

        assert_eq!(diff.kind, DiffKind::Commit);
        assert_eq!(diff.additions, 1);
        assert_eq!(diff.deletions, 1);
    }

    fn assert_parse_error<T>(result: GitResult<T>, expected: &str) {
        let Err(GitError::Parse(message)) = result else {
            panic!("expected parse error containing {expected}");
        };
        assert!(
            message.contains(expected),
            "expected parse error containing {expected:?}, got {message:?}",
        );
    }
}

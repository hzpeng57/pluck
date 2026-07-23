use crate::error::{GitError, GitResult};
use crate::git::cmd::{run_git, run_git_allow_exit_codes};
use crate::git::path::validate_repo_relative;
use serde::Serialize;
use std::path::{Path, PathBuf};

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

#[derive(Debug, Clone, Copy, Default)]
pub struct DiffOptions {
    pub ignore_whitespace: bool,
}

fn push_diff_options(args: &mut Vec<&str>, options: DiffOptions) {
    if options.ignore_whitespace {
        args.push("--ignore-all-space");
    }
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

fn repo_path(repo: &Path, path: &str) -> GitResult<PathBuf> {
    validate_repo_relative(path)?;
    Ok(repo.join(path))
}

fn ensure_removal_stays_in_repo(repo: &Path, full: &Path, path: &str) -> GitResult<()> {
    let repo = repo.canonicalize()?;
    let Some(parent) = full.parent() else {
        return Err(GitError::parse(format!("unsafe repository path: {path}")));
    };
    let parent = parent.canonicalize()?;
    if !parent.starts_with(&repo) {
        return Err(GitError::parse(format!("unsafe repository path: {path}")));
    }
    Ok(())
}

fn remove_worktree_path(repo: &Path, path: &str) -> GitResult<()> {
    let full = repo_path(repo, path)?;
    let meta = match full.symlink_metadata() {
        Ok(meta) => meta,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(err) => return Err(err.into()),
    };
    ensure_removal_stays_in_repo(repo, &full, path)?;
    if meta.file_type().is_dir() {
        std::fs::remove_dir_all(full)?;
    } else {
        std::fs::remove_file(full)?;
    }
    Ok(())
}

fn remove_worktree_file(repo: &Path, path: &str) -> GitResult<()> {
    let full = repo_path(repo, path)?;
    let meta = match full.symlink_metadata() {
        Ok(meta) => meta,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(err) => return Err(err.into()),
    };
    ensure_removal_stays_in_repo(repo, &full, path)?;
    if meta.file_type().is_dir() {
        return Err(GitError::parse(format!(
            "not an untracked worktree file: {path}"
        )));
    }
    std::fs::remove_file(full)?;
    Ok(())
}

fn diff_paths<'a>(path: &'a str, old_path: Option<&'a str>) -> Vec<&'a str> {
    match old_path {
        Some(old) if old != path => vec![old, path],
        _ => vec![path],
    }
}

async fn ensure_untracked_worktree_file(repo: &Path, path: &str) -> GitResult<()> {
    let candidates = untracked_worktree_candidates(repo, path).await?;
    let matches = candidates.iter().any(|candidate| candidate == path);
    if !matches {
        return Err(GitError::parse(format!(
            "not an untracked worktree file: {path}"
        )));
    }
    Ok(())
}

async fn untracked_worktree_candidates(repo: &Path, path: &str) -> GitResult<Vec<String>> {
    let out = run_git(
        repo,
        &["ls-files", "-z", "--others", "--exclude-standard", "--", path],
    )
    .await?;
    Ok(out
        .stdout
        .split_terminator('\0')
        .filter(|candidate| !candidate.is_empty())
        .map(str::to_string)
        .collect())
}

async fn remove_untracked_worktree_path(repo: &Path, path: &str) -> GitResult<()> {
    let candidates = untracked_worktree_candidates(repo, path).await?;
    if candidates.iter().any(|candidate| candidate == path) {
        return remove_worktree_file(repo, path);
    }
    if is_untracked_directory_entry(repo, path, &candidates).await? {
        return remove_untracked_directory_candidates(repo, path, &candidates);
    }
    Err(GitError::parse(format!(
        "not an untracked worktree path: {path}"
    )))
}

async fn is_untracked_directory_entry(
    repo: &Path,
    path: &str,
    candidates: &[String],
) -> GitResult<bool> {
    if candidates.is_empty() {
        return Ok(false);
    }

    let dir = path.trim_end_matches('/');
    if dir.is_empty() {
        return Ok(false);
    }

    let full = repo_path(repo, path)?;
    let meta = match full.symlink_metadata() {
        Ok(meta) => meta,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(err) => return Err(err.into()),
    };
    if !meta.file_type().is_dir() {
        return Ok(false);
    }
    ensure_removal_stays_in_repo(repo, &full, path)?;

    let dir_path = Path::new(dir);
    let all_inside_dir = candidates.iter().all(|candidate| {
        validate_repo_relative(candidate).is_ok()
            && Path::new(candidate).starts_with(dir_path)
            && Path::new(candidate) != dir_path
    });
    if !all_inside_dir {
        return Ok(false);
    }

    let ignored = run_git(
        repo,
        &[
            "ls-files",
            "-z",
            "--others",
            "--ignored",
            "--exclude-standard",
            "--",
            path,
        ],
    )
    .await?;
    if ignored
        .stdout
        .split_terminator('\0')
        .any(|candidate| !candidate.is_empty())
    {
        return Err(GitError::parse(format!(
            "untracked directory contains ignored files: {path}"
        )));
    }

    Ok(true)
}

fn remove_untracked_directory_candidates(
    repo: &Path,
    path: &str,
    candidates: &[String],
) -> GitResult<()> {
    let dir = path.trim_end_matches('/');
    let dir_path = Path::new(dir);

    for candidate in candidates {
        validate_repo_relative(candidate)?;
        let candidate_path = Path::new(candidate);
        if !candidate_path.starts_with(dir_path) || candidate_path == dir_path {
            return Err(GitError::parse(format!(
                "not an untracked worktree path: {path}"
            )));
        }
        remove_worktree_file(repo, candidate)?;
    }

    prune_empty_untracked_dirs(repo, dir_path, candidates)
}

fn prune_empty_untracked_dirs(
    repo: &Path,
    base_dir: &Path,
    candidates: &[String],
) -> GitResult<()> {
    for candidate in candidates {
        let mut current = Path::new(candidate).parent();
        while let Some(dir) = current {
            if dir.as_os_str().is_empty() || !dir.starts_with(base_dir) {
                break;
            }

            remove_empty_repo_dir(repo, dir)?;
            if dir == base_dir {
                break;
            }
            current = dir.parent();
        }
    }
    Ok(())
}

fn remove_empty_repo_dir(repo: &Path, path: &Path) -> GitResult<()> {
    let display_path = path.to_string_lossy();
    let full = repo_path(repo, &display_path)?;
    let meta = match full.symlink_metadata() {
        Ok(meta) => meta,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(err) => return Err(err.into()),
    };
    if !meta.file_type().is_dir() {
        return Ok(());
    }

    let repo = repo.canonicalize()?;
    let dir = full.canonicalize()?;
    if !dir.starts_with(&repo) {
        return Err(GitError::parse(format!(
            "unsafe repository path: {display_path}"
        )));
    }
    ensure_removal_stays_in_repo(&repo, &full, &display_path)?;

    match std::fs::remove_dir(full) {
        Ok(()) => Ok(()),
        Err(err)
            if matches!(
                err.kind(),
                std::io::ErrorKind::NotFound | std::io::ErrorKind::DirectoryNotEmpty
            ) =>
        {
            Ok(())
        }
        Err(err) => Err(err.into()),
    }
}

async fn ensure_index_added_file(repo: &Path, path: &str) -> GitResult<()> {
    let out = run_git(
        repo,
        &[
            "status",
            "--porcelain=v2",
            "-z",
            "--",
            path,
        ],
    )
    .await?;
    let matches = out
        .stdout
        .split_terminator('\0')
        .any(|record| status_record_is_added_path(record, path));
    if !matches {
        return Err(GitError::parse(format!("not an added index file: {path}")));
    }
    Ok(())
}

fn status_record_is_added_path(record: &str, path: &str) -> bool {
    let Some(rest) = record.strip_prefix("1 ") else {
        return false;
    };
    let parts: Vec<&str> = rest.splitn(8, ' ').collect();
    let Some(xy) = parts.first() else {
        return false;
    };
    let Some(candidate) = parts.get(7) else {
        return false;
    };
    xy.as_bytes().contains(&b'A') && *candidate == path
}

async fn ensure_renamed_file(repo: &Path, path: &str, old_path: &str) -> GitResult<()> {
    let out = run_git(
        repo,
        &[
            "diff",
            "--name-status",
            "--find-renames",
            "-z",
            "HEAD",
            "--",
            old_path,
            path,
        ],
    )
    .await?;
    let mut parts = out.stdout.split_terminator('\0');
    while let Some(status) = parts.next() {
        if status.starts_with('R') {
            let old = parts.next().unwrap_or("");
            let new = parts.next().unwrap_or("");
            if old == old_path && new == path {
                return Ok(());
            }
        } else {
            let _ = parts.next();
        }
    }
    Err(GitError::parse(format!(
        "not a renamed worktree file: {old_path} -> {path}"
    )))
}

async fn resolve_commit_oid(repo: &Path, hash: &str) -> GitResult<String> {
    if hash.is_empty() || hash.contains('\0') {
        return Err(GitError::parse("invalid commit hash"));
    }

    let commit_spec = format!("{hash}^{{commit}}");
    let out = run_git(
        repo,
        &[
            "rev-parse",
            "--verify",
            "--end-of-options",
            &commit_spec,
        ],
    )
    .await?;
    let oid = out.stdout.trim();
    if oid.is_empty() || oid.contains('\n') || oid.contains('\0') {
        return Err(GitError::parse("invalid commit hash"));
    }
    Ok(oid.to_string())
}

pub async fn rollback_file(
    repo: &Path,
    path: &str,
    old_path: Option<&str>,
    status: &str,
) -> GitResult<()> {
    validate_repo_relative(path)?;
    if let Some(old) = old_path {
        validate_repo_relative(old)?;
    }

    match status {
        "conflicted" => Err(GitError::parse(
            "Rollback for conflicted files is disabled in this version. Resolve or abort the in-progress operation first.",
        )),
        "untracked" => {
            remove_untracked_worktree_path(repo, path).await
        }
        "added" => {
            ensure_index_added_file(repo, path).await?;
            run_git(
                repo,
                &["rm", "-f", "--cached", "--ignore-unmatch", "--", path],
            )
            .await?;
            remove_worktree_path(repo, path)
        }
        "deleted" | "modified" => {
            run_git(
                repo,
                &["restore", "--staged", "--worktree", "--source=HEAD", "--", path],
            )
            .await?;
            Ok(())
        }
        "renamed" => {
            let old =
                old_path.ok_or_else(|| GitError::parse("missing old path for renamed file"))?;
            ensure_renamed_file(repo, path, old).await?;
            run_git(
                repo,
                &["restore", "--staged", "--worktree", "--source=HEAD", "--", old],
            )
            .await?;
            run_git(
                repo,
                &["rm", "-f", "--cached", "--ignore-unmatch", "--", path],
            )
            .await?;
            remove_worktree_path(repo, path)
        }
        _ => {
            run_git(
                repo,
                &["restore", "--staged", "--worktree", "--source=HEAD", "--", path],
            )
            .await?;
            Ok(())
        }
    }
}

pub async fn working_file_diff(
    repo: &Path,
    path: &str,
    old_path: Option<&str>,
    status: &str,
    options: DiffOptions,
) -> GitResult<FileDiff> {
    validate_repo_relative(path)?;
    if let Some(old) = old_path {
        validate_repo_relative(old)?;
    }

    let raw = if status == "untracked" {
        ensure_untracked_worktree_file(repo, path).await?;
        let mut args = vec!["diff", "--no-ext-diff"];
        push_diff_options(&mut args, options);
        args.extend(["--no-index", "--no-color", "-U3", "--", "/dev/null", path]);
        let (out, _) = run_git_allow_exit_codes(repo, &args, &[1]).await?;
        out.stdout
    } else {
        let mut args = vec![
            "diff",
            "--no-ext-diff",
            "--no-color",
            "--find-renames",
            "-U3",
        ];
        push_diff_options(&mut args, options);
        args.extend(["HEAD", "--"]);
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
    options: DiffOptions,
) -> GitResult<FileDiff> {
    validate_repo_relative(path)?;
    if let Some(old) = old_path {
        validate_repo_relative(old)?;
    }

    let oid = resolve_commit_oid(repo, hash).await?;
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
    ];
    push_diff_options(&mut args, options);
    args.extend([oid.as_str(), "--"]);
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

        let diff = working_file_diff(dir.path(), "a.txt", None, "modified", DiffOptions::default()).await.unwrap();

        assert_eq!(diff.path, "a.txt");
        assert_eq!(diff.additions, 2);
        assert_eq!(diff.deletions, 1);
        assert!(!diff.hunks.is_empty());
    }

    #[tokio::test]
    async fn untracked_file_diff_uses_no_index_exit_one_as_success() {
        let dir = repo();
        std::fs::write(dir.path().join("new.txt"), "hello\n").unwrap();

        let diff = working_file_diff(dir.path(), "new.txt", None, "untracked", DiffOptions::default()).await.unwrap();

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
                DiffOptions::default(),
            )
            .await,
            "unsafe repository path",
        );
        assert_parse_error(
            working_file_diff(dir.path(), "../a.txt", None, "modified", DiffOptions::default()).await,
            "unsafe repository path",
        );
    }

    #[tokio::test]
    async fn untracked_file_diff_rejects_git_metadata_paths() {
        let dir = repo();

        assert_parse_error(
            working_file_diff(dir.path(), ".git/config", None, "untracked", DiffOptions::default()).await,
            "unsafe repository path",
        );
    }

    #[tokio::test]
    async fn untracked_file_diff_requires_an_actual_untracked_file() {
        let dir = repo();

        assert_parse_error(
            working_file_diff(dir.path(), "a.txt", None, "untracked", DiffOptions::default()).await,
            "not an untracked worktree file",
        );
    }

    #[tokio::test]
    async fn untracked_file_diff_rejects_ignored_paths() {
        let dir = repo();
        std::fs::write(dir.path().join(".gitignore"), "*.secret\n").unwrap();
        std::fs::write(dir.path().join("token.secret"), "hidden\n").unwrap();

        assert_parse_error(
            working_file_diff(dir.path(), "token.secret", None, "untracked", DiffOptions::default()).await,
            "not an untracked worktree file",
        );
    }

    #[tokio::test]
    async fn working_file_diff_can_ignore_whitespace() {
        let dir = repo();
        std::fs::write(dir.path().join("a.txt"), "one\n  two\n").unwrap();

        let normal =
            working_file_diff(dir.path(), "a.txt", None, "modified", DiffOptions::default())
                .await
                .unwrap();
        let ignored = working_file_diff(
            dir.path(),
            "a.txt",
            None,
            "modified",
            DiffOptions { ignore_whitespace: true },
        )
        .await
        .unwrap();

        assert!(normal.additions > 0 || normal.deletions > 0);
        assert_eq!(ignored.additions, 0);
        assert_eq!(ignored.deletions, 0);
        assert!(ignored.hunks.is_empty());
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

        let diff =
            commit_file_diff(dir.path(), &hash, "a.txt", None, "modified", DiffOptions::default())
                .await
                .unwrap();

        assert_eq!(diff.kind, DiffKind::Commit);
        assert_eq!(diff.additions, 1);
        assert_eq!(diff.deletions, 1);
    }

    #[tokio::test]
    async fn commit_file_diff_rejects_option_like_hash() {
        let dir = repo();

        assert!(
            commit_file_diff(dir.path(), "--help", "a.txt", None, "modified", DiffOptions::default())
                .await
                .is_err(),
            "option-like hash should be rejected before diff-tree"
        );
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

#[cfg(test)]
mod rollback_tests {
    use super::*;
    use std::path::Path;
    use std::process::Command;
    use tempfile::tempdir;

    fn git(repo: &Path, args: &[&str]) {
        let status = Command::new("git").current_dir(repo).args(args).status().unwrap();
        assert!(status.success(), "git {:?} failed", args);
    }

    fn clean_repo() -> tempfile::TempDir {
        let dir = tempdir().unwrap();
        git(dir.path(), &["init", "-b", "main"]);
        git(dir.path(), &["config", "user.email", "t@t.t"]);
        git(dir.path(), &["config", "user.name", "t"]);
        std::fs::write(dir.path().join("a.txt"), "one\n").unwrap();
        git(dir.path(), &["add", "a.txt"]);
        git(dir.path(), &["commit", "-m", "init"]);
        dir
    }

    fn status(repo: &Path) -> String {
        let out = Command::new("git")
            .current_dir(repo)
            .args(["status", "--porcelain"])
            .output()
            .unwrap();
        String::from_utf8_lossy(&out.stdout).into_owned()
    }

    #[tokio::test]
    async fn rollback_modified_restores_head_version() {
        let dir = clean_repo();
        std::fs::write(dir.path().join("a.txt"), "changed\n").unwrap();

        rollback_file(dir.path(), "a.txt", None, "modified").await.unwrap();

        assert_eq!(std::fs::read_to_string(dir.path().join("a.txt")).unwrap(), "one\n");
        assert_eq!(status(dir.path()), "");
    }

    #[tokio::test]
    async fn rollback_untracked_removes_file() {
        let dir = clean_repo();
        std::fs::write(dir.path().join("new.txt"), "new\n").unwrap();

        rollback_file(dir.path(), "new.txt", None, "untracked").await.unwrap();

        assert!(!dir.path().join("new.txt").exists());
        assert_eq!(status(dir.path()), "");
    }

    #[tokio::test]
    async fn rollback_added_unstages_and_removes_file() {
        let dir = clean_repo();
        std::fs::write(dir.path().join("new.txt"), "new\n").unwrap();
        git(dir.path(), &["add", "new.txt"]);

        rollback_file(dir.path(), "new.txt", None, "added").await.unwrap();

        assert!(!dir.path().join("new.txt").exists());
        assert_eq!(status(dir.path()), "");
    }

    #[tokio::test]
    async fn rollback_intent_to_add_removes_index_entry_and_worktree_file() {
        let dir = clean_repo();
        std::fs::write(dir.path().join("intent.txt"), "intent\n").unwrap();
        git(dir.path(), &["add", "-N", "intent.txt"]);

        rollback_file(dir.path(), "intent.txt", None, "added").await.unwrap();

        assert!(!dir.path().join("intent.txt").exists());
        assert_eq!(status(dir.path()), "");
    }

    #[tokio::test]
    async fn rollback_untracked_directory_entry_removes_contained_files() {
        let dir = clean_repo();
        std::fs::create_dir(dir.path().join("dir")).unwrap();
        std::fs::write(dir.path().join("dir/a.txt"), "a\n").unwrap();

        rollback_file(dir.path(), "dir/", None, "untracked").await.unwrap();

        assert!(!dir.path().join("dir").exists());
        assert_eq!(status(dir.path()), "");
    }

    #[tokio::test]
    async fn rollback_untracked_directory_keeps_tracked_files_inside() {
        let dir = clean_repo();
        std::fs::create_dir(dir.path().join("dir")).unwrap();
        std::fs::write(dir.path().join("dir/tracked.txt"), "tracked\n").unwrap();
        git(dir.path(), &["add", "dir/tracked.txt"]);
        git(dir.path(), &["commit", "-m", "track dir file"]);
        std::fs::write(dir.path().join("dir/new.txt"), "new\n").unwrap();

        rollback_file(dir.path(), "dir/", None, "untracked").await.unwrap();

        assert_eq!(
            std::fs::read_to_string(dir.path().join("dir/tracked.txt")).unwrap(),
            "tracked\n"
        );
        assert!(!dir.path().join("dir/new.txt").exists());
        assert_eq!(status(dir.path()), "");
    }

    #[tokio::test]
    async fn rollback_renamed_restores_old_path_and_removes_new_path() {
        let dir = clean_repo();
        git(dir.path(), &["mv", "a.txt", "renamed.txt"]);

        rollback_file(dir.path(), "renamed.txt", Some("a.txt"), "renamed")
            .await
            .unwrap();

        assert_eq!(std::fs::read_to_string(dir.path().join("a.txt")).unwrap(), "one\n");
        assert!(!dir.path().join("renamed.txt").exists());
        assert_eq!(status(dir.path()), "");
    }

    #[tokio::test]
    async fn rollback_rejects_unsafe_paths() {
        let dir = clean_repo();

        assert_parse_error(
            rollback_file(dir.path(), "../a.txt", None, "modified").await,
            "unsafe repository path",
        );
        assert_parse_error(
            rollback_file(dir.path(), ".git/config", None, "untracked").await,
            "unsafe repository path",
        );
        assert_parse_error(
            rollback_file(dir.path(), ".", None, "untracked").await,
            "unsafe repository path",
        );
    }

    #[tokio::test]
    async fn rollback_conflicted_is_blocked_for_first_version() {
        let dir = clean_repo();

        let err = rollback_file(dir.path(), "a.txt", None, "conflicted")
            .await
            .unwrap_err();

        assert!(format!("{err}").contains("conflicted files"));
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

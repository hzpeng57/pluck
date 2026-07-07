# Readonly Diff And File Rollback Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a WebStorm-like readonly diff review flow and safe full-file rollback for working tree files.

**Architecture:** Add backend diff/rollback operations that continue to use the existing `run_git` command boundary, then add a frontend Review Mode where the diff viewer spans the current History + Inspector workspace. The right inspector remains a source/sidebar only; the diff itself gets a wide main area with stable line gutters, hunk rendering, and no cramped modal.

**Tech Stack:** Tauri 2, Rust git CLI operations, Vue 3 setup stores, Pinia, TailwindCSS utility classes, existing `gl-*` CSS tokens, existing lucide icon package already used by components.

## Global Constraints

- All backend Git operations must go through the existing `src-tauri/app/src/git/cmd.rs` command boundary; do not introduce `git2` or bypass the git CLI.
- Every mutation must end with `refresh_session()` and return a full `RepoSnapshot`.
- Do not use native `prompt`, `confirm`, or `alert`; use the existing Vue `ConfirmDialog`.
- Do not run full TypeScript/Nuxt/project typecheck after every small edit; use targeted Rust tests during implementation and run one final frontend build/backend check.
- Do not add Monaco, CodeMirror, or other editor dependencies in this first version.
- The diff viewer must not be constrained to the current 390px inspector width. Review Mode must give the diff main area at least a practical desktop width, with horizontal scrolling for long code lines.
- Commit messages, if commits are created while executing this plan, should be Chinese and follow existing style.

---

## File Structure

- Modify `src-tauri/app/src/git/cmd.rs`: add an allowed-exit-code helper that still uses `git_command`.
- Create `src-tauri/app/src/git/ops/diff.rs`: diff structs, unified patch parser, working tree diff, commit file diff, rollback logic, tests.
- Modify `src-tauri/app/src/git/ops/mod.rs`: export `diff`.
- Modify `src-tauri/app/src/commands.rs`: expose `working_file_diff`, `commit_file_diff`, and `rollback_file`.
- Modify `src-tauri/app/src/lib.rs`: register the new Tauri commands.
- Modify `src/types/git.ts`: add diff types and rollback-compatible status aliases.
- Modify `src/api/tauri.ts`: add frontend invoke wrappers.
- Modify `src/stores/repoState.ts`: add diff review state, load actions, rollback action, and close-review behavior.
- Create `src/components/DiffViewer.vue`: wide readonly diff surface.
- Create `src/components/DiffReviewWorkspace.vue`: two-column Review Mode shell that places the source panel on the left and `DiffViewer` in the wide main area.
- Modify `src/App.vue`: conditionally switch from `History + Inspector` to Review Mode spanning both areas.
- Modify `src/components/CommitPanel.vue`: separate checkbox selection from file review clicks; add working diff selection and rollback entry points.
- Modify `src/components/CommitDetailPanel.vue`: file clicks open commit file diff; preserve commit metadata/detail behavior.
- Modify `src/style.css`: add focused `gl-diff-*` utility classes for gutter, hunks, added/deleted/context rows, and review layout.

---

### Task 1: Backend Diff Command Boundary And Parser

**Files:**
- Modify: `src-tauri/app/src/git/cmd.rs`
- Create: `src-tauri/app/src/git/ops/diff.rs`
- Modify: `src-tauri/app/src/git/ops/mod.rs`

**Interfaces:**
- Produces: `run_git_allow_exit_codes(cwd: &Path, args: &[&str], allowed: &[i32]) -> GitResult<(GitOutput, i32)>`
- Produces: `parse_unified_diff(raw: &str, meta: DiffMeta) -> GitResult<FileDiff>`
- Produces serializable structs `FileDiff`, `DiffHunk`, `DiffLine`, `DiffLineKind`

- [ ] **Step 1: Add failing parser tests**

Add tests in `src-tauri/app/src/git/ops/diff.rs` before the implementation:

```rust
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
}
```

- [ ] **Step 2: Run parser tests and verify they fail**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml -p pluck-app git::ops::diff::tests -- --nocapture
```

Expected: compile failure because `diff` module and parser types do not exist.

- [ ] **Step 3: Add allowed exit-code helper through existing command boundary**

In `src-tauri/app/src/git/cmd.rs`, keep `run_git` unchanged and add:

```rust
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
```

This is required for `git diff --no-index`, which exits `1` when it successfully finds differences.

- [ ] **Step 4: Add `diff` module export**

In `src-tauri/app/src/git/ops/mod.rs`, add:

```rust
pub mod diff;
```

- [ ] **Step 5: Implement diff structs and parser**

Create `src-tauri/app/src/git/ops/diff.rs` with these public types and parser:

```rust
use crate::error::{GitError, GitResult};
use crate::git::cmd::{run_git, run_git_allow_exit_codes};
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

pub fn parse_unified_diff(raw: &str, meta: DiffMeta) -> GitResult<FileDiff> {
    let too_large = raw.len() > MAX_DIFF_BYTES;
    let content = if too_large { &raw[..MAX_DIFF_BYTES] } else { raw };
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
```

- [ ] **Step 6: Run parser tests and verify they pass**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml -p pluck-app git::ops::diff::tests -- --nocapture
```

Expected: parser tests pass.

---

### Task 2: Backend Working Tree And Commit Diff APIs

**Files:**
- Modify: `src-tauri/app/src/git/ops/diff.rs`
- Modify: `src-tauri/app/src/commands.rs`
- Modify: `src-tauri/app/src/lib.rs`

**Interfaces:**
- Produces: `working_file_diff(repo: &Path, path: &str, old_path: Option<&str>, status: &str) -> GitResult<FileDiff>`
- Produces: `commit_file_diff(repo: &Path, hash: &str, path: &str, old_path: Option<&str>, status: &str) -> GitResult<FileDiff>`
- Produces Tauri commands `working_file_diff` and `commit_file_diff`

- [ ] **Step 1: Add failing backend integration tests for working and untracked diffs**

Append tests in `src-tauri/app/src/git/ops/diff.rs`:

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
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
}
```

- [ ] **Step 2: Run tests and verify they fail**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml -p pluck-app git::ops::diff::integration_tests -- --nocapture
```

Expected: compile failure for missing `working_file_diff` and `commit_file_diff`.

- [ ] **Step 3: Implement path validation and diff functions**

Add to `src-tauri/app/src/git/ops/diff.rs`:

```rust
fn validate_repo_relative(path: &str) -> GitResult<()> {
    let p = Path::new(path);
    if path.is_empty() || p.is_absolute() || p.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
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
        let (out, _) = run_git_allow_exit_codes(
            repo,
            &["diff", "--no-index", "--no-color", "-U3", "--", "/dev/null", path],
            &[1],
        )
        .await?;
        out.stdout
    } else {
        let mut args = vec!["diff", "--no-ext-diff", "--no-color", "--find-renames", "-U3", "HEAD", "--"];
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
```

- [ ] **Step 4: Expose Tauri commands**

In `src-tauri/app/src/commands.rs`, import:

```rust
use crate::git::ops::diff::{commit_file_diff as git_commit_file_diff, working_file_diff as git_working_file_diff, FileDiff};
```

Add commands:

```rust
#[tauri::command]
pub async fn working_file_diff(
    id: String,
    path: String,
    old_path: Option<String>,
    status: String,
    state: State<'_, AppState>,
) -> GitResult<FileDiff> {
    let sess = state.get(&id).await.ok_or_else(|| GitError::parse("unknown repo id"))?;
    let repo = { sess.lock().await.path.clone() };
    git_working_file_diff(&repo, &path, old_path.as_deref(), &status).await
}

#[tauri::command]
pub async fn commit_file_diff(
    id: String,
    hash: String,
    path: String,
    old_path: Option<String>,
    status: String,
    state: State<'_, AppState>,
) -> GitResult<FileDiff> {
    let sess = state.get(&id).await.ok_or_else(|| GitError::parse("unknown repo id"))?;
    let repo = { sess.lock().await.path.clone() };
    git_commit_file_diff(&repo, &hash, &path, old_path.as_deref(), &status).await
}
```

- [ ] **Step 5: Register commands**

In `src-tauri/app/src/lib.rs`, add to `tauri::generate_handler!`:

```rust
commands::working_file_diff,
commands::commit_file_diff,
```

- [ ] **Step 6: Run backend diff tests**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml -p pluck-app git::ops::diff -- --nocapture
```

Expected: all diff parser/integration tests pass.

---

### Task 3: Backend Full-File Rollback

**Files:**
- Modify: `src-tauri/app/src/git/ops/diff.rs`
- Modify: `src-tauri/app/src/commands.rs`
- Modify: `src-tauri/app/src/lib.rs`

**Interfaces:**
- Produces: `rollback_file(repo: &Path, path: &str, old_path: Option<&str>, status: &str) -> GitResult<()>`
- Produces Tauri command `rollback_file(...) -> GitResult<RepoSnapshot>`

- [ ] **Step 1: Add failing rollback tests**

Append tests in `src-tauri/app/src/git/ops/diff.rs`:

```rust
#[cfg(test)]
mod rollback_tests {
    use super::*;
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
        let out = Command::new("git").current_dir(repo).args(["status", "--porcelain"]).output().unwrap();
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
    async fn rollback_conflicted_is_blocked_for_first_version() {
        let dir = clean_repo();

        let err = rollback_file(dir.path(), "a.txt", None, "conflicted").await.unwrap_err();

        assert!(format!("{err}").contains("conflicted files"));
    }
}
```

- [ ] **Step 2: Run rollback tests and verify they fail**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml -p pluck-app git::ops::diff::rollback_tests -- --nocapture
```

Expected: compile failure for missing `rollback_file`.

- [ ] **Step 3: Implement safe rollback helpers**

Add to `src-tauri/app/src/git/ops/diff.rs`:

```rust
fn repo_path(repo: &Path, path: &str) -> GitResult<PathBuf> {
    validate_repo_relative(path)?;
    Ok(repo.join(path))
}

fn remove_worktree_path(repo: &Path, path: &str) -> GitResult<()> {
    let full = repo_path(repo, path)?;
    if !full.exists() {
        return Ok(());
    }
    let meta = full.metadata()?;
    if meta.is_dir() {
        std::fs::remove_dir_all(full)?;
    } else {
        std::fs::remove_file(full)?;
    }
    Ok(())
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
        "conflicted" => {
            Err(GitError::parse("Rollback for conflicted files is disabled in this version. Resolve or abort the in-progress operation first."))
        }
        "untracked" => {
            remove_worktree_path(repo, path)
        }
        "added" => {
            let _ = run_git(repo, &["rm", "-f", "--cached", "--ignore-unmatch", "--", path]).await;
            remove_worktree_path(repo, path)
        }
        "deleted" | "modified" => {
            run_git(repo, &["restore", "--staged", "--worktree", "--source=HEAD", "--", path]).await?;
            Ok(())
        }
        "renamed" => {
            if let Some(old) = old_path {
                run_git(repo, &["restore", "--staged", "--worktree", "--source=HEAD", "--", old]).await?;
            }
            let _ = run_git(repo, &["rm", "-f", "--cached", "--ignore-unmatch", "--", path]).await;
            remove_worktree_path(repo, path)
        }
        _ => {
            run_git(repo, &["restore", "--staged", "--worktree", "--source=HEAD", "--", path]).await?;
            Ok(())
        }
    }
}
```

- [ ] **Step 4: Add rollback command returning a full snapshot**

In `src-tauri/app/src/commands.rs`, update the diff import:

```rust
use crate::git::ops::diff::{
    commit_file_diff as git_commit_file_diff,
    rollback_file as git_rollback_file,
    working_file_diff as git_working_file_diff,
    FileDiff,
};
```

Add command:

```rust
#[tauri::command]
pub async fn rollback_file(
    id: String,
    path: String,
    old_path: Option<String>,
    status: String,
    state: State<'_, AppState>,
) -> GitResult<RepoSnapshot> {
    let sess = state.get(&id).await.ok_or_else(|| GitError::parse("unknown repo id"))?;
    let repo = { sess.lock().await.path.clone() };
    git_rollback_file(&repo, &path, old_path.as_deref(), &status).await?;
    refresh_session(&sess).await
}
```

- [ ] **Step 5: Register rollback command**

In `src-tauri/app/src/lib.rs`, add:

```rust
commands::rollback_file,
```

- [ ] **Step 6: Run rollback tests**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml -p pluck-app git::ops::diff::rollback_tests -- --nocapture
```

Expected: rollback tests pass.

---

### Task 4: Frontend Types, API, And Review State

**Files:**
- Modify: `src/types/git.ts`
- Modify: `src/api/tauri.ts`
- Modify: `src/stores/repoState.ts`

**Interfaces:**
- Produces TypeScript `FileDiff`, `DiffHunk`, `DiffLine`
- Produces `api.workingFileDiff`, `api.commitFileDiff`, `api.rollbackFile`
- Produces store actions `openWorkingDiff`, `openCommitFileDiff`, `closeReviewMode`, `rollbackCurrentWorkingFile`

- [ ] **Step 1: Add diff types**

In `src/types/git.ts`, add:

```ts
export type DiffKind = "workingTree" | "commit";
export type DiffLineKind = "context" | "added" | "deleted" | "noNewline";

export interface DiffLine {
  kind: DiffLineKind;
  oldNumber: number | null;
  newNumber: number | null;
  content: string;
}

export interface DiffHunk {
  header: string;
  oldStart: number;
  oldLines: number;
  newStart: number;
  newLines: number;
  lines: DiffLine[];
}

export interface FileDiff {
  kind: DiffKind;
  path: string;
  oldPath: string | null;
  status: FileStatus | ChangedFileStatus | "copied" | "typechange";
  binary: boolean;
  tooLarge: boolean;
  additions: number;
  deletions: number;
  hunks: DiffHunk[];
}

export type DiffTarget =
  | { kind: "workingTree"; path: string; oldPath: string | null; status: FileStatus }
  | { kind: "commit"; hash: string; path: string; oldPath: string | null; status: ChangedFileStatus };
```

- [ ] **Step 2: Add API wrappers**

In `src/api/tauri.ts`, update imports to include `FileDiff`, `FileStatus`, and `ChangedFileStatus`, then add:

```ts
  workingFileDiff: (id: string, path: string, oldPath: string | null, status: FileStatus) =>
    invoke<FileDiff>("working_file_diff", { id, path, oldPath, status }),
  commitFileDiff: (id: string, hash: string, path: string, oldPath: string | null, status: ChangedFileStatus) =>
    invoke<FileDiff>("commit_file_diff", { id, hash, path, oldPath, status }),
  rollbackFile: (id: string, path: string, oldPath: string | null, status: FileStatus) =>
    invoke<RepoSnapshot>("rollback_file", { id, path, oldPath, status }),
```

- [ ] **Step 3: Add review state to store**

In `src/stores/repoState.ts`, update imports:

```ts
import type { RepoSnapshot, CommitDetail, Commit, WorkingFile, ChangedFile, FileDiff, DiffTarget } from "../types/git";
```

Add refs near existing selected commit state:

```ts
  const diffTarget = ref<DiffTarget | null>(null);
  const selectedDiff = ref<FileDiff | null>(null);
  const loadingDiff = ref(false);
  const diffError = ref<string | null>(null);
  let diffRequestId = 0;
```

Add actions:

```ts
  function closeReviewMode() {
    diffRequestId++;
    diffTarget.value = null;
    selectedDiff.value = null;
    loadingDiff.value = false;
    diffError.value = null;
  }

  async function openWorkingDiff(repoId: string, file: WorkingFile) {
    const requestId = ++diffRequestId;
    diffTarget.value = { kind: "workingTree", path: file.path, oldPath: file.oldPath, status: file.status };
    selectedDiff.value = null;
    diffError.value = null;
    loadingDiff.value = true;
    try {
      const diff = await api.workingFileDiff(repoId, file.path, file.oldPath, file.status);
      if (diffRequestId !== requestId) return;
      selectedDiff.value = diff;
    } catch (e: any) {
      if (diffRequestId === requestId) diffError.value = formatErr(e);
    } finally {
      if (diffRequestId === requestId) loadingDiff.value = false;
    }
  }

  async function openCommitFileDiff(repoId: string, commit: CommitDetail, file: ChangedFile) {
    const requestId = ++diffRequestId;
    diffTarget.value = { kind: "commit", hash: commit.hash, path: file.path, oldPath: file.oldPath, status: file.status };
    selectedDiff.value = null;
    diffError.value = null;
    loadingDiff.value = true;
    try {
      const diff = await api.commitFileDiff(repoId, commit.hash, file.path, file.oldPath, file.status);
      if (diffRequestId !== requestId) return;
      selectedDiff.value = diff;
    } catch (e: any) {
      if (diffRequestId === requestId) diffError.value = formatErr(e);
    } finally {
      if (diffRequestId === requestId) loadingDiff.value = false;
    }
  }

  async function rollbackCurrentWorkingFile(repoId: string) {
    const target = diffTarget.value;
    if (!target || target.kind !== "workingTree") return;
    const next = await api.rollbackFile(repoId, target.path, target.oldPath, target.status);
    snapshot.value = next;
    closeReviewMode();
  }
```

Call `closeReviewMode()` inside `clearRepoView()`.

- [ ] **Step 4: Export store state and actions**

Add to the returned object:

```ts
    diffTarget, selectedDiff, loadingDiff, diffError,
    openWorkingDiff, openCommitFileDiff, closeReviewMode, rollbackCurrentWorkingFile,
```

- [ ] **Step 5: Run lightweight TypeScript syntax check through build at final only**

Do not run `pnpm build` yet. Continue to component work first.

---

### Task 5: Wide Readonly Diff Viewer UI

**Files:**
- Create: `src/components/DiffViewer.vue`
- Modify: `src/style.css`

**Interfaces:**
- Consumes store state `selectedDiff`, `loadingDiff`, `diffError`, `diffTarget`
- Emits `back` and `rollback` to parent

- [ ] **Step 1: Add diff CSS utilities**

In `src/style.css` inside `@layer components`, add:

```css
  .gl-diff-shell {
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
    height: 100%;
    background: var(--bg);
  }

  .gl-diff-toolbar {
    min-height: 44px;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    border-bottom: 1px solid var(--border-soft);
    background: color-mix(in srgb, var(--panel) 88%, var(--bg));
  }

  .gl-diff-scroll {
    flex: 1;
    min-height: 0;
    overflow: auto;
    background: color-mix(in srgb, var(--bg) 92%, var(--panel));
  }

  .gl-diff-table {
    width: max-content;
    min-width: 100%;
    border-collapse: collapse;
    font-family: var(--mono);
    font-size: 12.5px;
    line-height: 1.55;
  }

  .gl-diff-row td {
    white-space: pre;
    vertical-align: top;
  }

  .gl-diff-line-no {
    width: 54px;
    min-width: 54px;
    padding: 0 8px;
    text-align: right;
    color: var(--fg-3);
    border-right: 1px solid var(--border-soft);
    background: color-mix(in srgb, var(--panel) 72%, var(--bg));
    user-select: none;
    -webkit-user-select: none;
  }

  .gl-diff-mark {
    width: 24px;
    min-width: 24px;
    padding: 0 6px;
    color: var(--fg-3);
    user-select: none;
    -webkit-user-select: none;
  }

  .gl-diff-code {
    padding: 0 14px 0 8px;
    color: var(--fg-2);
  }

  .gl-diff-row.is-added { background: color-mix(in srgb, var(--success-soft) 72%, transparent); }
  .gl-diff-row.is-added .gl-diff-code,
  .gl-diff-row.is-added .gl-diff-mark { color: var(--success); }
  .gl-diff-row.is-deleted { background: color-mix(in srgb, var(--danger-soft) 74%, transparent); }
  .gl-diff-row.is-deleted .gl-diff-code,
  .gl-diff-row.is-deleted .gl-diff-mark { color: var(--danger); }

  .gl-diff-hunk {
    position: sticky;
    top: 0;
    z-index: 1;
    background: color-mix(in srgb, var(--info-soft) 48%, var(--panel));
    color: var(--info);
    font-family: var(--mono);
    font-size: 12px;
  }
```

- [ ] **Step 2: Create `DiffViewer.vue`**

Create `src/components/DiffViewer.vue`:

```vue
<script setup lang="ts">
import { computed, ref } from "vue";
import { ArrowLeft, RotateCcw, WrapText } from "lucide-vue-next";
import { useRepoStateStore } from "../stores/repoState";
import type { DiffLine } from "../types/git";

const emit = defineEmits<{
  back: [];
  rollback: [];
}>();

const state = useRepoStateStore();
const wrap = ref(false);
const diff = computed(() => state.selectedDiff);
const canRollback = computed(() => state.diffTarget?.kind === "workingTree" && diff.value?.status !== "conflicted");

function rowClass(line: DiffLine) {
  return {
    "is-added": line.kind === "added",
    "is-deleted": line.kind === "deleted",
  };
}

function mark(line: DiffLine) {
  if (line.kind === "added") return "+";
  if (line.kind === "deleted") return "-";
  if (line.kind === "noNewline") return "\\";
  return "";
}
</script>

<template>
  <section class="gl-diff-shell">
    <div class="gl-diff-toolbar">
      <button class="gl-command-btn h-7 px-2" title="Back to History" @click="emit('back')">
        <ArrowLeft :size="13" />
        Back
      </button>
      <template v-if="diff">
        <span class="gl-badge">{{ diff.status }}</span>
        <span class="min-w-0 flex-1 truncate gl-mono text-[12px]" :title="diff.oldPath ? `${diff.oldPath} -> ${diff.path}` : diff.path">
          {{ diff.oldPath ? `${diff.oldPath} -> ${diff.path}` : diff.path }}
        </span>
        <span class="gl-badge" style="color: var(--success)">+{{ diff.additions }}</span>
        <span class="gl-badge" style="color: var(--danger)">-{{ diff.deletions }}</span>
      </template>
      <div v-else class="flex-1" />
      <button class="gl-command-btn h-7 px-2" :class="{ 'gl-btn-primary': wrap }" title="Toggle line wrap" @click="wrap = !wrap">
        <WrapText :size="13" />
        Wrap
      </button>
      <button v-if="state.diffTarget?.kind === 'workingTree'"
              class="gl-command-btn gl-btn-danger h-7 px-2"
              :disabled="!canRollback"
              title="Rollback this file"
              @click="emit('rollback')">
        <RotateCcw :size="13" />
        Rollback
      </button>
    </div>

    <div v-if="state.loadingDiff" class="gl-empty">
      <span class="gl-spinner" />
      <span>Loading diff...</span>
    </div>
    <div v-else-if="state.diffError" class="gl-empty" style="color: var(--danger)">
      <span class="text-[13px]">Diff failed</span>
      <span class="text-[12px]">{{ state.diffError }}</span>
    </div>
    <div v-else-if="!diff" class="gl-empty">
      <span class="text-[13px]">Select a file to review changes</span>
    </div>
    <div v-else-if="diff.binary" class="gl-empty">
      <span class="text-[13px]">Binary file diff is not available</span>
      <span class="gl-mono text-[12px]">{{ diff.path }}</span>
    </div>
    <div v-else-if="diff.hunks.length === 0" class="gl-empty">
      <span class="text-[13px]">No textual changes</span>
      <span class="gl-mono text-[12px]">{{ diff.path }}</span>
    </div>
    <div v-else class="gl-diff-scroll" :style="wrap ? { whiteSpace: 'pre-wrap' } : null">
      <table class="gl-diff-table">
        <tbody v-for="hunk in diff.hunks" :key="hunk.header">
          <tr>
            <td class="gl-diff-hunk px-3 py-1" colspan="4">{{ hunk.header }}</td>
          </tr>
          <tr v-for="(line, index) in hunk.lines"
              :key="`${hunk.header}:${index}`"
              class="gl-diff-row"
              :class="rowClass(line)">
            <td class="gl-diff-line-no">{{ line.oldNumber ?? "" }}</td>
            <td class="gl-diff-line-no">{{ line.newNumber ?? "" }}</td>
            <td class="gl-diff-mark">{{ mark(line) }}</td>
            <td class="gl-diff-code" :style="wrap ? { whiteSpace: 'pre-wrap', wordBreak: 'break-word' } : null">{{ line.content }}</td>
          </tr>
        </tbody>
      </table>
    </div>
  </section>
</template>
```

- [ ] **Step 3: Verify no card nesting in planned layout**

Ensure `DiffViewer.vue` does not create an extra `gl-panel` inside Review Mode. It should be the main surface within the workspace.

---

### Task 6: Review Mode Layout Spanning History And Inspector

**Files:**
- Create: `src/components/DiffReviewWorkspace.vue`
- Modify: `src/App.vue`

**Interfaces:**
- Consumes `state.diffTarget`
- Produces layout where diff main area is not constrained by `inspectorWidth`

- [ ] **Step 1: Create Review Workspace shell**

Create `src/components/DiffReviewWorkspace.vue`:

```vue
<script setup lang="ts">
import { computed } from "vue";
import { useReposStore } from "../stores/repos";
import { useRepoStateStore } from "../stores/repoState";
import CommitPanel from "./CommitPanel.vue";
import CommitDetailPanel from "./CommitDetailPanel.vue";
import DiffViewer from "./DiffViewer.vue";

const repos = useReposStore();
const state = useRepoStateStore();
const sourceIsCommit = computed(() => state.diffTarget?.kind === "commit");

async function rollback() {
  if (!repos.activeId || state.diffTarget?.kind !== "workingTree") return;
  const target = state.diffTarget;
  const destructive = target.status === "untracked" || target.status === "added" || target.status === "renamed";
  const ok = await state.confirmAction({
    title: "Rollback File",
    message: destructive
      ? `Rollback will remove "${target.path}" from your working tree.`
      : `Rollback will restore "${target.path}" to HEAD.`,
    confirmLabel: "Rollback",
    tone: destructive ? "danger" : "warning",
    confirmText: destructive ? target.path : undefined,
  });
  if (!ok) return;
  try {
    await state.rollbackCurrentWorkingFile(repos.activeId);
    state.pushToast("info", `Rolled back ${target.path}`);
  } catch (e: any) {
    state.pushToast("error", e?.data?.friendly ?? e?.message ?? String(e));
  }
}
</script>

<template>
  <div class="h-full min-h-0 min-w-0 grid" style="grid-template-columns: minmax(300px, 360px) minmax(620px, 1fr)">
    <aside class="min-h-0 overflow-hidden" style="border-right: 1px solid var(--border-soft); background: var(--panel)">
      <CommitDetailPanel v-if="sourceIsCommit" review-mode />
      <CommitPanel v-else review-mode />
    </aside>
    <DiffViewer @back="state.closeReviewMode()" @rollback="rollback" />
  </div>
</template>
```

- [ ] **Step 2: Modify App grid columns for Review Mode**

In `src/App.vue`, import:

```ts
import DiffReviewWorkspace from "./components/DiffReviewWorkspace.vue";
```

Replace the existing `gridCols` computed with:

```ts
const reviewMode = computed(() => state.diffTarget !== null);
const gridCols = computed(() =>
  reviewMode.value
    ? `${sideWidth.value}px 6px minmax(920px, 1fr)`
    : `${sideWidth.value}px 6px minmax(380px, 1fr) 6px ${inspectorWidth.value}px`
);
```

- [ ] **Step 3: Modify App template to span Review Mode**

Replace the middle/right panel block with:

```vue
        <template v-if="reviewMode">
          <div class="gl-panel overflow-hidden min-h-0 min-w-0">
            <DiffReviewWorkspace />
          </div>
        </template>
        <template v-else>
          <div class="gl-panel overflow-auto min-h-0 min-w-0">
            <LogPanel />
          </div>
          <div class="cursor-col-resize gl-splitter flex justify-center"
               @mousedown="startInspectorDrag"
               @dblclick="inspectorWidth = 390"
               title="Drag to resize inspector · double-click to reset">
            <div class="gl-splitter-line" />
          </div>
          <div class="gl-panel overflow-auto min-h-0 min-w-0">
            <CommitDetailPanel v-if="state.selectedCommit" />
            <CommitPanel v-else />
          </div>
        </template>
```

- [ ] **Step 4: Update Escape behavior**

In `src/App.vue` `onKey`, handle review mode before selected commit:

```ts
  if (e.key === "Escape" && state.diffTarget) {
    const tag = (document.activeElement as HTMLElement | null)?.tagName;
    if (tag !== "INPUT" && tag !== "TEXTAREA") {
      e.preventDefault();
      state.closeReviewMode();
      return;
    }
  }
```

- [ ] **Step 5: Manual layout check**

Run later with the dev server and verify a selected file gives this structure:

```text
RepoSwitcher | Branches | Review Workspace
                      source sidebar | wide DiffViewer
```

The diff main area must remain useful when the app window is around 1200px wide.

---

### Task 7: Wire File Selection And Rollback Entry Points

**Files:**
- Modify: `src/components/CommitPanel.vue`
- Modify: `src/components/CommitDetailPanel.vue`

**Interfaces:**
- Consumes store actions from Task 4
- Produces click-to-review behavior in working tree and commit detail file lists

- [ ] **Step 1: Add `reviewMode` prop to `CommitPanel.vue`**

Near the top of `src/components/CommitPanel.vue`:

```ts
defineProps<{ reviewMode?: boolean }>();
```

Add:

```ts
async function openDiff(f: WorkingFile) {
  if (!repos.activeId) return;
  await state.openWorkingDiff(repos.activeId, f);
}

function isDiffSelected(f: WorkingFile) {
  const target = state.diffTarget;
  return target?.kind === "workingTree" && target.path === f.path;
}
```

Update type import:

```ts
import type { WorkingFile } from "../types/git";
```

- [ ] **Step 2: Separate checkbox from row click**

Change file rows so the checkbox toggles commit inclusion and the row opens diff:

```vue
      <li v-for="f in files" :key="f.path"
          class="gl-row group"
          :class="{ 'is-selected': isDiffSelected(f) }"
          @click="openDiff(f)">
        <input type="checkbox" :checked="selected[f.path]" @click.stop
               @change="selected[f.path] = !selected[f.path]"
               class="w-3.5 h-3.5 rounded gl-checkbox" />
        ...
      </li>
```

This preserves commit selection while making file review the primary row action.

- [ ] **Step 3: Add `reviewMode` prop to `CommitDetailPanel.vue`**

Near the top:

```ts
defineProps<{ reviewMode?: boolean }>();
```

Add:

```ts
async function openCommitDiff(file: ChangedFile) {
  if (!repos.activeId || !detail.value) return;
  await state.openCommitFileDiff(repos.activeId, detail.value, file);
}

function isDiffSelected(file: ChangedFile) {
  const target = state.diffTarget;
  return target?.kind === "commit" && target.path === file.path;
}
```

Add import:

```ts
import { useReposStore } from "../stores/repos";
```

Initialize:

```ts
const repos = useReposStore();
```

- [ ] **Step 4: Make commit detail file rows clickable**

Update file rows:

```vue
        <li v-else
            :title="entry.file.oldPath ? `${entry.file.oldPath} -> ${entry.file.path}` : entry.file.path"
            class="gl-row group"
            :class="{ 'is-selected': isDiffSelected(entry.file) }"
            @click="openCommitDiff(entry.file)">
```

Keep folder rows unchanged.

- [ ] **Step 5: Preserve close behavior in Review Mode**

In `CommitDetailPanel.vue`, if `reviewMode` is true, the Close button should close Review Mode first:

```vue
      <button class="gl-command-btn h-7 px-2"
              @click="reviewMode ? state.closeReviewMode() : state.clearSelectedCommit()"
              title="Back">
```

Use the existing `X` icon and label.

---

### Task 8: Verification, Polish, And Final Checks

**Files:**
- Verify all files changed in previous tasks.
- Optionally update `CHANGELOG.md` only if the user asks to prepare a release; do not update it for ordinary feature implementation.

**Interfaces:**
- Consumes all previous tasks.
- Produces a locally verified feature ready for user testing.

- [ ] **Step 1: Run backend targeted tests**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml -p pluck-app git::ops::diff -- --nocapture
```

Expected: all diff tests pass.

- [ ] **Step 2: Run backend check once**

Run:

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: backend workspace checks successfully.

- [ ] **Step 3: Run frontend build once**

Run:

```bash
pnpm build
```

Expected: `vue-tsc --noEmit` and `vite build` pass.

- [ ] **Step 4: Start dev server for manual UI validation**

Run:

```bash
pnpm tauri dev
```

Expected: the Tauri app opens.

Manual scenarios:

- Modified tracked file: click file in `Changes`, Review Mode opens, diff occupies wide area, Rollback restores the file after confirmation.
- Untracked file: click file in `Changes`, diff shows full added content, Rollback danger confirmation requires the path text and removes the file.
- Commit detail file: select a commit, click a changed file, Review Mode opens with commit diff and no rollback action.
- Esc in Review Mode returns to `History + Inspector`.
- Long code lines do not compress the layout; horizontal scrolling works when Wrap is off.
- Wrap toggle makes long lines readable without changing layout width.
- Binary file displays a calm empty state instead of broken patch rows.

- [ ] **Step 5: Inspect final diff**

Run:

```bash
git diff --stat
git diff -- src-tauri/app/src/git src-tauri/app/src/commands.rs src-tauri/app/src/lib.rs src src/style.css
```

Expected:

- No unrelated refactors.
- No native browser dialogs.
- No new editor dependency.
- Review Mode grid spans center and inspector columns.
- Rollback command returns a full `RepoSnapshot`.

- [ ] **Step 6: Commit if requested**

Only if the user asks for a commit:

```bash
git add src-tauri/app/src/git/cmd.rs src-tauri/app/src/git/ops/diff.rs src-tauri/app/src/git/ops/mod.rs src-tauri/app/src/commands.rs src-tauri/app/src/lib.rs src/types/git.ts src/api/tauri.ts src/stores/repoState.ts src/components/DiffViewer.vue src/components/DiffReviewWorkspace.vue src/components/CommitPanel.vue src/components/CommitDetailPanel.vue src/App.vue src/style.css
git commit -m "feat(diff): 添加只读 diff 和文件回滚"
```

Do not push unless the user explicitly asks.

---

## Self-Review

**Spec coverage:** The plan covers readonly working tree diff, readonly commit file diff, wide Review Mode layout, full-file rollback, safe confirmation, binary/large states, and final verification.

**Placeholder scan:** No implementation step relies on an unspecified future placeholder. The plan deliberately excludes hunk rollback, partial commit, staged/local three-way diff, and conflict resolving from this first version.

**Type consistency:** Backend `FileDiff` maps to frontend `FileDiff`; backend `old_path` maps to frontend `oldPath`; `DiffKind::WorkingTree` maps to `"workingTree"`; rollback accepts only working-tree targets and returns `RepoSnapshot`.


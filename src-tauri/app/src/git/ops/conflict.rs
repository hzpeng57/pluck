use crate::error::{GitError, GitResult};
use crate::git::cmd::{run_git, run_git_bytes, run_git_bytes_with_stdin};
use crate::git::path::validate_repo_relative;
use serde::Serialize;
use std::{collections::BTreeMap, path::Path};

const MAX_CONFLICT_BYTES: usize = 1_500_000;

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConflictStage {
    pub mode: String,
    pub oid: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConflictFile {
    pub path: String,
    pub base: Option<ConflictStage>,
    pub stage2: Option<ConflictStage>,
    pub stage3: Option<ConflictStage>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConflictBlob {
    pub stage: ConflictStage,
    pub content: Option<String>,
    pub binary: bool,
    pub too_large: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConflictFileDetail {
    pub path: String,
    pub base: Option<ConflictBlob>,
    pub stage2: Option<ConflictBlob>,
    pub stage3: Option<ConflictBlob>,
}

pub async fn list_conflicts(repo: &Path) -> GitResult<Vec<ConflictFile>> {
    let output = run_git_bytes(repo, &["ls-files", "-u", "-z"]).await?;
    parse_unmerged_index(&output.stdout)
}

pub async fn conflict_detail(repo: &Path, path: &str) -> GitResult<ConflictFileDetail> {
    let conflict = unresolved_conflict(repo, path).await?;
    let base = load_stage(repo, path, 1, conflict.base).await?;
    let stage2 = load_stage(repo, path, 2, conflict.stage2).await?;
    let stage3 = load_stage(repo, path, 3, conflict.stage3).await?;

    Ok(ConflictFileDetail {
        path: path.to_string(),
        base,
        stage2,
        stage3,
    })
}

pub async fn resolve_conflict_text(repo: &Path, path: &str, content: &str) -> GitResult<()> {
    let conflict = unresolved_conflict(repo, path).await?;
    let mode = conflict
        .stage2
        .as_ref()
        .or(conflict.stage3.as_ref())
        .map(|stage| stage.mode.as_str())
        .ok_or_else(|| GitError::parse(format!("conflict has no selectable stage: {path}")))?;
    let output = run_git_bytes_with_stdin(repo, &["hash-object", "-w", "--stdin"], content.as_bytes()).await?;
    let oid = parse_hash_object_oid(&output.stdout)?;
    let cache_info = format!("{mode},{oid},{path}");

    run_git(repo, &["update-index", "--add", "--cacheinfo", &cache_info]).await?;
    run_git(repo, &["checkout-index", "--force", "--", path]).await?;
    Ok(())
}

pub async fn take_conflict_stage(repo: &Path, path: &str, stage: u8) -> GitResult<()> {
    let conflict = unresolved_conflict(repo, path).await?;
    let flag = match stage {
        2 if conflict.stage2.is_some() => "--ours",
        3 if conflict.stage3.is_some() => "--theirs",
        2 | 3 => return Err(GitError::parse(format!("conflict stage {stage} is absent: {path}"))),
        _ => return Err(GitError::parse("conflict stage must be 2 or 3")),
    };

    run_git(repo, &["checkout", flag, "--", path]).await?;
    run_git(repo, &["add", "--", path]).await?;
    Ok(())
}

pub async fn delete_conflict_path(repo: &Path, path: &str) -> GitResult<()> {
    unresolved_conflict(repo, path).await?;
    run_git(repo, &["rm", "-f", "--", path]).await?;
    Ok(())
}

pub async fn ensure_no_unresolved_conflicts(repo: &Path) -> GitResult<()> {
    let conflicts = list_conflicts(repo).await?;
    if !conflicts.is_empty() {
        return Err(GitError::parse(format!(
            "{} unresolved conflict(s). Resolve every conflict before continuing.",
            conflicts.len()
        )));
    }
    Ok(())
}

async fn unresolved_conflict(repo: &Path, path: &str) -> GitResult<ConflictFile> {
    validate_repo_relative(path)?;
    list_conflicts(repo)
        .await?
        .into_iter()
        .find(|conflict| conflict.path == path)
        .ok_or_else(|| GitError::parse(format!("not an unresolved conflict: {path}")))
}

fn parse_unmerged_index(raw: &[u8]) -> GitResult<Vec<ConflictFile>> {
    let mut conflicts = BTreeMap::new();
    for record in raw.split(|byte| *byte == b'\0').filter(|record| !record.is_empty()) {
        let (header, path) = split_record(record)?;
        validate_repo_relative(path)?;
        let mut fields = header.split(' ');
        let mode = fields.next().filter(|value| !value.is_empty());
        let oid = fields.next().filter(|value| !value.is_empty());
        let stage = fields.next().filter(|value| !value.is_empty());
        if fields.next().is_some() {
            return Err(GitError::parse("malformed unmerged index record"));
        }
        let (Some(mode), Some(oid), Some(stage)) = (mode, oid, stage) else {
            return Err(GitError::parse("malformed unmerged index record"));
        };
        if !mode.bytes().all(|byte| matches!(byte, b'0'..=b'7'))
            || !oid.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(GitError::parse("malformed unmerged index record"));
        }
        let stage = stage
            .parse::<u8>()
            .map_err(|_| GitError::parse("malformed unmerged index record"))?;
        let conflict = conflicts.entry(path.to_string()).or_insert_with(|| ConflictFile {
            path: path.to_string(),
            base: None,
            stage2: None,
            stage3: None,
        });
        let value = ConflictStage {
            mode: mode.to_string(),
            oid: oid.to_string(),
        };
        match stage {
            1 => conflict.base = Some(value),
            2 => conflict.stage2 = Some(value),
            3 => conflict.stage3 = Some(value),
            _ => return Err(GitError::parse("malformed unmerged index record")),
        }
    }
    Ok(conflicts.into_values().collect())
}

fn split_record(record: &[u8]) -> GitResult<(&str, &str)> {
    let delimiter = record
        .iter()
        .position(|byte| *byte == b'\t')
        .ok_or_else(|| GitError::parse("malformed unmerged index record"))?;
    let (header, path) = (&record[..delimiter], &record[delimiter + 1..]);
    let header = std::str::from_utf8(header)
        .map_err(|_| GitError::parse("unmerged index metadata is not UTF-8"))?;
    let path = std::str::from_utf8(path)
        .map_err(|_| GitError::parse("unmerged index path is not UTF-8"))?;
    if path.is_empty() {
        return Err(GitError::parse("malformed unmerged index record"));
    }
    Ok((header, path))
}

async fn load_stage(
    repo: &Path,
    path: &str,
    stage_number: u8,
    stage: Option<ConflictStage>,
) -> GitResult<Option<ConflictBlob>> {
    let Some(stage) = stage else {
        return Ok(None);
    };
    let spec = format!(":{stage_number}:{path}");
    let output = run_git_bytes(repo, &["show", &spec]).await?;
    let (content, binary, too_large) = safe_content(output.stdout);
    Ok(Some(ConflictBlob {
        stage,
        content,
        binary,
        too_large,
    }))
}

fn safe_content(bytes: Vec<u8>) -> (Option<String>, bool, bool) {
    if bytes.len() > MAX_CONFLICT_BYTES {
        return (None, false, true);
    }
    if bytes.contains(&b'\0') {
        return (None, true, false);
    }
    match String::from_utf8(bytes) {
        Ok(content) => (Some(content), false, false),
        Err(_) => (None, true, false),
    }
}

fn parse_hash_object_oid(raw: &[u8]) -> GitResult<&str> {
    let raw = raw.strip_suffix(b"\n").unwrap_or(raw);
    let oid = std::str::from_utf8(raw).map_err(|_| GitError::parse("invalid hash-object OID"))?;
    if oid.is_empty() || !oid.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(GitError::parse("invalid hash-object OID"));
    }
    Ok(oid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{path::Path, process::Command};
    use tempfile::tempdir;

    fn git(repo: &Path, args: &[&str]) {
        let output = Command::new("git").current_dir(repo).args(args).output().unwrap();
        assert!(
            output.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn git_fails(repo: &Path, args: &[&str]) {
        let output = Command::new("git").current_dir(repo).args(args).output().unwrap();
        assert!(
            !output.status.success(),
            "git {args:?} unexpectedly succeeded"
        );
    }

    fn merge_conflict(main: &[u8], feature: &[u8]) -> tempfile::TempDir {
        let repo = tempdir().unwrap();
        git(repo.path(), &["init", "-b", "main"]);
        git(repo.path(), &["config", "user.email", "t@t.t"]);
        git(repo.path(), &["config", "user.name", "t"]);
        std::fs::write(repo.path().join("conflict.txt"), b"base\n").unwrap();
        git(repo.path(), &["add", "conflict.txt"]);
        git(repo.path(), &["commit", "-m", "base"]);

        git(repo.path(), &["switch", "-c", "feature"]);
        std::fs::write(repo.path().join("conflict.txt"), feature).unwrap();
        git(repo.path(), &["commit", "-am", "feature"]);

        git(repo.path(), &["switch", "main"]);
        std::fs::write(repo.path().join("conflict.txt"), main).unwrap();
        git(repo.path(), &["commit", "-am", "main"]);
        git_fails(repo.path(), &["merge", "feature"]);
        repo
    }

    fn text_conflict() -> tempfile::TempDir {
        merge_conflict(b"main\n", b"feature\n")
    }

    #[tokio::test]
    async fn lists_conflicts_and_loads_lazy_stage_content() {
        let repo = text_conflict();

        let files = list_conflicts(repo.path()).await.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "conflict.txt");
        assert!(files[0].base.is_some());
        assert!(files[0].stage2.is_some());
        assert!(files[0].stage3.is_some());

        let detail = conflict_detail(repo.path(), "conflict.txt").await.unwrap();
        assert_eq!(detail.stage2.unwrap().content.as_deref(), Some("main\n"));
        assert_eq!(detail.stage3.unwrap().content.as_deref(), Some("feature\n"));
    }

    #[tokio::test]
    async fn rejects_unsafe_conflict_paths() {
        let repo = text_conflict();
        let error = conflict_detail(repo.path(), "../outside").await.unwrap_err();
        assert!(format!("{error}").contains("unsafe repository path"));
    }

    #[tokio::test]
    async fn takes_stage_two_or_three_and_marks_the_conflict_resolved() {
        let repo = text_conflict();
        take_conflict_stage(repo.path(), "conflict.txt", 2).await.unwrap();
        assert!(list_conflicts(repo.path()).await.unwrap().is_empty());
        assert_eq!(
            run_git_bytes(repo.path(), &["show", ":0:conflict.txt"])
                .await
                .unwrap()
                .stdout,
            b"main\n"
        );

        let repo = text_conflict();
        take_conflict_stage(repo.path(), "conflict.txt", 3).await.unwrap();
        assert!(list_conflicts(repo.path()).await.unwrap().is_empty());
        assert_eq!(
            run_git_bytes(repo.path(), &["show", ":0:conflict.txt"])
                .await
                .unwrap()
                .stdout,
            b"feature\n"
        );
    }

    #[tokio::test]
    async fn resolves_conflict_text_through_the_index() {
        let repo = text_conflict();
        resolve_conflict_text(repo.path(), "conflict.txt", "resolved\n")
            .await
            .unwrap();

        assert!(list_conflicts(repo.path()).await.unwrap().is_empty());
        assert_eq!(
            run_git_bytes(repo.path(), &["show", ":0:conflict.txt"])
                .await
                .unwrap()
                .stdout,
            b"resolved\n"
        );
    }

    #[tokio::test]
    async fn deletes_conflicted_path_through_git() {
        let repo = text_conflict();
        delete_conflict_path(repo.path(), "conflict.txt").await.unwrap();

        assert!(list_conflicts(repo.path()).await.unwrap().is_empty());
        assert!(!repo.path().join("conflict.txt").exists());
    }

    #[tokio::test]
    async fn reports_nul_containing_stage_content_as_binary() {
        let repo = merge_conflict(b"main\n", b"feature\0binary\n");
        let detail = conflict_detail(repo.path(), "conflict.txt").await.unwrap();
        let stage3 = detail.stage3.unwrap();

        assert_eq!(stage3.content, None);
        assert!(stage3.binary);
        assert!(!stage3.too_large);
    }

    #[tokio::test]
    async fn reports_large_stage_content_without_loading_it_into_the_webview() {
        let large = vec![b'x'; MAX_CONFLICT_BYTES + 1];
        let repo = merge_conflict(b"main\n", &large);
        let detail = conflict_detail(repo.path(), "conflict.txt").await.unwrap();
        let stage3 = detail.stage3.unwrap();

        assert_eq!(stage3.content, None);
        assert!(!stage3.binary);
        assert!(stage3.too_large);
    }

    #[tokio::test]
    async fn unresolved_guard_reports_the_conflict_count() {
        let repo = text_conflict();
        let error = ensure_no_unresolved_conflicts(repo.path()).await.unwrap_err();

        assert!(format!("{error}").contains("1 unresolved conflict(s)"));
    }

    #[test]
    fn parses_nul_delimited_records_without_splitting_newline_paths() {
        let parsed = parse_unmerged_index(
            b"100644 1111111111111111111111111111111111111111 2\talpha\nbeta\0",
        )
        .unwrap();

        assert_eq!(parsed[0].path, "alpha\nbeta");
    }
}

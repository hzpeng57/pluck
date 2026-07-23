use crate::git::git_dir;
use serde::Serialize;
use std::path::Path;
use tokio::fs;

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum GitOp {
    Merging { from: String },
    Rebasing { onto: String, head: String },
    CherryPicking,
    Reverting,
}

pub async fn detect_in_progress(repo: &Path) -> Option<GitOp> {
    let gd = git_dir(repo);
    if gd.join("rebase-merge").is_dir() || gd.join("rebase-apply").is_dir() {
        let base = if gd.join("rebase-merge").is_dir() { gd.join("rebase-merge") } else { gd.join("rebase-apply") };
        let onto = fs::read_to_string(base.join("onto")).await.unwrap_or_default().trim().to_string();
        let head = fs::read_to_string(base.join("head-name")).await.unwrap_or_default().trim().to_string();
        return Some(GitOp::Rebasing { onto, head });
    }
    if gd.join("MERGE_HEAD").exists() {
        let message = fs::read_to_string(gd.join("MERGE_MSG")).await.unwrap_or_default();
        let from = message.lines().find_map(|l| l.strip_prefix("Merge branch '").and_then(|x| x.split('\'').next())).unwrap_or("").to_string();
        return Some(GitOp::Merging { from });
    }
    if gd.join("CHERRY_PICK_HEAD").exists() {
        return Some(GitOp::CherryPicking);
    }
    if gd.join("REVERT_HEAD").exists() {
        return Some(GitOp::Reverting);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn detects_none_on_clean_repo() {
        let dir = tempdir().unwrap();
        std::fs::create_dir(dir.path().join(".git")).unwrap();
        assert_eq!(detect_in_progress(dir.path()).await, None);
    }

    #[tokio::test]
    async fn detects_merging() {
        let dir = tempdir().unwrap();
        std::fs::create_dir(dir.path().join(".git")).unwrap();
        std::fs::write(dir.path().join(".git/MERGE_HEAD"), "abc123\n").unwrap();
        std::fs::write(dir.path().join(".git/MERGE_MSG"), "Merge branch 'feature/x'\n").unwrap();
        assert_eq!(detect_in_progress(dir.path()).await, Some(GitOp::Merging { from: "feature/x".into() }));
    }

    #[tokio::test]
    async fn ignores_merge_message_without_merge_head() {
        let dir = tempdir().unwrap();
        std::fs::create_dir(dir.path().join(".git")).unwrap();
        std::fs::write(dir.path().join(".git/MERGE_MSG"), "commit message\n").unwrap();
        assert_eq!(detect_in_progress(dir.path()).await, None);
    }

    #[tokio::test]
    async fn detects_rebasing() {
        let dir = tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".git/rebase-merge")).unwrap();
        std::fs::write(dir.path().join(".git/rebase-merge/onto"), "abc123\n").unwrap();
        std::fs::write(dir.path().join(".git/rebase-merge/head-name"), "refs/heads/main\n").unwrap();
        match detect_in_progress(dir.path()).await.unwrap() {
            GitOp::Rebasing { onto, head } => { assert_eq!(onto, "abc123"); assert_eq!(head, "refs/heads/main"); }
            other => panic!("expected Rebasing, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn detects_rebasing_when_merge_message_exists() {
        let dir = tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".git/rebase-merge")).unwrap();
        std::fs::write(dir.path().join(".git/MERGE_MSG"), "rebased commit message\n").unwrap();
        std::fs::write(dir.path().join(".git/rebase-merge/onto"), "abc123\n").unwrap();
        std::fs::write(dir.path().join(".git/rebase-merge/head-name"), "refs/heads/feature\n").unwrap();

        assert_eq!(
            detect_in_progress(dir.path()).await,
            Some(GitOp::Rebasing { onto: "abc123".into(), head: "refs/heads/feature".into() }),
        );
    }

    #[tokio::test]
    async fn detects_rebasing_when_dot_git_points_to_real_gitdir() {
        let dir = tempdir().unwrap();
        let gitdir = dir.path().join("real-gitdir");
        std::fs::create_dir_all(gitdir.join("rebase-merge")).unwrap();
        std::fs::write(
            dir.path().join(".git"),
            format!("gitdir: {}\n", gitdir.display()),
        )
        .unwrap();
        std::fs::write(gitdir.join("rebase-merge/onto"), "abc123\n").unwrap();
        std::fs::write(gitdir.join("rebase-merge/head-name"), "refs/heads/feature\n").unwrap();

        match detect_in_progress(dir.path()).await.unwrap() {
            GitOp::Rebasing { onto, head } => {
                assert_eq!(onto, "abc123");
                assert_eq!(head, "refs/heads/feature");
            }
            other => panic!("expected Rebasing, got {:?}", other),
        }
    }
}

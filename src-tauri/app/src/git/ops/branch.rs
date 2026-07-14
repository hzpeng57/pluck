use crate::error::{GitError, GitResult};
use crate::git::cmd::{run_git, run_git_allow_exit_codes};
use serde::{Deserialize, Serialize};
use std::path::Path;

pub async fn create_branch(repo: &Path, name: &str, from: Option<&str>) -> GitResult<()> {
    if let Some(from) = from {
        run_git(repo, &["checkout", "--no-track", "-b", name, from]).await?;
    } else {
        run_git(repo, &["checkout", "-b", name]).await?;
    }
    Ok(())
}

pub async fn rename_branch(
    repo: &Path,
    old_name: &str,
    new_name: &str,
    unset_upstream: bool,
) -> GitResult<()> {
    run_git(repo, &["branch", "-m", old_name, new_name]).await?;
    if unset_upstream {
        run_git(repo, &["branch", "--unset-upstream", new_name]).await?;
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum BranchDeleteKind {
    Local,
    Remote,
}

pub async fn delete_branch(
    repo: &Path,
    name: &str,
    kind: BranchDeleteKind,
    force: bool,
) -> GitResult<()> {
    match kind {
        BranchDeleteKind::Local => {
            let flag = if force { "-D" } else { "-d" };
            run_git(repo, &["branch", flag, name]).await?;
        }
        BranchDeleteKind::Remote => {
            let (remote, branch) = split_remote_branch_name(name)?;
            run_git(repo, &["push", remote, "--delete", branch]).await?;
            let remote_tracking_ref = format!("refs/remotes/{name}");
            run_git(repo, &["update-ref", "-d", &remote_tracking_ref]).await?;
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeletePrecheck {
    pub exists: bool,
    pub is_current: bool,
    pub is_merged: bool,
    pub upstream: Option<String>,
    pub ahead_of_head: u32,
}

pub async fn delete_precheck(
    repo: &Path,
    name: &str,
    kind: BranchDeleteKind,
) -> GitResult<DeletePrecheck> {
    let head_branch = match run_git(repo, &["symbolic-ref", "--quiet", "--short", "HEAD"]).await {
        Ok(o) => Some(o.stdout.trim().to_string()),
        Err(_) => None,
    };

    let ref_prefix = match kind {
        BranchDeleteKind::Local => "refs/heads",
        BranchDeleteKind::Remote => "refs/remotes",
    };
    let full_ref = format!("{ref_prefix}/{name}");
    let exists = ref_exists(repo, &full_ref).await?;

    if !exists {
        return Ok(DeletePrecheck {
            exists: false,
            is_current: false,
            is_merged: false,
            upstream: None,
            ahead_of_head: 0,
        });
    }

    if kind == BranchDeleteKind::Remote {
        return Ok(DeletePrecheck {
            exists,
            is_current: false,
            is_merged: false,
            upstream: None,
            ahead_of_head: 0,
        });
    }

    let is_current = head_branch.as_deref() == Some(name);

    let upstream = run_git(
        repo,
        &[
            "for-each-ref",
            "--format=%(upstream:short)",
            &format!("refs/heads/{name}"),
        ],
    )
    .await
    .ok()
    .and_then(|o| {
        let s = o.stdout.trim().to_string();
        if s.is_empty() { None } else { Some(s) }
    });

    let ahead_of_head = if is_current {
        0
    } else {
        let head_ref = head_branch.as_deref().unwrap_or("HEAD");
        match run_git(
            repo,
            &["rev-list", "--count", &format!("{head_ref}..{name}")],
        )
        .await
        {
            Ok(o) => o.stdout.trim().parse().unwrap_or(0),
            Err(_) => 0,
        }
    };

    let is_merged = ahead_of_head == 0;

    Ok(DeletePrecheck {
        exists,
        is_current,
        is_merged,
        upstream,
        ahead_of_head,
    })
}

fn split_remote_branch_name(name: &str) -> GitResult<(&str, &str)> {
    let Some((remote, branch)) = name.split_once('/') else {
        return Err(GitError::parse(format!(
            "remote branch name must be in <remote>/<branch> form: {name}"
        )));
    };
    if remote.is_empty() || branch.is_empty() || branch == "HEAD" || branch.ends_with("/HEAD") {
        return Err(GitError::parse(format!(
            "remote branch name must be in <remote>/<branch> form: {name}"
        )));
    }
    Ok((remote, branch))
}

async fn ref_exists(repo: &Path, full_ref: &str) -> GitResult<bool> {
    let (_, code) =
        run_git_allow_exit_codes(repo, &["show-ref", "--verify", "--quiet", full_ref], &[1])
            .await?;
    Ok(code == 0)
}

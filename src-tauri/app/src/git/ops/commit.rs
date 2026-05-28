use crate::error::GitResult;
use crate::git::cmd::run_git;
use std::path::Path;

pub async fn commit_files(repo: &Path, files: &[String], message: &str, skip_hooks: bool) -> GitResult<()> {
    let mut add_args = vec!["add", "--"];
    for f in files { add_args.push(f.as_str()); }
    run_git(repo, &add_args).await?;

    let mut commit_args: Vec<&str> = vec!["commit", "--only", "-m", message];
    if skip_hooks { commit_args.push("-n"); }
    commit_args.push("--");
    for f in files { commit_args.push(f.as_str()); }
    run_git(repo, &commit_args).await?;
    Ok(())
}

pub mod cmd;
pub mod detect;
pub mod ops;
pub mod parse;
pub mod snapshot;

use std::path::{Path, PathBuf};

pub(crate) fn git_dir(repo: &Path) -> PathBuf {
    let dot_git = repo.join(".git");
    let Ok(contents) = std::fs::read_to_string(&dot_git) else {
        return dot_git;
    };
    let Some(path) = contents.trim().strip_prefix("gitdir:") else {
        return dot_git;
    };
    let path = PathBuf::from(path.trim());
    if path.is_absolute() {
        path
    } else {
        repo.join(path)
    }
}

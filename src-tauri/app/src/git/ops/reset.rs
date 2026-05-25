use crate::error::GitResult;
use crate::git::cmd::run_git;
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum ResetMode {
    Soft,
    Mixed,
    Hard,
    Keep,
}

impl ResetMode {
    fn flag(self) -> &'static str {
        match self {
            ResetMode::Soft => "--soft",
            ResetMode::Mixed => "--mixed",
            ResetMode::Hard => "--hard",
            ResetMode::Keep => "--keep",
        }
    }
}

pub async fn reset_to(repo: &Path, hash: &str, mode: ResetMode) -> GitResult<()> {
    run_git(repo, &["reset", mode.flag(), hash]).await?;
    Ok(())
}

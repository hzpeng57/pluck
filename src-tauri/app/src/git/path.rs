use crate::error::{GitError, GitResult};
use std::path::{Component, Path};

pub(crate) fn validate_repo_relative(path: &str) -> GitResult<()> {
    let p = Path::new(path);
    if path.is_empty()
        || path.contains('\0')
        || p.is_absolute()
        || p.components().any(|c| {
            matches!(
                c,
                Component::CurDir | Component::ParentDir | Component::Prefix(_) | Component::RootDir
            ) || matches!(
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

#[cfg(test)]
mod tests {
    use super::validate_repo_relative;

    #[test]
    fn rejects_unsafe_repository_paths() {
        assert!(validate_repo_relative("src/conflict.rs").is_ok());
        assert!(validate_repo_relative("../outside").is_err());
        assert!(validate_repo_relative("/tmp/outside").is_err());
        assert!(validate_repo_relative(".git/config").is_err());
        assert!(validate_repo_relative("a\0b").is_err());
    }
}

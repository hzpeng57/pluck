use crate::error::{GitError, GitResult};
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum FileStatus { Modified, Added, Deleted, Renamed, Untracked, Conflicted }

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkingFile {
    pub path: String,
    pub old_path: Option<String>,
    pub status: FileStatus,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HeadInfo {
    pub branch: Option<String>,
    pub detached_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StatusOutput {
    pub head: HeadInfo,
    pub upstream: Option<String>,
    pub ahead: u32,
    pub behind: u32,
    pub files: Vec<WorkingFile>,
}

pub fn parse_status_porcelain_v2(raw: &str) -> GitResult<StatusOutput> {
    let mut head = HeadInfo { branch: None, detached_at: None };
    let mut upstream = None;
    let mut ahead = 0u32;
    let mut behind = 0u32;
    let mut files = Vec::new();

    for line in raw.lines() {
        if let Some(rest) = line.strip_prefix("# branch.head ") {
            if rest == "(detached)" { head.branch = None } else { head.branch = Some(rest.to_string()) }
        } else if let Some(rest) = line.strip_prefix("# branch.oid ") {
            if head.branch.is_none() { head.detached_at = Some(rest.to_string()) }
        } else if let Some(rest) = line.strip_prefix("# branch.upstream ") {
            upstream = Some(rest.to_string());
        } else if let Some(rest) = line.strip_prefix("# branch.ab +") {
            let mut parts = rest.split(" -");
            ahead = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
            behind = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
        } else if let Some(rest) = line.strip_prefix("1 ") {
            // ordinary changed entry: XY sub mH mI mW hH hI path (8 fields, path at index 7)
            let parts: Vec<&str> = rest.splitn(8, ' ').collect();
            let xy = parts.first().ok_or_else(|| GitError::parse("bad 1 line"))?;
            let path = parts.get(7).ok_or_else(|| GitError::parse("missing path"))?;
            files.push(WorkingFile {
                path: (*path).to_string(),
                old_path: None,
                status: classify(xy),
            });
        } else if let Some(rest) = line.strip_prefix("2 ") {
            // renamed entry: XY sub mH mI mW hH hI Rscore <newPath>\t<oldPath> (9 fields, paths at index 8)
            let parts: Vec<&str> = rest.splitn(9, ' ').collect();
            let xy = parts.first().ok_or_else(|| GitError::parse("bad 2 line"))?;
            let combined = parts.get(8).ok_or_else(|| GitError::parse("missing rename paths"))?;
            let mut np = combined.splitn(2, '\t');
            let new_path = np.next().unwrap_or("").to_string();
            let old_path = np.next().map(|s| s.to_string());
            files.push(WorkingFile { path: new_path, old_path, status: classify(xy) });
        } else if let Some(rest) = line.strip_prefix("u ") {
            let parts: Vec<&str> = rest.splitn(11, ' ').collect();
            let path = parts.get(10).ok_or_else(|| GitError::parse("missing path in u"))?;
            files.push(WorkingFile { path: (*path).to_string(), old_path: None, status: FileStatus::Conflicted });
        } else if let Some(rest) = line.strip_prefix("? ") {
            files.push(WorkingFile { path: rest.to_string(), old_path: None, status: FileStatus::Untracked });
        }
    }

    Ok(StatusOutput { head, upstream, ahead, behind, files })
}

fn classify(xy: &str) -> FileStatus {
    let bytes = xy.as_bytes();
    let x = *bytes.first().unwrap_or(&b'.') as char;
    let y = *bytes.get(1).unwrap_or(&b'.') as char;
    match (x, y) {
        ('R', _) | (_, 'R') => FileStatus::Renamed,
        ('A', _) | (_, 'A') => FileStatus::Added,
        ('D', _) | (_, 'D') => FileStatus::Deleted,
        ('M', _) | (_, 'M') => FileStatus::Modified,
        _ => FileStatus::Modified,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_clean_status() {
        let raw = "# branch.oid abc123\n# branch.head main\n# branch.upstream origin/main\n# branch.ab +0 -0\n";
        let out = parse_status_porcelain_v2(raw).unwrap();
        assert_eq!(out.head.branch.as_deref(), Some("main"));
        assert_eq!(out.upstream.as_deref(), Some("origin/main"));
        assert_eq!(out.ahead, 0);
        assert_eq!(out.behind, 0);
        assert!(out.files.is_empty());
    }

    #[test]
    fn parses_modified_and_untracked() {
        let raw = "# branch.head main\n1 .M N... 100644 100644 100644 a b src/app.ts\n? docs/x.md\n";
        let out = parse_status_porcelain_v2(raw).unwrap();
        assert_eq!(out.files.len(), 2);
        assert_eq!(out.files[0].path, "src/app.ts");
        assert_eq!(out.files[0].status, FileStatus::Modified);
        assert_eq!(out.files[1].status, FileStatus::Untracked);
    }

    #[test]
    fn parses_rename() {
        let raw = "# branch.head main\n2 R. N... 100644 100644 100644 a b R100 new.ts\told.ts\n";
        let out = parse_status_porcelain_v2(raw).unwrap();
        assert_eq!(out.files[0].path, "new.ts");
        assert_eq!(out.files[0].old_path.as_deref(), Some("old.ts"));
        assert_eq!(out.files[0].status, FileStatus::Renamed);
    }

    #[test]
    fn parses_conflict() {
        let raw = "# branch.head main\nu UU N... 100644 100644 100644 100644 a b c d conflict.ts\n";
        let out = parse_status_porcelain_v2(raw).unwrap();
        assert_eq!(out.files[0].status, FileStatus::Conflicted);
        assert_eq!(out.files[0].path, "conflict.ts");
    }

    #[test]
    fn parses_ahead_behind() {
        let raw = "# branch.head main\n# branch.ab +3 -2\n";
        let out = parse_status_porcelain_v2(raw).unwrap();
        assert_eq!(out.ahead, 3);
        assert_eq!(out.behind, 2);
    }
}

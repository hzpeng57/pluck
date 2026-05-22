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
    let mut oid: Option<String> = None;
    let mut upstream = None;
    let mut ahead = 0u32;
    let mut behind = 0u32;
    let mut files = Vec::new();

    for line in raw.lines() {
        if let Some(rest) = line.strip_prefix("# branch.head ") {
            if rest == "(detached)" { head.branch = None } else { head.branch = Some(rest.to_string()) }
        } else if let Some(rest) = line.strip_prefix("# branch.oid ") {
            oid = Some(rest.to_string());
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
            let parts: Vec<&str> = rest.splitn(10, ' ').collect();
            let path = parts.get(9).ok_or_else(|| GitError::parse("missing path in u"))?;
            files.push(WorkingFile { path: (*path).to_string(), old_path: None, status: FileStatus::Conflicted });
        } else if let Some(rest) = line.strip_prefix("? ") {
            files.push(WorkingFile { path: rest.to_string(), old_path: None, status: FileStatus::Untracked });
        }
    }

    if head.branch.is_none() {
        head.detached_at = oid;
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

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum BranchKind { Local, Remote }

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Branch {
    pub name: String,
    pub kind: BranchKind,
    pub upstream: Option<String>,
    pub ahead: u32,
    pub behind: u32,
    pub is_current: bool,
    pub last_commit_short: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct BranchList { pub local: Vec<Branch>, pub remote: Vec<Branch> }

/// Format: <refname>%00<HEAD>%00<upstream>%00<upstream:track>%00<objectname:short>
/// Example invocation:
///   git for-each-ref --format='%(refname)%00%(HEAD)%00%(upstream:short)%00%(upstream:track)%00%(objectname:short)' refs/heads refs/remotes
pub fn parse_branches(raw: &str, current_branch: Option<&str>) -> Vec<Branch> {
    raw.lines().filter_map(|line| parse_branch_line(line, current_branch)).collect()
}

fn parse_branch_line(line: &str, current_branch: Option<&str>) -> Option<Branch> {
    let mut parts = line.split('\u{0000}');
    let refname = parts.next()?;
    let head_marker = parts.next().unwrap_or("");
    let upstream = parts.next().unwrap_or("");
    let track = parts.next().unwrap_or("");
    let short = parts.next().unwrap_or("").to_string();

    let (name, kind) = if let Some(r) = refname.strip_prefix("refs/heads/") {
        (r.to_string(), BranchKind::Local)
    } else if let Some(r) = refname.strip_prefix("refs/remotes/") {
        if r.ends_with("/HEAD") { return None }
        (r.to_string(), BranchKind::Remote)
    } else {
        return None;
    };

    let is_current = head_marker == "*" || current_branch == Some(&name);
    let (ahead, behind) = parse_track(track);

    Some(Branch {
        name, kind,
        upstream: if upstream.is_empty() { None } else { Some(upstream.to_string()) },
        ahead, behind, is_current, last_commit_short: short,
    })
}

fn parse_track(track: &str) -> (u32, u32) {
    // Examples: "" | "[gone]" | "[ahead 2]" | "[behind 1]" | "[ahead 2, behind 1]"
    let s = track.trim_start_matches('[').trim_end_matches(']');
    let mut ahead = 0u32;
    let mut behind = 0u32;
    for chunk in s.split(',') {
        let c = chunk.trim();
        if let Some(n) = c.strip_prefix("ahead ") { ahead = n.parse().unwrap_or(0) }
        else if let Some(n) = c.strip_prefix("behind ") { behind = n.parse().unwrap_or(0) }
    }
    (ahead, behind)
}

pub fn split_branch_list(all: Vec<Branch>) -> BranchList {
    let (local, remote): (Vec<_>, Vec<_>) = all.into_iter().partition(|b| matches!(b.kind, BranchKind::Local));
    BranchList { local, remote }
}

#[cfg(test)]
mod branch_tests {
    use super::*;

    #[test]
    fn parses_local_and_remote() {
        let raw = "refs/heads/main\u{0000}*\u{0000}origin/main\u{0000}[ahead 2]\u{0000}abc1234\n\
                   refs/heads/feat/x\u{0000}\u{0000}\u{0000}\u{0000}def5678\n\
                   refs/remotes/origin/main\u{0000}\u{0000}\u{0000}\u{0000}abc1234\n\
                   refs/remotes/origin/HEAD\u{0000}\u{0000}\u{0000}\u{0000}xxx\n";
        let branches = parse_branches(raw, Some("main"));
        assert_eq!(branches.len(), 3); // HEAD ref filtered
        let main = &branches[0];
        assert_eq!(main.name, "main");
        assert!(main.is_current);
        assert_eq!(main.ahead, 2);
        assert_eq!(main.upstream.as_deref(), Some("origin/main"));
        let split = split_branch_list(branches);
        assert_eq!(split.local.len(), 2);
        assert_eq!(split.remote.len(), 1);
    }

    #[test]
    fn parses_ahead_and_behind() {
        let (a, b) = parse_track("[ahead 3, behind 5]");
        assert_eq!((a, b), (3, 5));
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
        let raw = "# branch.head main\nu UU N... 100644 100644 100644 100644 hhh1 hhh2 hhh3 conflict.ts\n";
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

    #[test]
    fn normal_branch_does_not_set_detached_at() {
        // Real git emits oid BEFORE head
        let raw = "# branch.oid abc123def\n# branch.head main\n# branch.upstream origin/main\n# branch.ab +0 -0\n";
        let out = parse_status_porcelain_v2(raw).unwrap();
        assert_eq!(out.head.branch.as_deref(), Some("main"));
        assert_eq!(out.head.detached_at, None);
    }

    #[test]
    fn detached_head_sets_detached_at() {
        let raw = "# branch.oid abc123def\n# branch.head (detached)\n";
        let out = parse_status_porcelain_v2(raw).unwrap();
        assert_eq!(out.head.branch, None);
        assert_eq!(out.head.detached_at.as_deref(), Some("abc123def"));
    }
}

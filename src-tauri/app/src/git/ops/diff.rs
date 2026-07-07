use crate::error::{GitError, GitResult};
use serde::Serialize;

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

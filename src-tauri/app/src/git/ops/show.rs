use crate::error::{GitError, GitResult};
use crate::git::cmd::run_git;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChangedFile {
    pub status: String,    // "added" | "modified" | "deleted" | "renamed" | "copied" | "typechange"
    pub path: String,      // new path (or only path)
    pub old_path: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CommitDetail {
    pub hash: String,
    pub short: String,
    pub author: String,
    pub email: String,
    pub date_unix: i64,
    pub subject: String,
    pub body: String,
    pub parents: Vec<String>,
    pub refs: Vec<String>,
    pub files: Vec<ChangedFile>,
}

fn status_letter(c: char) -> &'static str {
    match c {
        'A' => "added",
        'M' => "modified",
        'D' => "deleted",
        'R' => "renamed",
        'C' => "copied",
        'T' => "typechange",
        _ => "modified",
    }
}

pub async fn commit_show(repo: &Path, hash: &str) -> GitResult<CommitDetail> {
    // Metadata: %H %h %an %ae %at %s\x00%B\x00%P\x00%D
    let fmt = "%H%n%h%n%an%n%ae%n%at%n%s%n%P%n%D%n%B";
    let meta = run_git(repo, &["show", "-s", &format!("--format={fmt}"), hash]).await?;
    let stdout = meta.stdout;
    let mut lines = stdout.lines();
    let h = lines.next().unwrap_or("").to_string();
    let short = lines.next().unwrap_or("").to_string();
    let author = lines.next().unwrap_or("").to_string();
    let email = lines.next().unwrap_or("").to_string();
    let date_unix: i64 = lines.next().unwrap_or("0").parse().unwrap_or(0);
    let subject = lines.next().unwrap_or("").to_string();
    let parents: Vec<String> = lines.next().unwrap_or("")
        .split_whitespace().map(|s| s.to_string()).collect();
    let refs: Vec<String> = lines.next().unwrap_or("")
        .split(", ").filter(|s| !s.is_empty()).map(|s| s.to_string()).collect();
    let body = lines.collect::<Vec<_>>().join("\n");

    // Changed files: name-status. Use diff-tree -r --no-commit-id; -m gives one diff per parent for merges.
    let files_out = run_git(
        repo,
        &["diff-tree", "--no-commit-id", "--name-status", "-r", "-m", "--first-parent", hash],
    )
    .await?;

    let mut files: Vec<ChangedFile> = Vec::new();
    let mut seen = std::collections::HashSet::<String>::new();
    for line in files_out.stdout.lines() {
        if line.is_empty() { continue; }
        let mut parts = line.split('\t');
        let status_field = parts.next().unwrap_or("");
        let first = parts.next().unwrap_or("").to_string();
        let second = parts.next().map(|s| s.to_string());
        let letter = status_field.chars().next().unwrap_or('M');
        let status = status_letter(letter).to_string();
        let (path, old_path) = match letter {
            'R' | 'C' => {
                let new_path = second.unwrap_or_else(|| first.clone());
                (new_path, Some(first))
            }
            _ => (first, None),
        };
        if seen.insert(path.clone()) {
            files.push(ChangedFile { status, path, old_path });
        }
    }

    if h.is_empty() {
        return Err(GitError::parse(format!("unknown commit: {hash}")));
    }
    Ok(CommitDetail {
        hash: h, short, author, email, date_unix, subject, body, parents, refs, files,
    })
}

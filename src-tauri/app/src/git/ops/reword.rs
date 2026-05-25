use crate::error::{GitError, GitResult};
use crate::git::cmd::run_git;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

/// Amend HEAD's commit message in place.
pub async fn amend_message(repo: &Path, new_message: &str) -> GitResult<()> {
    run_git(repo, &["commit", "--amend", "-m", new_message]).await?;
    Ok(())
}

/// Reword a non-HEAD commit by driving `git rebase -i` with scripted editors.
/// Avoids the bridge entirely: a tiny sed script flips the first `pick` to
/// `reword`, and a cat script replaces the commit message file with `new_message`.
/// On conflict mid-rebase, rebase state remains and the in-progress banner takes over.
pub async fn reword_commit(repo: &Path, hash: &str, new_message: &str) -> GitResult<()> {
    let pid = std::process::id();
    let nonce = format!(
        "{}-{}",
        pid,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    );

    let msg_path = std::env::temp_dir().join(format!("gitlite-msg-{nonce}.txt"));
    let seq_script = std::env::temp_dir().join(format!("gitlite-seq-{nonce}.sh"));
    let editor_script = std::env::temp_dir().join(format!("gitlite-editor-{nonce}.sh"));

    tokio::fs::write(&msg_path, new_message)
        .await
        .map_err(|e| GitError::spawn(format!("write msg: {e}")))?;

    // sequence editor: rewrite the very first "pick <hash>" line to "reword <hash>".
    // The todo file lives at $1.
    let seq_contents = "#!/bin/sh\nawk 'NR==1 && /^pick / { sub(/^pick /, \"reword \") } { print }' \"$1\" > \"$1.tmp\" && mv \"$1.tmp\" \"$1\"\n";
    tokio::fs::write(&seq_script, seq_contents)
        .await
        .map_err(|e| GitError::spawn(format!("write seq: {e}")))?;

    // commit-msg editor: dump our new message into the target file.
    let editor_contents = format!(
        "#!/bin/sh\ncat {msg} > \"$1\"\n",
        msg = shell_escape(msg_path.to_string_lossy().as_ref())
    );
    tokio::fs::write(&editor_script, editor_contents)
        .await
        .map_err(|e| GitError::spawn(format!("write editor: {e}")))?;

    for p in [&seq_script, &editor_script] {
        let mut perms = tokio::fs::metadata(p)
            .await
            .map_err(|e| GitError::spawn(format!("stat: {e}")))?
            .permissions();
        perms.set_mode(0o755);
        tokio::fs::set_permissions(p, perms)
            .await
            .map_err(|e| GitError::spawn(format!("chmod: {e}")))?;
    }

    let cleanup = || {
        let _ = std::fs::remove_file(&msg_path);
        let _ = std::fs::remove_file(&seq_script);
        let _ = std::fs::remove_file(&editor_script);
    };

    let output = tokio::process::Command::new("git")
        .current_dir(repo)
        .env("GIT_SEQUENCE_EDITOR", &seq_script)
        .env("GIT_EDITOR", &editor_script)
        .args(["rebase", "-i", &format!("{hash}^")])
        .output()
        .await
        .map_err(|e| GitError::spawn(e.to_string()));

    cleanup();

    let output = output?;
    if output.status.success() {
        return Ok(());
    }
    if repo.join(".git/rebase-merge").exists() || repo.join(".git/rebase-apply").exists() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(GitError::from_stderr(output.status.code().unwrap_or(-1), &stderr))
}

fn shell_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('\'');
    for c in s.chars() {
        if c == '\'' {
            out.push_str("'\\''");
        } else {
            out.push(c);
        }
    }
    out.push('\'');
    out
}

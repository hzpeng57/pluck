use crate::error::GitResult;
use crate::git::snapshot::{build_snapshot, RepoSnapshot};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

const DEBOUNCE_MS: u64 = 200;

pub struct RepoSession {
    pub id: String,
    pub path: PathBuf,
    pub log_branch: Option<String>,
    pub last_snapshot: Option<RepoSnapshot>,
    pub last_refresh: Instant,
    refreshing: bool,
}

#[derive(Default)]
pub struct AppState {
    pub sessions: Mutex<HashMap<String, Arc<Mutex<RepoSession>>>>,
}

impl AppState {
    pub async fn add(&self, id: String, path: PathBuf) -> Arc<Mutex<RepoSession>> {
        let sess = Arc::new(Mutex::new(RepoSession {
            id: id.clone(), path, log_branch: None,
            last_snapshot: None,
            last_refresh: Instant::now() - Duration::from_secs(3600),
            refreshing: false,
        }));
        self.sessions.lock().await.insert(id, sess.clone());
        sess
    }

    pub async fn get(&self, id: &str) -> Option<Arc<Mutex<RepoSession>>> {
        self.sessions.lock().await.get(id).cloned()
    }
}

pub async fn refresh_session(sess: &Arc<Mutex<RepoSession>>) -> GitResult<RepoSnapshot> {
    refresh_session_inner(sess, false).await
}

pub async fn refresh_session_force(sess: &Arc<Mutex<RepoSession>>) -> GitResult<RepoSnapshot> {
    refresh_session_inner(sess, true).await
}

async fn refresh_session_inner(
    sess: &Arc<Mutex<RepoSession>>,
    force: bool,
) -> GitResult<RepoSnapshot> {
    {
        let mut s = sess.lock().await;
        if s.refreshing { /* fall through; second caller will see new snapshot */ }
        else if !force && s.last_refresh.elapsed() < Duration::from_millis(DEBOUNCE_MS) {
            if let Some(snap) = &s.last_snapshot { return Ok(snap.clone()) }
        }
        s.refreshing = true;
    }
    let (path, log_branch) = { let s = sess.lock().await; (s.path.clone(), s.log_branch.clone()) };
    let snap = build_snapshot(&path, log_branch.as_deref()).await?;
    let mut s = sess.lock().await;
    s.last_snapshot = Some(snap.clone());
    s.last_refresh = Instant::now();
    s.refreshing = false;
    Ok(snap)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use tempfile::tempdir;

    #[tokio::test]
    async fn refresh_caches_within_debounce() {
        let dir = tempdir().unwrap();
        Command::new("git").current_dir(dir.path()).args(["init", "-b", "main"]).status().unwrap();
        Command::new("git").current_dir(dir.path()).args(["config", "user.email", "t@t.t"]).status().unwrap();
        Command::new("git").current_dir(dir.path()).args(["config", "user.name", "t"]).status().unwrap();
        Command::new("git").current_dir(dir.path()).args(["commit", "--allow-empty", "-m", "init"]).status().unwrap();

        let state = AppState::default();
        let sess = state.add("r1".into(), dir.path().into()).await;

        let snap1 = refresh_session(&sess).await.unwrap();
        let snap2 = refresh_session(&sess).await.unwrap();
        assert_eq!(snap1.head.branch, snap2.head.branch);
    }

    #[tokio::test]
    async fn forced_refresh_bypasses_debounce_cache() {
        let dir = tempdir().unwrap();
        Command::new("git").current_dir(dir.path()).args(["init", "-b", "main"]).status().unwrap();
        Command::new("git").current_dir(dir.path()).args(["config", "user.email", "t@t.t"]).status().unwrap();
        Command::new("git").current_dir(dir.path()).args(["config", "user.name", "t"]).status().unwrap();
        Command::new("git").current_dir(dir.path()).args(["commit", "--allow-empty", "-m", "init"]).status().unwrap();

        let state = AppState::default();
        let sess = state.add("r1".into(), dir.path().into()).await;

        let snap1 = refresh_session(&sess).await.unwrap();
        assert_eq!(snap1.files.len(), 0);

        std::fs::write(dir.path().join("fresh.txt"), "fresh\n").unwrap();

        let cached = refresh_session(&sess).await.unwrap();
        assert_eq!(cached.files.len(), 0);

        let fresh = refresh_session_force(&sess).await.unwrap();
        assert_eq!(fresh.files.len(), 1);
        assert_eq!(fresh.files[0].path, "fresh.txt");
    }
}

use crate::error::{GitError, GitResult};
use crate::git::ops::branch::{create_branch, delete_branch};
use crate::git::ops::checkout::checkout_branch;
use crate::git::ops::commit::commit_files;
use crate::git::ops::fetch::fetch_all;
use crate::git::ops::merge::{merge_abort, merge_continue, merge_into_current};
use crate::git::ops::push::push;
use crate::git::snapshot::RepoSnapshot;
use crate::state::{refresh_session, AppState};
use serde::Serialize;
use std::path::PathBuf;
use tauri::State;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RepoMeta {
    pub id: String,
    pub path: PathBuf,
    pub name: String,
}

fn make_id(p: &std::path::Path) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    p.hash(&mut h);
    format!("{:x}", h.finish())
}

#[tauri::command]
pub async fn repo_add(path: String, state: State<'_, AppState>) -> GitResult<RepoMeta> {
    let p = PathBuf::from(&path);
    if !p.join(".git").exists() {
        return Err(GitError::parse(format!("Not a git repository: {path}")));
    }
    let id = make_id(&p);
    let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("repo").to_string();
    state.add(id.clone(), p.clone()).await;
    Ok(RepoMeta { id, path: p, name })
}

#[tauri::command]
pub async fn repo_refresh(
    id: String,
    log_branch: Option<String>,
    state: State<'_, AppState>,
) -> GitResult<RepoSnapshot> {
    let sess = state
        .get(&id)
        .await
        .ok_or_else(|| GitError::parse("unknown repo id"))?;
    if let Some(b) = log_branch {
        sess.lock().await.log_branch = Some(b);
    }
    refresh_session(&sess).await
}

#[tauri::command]
pub async fn repo_open(id: String, state: State<'_, AppState>) -> GitResult<RepoSnapshot> {
    repo_refresh(id, None, state).await
}

#[tauri::command]
pub async fn branch_checkout(
    id: String,
    name: String,
    state: State<'_, AppState>,
) -> GitResult<RepoSnapshot> {
    let sess = state
        .get(&id)
        .await
        .ok_or_else(|| GitError::parse("unknown repo id"))?;
    let path = { sess.lock().await.path.clone() };
    checkout_branch(&path, &name).await?;
    refresh_session(&sess).await
}

#[tauri::command]
pub async fn branch_create(
    id: String,
    name: String,
    from: Option<String>,
    state: State<'_, AppState>,
) -> GitResult<RepoSnapshot> {
    let sess = state
        .get(&id)
        .await
        .ok_or_else(|| GitError::parse("unknown repo id"))?;
    let path = { sess.lock().await.path.clone() };
    create_branch(&path, &name, from.as_deref()).await?;
    refresh_session(&sess).await
}

#[tauri::command]
pub async fn branch_delete(
    id: String,
    name: String,
    force: bool,
    state: State<'_, AppState>,
) -> GitResult<RepoSnapshot> {
    let sess = state
        .get(&id)
        .await
        .ok_or_else(|| GitError::parse("unknown repo id"))?;
    let path = { sess.lock().await.path.clone() };
    delete_branch(&path, &name, force).await?;
    refresh_session(&sess).await
}

#[tauri::command]
pub async fn commit(id: String, files: Vec<String>, message: String, skip_hooks: bool, state: State<'_, AppState>) -> GitResult<RepoSnapshot> {
    let sess = state.get(&id).await.ok_or_else(|| GitError::parse("unknown repo id"))?;
    let path = { sess.lock().await.path.clone() };
    commit_files(&path, &files, &message, skip_hooks).await?;
    refresh_session(&sess).await
}

#[tauri::command]
pub async fn push_branch(id: String, force_with_lease: bool, state: State<'_, AppState>) -> GitResult<RepoSnapshot> {
    let sess = state.get(&id).await.ok_or_else(|| GitError::parse("unknown repo id"))?;
    let path = { sess.lock().await.path.clone() };
    push(&path, force_with_lease).await?;
    refresh_session(&sess).await
}

#[tauri::command]
pub async fn fetch(id: String, state: State<'_, AppState>) -> GitResult<RepoSnapshot> {
    let sess = state.get(&id).await.ok_or_else(|| GitError::parse("unknown repo id"))?;
    let path = { sess.lock().await.path.clone() };
    fetch_all(&path).await?;
    refresh_session(&sess).await
}

#[tauri::command]
pub async fn merge(id: String, branch: String, state: State<'_, AppState>) -> GitResult<RepoSnapshot> {
    let sess = state.get(&id).await.ok_or_else(|| GitError::parse("unknown repo id"))?;
    let path = { sess.lock().await.path.clone() };
    merge_into_current(&path, &branch).await?;
    refresh_session(&sess).await
}

#[tauri::command]
pub async fn merge_abort_cmd(id: String, state: State<'_, AppState>) -> GitResult<RepoSnapshot> {
    let sess = state.get(&id).await.ok_or_else(|| GitError::parse("unknown repo id"))?;
    let path = { sess.lock().await.path.clone() };
    merge_abort(&path).await?;
    refresh_session(&sess).await
}

#[tauri::command]
pub async fn merge_continue_cmd(id: String, state: State<'_, AppState>) -> GitResult<RepoSnapshot> {
    let sess = state.get(&id).await.ok_or_else(|| GitError::parse("unknown repo id"))?;
    let path = { sess.lock().await.path.clone() };
    merge_continue(&path).await?;
    refresh_session(&sess).await
}

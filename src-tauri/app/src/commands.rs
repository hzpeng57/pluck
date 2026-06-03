use crate::error::{GitError, GitResult};
use crate::git::ops::branch::{
    create_branch, delete_branch, delete_precheck, rename_branch, DeletePrecheck,
};
use crate::git::ops::checkout::checkout_branch;
use crate::git::ops::cherry_pick::{cherry_pick, cherry_pick_abort, cherry_pick_continue};
use crate::git::ops::commit::commit_files;
use crate::git::ops::fetch::fetch_all;
use crate::git::ops::merge::{merge_abort, merge_continue, merge_into_current};
use crate::git::ops::pull::{pull_into_rebase, pull_rebase};
use crate::git::ops::push::push;
use crate::git::ops::rebase::{rebase_abort, rebase_continue, rebase_interactive};
use crate::git::ops::reset::{reset_to, ResetMode};
use crate::git::ops::revert::{revert, revert_abort, revert_continue};
use crate::git::ops::reword::{amend_message, reword_commit};
use crate::git::ops::show::{commit_show, CommitDetail};
use crate::git::parse::Commit;
use crate::git::snapshot::{log_page, log_search, RepoSnapshot};
use crate::rebase_editor::{deliver_reply, socket_path, EditReply, RebaseBridge};
use crate::state::{refresh_session, AppState};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Arc;
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
pub async fn branch_rename(
    id: String,
    old_name: String,
    new_name: String,
    unset_upstream: bool,
    state: State<'_, AppState>,
) -> GitResult<RepoSnapshot> {
    let sess = state
        .get(&id)
        .await
        .ok_or_else(|| GitError::parse("unknown repo id"))?;
    let path = { sess.lock().await.path.clone() };
    rename_branch(&path, &old_name, &new_name, unset_upstream).await?;
    {
        let mut s = sess.lock().await;
        if s.log_branch.as_deref() == Some(old_name.as_str()) {
            s.log_branch = Some(new_name);
        }
    }
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
    let res = delete_branch(&path, &name, force).await;
    // 不论成败都刷新一次：失败时也让 UI 拿到真实快照，避免 stale 状态
    let snapshot = refresh_session(&sess).await;
    res?;
    snapshot
}

#[tauri::command]
pub async fn branch_delete_precheck(
    id: String,
    name: String,
    state: State<'_, AppState>,
) -> GitResult<DeletePrecheck> {
    let sess = state
        .get(&id)
        .await
        .ok_or_else(|| GitError::parse("unknown repo id"))?;
    let path = { sess.lock().await.path.clone() };
    delete_precheck(&path, &name).await
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

#[tauri::command]
pub async fn pull(id: String, target_branch: String, state: State<'_, AppState>) -> GitResult<RepoSnapshot> {
    let sess = state.get(&id).await.ok_or_else(|| GitError::parse("unknown repo id"))?;
    let path = { sess.lock().await.path.clone() };
    pull_rebase(&path, &target_branch).await?;
    refresh_session(&sess).await
}

#[tauri::command]
pub async fn pull_into_current_rebase(
    id: String,
    source: String,
    state: State<'_, AppState>,
) -> GitResult<RepoSnapshot> {
    let sess = state.get(&id).await.ok_or_else(|| GitError::parse("unknown repo id"))?;
    let path = { sess.lock().await.path.clone() };
    pull_into_rebase(&path, &source).await?;
    refresh_session(&sess).await
}

#[tauri::command]
pub async fn rebase_interactive_start(
    id: String,
    from_commit: String,
    state: State<'_, AppState>,
) -> GitResult<RepoSnapshot> {
    let sess = state
        .get(&id)
        .await
        .ok_or_else(|| GitError::parse("unknown repo id"))?;
    let path = { sess.lock().await.path.clone() };
    let bridge_bin = current_bridge_path()?;
    rebase_interactive(&path, &from_commit, &bridge_bin, &socket_path()).await?;
    refresh_session(&sess).await
}

#[tauri::command]
pub async fn rebase_reply(
    reply: EditReply,
    bridge: State<'_, Arc<RebaseBridge>>,
) -> GitResult<()> {
    deliver_reply(&bridge, reply).await
}

#[tauri::command]
pub async fn rebase_continue_cmd(
    id: String,
    state: State<'_, AppState>,
) -> GitResult<RepoSnapshot> {
    let sess = state
        .get(&id)
        .await
        .ok_or_else(|| GitError::parse("unknown repo id"))?;
    let path = { sess.lock().await.path.clone() };
    rebase_continue(&path).await?;
    refresh_session(&sess).await
}

#[tauri::command]
pub async fn rebase_abort_cmd(
    id: String,
    state: State<'_, AppState>,
) -> GitResult<RepoSnapshot> {
    let sess = state
        .get(&id)
        .await
        .ok_or_else(|| GitError::parse("unknown repo id"))?;
    let path = { sess.lock().await.path.clone() };
    rebase_abort(&path).await?;
    refresh_session(&sess).await
}

#[tauri::command]
pub async fn commit_detail(
    id: String,
    hash: String,
    state: State<'_, AppState>,
) -> GitResult<CommitDetail> {
    let sess = state
        .get(&id)
        .await
        .ok_or_else(|| GitError::parse("unknown repo id"))?;
    let path = { sess.lock().await.path.clone() };
    commit_show(&path, &hash).await
}

#[tauri::command]
pub async fn cherry_pick_cmd(
    id: String,
    hashes: Vec<String>,
    state: State<'_, AppState>,
) -> GitResult<RepoSnapshot> {
    let sess = state.get(&id).await.ok_or_else(|| GitError::parse("unknown repo id"))?;
    let path = { sess.lock().await.path.clone() };
    cherry_pick(&path, &hashes).await?;
    refresh_session(&sess).await
}

#[tauri::command]
pub async fn cherry_pick_continue_cmd(id: String, state: State<'_, AppState>) -> GitResult<RepoSnapshot> {
    let sess = state.get(&id).await.ok_or_else(|| GitError::parse("unknown repo id"))?;
    let path = { sess.lock().await.path.clone() };
    cherry_pick_continue(&path).await?;
    refresh_session(&sess).await
}

#[tauri::command]
pub async fn cherry_pick_abort_cmd(id: String, state: State<'_, AppState>) -> GitResult<RepoSnapshot> {
    let sess = state.get(&id).await.ok_or_else(|| GitError::parse("unknown repo id"))?;
    let path = { sess.lock().await.path.clone() };
    cherry_pick_abort(&path).await?;
    refresh_session(&sess).await
}

#[tauri::command]
pub async fn revert_cmd(
    id: String,
    hashes: Vec<String>,
    state: State<'_, AppState>,
) -> GitResult<RepoSnapshot> {
    let sess = state.get(&id).await.ok_or_else(|| GitError::parse("unknown repo id"))?;
    let path = { sess.lock().await.path.clone() };
    revert(&path, &hashes).await?;
    refresh_session(&sess).await
}

#[tauri::command]
pub async fn revert_continue_cmd(id: String, state: State<'_, AppState>) -> GitResult<RepoSnapshot> {
    let sess = state.get(&id).await.ok_or_else(|| GitError::parse("unknown repo id"))?;
    let path = { sess.lock().await.path.clone() };
    revert_continue(&path).await?;
    refresh_session(&sess).await
}

#[tauri::command]
pub async fn revert_abort_cmd(id: String, state: State<'_, AppState>) -> GitResult<RepoSnapshot> {
    let sess = state.get(&id).await.ok_or_else(|| GitError::parse("unknown repo id"))?;
    let path = { sess.lock().await.path.clone() };
    revert_abort(&path).await?;
    refresh_session(&sess).await
}

#[tauri::command]
pub async fn reset_to_commit(
    id: String,
    hash: String,
    mode: ResetMode,
    state: State<'_, AppState>,
) -> GitResult<RepoSnapshot> {
    let sess = state.get(&id).await.ok_or_else(|| GitError::parse("unknown repo id"))?;
    let path = { sess.lock().await.path.clone() };
    reset_to(&path, &hash, mode).await?;
    refresh_session(&sess).await
}

#[tauri::command]
pub async fn amend_head_message(
    id: String,
    message: String,
    state: State<'_, AppState>,
) -> GitResult<RepoSnapshot> {
    let sess = state.get(&id).await.ok_or_else(|| GitError::parse("unknown repo id"))?;
    let path = { sess.lock().await.path.clone() };
    amend_message(&path, &message).await?;
    refresh_session(&sess).await
}

#[tauri::command]
pub async fn log_page_cmd(
    id: String,
    branch: Option<String>,
    skip: u32,
    limit: u32,
    state: State<'_, AppState>,
) -> GitResult<Vec<Commit>> {
    let sess = state
        .get(&id)
        .await
        .ok_or_else(|| GitError::parse("unknown repo id"))?;
    let path = { sess.lock().await.path.clone() };
    let target = branch.as_deref().unwrap_or("HEAD");
    log_page(&path, target, skip, limit).await
}

#[tauri::command]
pub async fn log_search_cmd(
    id: String,
    branch: Option<String>,
    query: String,
    author: String,
    limit: u32,
    state: State<'_, AppState>,
) -> GitResult<Vec<Commit>> {
    let sess = state
        .get(&id)
        .await
        .ok_or_else(|| GitError::parse("unknown repo id"))?;
    let path = { sess.lock().await.path.clone() };
    let target = branch.as_deref().unwrap_or("HEAD");
    log_search(&path, target, &query, &author, limit).await
}

#[tauri::command]
pub async fn reword_ancestor(
    id: String,
    hash: String,
    message: String,
    state: State<'_, AppState>,
) -> GitResult<RepoSnapshot> {
    let sess = state.get(&id).await.ok_or_else(|| GitError::parse("unknown repo id"))?;
    let path = { sess.lock().await.path.clone() };
    reword_commit(&path, &hash, &message).await?;
    refresh_session(&sess).await
}

fn current_bridge_path() -> GitResult<PathBuf> {
    let exe = std::env::current_exe().map_err(|e| GitError::parse(e.to_string()))?;
    if let Some(parent) = exe.parent() {
        let bundled = parent.join("../Resources/binaries/pluck-git-bridge");
        if bundled.exists() {
            return Ok(bundled);
        }
        let sibling = parent.join("pluck-git-bridge");
        if sibling.exists() {
            return Ok(sibling);
        }
    }
    let dev = std::env::current_dir()
        .unwrap_or_default()
        .join("src-tauri/target/debug/pluck-git-bridge");
    if dev.exists() {
        return Ok(dev);
    }
    Err(GitError::parse(format!(
        "bridge binary not found (searched bundled, sibling of {:?}, and {:?})",
        exe, dev
    )))
}

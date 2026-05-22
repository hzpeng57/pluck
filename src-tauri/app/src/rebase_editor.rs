use crate::error::{GitError, GitResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use tokio::sync::{oneshot, Mutex};

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EditPayload {
    pub kind: String,
    pub path: String,
    pub content: String,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EditReply {
    pub action: String,
    pub content: Option<String>,
}

#[derive(Default)]
pub struct RebaseBridge {
    pub pending: Mutex<Option<oneshot::Sender<EditReply>>>,
}

pub fn socket_path() -> PathBuf {
    std::env::temp_dir().join(format!("git-lite-bridge-{}.sock", std::process::id()))
}

pub async fn start_listener(app: AppHandle, bridge: Arc<RebaseBridge>) -> GitResult<()> {
    let path = socket_path();
    let _ = std::fs::remove_file(&path);
    let listener =
        UnixListener::bind(&path).map_err(|e| GitError::parse(format!("bind socket: {e}")))?;

    tauri::async_runtime::spawn(async move {
        loop {
            let (stream, _) = match listener.accept().await {
                Ok(s) => s,
                Err(_) => continue,
            };
            let app = app.clone();
            let bridge = bridge.clone();
            tauri::async_runtime::spawn(async move {
                let mut stream = stream;
                let (rx, mut tx) = stream.split();
                let mut reader = BufReader::new(rx);
                let mut line = String::new();
                if reader.read_line(&mut line).await.is_err() {
                    return;
                }
                let req: EditPayload = match serde_json::from_str(line.trim()) {
                    Ok(p) => p,
                    Err(_) => return,
                };
                let (reply_tx, reply_rx) = oneshot::channel();
                {
                    *bridge.pending.lock().await = Some(reply_tx);
                }
                let _ = app.emit("rebase:edit", req);
                if let Ok(reply) = reply_rx.await {
                    let line = serde_json::to_string(&reply).unwrap_or_default();
                    let _ = tx.write_all(line.as_bytes()).await;
                    let _ = tx.shutdown().await;
                }
            });
        }
    });
    Ok(())
}

pub async fn deliver_reply(bridge: &Arc<RebaseBridge>, reply: EditReply) -> GitResult<()> {
    if let Some(tx) = bridge.pending.lock().await.take() {
        let _ = tx.send(reply);
        Ok(())
    } else {
        Err(GitError::parse("no pending rebase edit"))
    }
}

use std::time::Duration;
use tempfile::tempdir;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bridge_relays_todo_to_socket() {
    // Workspace target is at <repo>/src-tauri/target. Tests run with CWD set to the package dir
    // (src-tauri/app) by cargo, so compute the bridge path relative to the workspace root.
    let bridge_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap() // src-tauri/
        .join("target/debug/taptap-git-bridge");
    assert!(
        bridge_path.exists(),
        "build bridge first: cargo build -p taptap-git-bridge ({})",
        bridge_path.display()
    );

    let dir = tempdir().unwrap();
    let todo = dir.path().join("git-rebase-todo");
    std::fs::write(&todo, "pick abc123 first\npick def456 second\n").unwrap();
    let sock = dir.path().join("test.sock");

    let listener = UnixListener::bind(&sock).unwrap();
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let (rx, mut tx) = stream.split();
        let mut reader = BufReader::new(rx);
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        // The bridge sends a JSON line: {"kind":"sequence","path":"...","content":"pick abc123 first\npick def456 second\n"}
        // The JSON-encoded content escapes \n as \\n, so the literal "pick abc123 first" substring is still present.
        assert!(line.contains("pick abc123 first"), "request line: {line}");
        // The bridge uses read_to_string (reads until EOF). Drop the write half by shutting it down.
        let reply = r#"{"action":"save","content":"pick abc123 first\nsquash def456 second\n"}"#;
        tx.write_all(reply.as_bytes()).await.unwrap();
        tx.shutdown().await.unwrap();
    });

    let mut child = tokio::process::Command::new(&bridge_path)
        .arg(todo.to_str().unwrap())
        .env("TTGIT_SOCK", sock.to_str().unwrap())
        .spawn()
        .unwrap();

    tokio::time::timeout(Duration::from_secs(5), server)
        .await
        .unwrap()
        .unwrap();
    let status = tokio::time::timeout(Duration::from_secs(5), child.wait())
        .await
        .unwrap()
        .unwrap();
    assert!(status.success());

    let after = std::fs::read_to_string(&todo).unwrap();
    assert!(after.contains("squash def456"), "todo after: {after}");
}

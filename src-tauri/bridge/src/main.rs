use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EditRequest {
    kind: String,        // "sequence" | "commitMsg"
    path: String,
    content: String,
}

#[derive(Deserialize)]
struct EditReply {
    action: String,      // "save" | "abort"
    content: Option<String>,
}

fn main() -> ExitCode {
    let sock_path = match env::var("TTGIT_SOCK") { Ok(s) => s, Err(_) => return ExitCode::from(0) };
    let argv1 = match env::args().nth(1) { Some(a) => a, None => return ExitCode::from(0) };
    let file = PathBuf::from(&argv1);
    let kind = if argv1.ends_with("git-rebase-todo") { "sequence" } else { "commitMsg" };
    let content = match fs::read_to_string(&file) { Ok(c) => c, Err(_) => String::new() };

    let stream = match connect_with_timeout(&sock_path, Duration::from_secs(5)) {
        Ok(s) => s,
        Err(_) => return ExitCode::from(0), // safe fallback: leave file unchanged
    };
    let mut stream = stream;

    let req = EditRequest { kind: kind.into(), path: argv1, content };
    let line = serde_json::to_string(&req).unwrap() + "\n";
    if stream.write_all(line.as_bytes()).is_err() { return ExitCode::from(0); }

    let mut buf = String::new();
    if stream.read_to_string(&mut buf).is_err() { return ExitCode::from(0); }

    let reply: EditReply = match serde_json::from_str(buf.trim()) { Ok(r) => r, Err(_) => return ExitCode::from(0) };
    if reply.action == "abort" { return ExitCode::from(1); }
    if let Some(new_content) = reply.content {
        let _ = fs::write(&file, new_content);
    }
    ExitCode::from(0)
}

fn connect_with_timeout(path: &str, timeout: Duration) -> std::io::Result<UnixStream> {
    let deadline = std::time::Instant::now() + timeout;
    loop {
        match UnixStream::connect(path) {
            Ok(s) => return Ok(s),
            Err(e) if std::time::Instant::now() >= deadline => return Err(e),
            Err(_) => std::thread::sleep(Duration::from_millis(100)),
        }
    }
}

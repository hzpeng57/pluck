fn main() {
    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "debug".into());
    let workspace = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("app crate must live in workspace");
    let target = std::env::var("TARGET").ok();
    let src = match target {
        Some(t) => {
            let with_target = workspace.join(format!("target/{t}/{profile}/taptap-git-bridge"));
            if with_target.exists() {
                with_target
            } else {
                workspace.join(format!("target/{profile}/taptap-git-bridge"))
            }
        }
        None => workspace.join(format!("target/{profile}/taptap-git-bridge")),
    };
    let dst_dir = workspace.join("binaries");
    let _ = std::fs::create_dir_all(&dst_dir);
    let dst = dst_dir.join("taptap-git-bridge");
    if src.exists() {
        let _ = std::fs::copy(&src, &dst);
    } else if !dst.exists() {
        // Ensure the resource file exists so tauri_build doesn't fail validation.
        // The real binary will be copied here on subsequent builds once the
        // bridge crate has been compiled.
        let _ = std::fs::File::create(&dst);
    }
    println!("cargo:rerun-if-changed={}", src.display());

    tauri_build::build();
}

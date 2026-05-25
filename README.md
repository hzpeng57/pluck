<p align="center">
  <img src="src-tauri/app/icons/128x128@2x.png" width="128" alt="Pluck app icon" />
</p>

<h1 align="center">Pluck</h1>

<p align="center">A native, keyboard-friendly git client for macOS — built with Tauri 2 + Vue 3.</p>

<p align="center">
  <strong>English</strong> · <a href="README.zh-CN.md">简体中文</a>
</p>

---

## Status

Personal-use alpha. macOS arm64 (Apple Silicon) only for now; Intel Mac and Windows are planned.

## Install

1. Download the latest `Pluck_<version>_aarch64.dmg` from the [Releases page](https://github.com/hzpeng57/pluck/releases/latest).
2. Open the DMG and drag **Pluck** into `/Applications`.
3. The app is not yet signed with an Apple Developer ID, so macOS Gatekeeper will mark it as "damaged" on first launch. Remove the quarantine attribute once:

   ```bash
   xattr -dr com.apple.quarantine /Applications/Pluck.app
   ```

4. Launch from Spotlight, Launchpad, or `open -a Pluck`.

> Proper Developer ID signing + notarization is on the roadmap; once shipped, the `xattr` step will no longer be needed.

## Updates

Pluck ships with an in-app updater:

- Checks GitHub Releases once at launch, then re-checks every 6 hours;
- Click the `pluck v0.x.y` label in the bottom-right status bar to check on demand;
- When a new version is available, a banner appears at the top of the window — one click downloads, verifies the ed25519 signature, and restarts into the new build.

## Run from source

Requirements: Node 20+, pnpm 9+, Rust stable, Xcode Command Line Tools.

```bash
pnpm install
pnpm tauri dev
```

To produce a local release bundle:

```bash
pnpm tauri build
# output: src-tauri/target/release/bundle/macos/Pluck.app
```

## Architecture

- **`src/`** — Vue 3 + Pinia frontend (status, log, diff, rebase, etc.).
- **`src-tauri/app/`** — Tauri host. Owns the git command layer and exposes commands to the frontend.
- **`src-tauri/bridge/`** — `pluck-git-bridge`, a small sidecar binary that git invokes as `$GIT_EDITOR` / `$GIT_SEQUENCE_EDITOR` during interactive rebase and commit. It talks to the host over a per-process unix socket so the UI can drive todo edits and commit-message reword without dropping into the terminal.

## Tech stack

- Tauri 2 (Rust backend, native macOS WKWebView)
- Vue 3 + Pinia + Vite
- TailwindCSS 4
- TypeScript

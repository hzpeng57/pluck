<p align="center">
  <img src="src-tauri/app/icons/128x128@2x.png" width="128" alt="Pluck app icon" />
</p>

<h1 align="center">Pluck</h1>

<p align="center">一个轻量的 macOS git 客户端，参考 JetBrains 系列 IDE 的 git 体验。</p>

<p align="center">
  <a href="README.md">English</a> · <strong>简体中文</strong>
</p>

---

## 状态

个人使用 alpha 阶段。目前只支持 macOS arm64（Apple Silicon），Intel Mac 与 Windows 在规划中。

## 安装

1. 从 [Releases 页面](https://github.com/hzpeng57/pluck/releases/latest) 下载最新的 `Pluck_<版本>_aarch64.dmg`。
2. 双击 DMG，把 **Pluck** 拖到 `/Applications` 目录。
3. 由于尚未通过 Apple Developer ID 签名，macOS Gatekeeper 首次启动时会提示「应用已损坏」。执行一次以下命令去掉隔离属性：

   ```bash
   xattr -dr com.apple.quarantine /Applications/Pluck.app
   ```

4. 从 Spotlight、Launchpad，或者用 `open -a Pluck` 启动。

> 正式的 Developer ID 签名 + 公证已经在 roadmap 上；上线之后，上面的 `xattr` 步骤就不再需要。

## 自动更新

Pluck 内置自动更新：

- 启动时立即检查一次 GitHub Releases，之后每 6 小时复查；
- 状态栏右下角的「pluck v0.x.y」可以点击，随时手动检查；
- 有新版本时会在窗口顶部弹出 banner，一键完成下载 → ed25519 签名校验 → 重启进入新版本。

## 从源码运行

依赖：Node 20+、pnpm 9+、Rust stable、Xcode Command Line Tools。

```bash
pnpm install
pnpm tauri dev
```

要打本地 release 包：

```bash
pnpm tauri build
# 产物：src-tauri/target/release/bundle/macos/Pluck.app
```

## 架构

- **`src/`** —— Vue 3 + Pinia 前端（状态面板、提交日志、diff、rebase 等）。
- **`src-tauri/app/`** —— Tauri 宿主进程。封装 git 命令层，向前端暴露 commands。
- **`src-tauri/bridge/`** —— `pluck-git-bridge`，一个小型 sidecar 二进制。git 在交互式 rebase 与 commit 时把它当作 `$GIT_EDITOR` / `$GIT_SEQUENCE_EDITOR` 调起，通过每进程独立的 unix socket 跟宿主通信，让 UI 接管 todo 编辑和 commit message 修改，不需要掉进终端。

## 技术栈

- Tauri 2（Rust 后端，原生 macOS WKWebView）
- Vue 3 + Pinia + Vite
- TailwindCSS 4
- TypeScript

## 给 AI Agent

如果你是接手这个 repo 的 AI 编码 agent，先读 [AGENTS.md](AGENTS.md)：目录拓扑、架构红线、常用命令、发版流程都在里面。

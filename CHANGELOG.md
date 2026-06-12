# Changelog

本项目遵循 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/) 规范，版本号采用 [SemVer](https://semver.org/lang/zh-CN/)。

## [0.1.9] - 2026-06-12

### Fixed
- 修复从 macOS Dock / Finder 启动 Pluck 时，Git LFS 安装在 Homebrew 路径下无法被 `git` 子进程找到，导致 checkout / 新建分支等操作报 `git-lfs filter-process: git-lfs: command not found` 的问题。
- 统一 merge、pull、rebase、cherry-pick、revert、reword 等 Git 操作的子进程 PATH 处理，避免同类 LFS filter 问题在其他操作中复现。

## [0.1.8] - 2026-06-08

### Fixed
- 修复发布包中 Interactive rebase 无法找到 `pluck-git-bridge`，导致 `Interactively rebase from here...` 报错的问题。
- 修复 rebase 等操作收到结构化错误时 toast 显示 `[object Object]` 的问题。

### Changed
- 发布构建缺少 `pluck-git-bridge` 时直接失败，避免生成缺少 rebase bridge 的安装包。

## [0.1.7] - 2026-06-03

### Added
- 分支右键菜单新增 Rename 功能，支持本地分支重命名，并可选择同时取消 upstream。

### Changed
- `New branch from here...` 创建分支时显式使用 `--no-track`，避免从远端分支创建新分支时自动跟踪原远端分支。

## [0.1.6] - 2026-06-02

### Fixed
- 修复分支名与路径同名时 History 执行 `git log` 报 `fatal: ambiguous argument` 的问题，例如 taptap-pc 中的 `main`。
- 修复仓库切换或打开失败时旧仓库快照回写，导致分支列表显示成上一个仓库的问题。
- 修复危险按钮 hover 与禁用按钮 cursor/hover 状态不清晰的问题。

## [0.1.5] - 2026-06-01

### Fixed
- 删除当前 History 选中的本地分支后，刷新自动回退到当前 HEAD，避免 `fatal: ambiguous argument` 报错。

## [0.1.4] - 2026-05-28

### Added
- 新增 App 内统一确认弹窗，用于 hard reset 与 force-with-lease push 等高风险操作。

### Changed
- 远程分支右键 checkout 改为创建本地 tracking branch，避免进入 detached HEAD。
- 「Pull into "<current>" using rebase」和顶部 Pull 遇到冲突时保留 rebase 中态，由 InProgress banner 接管 continue/abort。
- Merge 遇到冲突时保留 merge 中态并刷新快照，不再以普通失败 toast 中断流程。
- 文件级 commit 改用 `git commit --only`，保留用户已 staged 但未勾选的文件。
- 交互式 rebase 仅允许在当前分支 log 上触发，避免对非当前分支产生歧义操作。

### Fixed
- 新建分支成功或失败后自动关闭弹窗，并让 toast 不再被蒙层遮挡。
- 修复 Git linked worktree 下 merge/rebase 中态检测漏判的问题。
- Repo 移除等轻量提示改用 toast，继续移除原生浏览器弹窗。

## [0.1.3] - 2026-05-26

### Added
- History 面板新增服务端搜索：按 hash 前缀或 message 关键字过滤，命中走 `git log --grep`，hash 命中走 `git rev-parse` 单点解析。
- History 面板新增作者过滤，搜索框旁直接选「me」一键按当前 git 身份过滤。
- History 面板支持上下分栏（CommitDetail / Commit ↔ History），分隔条比例持久化。

### Changed
- 分支删除/创建等弹窗替换为 Vue 自绘对话框，统一与 App 内观感，移除原生 `prompt/confirm`。
- Updater 启动即检查 + 每 6 小时复查 + 状态栏新增「Check for Updates」入口。

### Fixed
- 长 history 滚动到底加载下一页：由 IntersectionObserver 改为 scroll 事件 + rAF，规避 nested 容器边界判定问题。

## [0.1.2] - 2026-05-25

### Changed
- 重写 README，介绍定位、能力、构建方式。
- 内部 sidecar 重命名为 `pluck-git-bridge`。

## [0.1.1] - 2026-05-25

### Added
- 接入 `tauri-plugin-updater` + GitHub Actions 自动发版流程。
- 启用 Pluck 暗色品牌 icon，整体 UI 切换为 Linear/Fleet 风格的暗色优先设计。
- Commit 右键菜单：cherry-pick / revert / 编辑信息 / reset，后端补齐对应能力。
- Commit Detail 视图，含可折叠的文件树。
- Branches 面板：搜索框（⌘F + 自动展开）、置顶、按前缀折叠的树。
- 「Pull into "<current>" using rebase」语义，对齐 WebStorm。
- 仓库侧栏支持右键移除。

### Changed
- 项目改名 `git-lite` → `pluck`。
- 左侧面板可拖拽调宽；fetch/pull/push 接入 toast + loading 态。
- Commit 时间显示重构、分支名一键复制。

### Fixed
- App 重启后 repo session 重新注册。
- 用户主动中断 rebase / 冲突中途退出时不再误报错误。
- CommitDetail 点击无响应（Pinia HMR 残留）：拆出 `onCommitClick` + 加载占位。

## [0.1.0] - 2026-05-22

首个 code-complete 版本。

### Added
- 基于 git CLI 的快照模型：`status --porcelain=v2` / `for-each-ref` / `log` 并行采集，组装成 `RepoSnapshot`。
- 检测 merge / rebase / cherry-pick 进行中状态，UI 显示 banner 并提供 continue / abort。
- 基础操作：branch checkout/create/delete、文件级 commit（含 skip-hooks）、push `--force-with-lease`、fetch `--all --prune`、pull `--rebase`、merge 含 abort/continue。
- Interactive rebase：通过自实现 bridge 二进制 + Unix socket 与 GIT_SEQUENCE_EDITOR 通信，前端弹 todo 对话框。
- Tauri 命令层 + Pinia stores + repo 切换 shell layout。
- 快捷键：⌘K commit / ⌘⇧K push / ⌘T fetch / ⌘R refresh。
- Toast tray 替代内联 lastError。

[0.1.9]: https://github.com/hzpeng57/pluck/compare/v0.1.8...v0.1.9
[0.1.8]: https://github.com/hzpeng57/pluck/compare/v0.1.7...v0.1.8
[0.1.7]: https://github.com/hzpeng57/pluck/compare/v0.1.6...v0.1.7
[0.1.6]: https://github.com/hzpeng57/pluck/compare/v0.1.5...v0.1.6
[0.1.5]: https://github.com/hzpeng57/pluck/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/hzpeng57/pluck/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/hzpeng57/pluck/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/hzpeng57/pluck/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/hzpeng57/pluck/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/hzpeng57/pluck/releases/tag/v0.1.0

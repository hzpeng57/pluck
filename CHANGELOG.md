# Changelog

本项目遵循 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/) 规范，版本号采用 [SemVer](https://semver.org/lang/zh-CN/)。

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

[0.1.4]: https://github.com/hzpeng57/pluck/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/hzpeng57/pluck/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/hzpeng57/pluck/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/hzpeng57/pluck/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/hzpeng57/pluck/releases/tag/v0.1.0

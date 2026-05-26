# AGENTS.md

给 fresh session 接手 Pluck 的最小上下文。先读这一份，再动手。

## 项目定位

Pluck 是一个 macOS 原生 git 客户端：Tauri 2 壳，Vue 3 + Pinia + TailwindCSS 4 前端，Rust 后端通过 shell 调用 `git` CLI。目标用户是从 WebStorm / Fork / Tower 切过来的开发者。

## 目录拓扑

```
src/                       Vue 3 前端
  components/              UI 组件
  stores/                  Pinia setup stores（repos / repoState / branchPrefs）
  api/tauri.ts             所有 invoke 包装，类型来自 src/types/git.ts
  types/git.ts             与 Rust 端 serde 结构镜像

src-tauri/                 Cargo workspace
  app/                     主进程（pluck-app）
    src/commands.rs        所有 #[tauri::command]，薄壳，只做参数解包 + refresh
    src/git/snapshot.rs    RepoSnapshot 装配
    src/git/ops/           每个 git 动作一个文件
    src/git/cmd.rs         run_git shell exec 唯一入口
    src/git/parse.rs       porcelain v2 / for-each-ref / log 解析
    src/state.rs           AppState + RepoSession，含 refresh debounce
    src/rebase_editor.rs   交互式 rebase 的 Unix socket 桥
  bridge/                  独立二进制 pluck-git-bridge（rebase 时的 GIT_SEQUENCE_EDITOR）
  binaries/                构建产物（sidecar）
  app/tauri.conf.json      版本号、updater、bundle 配置
```

## 架构红线

- **后端 git 操作全走 `run_git` shell exec**，不引入 `git2` / libgit2。Pluck 的底层逻辑就是「git CLI 的可视化壳」，绕过 CLI 等于砸地基。
- **快照模型**：所有 mutation（commit/checkout/rebase/...）末尾必须 `refresh_session()` 返回完整 `RepoSnapshot`，前端 store 直接整体替换。**不要**单独返回 diff 片段让前端拼装。
- **Tauri invoke 参数 camelCase ↔ snake_case 自动转**：Rust 写 snake_case + `#[serde(rename_all = "camelCase")]`，TS 端传 camelCase。
- **交互式 rebase 走 bridge + Unix socket**：`pluck-git-bridge` 二进制作为 `GIT_SEQUENCE_EDITOR`，通过 socket 把 todo list 推给前端、等回写。**不要**尝试在主进程内拦截 editor。
- **前端禁用原生 `prompt` / `confirm` / `alert`**：统一走 Vue 自绘对话框（参考 `BranchesPanel` / `App.vue` 里的 dialog 模式）。

## 前端约定

- Pinia 全用 setup store（`defineStore('x', () => { ... })`），不用 options API。
- 样式优先 Tailwind utility；自绘 token 在 `src/style.css` 的 `@layer components`，命名 `gl-*`（`gl-input` / `gl-menu` / `gl-spinner` / `gl-chip` ...）。颜色走 CSS 变量（`--fg` / `--accent` / `--accent-soft` / `--hover` / `--danger` 等），不写死色值。
- 长列表用 scroll 事件 + `requestAnimationFrame` 实现无限滚动，**不**用 IntersectionObserver（nested scroll container 上踩过坑）。

## 常用命令

```bash
pnpm install                                              # 装依赖
pnpm dev                                                  # 纯前端 dev server
pnpm tauri dev                                            # 完整 App dev（前端 + Rust）
pnpm build                                                # vue-tsc + vite build
cargo check --manifest-path src-tauri/Cargo.toml          # 后端类型检查（快）
cargo build --manifest-path src-tauri/Cargo.toml -p pluck-git-bridge --release   # 单独构建 bridge
```

后端改动后跑 `cargo check`，前端类型改动后跑 `pnpm build`（`vue-tsc --noEmit` 在里面）。

## 发版流程

tag 推上去会触发 `.github/workflows/release.yml`，构建 macOS aarch64 产物 + 写 GitHub Release + 更新 `latest.json`（updater 弹窗用）。

闭环动作：

1. `CHANGELOG.md` 顶部新增 `## [x.y.z] - YYYY-MM-DD` 段，写 Added / Changed / Fixed
2. 同步 bump **4 处**版本号，少一处 CI 就跪：
   - `package.json`
   - `src-tauri/app/Cargo.toml`
   - `src-tauri/app/tauri.conf.json`
   - `src-tauri/Cargo.lock`（搜 `name = "pluck-app"` 改下面那行 version）
3. `git commit -m "chore: 发布 vx.y.z，..."`
4. `git tag vx.y.z`
5. 等用户确认后 `git push origin main && git push origin vx.y.z`

release notes 由 workflow 从 CHANGELOG 抽 `## [x.y.z]` 到下一个 `## [` 之间的内容，所以 CHANGELOG 段落写得好 = release / updater 弹窗都漂亮。

## 红线

- **`~/Desktop/pluck-updater-key` 是 updater 私钥，永远不进 repo**。`.gitignore` 已防，但 agent 不要主动 cat / 移动这个文件。
- **push 前必须问用户**。已有 commit / tag 也要问；这是用户的全局规则（见 `~/.claude/CLAUDE.md`）。
- **commit message / PR 描述用中文**，scope 风格参考 `git log --oneline | head -30`：`feat(log): ...` / `fix(rebase): ...` / `chore: ...` / `docs: ...`。不加 ticket id、不加 `#no-ticket`。
- **不要在 main 上做实验性大改**。要重构先开分支。

## 调试提示

- `RUST_LOG=pluck=debug pnpm tauri dev` 看后端 git 调用。
- 卡在 rebase / merge / cherry-pick 中态时检查 `.git/REBASE_HEAD` / `.git/MERGE_HEAD` / `.git/CHERRY_PICK_HEAD`，对应 `detect_in_progress` 的判定。
- updater 弹窗不出现：检查 `latest.json` 里的 `signature` 是否匹配本地 `~/Desktop/pluck-updater-key.pub`，以及 `tauri.conf.json` 里 `endpoints` 是否指向当前 repo 的 `latest.json` raw 链接。

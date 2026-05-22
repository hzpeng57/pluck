# Lightweight Git Tool — Design Spec

- **Date:** 2026-05-22
- **Working name:** `git-lite` (renameable)
- **Owner:** hzp
- **Status:** Design approved, pending implementation plan

## 1. Background and Goal

WebStorm's Git UI is the user's daily driver, but running WebStorm just for Git operations is wasteful: 1.5–3 GB RAM per project window, multi-second cold start. Existing alternatives (VSCode, Fork, Sourcetree, lazygit) all use different command vocabularies and interaction patterns, breaking the user's WebStorm muscle memory.

**Goal:** A lightweight desktop app that replicates WebStorm's Git operations and interaction model in ~150 MB RAM, opens instantly, and works against multiple repositories from a single window.

**Non-goal:** Feature parity with WebStorm. Anything not in §3 is explicitly out of v1.

## 2. Core Principles

1. **WebStorm operation parity over invention.** Every command's name, default, and behavior maps to a recognizable WebStorm action.
2. **Lean by default.** No fs watchers, no background polling, no graph layout engine in v1. Refresh on focus + after operations only.
3. **System git is the source of truth.** All Git operations shell out to the user's installed `git`. The user's hooks, credential helpers, GPG signing, and `.gitconfig` work unchanged.
4. **Interaction quality is non-negotiable.** Keyboard shortcuts, right-click menus, type-to-search, and inline progress mirror WebStorm's polish.

## 3. v1 Scope

### In scope
- **Repo switcher** (left sidebar): add / open / remove repos, persisted across sessions.
- **Branches panel** (foldable Local / Remote sections):
  - List with current branch highlight and `[ahead/behind upstream]` decorations.
  - Right-click: checkout, new branch from here, merge into current, pull `--rebase`, delete.
- **Commit panel**:
  - File-level checkbox staging (no hunk-level in v1).
  - Commit message input.
  - "Skip hooks" checkbox (→ `git commit -n`).
  - `[Commit]` and `[Commit & Push]` buttons.
- **Push** with optional `--force-with-lease` (never bare `--force`).
- **Fetch all + prune.**
- **Merge selected branch into current.**
- **Pull `--rebase`** into the selected branch.
- **Interactive Rebase from Here** (right-click any commit in the log).
- **Log view for the currently selected branch only** (linear list, no graph topology).
- **In-progress detection**: surface merge / rebase / cherry-pick state with `[Continue] [Abort]` banner.

### Out of scope (deferred)
- Hunk- or line-level staging
- Built-in three-pane conflict resolver
- Cherry-pick / revert / reset right-click actions
- Multi-branch log graph topology
- Blame / file history
- Stash management
- Submodules / worktrees
- Multiple windows
- GPG passphrase prompts (delegated to system ssh-agent / pinentry)

## 4. Architecture

### 4.1 Stack
- **Shell:** Tauri 2.x (Rust core + system WebView)
- **Frontend:** Vue 3 + Pinia + TailwindCSS
- **Backend:** Rust, shelling out to system `git`

### 4.2 Process Topology
Single OS process, single window. The Rust core holds an in-memory `RepoSession` per opened repo (cached `RepoSnapshot`). Switching active repo in the sidebar swaps which session the UI reads from; the WebView is not recreated.

### 4.3 Layout

```
┌──────────────────────────────────────────────────────────────┐
│ TitleBar  [Repo: aisync]   [Fetch] [Push] [Pull --rebase ▾] │
├──────────┬───────────────────────────────────────────────────┤
│ Repos    │ ┌─ Branches ─────────┐ ┌─ Commit ──────────────┐ │
│ • aisync │ │ Local         ▾    │ │ ☐ src/app.ts   M      │ │
│ • beauti │ │   ● main           │ │ ☑ README.md    M      │ │
│ • ccm    │ │   feat/x [→origin] │ │ ☐ docs/x.md    ??     │ │
│ + Add..  │ │ Remote        ▾    │ │ ─────────────────────  │ │
│ ⚙ Setting│ │   origin/main      │ │ [commit message...]    │ │
│          │ │   origin/dev       │ │ ☐ Skip hooks (-n)      │ │
│          │ └────────────────────┘ │ [Commit] [Commit&Push] │ │
│          │                        └────────────────────────┘ │
│          │ ┌─ Log: main ──────────────────────────────────┐  │
│          │ │ ● a1b2c3  fix: ...     hzp   2h              │  │
│          │ │ ● d4e5f6  feat: ...    hzp   yesterday  ⋮    │  │
│          │ │ │  Right-click ⋮: Interactive Rebase from Here│  │
│          │ └──────────────────────────────────────────────┘  │
└──────────┴───────────────────────────────────────────────────┘
                                          [status bar: ↑2 ↓0 ✏3]
```

### 4.4 Frontend modules

| Module | Responsibility | Depends on |
|---|---|---|
| `stores/repos.ts` | Repo list (localStorage), active repo id | — |
| `stores/repoState.ts` | Current repo snapshot; single `refresh()` entry point with 200 ms debounce | tauri `invoke` |
| `components/RepoSwitcher.vue` | Left sidebar | `repos` |
| `components/BranchesPanel.vue` | Foldable Local / Remote sections, right-click menu | `repoState` |
| `components/CommitPanel.vue` | File checklist, message, skip-hooks, commit buttons | `repoState` |
| `components/LogPanel.vue` | Selected-branch commit list, right-click menu | `repoState` |
| `components/RebaseTodoDialog.vue` | Interactive rebase todo editor (pick / reword / squash / fixup / drop + reorder) | tauri events |
| `components/InProgressBanner.vue` | Shows when merge / rebase / cherry-pick is in progress | `repoState` |
| `components/ToastTray.vue` | Non-blocking error and success feedback | — |

### 4.5 Backend modules (Rust)

| Module | Responsibility |
|---|---|
| `git/cmd.rs` | Wraps `Command::new("git")`: stdin / stdout / stderr capture, exit code handling, friendly mapping of `.git/index.lock` and other common failures |
| `git/parse.rs` | Parsers for `status --porcelain=v2`, `for-each-ref`, `log --format=...` |
| `git/ops/{commit,branch,push,fetch,merge,pull,rebase,checkout}.rs` | One file per operation; pure functions over a `RepoCtx` |
| `git/detect.rs` | Reads `.git/MERGE_HEAD`, `.git/rebase-merge/`, `.git/CHERRY_PICK_HEAD` to populate `in_progress` |
| `rebase_editor.rs` | Bridge for `GIT_SEQUENCE_EDITOR` / `GIT_EDITOR` (see §6) |
| `commands.rs` | Tauri command registration |
| `state.rs` | `RepoSession` cache + 200 ms refresh debounce |

## 5. Data Model

```rust
struct RepoMeta { id: String, path: PathBuf, name: String, last_opened: i64 }

struct RepoSnapshot {
  head: HeadInfo,                  // current branch or detached
  files: Vec<WorkingFile>,         // working-tree changes
  branches: BranchList,            // local + remote
  log: Vec<Commit>,                // for the selected branch only
  remote_status: RemoteStatus,     // ahead / behind upstream
  in_progress: Option<GitOp>,      // merge / rebase / cherry-pick
}

struct HeadInfo { branch: Option<String>, detached_at: Option<String> }
struct RemoteStatus { upstream: Option<String>, ahead: u32, behind: u32 }

enum FileStatus { Modified, Added, Deleted, Renamed, Untracked, Conflicted }
struct WorkingFile { path: String, old_path: Option<String>, status: FileStatus }

struct BranchList { local: Vec<Branch>, remote: Vec<Branch> }
struct Branch {
  name: String,                    // "main" / "origin/main"
  kind: BranchKind,                // Local | Remote
  upstream: Option<String>,        // for local branches
  ahead: u32, behind: u32,         // vs upstream
  is_current: bool,
  last_commit_short: String,
}

struct Commit {
  hash: String, short: String,
  author: String, email: String, date_unix: i64,
  subject: String, body: String,
  parents: Vec<String>,
  refs: Vec<String>,               // matching tags / branches
}

enum GitOp { Merging{ from: String }, Rebasing{ onto: String, head: String }, CherryPicking }
```

**Contract:** The backend exposes snapshot queries and command invocations only — no incremental events. After every command and on window focus, the frontend calls `repo_refresh(id)` and re-renders from the returned snapshot. A snapshot is ~10–50 KB serialized.

## 6. Tauri Command Surface (v1 complete list)

| Command | Args | Behavior |
|---|---|---|
| `repo_add(path)` | absolute path | Validate `.git/`, persist to repo list, return `RepoMeta` |
| `repo_open(id)` | repo id | Set active session, return `RepoSnapshot` |
| `repo_refresh(id)` | id | Force re-pull snapshot (debounced 200 ms) |
| `branch_checkout(id, name)` | | `git checkout <name>` |
| `branch_create(id, name, from)` | | `git checkout -b <name> <from>` |
| `branch_delete(id, name, force)` | | `git branch -d` / `-D` |
| `commit(id, files, message, skip_hooks)` | | `git add <files>` then `git commit -m <msg> [-n]` |
| `push(id, force_with_lease)` | | `git push [--force-with-lease]` — never bare `--force` |
| `fetch(id)` | | `git fetch --all --prune` |
| `merge_into_current(id, branch)` | | `git merge <branch>` |
| `pull_rebase(id, target_branch)` | | If current branch: `git pull --rebase`. Else: `git fetch <remote> <branch>:<branch>` for fast-forward; non-ff returns an error asking the user to checkout first. |
| `rebase_interactive_start(id, from_commit)` | | Launch bridge (§7) |
| `rebase_continue(id)` / `rebase_abort(id)` | | |
| `merge_continue(id)` / `merge_abort(id)` | | |

## 7. Interactive Rebase Bridge

This is the only piece of v1 that requires custom infrastructure. `git rebase -i` opens two external editors: the **sequence editor** (the todo list) and the **commit editor** (when reword / edit lands). Both must be hijacked into the Tauri UI.

### 7.1 Components

- `taptap-git-bridge` — a small standalone Rust binary (~300 KB), bundled in `Resources/`.
- A Unix domain socket created by the Tauri main process before each rebase.
- An RPC pair: `rebase:edit` event (main → frontend) and `rebase:reply` invoke (frontend → main).

### 7.2 Flow

```
Tauri main process              bridge binary              git
─────────────────              ─────────────              ───
rebase_interactive_start
  ├ create unix socket at /tmp/...
  ├ spawn `git rebase -i <from>^`
  │   env GIT_SEQUENCE_EDITOR=/path/to/bridge
  │       GIT_EDITOR=/path/to/bridge
  │       TTGIT_SOCK=/tmp/...
  │                              git invokes bridge with
  │                              the todo file as argv[1]
  │                                ├ read file
  │                                ├ open socket
  │                                ├ send { kind, content }
  │                                └ block on reply
  ├ recv socket message
  ├ emit("rebase:edit", payload)
  ↓ frontend opens RebaseTodoDialog
  ↑ user submits
  ├ invoke("rebase:reply", { content })
  ├ write content to todo file
  └ signal bridge to exit
                                  └ exit 0
                                                          continue rebase
                                                          (next prompt or finish)
```

### 7.3 Failure Modes
- **Socket not reachable within 5 s:** bridge exits 0 without modifying the file. Result: git proceeds with the unchanged todo (all `pick`), which is a safe no-op rebase.
- **Conflict mid-rebase:** snapshot's `in_progress` field flips to `Rebasing{...}`. UI shows banner: "Rebase in progress. Resolve conflicts in your editor, then click `[Continue]`." (v1 has no in-app conflict resolver.)
- **User cancels dialog:** frontend replies with sentinel "abort"; backend runs `git rebase --abort` and surfaces a toast.

## 8. Performance Budget

| Operation | Target | Method |
|---|---|---|
| Initial repo snapshot refresh | < 150 ms (≤ 5 k file repo) | `tokio::join!` `git status` / `for-each-ref` / `log -n 200` in parallel |
| Subsequent refresh after action | < 200 ms | Same, with 200 ms debounce |
| RAM for 5 repos open | < 200 MB total | Tauri shared WebKit + per-repo snapshot ~1–5 MB |
| Cold start to usable | < 1 s | No background indexing |

### Known boundaries
- **Monorepos (10 k+ files):** `git status` alone may exceed 500 ms. Mitigation: documentation pointing users at `core.fsmonitor` + `core.untrackedCache` (built into modern git). Same limit applies to WebStorm.
- **Branches > 1 000:** virtualize the branches list (Vue virtual scroller).
- **Log > 1 000 commits:** load first 200, paginate with `--skip` + `-n` on scroll. List virtualized.

## 9. Refresh Strategy

| Trigger | Action |
|---|---|
| Window regains focus | `repo_refresh(active_id)` |
| Any `commit` / `push` / `fetch` / `merge` / `pull` / `rebase*` / `branch_*` command succeeds | `repo_refresh(active_id)` |
| Repo switched in sidebar | `repo_open(new_id)` returns fresh snapshot |

All refreshes are debounced 200 ms in the Rust core; at most one refresh is in flight per repo.

## 10. Error Handling

- Every Tauri command returns `Result<T, GitError>` where `GitError` carries `exit_code`, `stderr` (raw), and `friendly_message` (translated for common cases: `index.lock` exists, no upstream configured, non-fast-forward push, dirty working tree blocking checkout, etc.).
- Frontend renders friendly message in a toast; "Show details" reveals raw stderr.
- A persistent error log panel (collapsed by default) keeps the last 50 failures for diagnosis.

## 11. Testing Strategy

| Layer | Tooling | Coverage |
|---|---|---|
| Rust ops | `cargo test` against fixture repos (`tempfile` + `git init`) | One happy path + one failure path per `ops/*.rs` |
| Porcelain parsing | `cargo test` snapshot tests on captured real `git` outputs | All `FileStatus` variants + rename + conflict |
| Bridge IPC | End-to-end: spawn bridge → send socket message → assert file rewritten | Full rebase-i todo loop |
| Frontend stores | Vitest with mocked `invoke` | Repo switching, refresh debounce, file selection state |
| End-to-end | Playwright driving the built Tauri binary against scratch repos | Open repo → modify files → commit subset → push to local bare remote |

### v1 manual acceptance checklist
- [ ] Switch between `personal/aisync`, `personal/beautiful-css`, `personal/claude-code-main` without lag
- [ ] Modify 5 files; check 2; commit; verify the other 3 remain in working tree
- [ ] With "Skip hooks" checked, `commit-msg` hook (lefthook / husky) does not fire
- [ ] Push to `main` disables force by default; feature branch allows `--force-with-lease`
- [ ] Pull `--rebase` that conflicts surfaces banner with `[Continue]` button
- [ ] Interactive rebase from `HEAD~5`, mark two commits as `squash`, result is correct

## 12. Open Questions

1. **Project final name.** Working name `git-lite` is generic; user to confirm before public release.
2. **Code signing & notarization.** Required for macOS distribution outside the App Store; not blocking v1 local dev builds.
3. **Auto-update channel.** Out of v1; user runs from `cargo tauri build` artifact initially.

## 13. Glossary

- **Snapshot:** A single serializable view of a repo's state (head, files, branches, log, remote status, in-progress op). Always replaces the previous snapshot in full; no diffing.
- **Session:** The Rust-side cache of the most recent snapshot for one repo. One per opened repo, lifetime = while the repo is in the sidebar.
- **Bridge:** The standalone binary invoked by `git` as its editor during interactive rebase; relays todo / commit-message editing into the Tauri UI over a Unix socket.

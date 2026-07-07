# Task 1 Report: Design System Foundation

## 状态

已完成 Task 1，改动范围严格限制在 `package.json`、`pnpm-lock.yaml` 和 `src/style.css`。

## 实施内容

1. 新增 `lucide-vue-next` 依赖，并同步更新 lockfile。
2. 按 brief 原样替换 `src/style.css` 根级深色 token 为 graphite workbench palette。
3. 按 brief 原样替换 `src/style.css` light mode token 为 graphite-compatible light theme。
4. 在 `@layer components` 中新增以下复用类：
   - `gl-app-shell`
   - `gl-panel`
   - `gl-panel-header`
   - `gl-toolbar`
   - `gl-search`
   - `gl-list`
   - `gl-empty`
   - `gl-badge`
   - `gl-status-dot`
   - `gl-command-btn`
   - `gl-kbd`
   - `gl-dialog-shell`
   - `gl-banner`
   - `gl-toast`
5. 保留并对齐现有基础类的视觉体系：
   - `gl-btn`
   - `gl-input`
   - `gl-row`
   - `gl-menu`
   - `gl-chip`
   - `gl-icon-btn`
   - `gl-spinner`
   - `gl-splitter`
   - `gl-overlay`
   - `gl-dialog`
   - `gl-mono`
   - `gl-section-title`

## 验证

- 轻量验证已执行：`./node_modules/.bin/vite --version`
- 结果：`vite/6.4.2 darwin-arm64 node-v24.13.0`

## 说明

- 按 brief 要求没有执行 `pnpm build`。
- 运行 `pnpm add lucide-vue-next` 时，仓库现有 `node_modules` 使用的是旧 pnpm store，直接调用本机 `pnpm` 会报 store version 冲突，因此改为使用兼容的 `npx pnpm@10.34.4 add lucide-vue-next` 完成依赖写入。
- `lucide-vue-next@1.0.0` 上游已标记 deprecated，但本任务明确要求仅引入该包，因此保持 brief 指定实现，不擅自替换为其他图标包。

## 产出文件

- `/Users/hzp/personal/pluck/package.json`
- `/Users/hzp/personal/pluck/pnpm-lock.yaml`
- `/Users/hzp/personal/pluck/src/style.css`

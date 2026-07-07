<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import {
  GitCommitHorizontal,
  Search,
  UserRound,
  X,
} from "lucide-vue-next";
import { useRepoStateStore } from "../stores/repoState";
import { useReposStore } from "../stores/repos";
import { api } from "../api/tauri";
import { formatErr } from "../lib/errors";
import { relativeTime } from "../lib/format";
import type { Commit, RepoSnapshot } from "../types/git";

const state = useRepoStateStore();
const repos = useReposStore();
const allLog = computed(() => state.snapshot?.log ?? []);
const headBranch = computed(() => state.snapshot?.head.branch ?? null);

// 搜索走服务端 `git log --grep` 一次扫全 history（带 hash 前缀单点解析）；
// 浏览态走分页 allLog + 无限滚动。两条路径互不干扰。
const SEARCH_LIMIT = 500;
const SEARCH_DEBOUNCE_MS = 300;

const query = ref("");
const author = ref("");
const hasQuery = computed(() => query.value.trim().length > 0);
const hasAuthor = computed(() => author.value.trim().length > 0);
const hasFilter = computed(() => hasQuery.value || hasAuthor.value);

// 当前 git 用户：从 snapshot.me 拿。空字符串表示 git config 未设。
const me = computed(() => state.snapshot?.me ?? null);
const canPickMe = computed(() => !!me.value?.name);
const isMeSelected = computed(() =>
  !!me.value?.name && author.value.trim() === me.value.name
);
function pickMe() {
  if (me.value?.name) author.value = me.value.name;
}
const searchResults = ref<Commit[] | null>(null);
const searching = ref(false);
const searchError = ref<string | null>(null);
let searchSeq = 0;
let searchTimer: number | null = null;

async function runSearch(q: string, a: string, seq: number) {
  if (!repos.activeId) return;
  searching.value = true;
  searchError.value = null;
  try {
    const res = await api.logSearch(repos.activeId, state.selectedLogBranch, q, a, SEARCH_LIMIT);
    if (seq !== searchSeq) return; // stale
    searchResults.value = res;
  } catch (e: any) {
    if (seq !== searchSeq) return;
    searchError.value = formatErr(e);
    searchResults.value = [];
  } finally {
    if (seq === searchSeq) searching.value = false;
  }
}

watch([query, author], ([q, a]) => {
  const tq = q.trim();
  const ta = a.trim();
  searchSeq++;
  if (searchTimer !== null) { clearTimeout(searchTimer); searchTimer = null; }
  if (!tq && !ta) {
    searchResults.value = null;
    searching.value = false;
    searchError.value = null;
    return;
  }
  // 进入 debounce 窗口立刻置为 searching，避免 0~300ms 内闪 "No match"
  searching.value = true;
  searchError.value = null;
  const seq = searchSeq;
  searchTimer = window.setTimeout(() => { void runSearch(tq, ta, seq); }, SEARCH_DEBOUNCE_MS);
});

const log = computed<Commit[]>(() => {
  if (hasFilter.value) return searchResults.value ?? [];
  return allLog.value;
});
const searchHitCap = computed(() =>
  hasFilter.value && (searchResults.value?.length ?? 0) >= SEARCH_LIMIT
);
const onCurrentBranchLog = computed(() =>
  headBranch.value !== null && state.selectedLogBranch === headBranch.value
);

const menu = ref<{ x: number; y: number; commit: Commit } | null>(null);
window.addEventListener("click", () => menu.value = null);

function isSelected(hash: string) { return state.selectedHashes.has(hash); }

function onCommitClick(e: MouseEvent, c: Commit) {
  if (!repos.activeId) return;
  if (e.shiftKey) {
    state.selectRange(log.value, c.hash);
    return;
  }
  if (e.metaKey || e.ctrlKey) {
    state.toggleSelection(c.hash);
    return;
  }
  state.setSingleSelection(repos.activeId, c.hash);
}

function onContext(e: MouseEvent, c: Commit) {
  if (!repos.activeId) return;
  if (!isSelected(c.hash)) state.setSingleSelection(repos.activeId, c.hash);
  menu.value = { x: e.clientX, y: e.clientY, commit: c };
}

const selectedCount = computed(() => state.selectionCount);

// Selected commits in log order (oldest → newest replay order).
const selectedInOrder = computed<Commit[]>(() => {
  const set = state.selectedHashes;
  return log.value.filter(c => set.has(c.hash)).slice().reverse();
});

const singleSelected = computed<Commit | null>(() => {
  if (state.selectionCount !== 1) return null;
  return log.value.find(c => state.selectedHashes.has(c.hash)) ?? null;
});

// Edit Message enablement: single-select, non-merge, reachable from HEAD
// (we approximate "reachable from HEAD" as: viewing the current HEAD branch's log).
const canEditMessage = computed(() => {
  const c = singleSelected.value;
  if (!c) return false;
  if (c.parents.length !== 1) return false;
  return onCurrentBranchLog.value;
});

const editMessageMode = computed<"amend" | "reword" | null>(() => {
  const c = singleSelected.value;
  if (!c || !canEditMessage.value) return null;
  return log.value[0]?.hash === c.hash ? "amend" : "reword";
});

const canReset = computed(() => state.selectionCount === 1 && onCurrentBranchLog.value);

async function doCherryPick() {
  if (!repos.activeId || selectedCount.value === 0) return;
  const id = repos.activeId;
  const hashes = selectedInOrder.value.map(c => c.hash);
  menu.value = null;
  try { state.snapshot = await api.cherryPick(id, hashes); }
  catch (e: any) { state.pushToast("error", formatErr(e)); }
}

async function doRevert() {
  if (!repos.activeId || selectedCount.value === 0) return;
  const id = repos.activeId;
  // Revert wants newest → oldest so the working tree stays sane.
  const hashes = selectedInOrder.value.map(c => c.hash).reverse();
  menu.value = null;
  try { state.snapshot = await api.revert(id, hashes); }
  catch (e: any) { state.pushToast("error", formatErr(e)); }
}

function doEditMessage() {
  const c = singleSelected.value;
  const mode = editMessageMode.value;
  if (!c || !mode) return;
  const initial = c.body ? `${c.subject}\n\n${c.body}` : c.subject;
  menu.value = null;
  state.openEditMessageDialog(c.hash, initial, mode);
}

function doReset() {
  const c = singleSelected.value;
  if (!c) return;
  menu.value = null;
  state.openResetDialog(c.hash, c.short, c.subject);
}

async function interactiveRebase() {
  if (!menu.value || !repos.activeId || !onCurrentBranchLog.value) return;
  const id = repos.activeId; const from = menu.value.commit.hash;
  menu.value = null;
  try { state.snapshot = await invoke<RepoSnapshot>("rebase_interactive_start", { id, fromCommit: from }); }
  catch (e: any) { state.pushToast("error", formatErr(e)); }
}

function authorInitial(name: string) {
  return name.trim().split(/\s+/).map(w => w[0]).slice(0, 2).join("").toUpperCase() || "·";
}
const graphPalette = [
  "var(--graph-1)", "var(--graph-2)", "var(--graph-3)", "var(--graph-4)",
  "var(--graph-5)", "var(--graph-6)", "var(--graph-7)", "var(--graph-8)",
];
function authorColor(name: string) {
  let h = 0; for (const c of name) h = (h * 31 + c.charCodeAt(0)) >>> 0;
  return graphPalette[h % graphPalette.length];
}

// === infinite scroll ===
// 用 scroll 事件而不是 IntersectionObserver：nested scroll container 上
// observer 的 root 边界判断、refresh 后元素复用都踩过坑，scroll 事件简单可控。
const scrollRoot = ref<HTMLElement | null>(null);
let scrollRaf = 0;
function onScroll() {
  if (hasFilter.value) return; // 搜索/筛选态结果是一次性 grep，不分页
  if (scrollRaf) return;
  scrollRaf = requestAnimationFrame(() => {
    scrollRaf = 0;
    const el = scrollRoot.value;
    if (!el || !repos.activeId) return;
    if (el.scrollHeight - el.scrollTop - el.clientHeight < 200) {
      void state.loadMoreLog(repos.activeId);
    }
  });
}
</script>

<template>
  <div class="flex flex-col h-full">
    <div class="gl-panel-header">
      <GitCommitHorizontal :size="15" style="color: var(--accent)" />
      <span class="gl-section-title">History</span>
      <span class="gl-badge">{{ state.selectedLogBranch ?? "—" }}</span>
      <div class="flex-1" />
      <span v-if="selectedCount > 1" class="gl-badge" style="color: var(--accent-2)">
        {{ selectedCount }} selected
      </span>
    </div>
    <div class="px-3 py-2 flex items-center gap-2" style="border-bottom: 1px solid var(--border-soft)">
      <div class="gl-search flex-[3] min-w-0">
        <Search class="absolute left-2 pointer-events-none top-1/2 -translate-y-1/2"
                :size="14"
                style="color: var(--fg-3)" />
        <input v-model="query" type="text" placeholder="Search hash or message…"
               class="gl-input w-full pl-8 pr-8" />
        <button v-if="hasQuery" @click="query = ''"
                class="gl-icon-btn absolute right-1 top-1/2 -translate-y-1/2"
                title="Clear">
          <X :size="13" />
        </button>
      </div>
      <div class="gl-search flex-[2] min-w-0">
        <UserRound class="absolute left-2 pointer-events-none top-1/2 -translate-y-1/2"
                   :size="14"
                   style="color: var(--fg-3)" />
        <input v-model="author" type="text" placeholder="Author…"
               class="gl-input w-full pl-8" :class="canPickMe ? 'pr-14' : 'pr-8'" />
        <button v-if="canPickMe && !isMeSelected"
                @click="pickMe"
                class="absolute top-1/2 -translate-y-1/2 gl-badge"
                :class="hasAuthor ? 'right-7' : 'right-2'"
                :title="`Filter by me (${me?.name})`">me</button>
        <button v-if="hasAuthor" @click="author = ''"
                class="gl-icon-btn absolute right-1 top-1/2 -translate-y-1/2"
                title="Clear">
          <X :size="13" />
        </button>
      </div>
    </div>
    <ul ref="scrollRoot" @scroll.passive="onScroll" class="flex-1 overflow-auto px-2 flex flex-col gap-0.5">
      <li v-for="c in log" :key="c.hash"
          @click="onCommitClick($event, c)"
          @contextmenu.prevent="onContext($event, c)"
          :title="c.subject"
          class="flex items-center gap-2.5 px-2.5 h-9 rounded-md cursor-pointer transition-colors"
          :class="{ 'gl-row-active': isSelected(c.hash) }"
          @mouseover="(e: any) => { if (!isSelected(c.hash)) e.currentTarget.style.background = 'var(--hover)' }"
          @mouseleave="(e: any) => { if (!isSelected(c.hash)) e.currentTarget.style.background = '' }">
        <span class="gl-badge shrink-0">{{ c.short }}</span>
        <span class="inline-flex items-center justify-center w-5 h-5 rounded-full text-[10px] font-semibold shrink-0"
              :style="{ background: authorColor(c.author), color: 'var(--on-graph)' }"
              :title="c.author">{{ authorInitial(c.author) }}</span>
        <span class="flex-1 truncate text-[13px]" style="color: var(--fg)">{{ c.subject }}</span>
        <span class="text-[12px] shrink-0" style="color: var(--fg-3)">{{ relativeTime(c.dateUnix) }}</span>
      </li>
      <li v-if="log.length === 0 && !hasFilter" class="gl-empty">
        <GitCommitHorizontal :size="24" />
        <span class="text-[13px]">No commits</span>
      </li>
      <li v-else-if="hasFilter && searching" class="gl-empty">
        <span class="gl-spinner" />
        <span class="text-[12px]">Searching…</span>
      </li>
      <li v-else-if="hasFilter && searchError" class="gl-empty" style="color: var(--danger)">
        <span class="text-[13px]">Search failed</span>
        <span class="text-[12px]">{{ searchError }}</span>
      </li>
      <li v-else-if="hasFilter && log.length === 0" class="gl-empty">
        <Search :size="22" />
        <span class="text-[13px]">No match</span>
      </li>
      <li v-else-if="hasFilter"
          class="flex items-center justify-center py-3 text-[12px]"
          style="color: var(--fg-3)">
        <span v-if="searchHitCap">Showing first {{ log.length }} matches · refine filter for more</span>
        <span v-else>{{ log.length }} match{{ log.length === 1 ? "" : "es" }}</span>
      </li>
      <li v-else-if="!state.logEnd"
          class="flex items-center justify-center py-3 text-[12px]"
          style="color: var(--fg-3)">
        <span v-if="state.logLoadingMore" class="gl-spinner mr-2" />
        <span>{{ state.logLoadingMore ? "Loading more…" : "Scroll for more" }}</span>
      </li>
      <li v-else-if="allLog.length >= 200"
          class="flex items-center justify-center py-3 text-[12px]"
          style="color: var(--fg-3)">
        End of history · {{ allLog.length }} commits
      </li>
    </ul>
    <div v-if="menu" :style="{ top: menu.y + 'px', left: menu.x + 'px' }" class="gl-menu">
      <button class="gl-menu-item" @click="doCherryPick">
        Cherry-Pick{{ selectedCount > 1 ? ` ${selectedCount} commits` : "" }}
      </button>
      <button class="gl-menu-item" @click="doRevert">
        Revert{{ selectedCount > 1 ? ` ${selectedCount} commits` : " Commit" }}
      </button>
      <button class="gl-menu-item"
              :disabled="!canEditMessage"
              :title="canEditMessage ? '' : 'Only single non-merge commits on the current branch can be edited'"
              @click="doEditMessage">
        Edit Commit Message…
      </button>
      <button class="gl-menu-item"
              :disabled="!canReset"
              :title="canReset ? '' : 'Select a single commit on the current branch'"
              @click="doReset">
        Reset Current Branch to Here…
      </button>
      <div class="gl-menu-sep" />
      <button class="gl-menu-item"
              :disabled="!onCurrentBranchLog"
              :title="onCurrentBranchLog ? '' : 'Switch the log to the current branch first'"
              @click="interactiveRebase">
        Interactively rebase from here…
      </button>
    </div>
  </div>
</template>

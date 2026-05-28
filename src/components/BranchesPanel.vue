<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount, watch } from "vue";
import { useRepoStateStore } from "../stores/repoState";
import { useReposStore } from "../stores/repos";
import { useBranchPrefsStore } from "../stores/branchPrefs";
import type { Branch, RepoSnapshot } from "../types/git";
import { ops } from "../api/tauri";
import { invoke } from "@tauri-apps/api/core";
import { buildTree, type TreeEntry } from "../lib/branchTree";

const state = useRepoStateStore();
const repos = useReposStore();
const prefs = useBranchPrefsStore();

const showPinned = ref(true);
const showLocal = ref(true);
const showRemote = ref(true);
const search = ref("");
const searchInput = ref<HTMLInputElement | null>(null);

const menu = ref<{ x: number; y: number; branch: Branch } | null>(null);

const repoId = computed(() => repos.activeId ?? "");
const allLocal = computed<Branch[]>(() => state.snapshot?.branches.local ?? []);
const allRemote = computed<Branch[]>(() => state.snapshot?.branches.remote ?? []);

const query = computed(() => search.value.trim().toLowerCase());
const isSearching = computed(() => query.value.length > 0);
function matches(b: Branch): boolean {
  if (!isSearching.value) return true;
  return b.name.toLowerCase().includes(query.value);
}

const pinnedBranches = computed<Branch[]>(() => {
  const set = new Set(prefs.pinned(repoId.value));
  return [...allLocal.value, ...allRemote.value].filter(b => set.has(b.name) && matches(b));
});
const unpinnedLocal = computed<Branch[]>(() => {
  const set = new Set(prefs.pinned(repoId.value));
  return allLocal.value.filter(b => !set.has(b.name) && matches(b));
});
const unpinnedRemote = computed<Branch[]>(() => {
  const set = new Set(prefs.pinned(repoId.value));
  return allRemote.value.filter(b => !set.has(b.name) && matches(b));
});

// During search the persisted collapse state is bypassed (default open),
// but the user can still toggle individual folders via a local override
// that lives only for the duration of the search.
const searchCollapsed = ref<Set<string>>(new Set());
watch(isSearching, v => { if (!v) searchCollapsed.value = new Set(); });

function collapseLookup(scope: "local" | "remote") {
  return (p: string) => {
    const key = `${scope}:${p}`;
    return isSearching.value
      ? searchCollapsed.value.has(key)
      : prefs.isCollapsed(repoId.value, key);
  };
}

const localTree = computed<TreeEntry[]>(() => buildTree(unpinnedLocal.value, collapseLookup("local")));
const remoteTree = computed<TreeEntry[]>(() => buildTree(unpinnedRemote.value, collapseLookup("remote")));

const totalMatches = computed(() =>
  pinnedBranches.value.length + unpinnedLocal.value.length + unpinnedRemote.value.length
);

function onKeydown(e: KeyboardEvent) {
  if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "f" && !e.shiftKey) {
    e.preventDefault();
    searchInput.value?.focus();
    searchInput.value?.select();
  } else if (e.key === "Escape" && document.activeElement === searchInput.value) {
    search.value = "";
    searchInput.value?.blur();
  }
}
onMounted(() => window.addEventListener("keydown", onKeydown));
onBeforeUnmount(() => window.removeEventListener("keydown", onKeydown));

function pickForLog(b: Branch) {
  if (!repos.activeId) return;
  state.setLogBranch(repos.activeId, b.name);
}
function togglePin(e: MouseEvent, b: Branch) {
  e.stopPropagation();
  if (!repos.activeId) return;
  prefs.togglePin(repos.activeId, b.name);
}
function toggleFolder(scope: "local" | "remote", prefix: string) {
  if (!repos.activeId) return;
  const key = `${scope}:${prefix}`;
  if (isSearching.value) {
    const next = new Set(searchCollapsed.value);
    if (next.has(key)) next.delete(key); else next.add(key);
    searchCollapsed.value = next;
  } else {
    prefs.toggleCollapse(repos.activeId, key);
  }
}

function onContext(e: MouseEvent, b: Branch) {
  menu.value = { x: e.clientX, y: e.clientY, branch: b };
}

async function checkout() {
  if (!menu.value || !repos.activeId) return;
  const id = repos.activeId, name = menu.value.branch.name; menu.value = null;
  try { state.snapshot = await ops.branchCheckout(id, name); }
  catch (e: any) { state.pushToast("error", e?.data?.friendly ?? String(e)); }
}
function newFromHere() {
  if (!menu.value || !repos.activeId) return;
  const from = menu.value.branch.name; menu.value = null;
  state.openBranchCreateDialog(from);
}
function del() {
  if (!menu.value || !repos.activeId) return;
  const name = menu.value.branch.name; menu.value = null;
  state.openBranchDeleteDialog(name);
}
async function mergeIntoCurrent() {
  if (!menu.value || !repos.activeId) return;
  const id = repos.activeId, name = menu.value.branch.name; menu.value = null;
  try { state.snapshot = await ops.merge(id, name); }
  catch (e: any) { state.pushToast("error", e?.data?.friendly ?? String(e)); }
}
async function pullIntoCurrentRebase() {
  if (!menu.value || !repos.activeId) return;
  const id = repos.activeId, source = menu.value.branch.name; menu.value = null;
  try { state.snapshot = await invoke<RepoSnapshot>("pull_into_current_rebase", { id, source }); }
  catch (e: any) { state.pushToast("error", e?.data?.friendly ?? String(e)); }
}
const currentBranchName = computed(() => state.snapshot?.head.branch ?? null);
function toggleMenuPin() {
  if (!menu.value || !repos.activeId) return;
  prefs.togglePin(repos.activeId, menu.value.branch.name);
  menu.value = null;
}

window.addEventListener("click", () => (menu.value = null));
</script>

<template>
  <div class="flex flex-col p-2 gap-1">
    <!-- Search -->
    <div class="sticky top-0 z-10 pb-1 -mx-2 px-2 pt-0"
         style="background: var(--panel)">
      <div class="relative">
        <span class="absolute left-2.5 top-1/2 -translate-y-1/2 pointer-events-none inline-flex"
              style="color: var(--fg-3)">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor"
               stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="11" cy="11" r="7" />
            <path d="m20 20-3.5-3.5" />
          </svg>
        </span>
        <input ref="searchInput" v-model="search" type="text"
               placeholder="Search branches  ⌘F"
               class="gl-input pl-8 pr-7 py-1.5 text-[13px] h-8"
               style="border-radius: 6px" />
        <button v-if="search" @click="search = ''"
                class="absolute right-2 top-1/2 -translate-y-1/2 text-[13px] transition-colors"
                style="color: var(--fg-3)"
                @mouseover="(e: any) => (e.currentTarget.style.color = 'var(--fg)')"
                @mouseleave="(e: any) => (e.currentTarget.style.color = 'var(--fg-3)')"
                title="Clear (Esc)">✕</button>
      </div>
      <div v-if="isSearching" class="text-[11px] mt-1 px-0.5"
           style="color: var(--fg-3)">
        {{ totalMatches }} match{{ totalMatches === 1 ? "" : "es" }}
      </div>
    </div>

    <!-- No matches -->
    <div v-if="isSearching && totalMatches === 0"
         class="flex flex-col items-center justify-center py-6 gap-1 text-center"
         style="color: var(--fg-3)">
      <span class="text-2xl">∅</span>
      <span class="text-[13px]">No branch matches "{{ search }}"</span>
    </div>

    <!-- Pinned -->
    <template v-if="pinnedBranches.length">
      <button class="flex items-center gap-1.5 px-2 py-1 rounded-md transition-colors"
              style="color: var(--fg-3)"
              @click="showPinned = !showPinned"
              @mouseover="(e: any) => (e.currentTarget.style.color = 'var(--fg)')"
              @mouseleave="(e: any) => (e.currentTarget.style.color = 'var(--fg-3)')">
        <span class="text-[11px] transition-transform"
              :style="{ transform: showPinned ? 'rotate(90deg)' : 'rotate(0)' }">▶</span>
        <span class="gl-section-title">Pinned</span>
        <span class="ml-auto gl-mono text-[11px]" style="color: var(--fg-3)">{{ pinnedBranches.length }}</span>
      </button>
      <ul v-if="showPinned" class="flex flex-col gap-0.5">
        <li v-for="b in pinnedBranches" :key="'pin:' + b.name"
            @click="pickForLog(b)"
            @contextmenu.prevent="onContext($event, b)"
            :title="b.name"
            class="gl-row group"
            :class="{ 'is-selected': b.name === state.selectedLogBranch }">
          <span class="w-3 inline-flex justify-center"
                :style="{ color: b.isCurrent ? 'var(--success)' : 'transparent' }">●</span>
          <button @click="togglePin($event, b)" title="Unpin"
                  class="text-[12px] transition-colors"
                  style="color: var(--warning)">★</button>
          <span class="truncate flex-1 text-[13.5px]"
                :style="b.isCurrent ? 'font-weight: 600' : ''">{{ b.name }}</span>
          <span v-if="b.ahead" class="gl-chip gl-chip-ahead">↑{{ b.ahead }}</span>
          <span v-if="b.behind" class="gl-chip gl-chip-behind">↓{{ b.behind }}</span>
        </li>
      </ul>
    </template>

    <!-- Local -->
    <button class="flex items-center gap-1.5 px-2 py-1 rounded-md transition-colors mt-1"
            style="color: var(--fg-3)"
            @click="showLocal = !showLocal"
            @mouseover="(e: any) => (e.currentTarget.style.color = 'var(--fg)')"
            @mouseleave="(e: any) => (e.currentTarget.style.color = 'var(--fg-3)')">
      <span class="text-[11px] transition-transform"
            :style="{ transform: showLocal ? 'rotate(90deg)' : 'rotate(0)' }">▶</span>
      <span class="gl-section-title">Local</span>
      <span class="ml-auto gl-mono text-[11px]" style="color: var(--fg-3)">{{ unpinnedLocal.length }}</span>
    </button>
    <ul v-if="showLocal" class="flex flex-col gap-0.5">
      <template v-for="entry in localTree" :key="'l:' + (entry.kind === 'folder' ? entry.prefix : entry.branch.name)">
        <!-- folder -->
        <li v-if="entry.kind === 'folder'"
            @click="toggleFolder('local', entry.prefix)"
            @contextmenu.prevent="entry.selfBranch && onContext($event, entry.selfBranch)"
            :title="entry.prefix"
            class="gl-row group">
          <span :style="{ paddingLeft: (entry.depth * 12) + 'px' }" class="inline-flex" />
          <span class="text-[11px] w-3 inline-flex justify-center transition-transform"
                style="color: var(--fg-3)"
                :style="{ transform: entry.collapsed ? 'rotate(0)' : 'rotate(90deg)' }">▶</span>
          <span class="text-[13px]" style="color: var(--accent-2)">▦</span>
          <span class="truncate flex-1 text-[13.5px]"
                :style="entry.selfBranch?.isCurrent ? 'font-weight: 600; color: var(--fg)' : 'color: var(--fg)'">
            {{ entry.label }}<span v-if="entry.selfBranch" class="gl-mono text-[11px] ml-1"
                                   style="color: var(--fg-3)">·branch</span>
          </span>
          <span class="gl-chip">{{ entry.childCount }}</span>
          <span v-if="entry.collapsed && entry.ahead" class="gl-chip gl-chip-ahead">↑{{ entry.ahead }}</span>
          <span v-if="entry.collapsed && entry.behind" class="gl-chip gl-chip-behind">↓{{ entry.behind }}</span>
        </li>
        <!-- branch leaf -->
        <li v-else
            @click="pickForLog(entry.branch)"
            @contextmenu.prevent="onContext($event, entry.branch)"
            :title="entry.branch.name"
            class="gl-row group"
            :class="{ 'is-selected': entry.branch.name === state.selectedLogBranch }">
          <span :style="{ paddingLeft: (entry.depth * 12) + 'px' }" class="inline-flex" />
          <span class="w-3 inline-flex justify-center"
                :style="{ color: entry.branch.isCurrent ? 'var(--success)' : 'transparent' }">●</span>
          <button @click="togglePin($event, entry.branch)" title="Pin"
                  class="text-[12px] opacity-0 group-hover:opacity-60 hover:!opacity-100 transition-opacity"
                  style="color: var(--fg-3)">☆</button>
          <span class="truncate flex-1 text-[13.5px]"
                :style="entry.branch.isCurrent ? 'font-weight: 600' : ''">{{ entry.displayLabel }}</span>
          <span v-if="entry.branch.ahead" class="gl-chip gl-chip-ahead">↑{{ entry.branch.ahead }}</span>
          <span v-if="entry.branch.behind" class="gl-chip gl-chip-behind">↓{{ entry.branch.behind }}</span>
        </li>
      </template>
    </ul>

    <!-- Remote -->
    <button class="flex items-center gap-1.5 px-2 py-1 rounded-md transition-colors mt-2"
            style="color: var(--fg-3)"
            @click="showRemote = !showRemote"
            @mouseover="(e: any) => (e.currentTarget.style.color = 'var(--fg)')"
            @mouseleave="(e: any) => (e.currentTarget.style.color = 'var(--fg-3)')">
      <span class="text-[11px] transition-transform"
            :style="{ transform: showRemote ? 'rotate(90deg)' : 'rotate(0)' }">▶</span>
      <span class="gl-section-title">Remote</span>
      <span class="ml-auto gl-mono text-[11px]" style="color: var(--fg-3)">{{ unpinnedRemote.length }}</span>
    </button>
    <ul v-if="showRemote" class="flex flex-col gap-0.5">
      <template v-for="entry in remoteTree" :key="'r:' + (entry.kind === 'folder' ? entry.prefix : entry.branch.name)">
        <li v-if="entry.kind === 'folder'"
            @click="toggleFolder('remote', entry.prefix)"
            :title="entry.prefix"
            class="gl-row group" style="cursor: pointer">
          <span :style="{ paddingLeft: (entry.depth * 12) + 'px' }" class="inline-flex" />
          <span class="text-[11px] w-3 inline-flex justify-center transition-transform"
                style="color: var(--fg-3)"
                :style="{ transform: entry.collapsed ? 'rotate(0)' : 'rotate(90deg)' }">▶</span>
          <span class="text-[13px]" style="color: var(--fg-3)">▦</span>
          <span class="truncate flex-1 text-[13.5px]" style="color: var(--fg-2)">{{ entry.label }}</span>
          <span class="gl-chip">{{ entry.childCount }}</span>
        </li>
        <li v-else
            @contextmenu.prevent="onContext($event, entry.branch)"
            :title="entry.branch.name"
            class="gl-row group" style="cursor: default">
          <span :style="{ paddingLeft: (entry.depth * 12) + 'px' }" class="inline-flex" />
          <span class="w-3 inline-flex justify-center" style="color: var(--fg-3)">⬡</span>
          <button @click="togglePin($event, entry.branch)" title="Pin"
                  class="text-[12px] opacity-0 group-hover:opacity-60 hover:!opacity-100 transition-opacity"
                  style="color: var(--fg-3)">☆</button>
          <span class="truncate flex-1 text-[13.5px]" style="color: var(--fg-2)">{{ entry.displayLabel }}</span>
        </li>
      </template>
    </ul>

    <!-- Context menu -->
    <div v-if="menu" :style="{ top: menu.y + 'px', left: menu.x + 'px' }" class="gl-menu">
      <button class="gl-menu-item" @click="toggleMenuPin">
        {{ repoId && prefs.isPinned(repoId, menu.branch.name) ? "★ Unpin" : "☆ Pin to top" }}
      </button>
      <div class="my-1 h-px" style="background: var(--border)"></div>
      <button class="gl-menu-item" @click="checkout" :disabled="menu.branch.isCurrent">
        {{ menu.branch.kind === "remote" ? "Checkout as local branch" : "Checkout" }}
      </button>
      <button class="gl-menu-item" @click="newFromHere">New branch from here…</button>
      <button class="gl-menu-item" @click="mergeIntoCurrent" :disabled="menu.branch.isCurrent">Merge into current</button>
      <button class="gl-menu-item"
              @click="pullIntoCurrentRebase"
              :disabled="menu.branch.isCurrent || !currentBranchName">
        Pull into "{{ currentBranchName ?? "current" }}" using rebase
      </button>
      <div class="my-1 h-px" style="background: var(--border)"></div>
      <button class="gl-menu-item is-danger" @click="del" :disabled="menu.branch.isCurrent">Delete</button>
    </div>
  </div>
</template>

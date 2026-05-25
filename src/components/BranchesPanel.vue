<script setup lang="ts">
import { ref } from "vue";
import { useRepoStateStore } from "../stores/repoState";
import { useReposStore } from "../stores/repos";
import type { Branch, RepoSnapshot } from "../types/git";
import { ops } from "../api/tauri";
import { invoke } from "@tauri-apps/api/core";

const state = useRepoStateStore();
const repos = useReposStore();
const showLocal = ref(true);
const showRemote = ref(true);

const menu = ref<{ x: number; y: number; branch: Branch } | null>(null);

function pickForLog(b: Branch) {
  if (!repos.activeId) return;
  state.setLogBranch(repos.activeId, b.name);
}

function onContext(e: MouseEvent, b: Branch) {
  menu.value = { x: e.clientX, y: e.clientY, branch: b };
}

async function checkout() {
  if (!menu.value || !repos.activeId) return;
  const id = repos.activeId;
  const name = menu.value.branch.name;
  menu.value = null;
  try { state.snapshot = await ops.branchCheckout(id, name); }
  catch (e: any) { state.pushToast("error", e?.data?.friendly ?? String(e)); }
}

async function newFromHere() {
  if (!menu.value || !repos.activeId) return;
  const name = prompt("New branch name:")?.trim();
  if (!name) { menu.value = null; return; }
  const id = repos.activeId; const from = menu.value.branch.name;
  menu.value = null;
  try { state.snapshot = await ops.branchCreate(id, name, from); }
  catch (e: any) { state.pushToast("error", e?.data?.friendly ?? String(e)); }
}
async function del() {
  if (!menu.value || !repos.activeId) return;
  const id = repos.activeId; const name = menu.value.branch.name;
  menu.value = null;
  if (!confirm(`Delete branch ${name}?`)) return;
  try { state.snapshot = await ops.branchDelete(id, name, false); }
  catch (e: any) {
    if (confirm(`${e?.data?.friendly ?? e}\n\nForce delete?`)) {
      try { state.snapshot = await ops.branchDelete(id, name, true); }
      catch (e2: any) { state.pushToast("error", e2?.data?.friendly ?? String(e2)); }
    }
  }
}

async function mergeIntoCurrent() {
  if (!menu.value || !repos.activeId) return;
  const id = repos.activeId; const name = menu.value.branch.name;
  menu.value = null;
  try { state.snapshot = await ops.merge(id, name); }
  catch (e: any) { state.pushToast("error", e?.data?.friendly ?? String(e)); }
}

async function pullInto() {
  if (!menu.value || !repos.activeId) return;
  const id = repos.activeId; const name = menu.value.branch.name;
  menu.value = null;
  try { state.snapshot = await invoke<RepoSnapshot>("pull", { id, targetBranch: name }); }
  catch (e: any) { state.pushToast("error", e?.data?.friendly ?? String(e)); }
}

window.addEventListener("click", () => (menu.value = null));
</script>

<template>
  <div class="flex flex-col p-2 gap-1">
    <button class="flex items-center gap-1.5 px-2 py-1 rounded-md transition-colors"
            style="color: var(--fg-3)"
            @click="showLocal = !showLocal"
            @mouseover="(e: any) => (e.currentTarget.style.color = 'var(--fg)')"
            @mouseleave="(e: any) => (e.currentTarget.style.color = 'var(--fg-3)')">
      <span class="text-[10px] transition-transform" :style="{ transform: showLocal ? 'rotate(90deg)' : 'rotate(0)' }">▶</span>
      <span class="gl-section-title">Local</span>
      <span class="ml-auto gl-mono text-[10px]" style="color: var(--fg-3)">{{ state.snapshot?.branches.local.length ?? 0 }}</span>
    </button>
    <ul v-if="showLocal" class="flex flex-col gap-0.5">
      <li v-for="b in state.snapshot?.branches.local ?? []" :key="b.name"
          @click="pickForLog(b)"
          @contextmenu.prevent="onContext($event, b)"
          class="gl-row"
          :class="{ 'is-selected': b.name === state.selectedLogBranch }">
        <span class="w-3 inline-flex justify-center"
              :style="{ color: b.isCurrent ? 'var(--success)' : 'transparent' }">●</span>
        <span class="truncate flex-1 text-[13px]"
              :style="b.isCurrent ? 'font-weight: 600' : ''">{{ b.name }}</span>
        <span v-if="b.ahead" class="gl-chip gl-chip-ahead">↑{{ b.ahead }}</span>
        <span v-if="b.behind" class="gl-chip gl-chip-behind">↓{{ b.behind }}</span>
      </li>
    </ul>

    <button class="flex items-center gap-1.5 px-2 py-1 rounded-md transition-colors mt-2"
            style="color: var(--fg-3)"
            @click="showRemote = !showRemote"
            @mouseover="(e: any) => (e.currentTarget.style.color = 'var(--fg)')"
            @mouseleave="(e: any) => (e.currentTarget.style.color = 'var(--fg-3)')">
      <span class="text-[10px] transition-transform" :style="{ transform: showRemote ? 'rotate(90deg)' : 'rotate(0)' }">▶</span>
      <span class="gl-section-title">Remote</span>
      <span class="ml-auto gl-mono text-[10px]" style="color: var(--fg-3)">{{ state.snapshot?.branches.remote.length ?? 0 }}</span>
    </button>
    <ul v-if="showRemote" class="flex flex-col gap-0.5">
      <li v-for="b in state.snapshot?.branches.remote ?? []" :key="b.name"
          class="gl-row" style="cursor: default">
        <span class="w-3 inline-flex justify-center" style="color: var(--fg-3)">⬡</span>
        <span class="truncate flex-1 text-[13px]" style="color: var(--fg-2)">{{ b.name }}</span>
      </li>
    </ul>

    <div v-if="menu" :style="{ top: menu.y + 'px', left: menu.x + 'px' }" class="gl-menu">
      <button class="gl-menu-item" @click="checkout" :disabled="menu.branch.isCurrent">Checkout</button>
      <button class="gl-menu-item" @click="newFromHere">New branch from here…</button>
      <button class="gl-menu-item" @click="mergeIntoCurrent" :disabled="menu.branch.isCurrent">Merge into current</button>
      <button class="gl-menu-item" @click="pullInto">Pull --rebase</button>
      <div class="my-1 h-px" style="background: var(--border)"></div>
      <button class="gl-menu-item is-danger" @click="del" :disabled="menu.branch.isCurrent">Delete</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { useRepoStateStore } from "../stores/repoState";
import { useReposStore } from "../stores/repos";
import type { Branch } from "../types/git";
import { ops } from "../api/tauri";

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
  try {
    state.snapshot = await ops.branchCheckout(id, name);
  } catch (e: any) {
    state.lastError = e?.data?.friendly ?? String(e);
  }
}

window.addEventListener("click", () => (menu.value = null));
</script>

<template>
  <div class="flex flex-col text-sm">
    <button class="text-left px-2 py-1 font-semibold flex items-center gap-1" @click="showLocal = !showLocal">
      <span>{{ showLocal ? "▾" : "▸" }}</span> Local
    </button>
    <ul v-if="showLocal" class="pl-2">
      <li v-for="b in state.snapshot?.branches.local ?? []" :key="b.name"
          @click="pickForLog(b)"
          @contextmenu.prevent="onContext($event, b)"
          :class="['px-2 py-0.5 cursor-pointer rounded hover:bg-neutral-200 dark:hover:bg-neutral-800 flex items-center gap-1',
                   b.name === state.selectedLogBranch ? 'bg-blue-100 dark:bg-blue-900/40' : '']">
        <span :class="b.isCurrent ? 'text-emerald-600 font-semibold' : ''">{{ b.isCurrent ? "●" : " " }}</span>
        <span class="truncate flex-1">{{ b.name }}</span>
        <span class="opacity-60 text-xs" v-if="b.ahead || b.behind">↑{{ b.ahead }} ↓{{ b.behind }}</span>
      </li>
    </ul>
    <button class="text-left px-2 py-1 font-semibold flex items-center gap-1" @click="showRemote = !showRemote">
      <span>{{ showRemote ? "▾" : "▸" }}</span> Remote
    </button>
    <ul v-if="showRemote" class="pl-2">
      <li v-for="b in state.snapshot?.branches.remote ?? []" :key="b.name"
          class="px-2 py-0.5 truncate opacity-90 hover:bg-neutral-200 dark:hover:bg-neutral-800 rounded">
        {{ b.name }}
      </li>
    </ul>
    <div v-if="menu" :style="{ top: menu.y + 'px', left: menu.x + 'px' }"
         class="fixed z-50 bg-white dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700 rounded shadow text-sm min-w-40">
      <button class="block w-full text-left px-3 py-1 hover:bg-neutral-100 dark:hover:bg-neutral-700"
              @click="checkout" :disabled="menu.branch.isCurrent">Checkout</button>
    </div>
  </div>
</template>

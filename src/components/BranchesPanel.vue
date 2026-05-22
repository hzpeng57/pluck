<script setup lang="ts">
import { ref } from "vue";
import { useRepoStateStore } from "../stores/repoState";
import { useReposStore } from "../stores/repos";
import type { Branch } from "../types/git";

const state = useRepoStateStore();
const repos = useReposStore();
const showLocal = ref(true);
const showRemote = ref(true);

function pickForLog(b: Branch) {
  if (!repos.activeId) return;
  state.setLogBranch(repos.activeId, b.name);
}
</script>

<template>
  <div class="flex flex-col text-sm">
    <button class="text-left px-2 py-1 font-semibold flex items-center gap-1" @click="showLocal = !showLocal">
      <span>{{ showLocal ? "▾" : "▸" }}</span> Local
    </button>
    <ul v-if="showLocal" class="pl-2">
      <li v-for="b in state.snapshot?.branches.local ?? []" :key="b.name"
          @click="pickForLog(b)"
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
  </div>
</template>

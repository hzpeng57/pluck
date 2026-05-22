<script setup lang="ts">
import { open } from "@tauri-apps/plugin-dialog";
import { useReposStore } from "../stores/repos";
import { api } from "../api/tauri";

const repos = useReposStore();

async function addRepo() {
  const dir = await open({ directory: true, multiple: false });
  if (typeof dir !== "string") return;
  try {
    const meta = await api.repoAdd(dir);
    repos.add(meta);
  } catch (e: any) { alert(e?.data?.friendly ?? String(e)); }
}
</script>

<template>
  <div class="flex flex-col p-2 gap-1">
    <div class="text-xs uppercase opacity-50 px-1 pb-1">Repos</div>
    <button
      v-for="r in repos.all" :key="r.id"
      @click="repos.setActive(r.id)"
      :class="['text-left px-2 py-1 rounded truncate',
               r.id === repos.activeId ? 'bg-blue-100 dark:bg-blue-900/40' : 'hover:bg-neutral-200 dark:hover:bg-neutral-800']"
    >{{ r.name }}</button>
    <button class="text-left px-2 py-1 opacity-70 hover:opacity-100" @click="addRepo">+ Add…</button>
  </div>
</template>

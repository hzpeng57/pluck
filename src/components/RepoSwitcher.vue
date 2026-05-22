<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { useReposStore } from "../stores/repos";
import { api } from "../api/tauri";
import type { RepoMeta } from "../types/git";

const repos = useReposStore();
const menu = ref<{ x: number; y: number; repo: RepoMeta } | null>(null);

async function addRepo() {
  const dir = await open({ directory: true, multiple: false });
  if (typeof dir !== "string") return;
  try {
    const meta = await api.repoAdd(dir);
    repos.add(meta);
  } catch (e: any) { alert(e?.data?.friendly ?? String(e)); }
}

function onContext(e: MouseEvent, r: RepoMeta) {
  menu.value = { x: e.clientX, y: e.clientY, repo: r };
}
function removeCurrent() {
  if (!menu.value) return;
  repos.remove(menu.value.repo.id);
  menu.value = null;
}
const closeMenu = () => { menu.value = null; };
onMounted(() => window.addEventListener("click", closeMenu));
onBeforeUnmount(() => window.removeEventListener("click", closeMenu));
</script>

<template>
  <div class="flex flex-col p-2 gap-1">
    <div class="text-xs uppercase opacity-50 px-1 pb-1">Repos</div>
    <button
      v-for="r in repos.all" :key="r.id"
      @click="repos.setActive(r.id)"
      @contextmenu.prevent="onContext($event, r)"
      :class="['text-left px-2 py-1 rounded truncate',
               r.id === repos.activeId ? 'bg-blue-100 dark:bg-blue-900/40' : 'hover:bg-neutral-200 dark:hover:bg-neutral-800']"
    >{{ r.name }}</button>
    <button class="text-left px-2 py-1 opacity-70 hover:opacity-100" @click="addRepo">+ Add…</button>

    <div v-if="menu" :style="{ top: menu.y + 'px', left: menu.x + 'px' }"
         class="fixed z-50 bg-white dark:bg-neutral-800 border rounded shadow text-sm min-w-44">
      <button class="block w-full text-left px-3 py-1 hover:bg-neutral-100 dark:hover:bg-neutral-700"
              @click="removeCurrent">Remove from list</button>
    </div>
  </div>
</template>

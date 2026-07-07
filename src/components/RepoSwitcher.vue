<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, computed } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { Plus, Trash2 } from "lucide-vue-next";
import { useReposStore } from "../stores/repos";
import { useRepoStateStore } from "../stores/repoState";
import { api } from "../api/tauri";
import type { RepoMeta } from "../types/git";

const repos = useReposStore();
const state = useRepoStateStore();
const menu = ref<{ x: number; y: number; repo: RepoMeta } | null>(null);

async function addRepo() {
  const dir = await open({ directory: true, multiple: false });
  if (typeof dir !== "string") return;
  try {
    const meta = await api.repoAdd(dir);
    repos.add(meta);
  } catch (e: any) { state.pushToast("error", e?.data?.friendly ?? String(e)); }
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

function initial(name: string) {
  return name.replace(/[^a-zA-Z0-9]/g, "").slice(0, 2).toUpperCase() || "·";
}
const palette = [
  "var(--graph-1)", "var(--graph-2)", "var(--graph-3)", "var(--graph-4)",
  "var(--graph-5)", "var(--graph-6)", "var(--graph-7)", "var(--graph-8)",
];
function color(id: string) {
  let h = 0; for (const c of id) h = (h * 31 + c.charCodeAt(0)) >>> 0;
  return palette[h % palette.length];
}
const items = computed(() => repos.all);
</script>

<template>
  <aside class="w-[60px] shrink-0 flex flex-col items-center py-3 gap-2"
         style="background: color-mix(in srgb, var(--bg) 88%, black); border-right: 1px solid var(--border-soft)">
    <button
      v-for="r in items" :key="r.id"
      @click="repos.setActive(r.id)"
      @contextmenu.prevent="onContext($event, r)"
      :title="r.name"
      class="relative w-10 h-10 rounded-xl flex items-center justify-center text-[12px] font-semibold tracking-wide transition-all"
      :class="r.id === repos.activeId ? 'scale-100' : 'scale-95 opacity-75 hover:opacity-100 hover:scale-100'"
      :style="{
        background: r.id === repos.activeId ? color(r.id) : 'var(--raised)',
        color: r.id === repos.activeId ? 'var(--on-graph)' : 'var(--fg-2)',
        border: '1px solid ' + (r.id === repos.activeId ? color(r.id) : 'var(--border)')
      }"
    >
      <span
        v-if="r.id === repos.activeId"
        class="absolute -left-3 top-1.5 bottom-1.5 w-1 rounded-r-md"
        :style="{ background: color(r.id) }"
      />
      {{ initial(r.name) }}
    </button>

    <button
      class="w-10 h-10 rounded-xl flex items-center justify-center text-lg font-light transition-colors border border-dashed hover:text-[var(--fg)]"
      style="color: var(--fg-3); border-color: var(--border)"
      @click="addRepo"
      title="Add repository"
    >
      <Plus :size="18" />
    </button>

    <div v-if="menu" :style="{ top: menu.y + 'px', left: menu.x + 'px' }" class="gl-menu">
      <button class="gl-menu-item is-danger" @click="removeCurrent">
        <Trash2 :size="14" class="shrink-0" />
        <span class="truncate">Remove from list</span>
      </button>
    </div>
  </aside>
</template>

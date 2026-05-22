<script setup lang="ts">
import { computed, reactive, ref } from "vue";
import { useRepoStateStore } from "../stores/repoState";
import { ops } from "../api/tauri";
import { useReposStore } from "../stores/repos";

const state = useRepoStateStore();
const repos = useReposStore();
const selected = reactive<Record<string, boolean>>({});
const message = ref("");
const skipHooks = ref(false);

const files = computed(() => state.snapshot?.files ?? []);
const checkedFiles = computed(() => files.value.filter(f => selected[f.path]).map(f => f.path));

function toggleAll(on: boolean) {
  for (const f of files.value) selected[f.path] = on;
}

async function doCommit() {
  if (!repos.activeId) return;
  try {
    state.snapshot = await ops.commit(repos.activeId, checkedFiles.value, message.value, skipHooks.value);
    message.value = ""; for (const f of files.value) selected[f.path] = false;
  } catch (e: any) { state.pushToast("error", e?.data?.friendly ?? String(e)); }
}
</script>

<template>
  <div class="flex flex-col h-full">
    <div class="px-2 py-1 text-xs uppercase opacity-50 flex items-center gap-2">
      <span>Changes ({{ files.length }})</span>
      <button class="opacity-70 hover:opacity-100" @click="toggleAll(true)">all</button>
      <button class="opacity-70 hover:opacity-100" @click="toggleAll(false)">none</button>
    </div>
    <ul class="flex-1 overflow-auto px-2">
      <li v-for="f in files" :key="f.path" class="flex items-center gap-2 py-0.5 hover:bg-neutral-200 dark:hover:bg-neutral-800 rounded px-1">
        <input type="checkbox" v-model="selected[f.path]" />
        <span class="truncate flex-1">{{ f.path }}</span>
        <span class="text-xs opacity-60">{{ f.status }}</span>
      </li>
      <li v-if="files.length === 0" class="opacity-50 py-2">Working tree clean</li>
    </ul>
    <div class="p-2 border-t border-neutral-200 dark:border-neutral-800 flex flex-col gap-2">
      <textarea v-model="message" rows="3" placeholder="Commit message..."
                class="w-full px-2 py-1 rounded bg-white dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700"/>
      <label class="flex items-center gap-2 text-xs">
        <input type="checkbox" v-model="skipHooks" /> Skip hooks (-n)
      </label>
      <div class="flex gap-2">
        <button class="px-3 py-1 rounded bg-blue-600 text-white disabled:opacity-40"
                :disabled="checkedFiles.length === 0 || !message.trim()"
                @click="doCommit">Commit</button>
        <button class="px-3 py-1 rounded border" disabled>Commit & Push</button>
      </div>
    </div>
  </div>
</template>

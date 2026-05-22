<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from "vue";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

interface TodoRow { action: string; hash: string; rest: string }

const visible = ref(false);
const kind = ref<"sequence" | "commitMsg">("sequence");
const rows = ref<TodoRow[]>([]);
const message = ref("");
let unlisten: (() => void) | null = null;

const ACTIONS = ["pick", "reword", "edit", "squash", "fixup", "drop"];

function parseTodo(content: string): TodoRow[] {
  return content.split("\n").filter(l => l && !l.startsWith("#")).map(line => {
    const [action, hash, ...rest] = line.split(" ");
    return { action, hash: hash ?? "", rest: rest.join(" ") };
  });
}

function serializeTodo(rs: TodoRow[]): string {
  return rs.map(r => `${r.action} ${r.hash} ${r.rest}`).join("\n") + "\n";
}

async function save() {
  const content = kind.value === "sequence" ? serializeTodo(rows.value) : message.value;
  await invoke("rebase_reply", { reply: { action: "save", content } });
  visible.value = false;
}

async function abort() {
  await invoke("rebase_reply", { reply: { action: "abort", content: null } });
  visible.value = false;
}

function moveUp(i: number) { if (i > 0) { const a = rows.value.splice(i, 1)[0]; rows.value.splice(i - 1, 0, a); } }
function moveDown(i: number) { if (i < rows.value.length - 1) { const a = rows.value.splice(i, 1)[0]; rows.value.splice(i + 1, 0, a); } }

onMounted(async () => {
  unlisten = await listen<{ kind: string; path?: string; content: string }>("rebase:edit", e => {
    kind.value = e.payload.kind as any;
    if (e.payload.kind === "sequence") { rows.value = parseTodo(e.payload.content); }
    else { message.value = e.payload.content; }
    visible.value = true;
  });
});
onBeforeUnmount(() => unlisten?.());
</script>

<template>
  <div v-if="visible" class="fixed inset-0 bg-black/40 flex items-center justify-center z-50">
    <div class="bg-white dark:bg-neutral-900 rounded-lg shadow-xl w-[640px] max-h-[80vh] flex flex-col">
      <div class="px-4 py-2 border-b font-semibold">
        {{ kind === "sequence" ? "Interactive Rebase" : "Edit Commit Message" }}
      </div>
      <div class="flex-1 overflow-auto p-3">
        <template v-if="kind === 'sequence'">
          <table class="w-full text-sm">
            <tr v-for="(r, i) in rows" :key="i" class="border-b">
              <td class="py-1 w-32">
                <select v-model="r.action" class="bg-transparent border rounded px-1">
                  <option v-for="a in ACTIONS" :key="a" :value="a">{{ a }}</option>
                </select>
              </td>
              <td class="font-mono opacity-60 w-20">{{ r.hash }}</td>
              <td class="truncate">{{ r.rest }}</td>
              <td class="w-16 text-right">
                <button class="px-1" @click="moveUp(i)">↑</button>
                <button class="px-1" @click="moveDown(i)">↓</button>
              </td>
            </tr>
          </table>
        </template>
        <textarea v-else v-model="message" class="w-full h-48 bg-transparent border rounded p-2"/>
      </div>
      <div class="p-3 border-t flex gap-2 justify-end">
        <button class="px-3 py-1 rounded border" @click="abort">Abort rebase</button>
        <button class="px-3 py-1 rounded bg-blue-600 text-white" @click="save">Save & continue</button>
      </div>
    </div>
  </div>
</template>

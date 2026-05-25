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
const ACTION_COLOR: Record<string, string> = {
  pick: "var(--success)",
  reword: "var(--info)",
  edit: "var(--warning)",
  squash: "var(--accent-2)",
  fixup: "var(--accent)",
  drop: "var(--danger)",
};

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
  <div v-if="visible"
       class="fixed inset-0 flex items-center justify-center z-50"
       style="background: rgba(11, 13, 16, 0.7); backdrop-filter: blur(4px)">
    <div class="rounded-xl w-[680px] max-h-[80vh] flex flex-col overflow-hidden"
         style="background: var(--panel); border: 1px solid var(--border); box-shadow: 0 24px 64px rgba(0,0,0,0.6)">
      <div class="flex items-center gap-2 px-4 h-12 shrink-0" style="border-bottom: 1px solid var(--border-soft)">
        <span class="w-2 h-2 rounded-full" style="background: var(--accent)" />
        <span class="font-semibold text-[13px]">
          {{ kind === "sequence" ? "Interactive Rebase" : "Edit Commit Message" }}
        </span>
        <span class="gl-mono text-[10.5px] px-1.5 py-0.5 rounded ml-1"
              v-if="kind === 'sequence'"
              style="background: var(--hover); color: var(--fg-3)">
          {{ rows.length }} commits
        </span>
      </div>

      <div class="flex-1 overflow-auto p-3">
        <template v-if="kind === 'sequence'">
          <div class="flex flex-col gap-1">
            <div v-for="(r, i) in rows" :key="i"
                 class="flex items-center gap-2 px-2 py-1.5 rounded-md transition-colors"
                 @mouseover="(e: any) => (e.currentTarget.style.background = 'var(--hover)')"
                 @mouseleave="(e: any) => (e.currentTarget.style.background = '')">
              <span class="w-1 h-6 rounded-full shrink-0" :style="{ background: ACTION_COLOR[r.action] ?? 'var(--fg-3)' }" />
              <select v-model="r.action"
                      class="gl-mono text-[11.5px] px-2 py-1 rounded w-24 cursor-pointer"
                      :style="{ background: 'var(--bg)', border: '1px solid var(--border)', color: ACTION_COLOR[r.action] ?? 'var(--fg)' }">
                <option v-for="a in ACTIONS" :key="a" :value="a">{{ a }}</option>
              </select>
              <span class="gl-mono text-[11px] shrink-0" style="color: var(--fg-3)">{{ r.hash }}</span>
              <span class="flex-1 truncate text-[12.5px]" style="color: var(--fg)">{{ r.rest }}</span>
              <button class="gl-btn px-1.5" @click="moveUp(i)" title="Move up">↑</button>
              <button class="gl-btn px-1.5" @click="moveDown(i)" title="Move down">↓</button>
            </div>
          </div>
        </template>
        <textarea v-else v-model="message"
                  class="gl-input gl-mono text-[12.5px] resize-none w-full h-48" />
      </div>

      <div class="flex items-center gap-2 px-4 py-3 shrink-0" style="border-top: 1px solid var(--border-soft)">
        <span class="text-[11px]" style="color: var(--fg-3)">
          pick · reword · edit · squash · fixup · drop
        </span>
        <div class="flex-1" />
        <button class="gl-btn gl-btn-ghost" @click="abort">Abort rebase</button>
        <button class="gl-btn gl-btn-primary" @click="save">Save & continue</button>
      </div>
    </div>
  </div>
</template>

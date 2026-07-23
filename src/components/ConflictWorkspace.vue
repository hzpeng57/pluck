<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { Ban, Check, RefreshCw, Trash2 } from "lucide-vue-next";
import { useReposStore } from "../stores/repos";
import { useRepoStateStore } from "../stores/repoState";
import type { ConflictBlob, ConflictStageChoice, GitOp } from "../types/git";
import ConflictEditor from "./ConflictEditor.vue";
import ThreeWayMerge from "./ThreeWayMerge.vue";

const repos = useReposStore();
const state = useRepoStateStore();
const sourceTab = ref<"base" | 2 | 3>("base");
const editorValue = ref("");
const unresolvedBlockCount = ref(0);

const detail = computed(() => state.selectedConflict);
const operation = computed(() => state.snapshot?.inProgress ?? null);
const unresolvedCount = computed(() => state.conflictFiles.length);
const activeRepoId = computed(() => repos.activeId);

function stageLabels(op: GitOp | null) {
  switch (op?.type) {
    case "rebasing": return { 2: "Current branch", 3: "Commit being replayed" };
    case "cherryPicking": return { 2: "Current branch", 3: "Commit being applied" };
    case "reverting": return { 2: "Current branch", 3: "Revert patch" };
    case "merging":
    default: return { 2: "Current branch", 3: "Incoming branch" };
  }
}

const labels = computed(() => stageLabels(operation.value));
const operationTitle = computed(() => {
  const op = operation.value;
  if (!op) return "Conflict resolution";
  switch (op.type) {
    case "rebasing": return `Rebasing ${op.head} onto ${op.onto}`;
    case "merging": return `Merging ${op.from}`;
    case "cherryPicking": return "Cherry-picking";
    case "reverting": return "Reverting";
  }
});

function isTextual(blob: ConflictBlob | null | undefined): blob is ConflictBlob & { content: string } {
  return !!blob && !blob.binary && !blob.tooLarge && typeof blob.content === "string";
}

function blobForTab(tab: "base" | 2 | 3) {
  if (tab === "base") return detail.value?.base ?? null;
  return tab === 2 ? detail.value?.stage2 ?? null : detail.value?.stage3 ?? null;
}

function blobExplanation(blob: ConflictBlob | null) {
  if (!blob) return "This version is not present for the selected path.";
  if (blob.binary) return "Binary content cannot be edited in Pluck. Use a complete version or resolve the path as deletion.";
  if (blob.tooLarge) return "This file is too large for the in-app editor. Use a complete version or resolve the path as deletion.";
  return "No textual content is available for this version.";
}

const selectedBlob = computed(() => blobForTab(sourceTab.value));
const initialStage = computed(() => {
  const current = detail.value?.stage2;
  if (isTextual(current)) return current;
  const incoming = detail.value?.stage3;
  return isTextual(incoming) ? incoming : null;
});
const baseAvailableForMerge = computed(() => !detail.value?.base || isTextual(detail.value.base));
const canThreeWayMerge = computed(() => (
  !!detail.value
  && isTextual(detail.value.stage2)
  && isTextual(detail.value.stage3)
  && baseAvailableForMerge.value
));
const mergeBase = computed(() => isTextual(detail.value?.base) ? detail.value.base.content : "");
const mergeKey = computed(() => {
  const selected = detail.value;
  if (!selected) return "none";
  return [selected.path, selected.base?.stage.oid, selected.stage2?.stage.oid, selected.stage3?.stage.oid].join(":");
});
const canEditFallback = computed(() => !!initialStage.value);
const canMarkResolved = computed(() => (
  !!detail.value
  && !state.loading
  && (canThreeWayMerge.value ? unresolvedBlockCount.value === 0 : canEditFallback.value)
));
const markResolvedTitle = computed(() => {
  if (canThreeWayMerge.value && unresolvedBlockCount.value > 0) {
    return `${unresolvedBlockCount.value} conflict ${unresolvedBlockCount.value === 1 ? "block remains" : "blocks remain"}`;
  }
  return "Write the Result to the Git index";
});

watch(detail, next => {
  editorValue.value = next?.stage2?.content ?? next?.stage3?.content ?? "";
  unresolvedBlockCount.value = next && isTextual(next.stage2) && isTextual(next.stage3) ? 1 : 0;
  sourceTab.value = "base";
}, { immediate: true });

function selectFile(path: string) {
  if (activeRepoId.value) void state.selectConflict(activeRepoId.value, path);
}

function takeStage(stage: ConflictStageChoice) {
  if (!activeRepoId.value || !detail.value) return;
  void state.takeConflictStage(activeRepoId.value, detail.value.path, stage);
}

function resolveText() {
  if (!activeRepoId.value || !detail.value || !canMarkResolved.value) return;
  void state.resolveConflictText(activeRepoId.value, detail.value.path, editorValue.value);
}

function deletePath() {
  if (!activeRepoId.value || !detail.value) return;
  void state.deleteConflictPath(activeRepoId.value, detail.value.path);
}

function refresh() {
  if (activeRepoId.value) void state.refreshConflictWorkspace(activeRepoId.value);
}

function abort() {
  if (activeRepoId.value) void state.abortInProgress(activeRepoId.value);
}

function continueOperation() {
  if (activeRepoId.value && unresolvedCount.value === 0) void state.continueInProgress(activeRepoId.value);
}
</script>

<template>
  <div class="h-full min-h-0 min-w-0 grid" style="grid-template-columns: 260px 6px minmax(620px, 1fr)">
    <aside class="min-h-0 overflow-auto" style="background: var(--panel)">
      <div class="gl-panel-header">
        <span class="font-semibold">Conflicts</span>
        <span class="gl-badge ml-auto">{{ unresolvedCount }}</span>
      </div>
      <div v-if="state.conflictError" class="gl-conflict-empty" style="color: var(--danger)">{{ state.conflictError }}</div>
      <div v-if="state.loadingConflicts" class="gl-conflict-empty"><span class="gl-spinner" /> Loading conflicts</div>
      <div v-else-if="state.conflictFiles.length === 0" class="gl-conflict-empty">No unresolved conflicts</div>
      <div v-else class="p-2 space-y-1">
        <button v-for="file in state.conflictFiles" :key="file.path"
                class="gl-conflict-file w-full text-left"
                :class="{ 'gl-conflict-file-active': file.path === state.selectedConflictPath }"
                @click="selectFile(file.path)">
          <span class="truncate gl-mono">{{ file.path }}</span>
          <span class="gl-badge ml-auto">{{ [file.stage2, file.stage3].filter(Boolean).length }} stages</span>
        </button>
      </div>
    </aside>

    <div class="gl-splitter flex justify-center"><div class="gl-splitter-line" /></div>

    <main class="min-h-0 min-w-0 overflow-hidden flex flex-col" style="background: var(--panel)">
      <template v-if="detail">
        <header class="gl-panel-header shrink-0">
          <div class="min-w-0">
            <div class="font-semibold truncate">{{ operationTitle }}</div>
            <div class="text-[12px]" style="color: var(--fg-3)">{{ unresolvedCount }} unresolved {{ unresolvedCount === 1 ? "file" : "files" }}</div>
          </div>
          <span v-if="state.loadingConflictDetail" class="gl-spinner ml-auto" />
        </header>

        <section class="shrink-0 px-3 py-2 border-b" style="border-color: var(--border-soft)">
          <div class="gl-mono text-[13px] truncate">{{ detail.path }}</div>
          <div class="flex flex-wrap gap-2 mt-1 text-[11px]" style="color: var(--fg-3)">
            <span>base: {{ detail.base ? detail.base.stage.oid.slice(0, 8) : "missing" }}</span>
            <span>stage 2: {{ detail.stage2 ? detail.stage2.stage.oid.slice(0, 8) : "missing" }}</span>
            <span>stage 3: {{ detail.stage3 ? detail.stage3.stage.oid.slice(0, 8) : "missing" }}</span>
          </div>
        </section>

        <section v-if="canThreeWayMerge" class="min-h-0 flex-1 overflow-hidden">
          <ThreeWayMerge :key="mergeKey"
                         :base="mergeBase"
                         :current="detail.stage2!.content!"
                         :incoming="detail.stage3!.content!"
                         :current-label="labels[2]"
                         :incoming-label="labels[3]"
                         @update:result="editorValue = $event"
                         @update:unresolved="unresolvedBlockCount = $event" />
        </section>

        <section v-else class="min-h-0 flex-1 overflow-auto p-3 space-y-3">
          <div class="gl-segmented" role="tablist" aria-label="Conflict sources">
            <button class="gl-segmented-btn px-2 w-auto" :class="{ 'is-active': sourceTab === 'base' }" @click="sourceTab = 'base'">Common base</button>
            <button class="gl-segmented-btn px-2 w-auto" :class="{ 'is-active': sourceTab === 2 }" @click="sourceTab = 2">{{ labels[2] }}</button>
            <button class="gl-segmented-btn px-2 w-auto" :class="{ 'is-active': sourceTab === 3 }" @click="sourceTab = 3">{{ labels[3] }}</button>
          </div>
          <div class="gl-conflict-source">
            <pre v-if="isTextual(selectedBlob)">{{ selectedBlob.content }}</pre>
            <div v-else class="gl-conflict-empty">{{ blobExplanation(selectedBlob) }}</div>
          </div>
          <div v-if="canEditFallback" class="min-h-0 flex flex-col">
            <div class="font-semibold mb-2">Result</div>
            <div class="gl-conflict-editor-wrap">
              <ConflictEditor v-model="editorValue" />
            </div>
          </div>
        </section>

        <div class="shrink-0 flex flex-wrap items-center gap-2 px-3 py-2 border-t" style="border-color: var(--border-soft)">
          <button v-if="detail.stage2" class="gl-command-btn" :disabled="state.loading" @click="takeStage(2)">Use entire {{ labels[2] }}</button>
          <button v-if="detail.stage3" class="gl-command-btn" :disabled="state.loading" @click="takeStage(3)">Use entire {{ labels[3] }}</button>
          <button class="gl-command-btn" :disabled="state.loading" @click="deletePath"><Trash2 :size="14" /> Resolve as deletion</button>
          <button class="gl-command-btn gl-btn-primary ml-auto" :disabled="!canMarkResolved" :title="markResolvedTitle" @click="resolveText">
            <Check :size="14" /> Mark resolved
          </button>
        </div>
      </template>

      <div v-else class="gl-conflict-empty flex-1">{{ state.loadingConflictDetail ? "Loading conflict" : "Select a conflict to resolve" }}</div>

      <footer class="shrink-0 flex items-center gap-2 px-3 py-2 border-t" style="border-color: var(--border-soft)">
        <button class="gl-command-btn" :disabled="state.loadingConflicts || state.loading" @click="refresh"><RefreshCw :size="14" /> Refresh</button>
        <button class="gl-command-btn" :disabled="state.loading" @click="abort"><Ban :size="14" /> Abort</button>
        <button class="gl-command-btn gl-btn-primary ml-auto" :disabled="unresolvedCount > 0 || state.loading" @click="continueOperation">Continue</button>
      </footer>
    </main>
  </div>
</template>

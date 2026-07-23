<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from "vue";
import { ArrowLeft, ArrowRight, GitCompareArrows } from "lucide-vue-next";
import {
  buildSourceRows,
  createThreeWayMerge,
  type MergeConflictBlock,
  type MergeSourceRow,
  type MergeSourceSide,
  type ThreeWayMergeModel,
} from "../lib/threeWayMerge";
import type { ResultRange } from "../lib/mergeResultRanges";
import MergeResultEditor from "./MergeResultEditor.vue";

const props = defineProps<{
  base: string;
  current: string;
  incoming: string;
  currentLabel: string;
  incomingLabel: string;
}>();
const emit = defineEmits<{
  "update:result": [value: string];
  "update:unresolved": [count: number];
}>();

const editor = ref<InstanceType<typeof MergeResultEditor> | null>(null);
const model = ref<ThreeWayMergeModel>(createModel());
const result = ref(model.value.initialResult);
const ranges = ref<ResultRange[]>(makeRanges(model.value.conflicts));
const selectedConflictId = ref<string | null>(model.value.conflicts[0]?.id ?? null);
const showBase = ref(false);
const currentRows = computed(() => buildSourceRows(model.value.currentLines, model.value.conflicts, "current"));
const incomingRows = computed(() => buildSourceRows(model.value.incomingLines, model.value.conflicts, "incoming"));
const unresolvedCount = computed(() => ranges.value.filter(range => range.resolution === "unresolved").length);
const selectedConflict = computed(() => (
  model.value.conflicts.find(block => block.id === selectedConflictId.value) ?? null
));

function createModel() {
  return createThreeWayMerge(props.current, props.base, props.incoming);
}

function makeRanges(conflicts: MergeConflictBlock[]): ResultRange[] {
  return conflicts.map(block => ({
    id: block.id,
    from: block.resultStart,
    to: block.resultEnd,
    resolution: block.resolution,
  }));
}

function reset() {
  model.value = createModel();
  result.value = model.value.initialResult;
  ranges.value = makeRanges(model.value.conflicts);
  selectedConflictId.value = model.value.conflicts[0]?.id ?? null;
  showBase.value = false;
  emit("update:result", result.value);
  emit("update:unresolved", unresolvedCount.value);
}

function displayLine(content: string) {
  return content.replace(/(?:\r\n|\r|\n)$/, "");
}

function resolutionFor(id: string | null) {
  return ranges.value.find(range => range.id === id)?.resolution ?? null;
}

function blockFor(id: string | null) {
  return model.value.conflicts.find(block => block.id === id) ?? null;
}

function accept(row: MergeSourceRow, side: MergeSourceSide) {
  if (!row.conflictId) return;
  const block = blockFor(row.conflictId);
  if (!block) return;
  selectedConflictId.value = block.id;
  const content = side === "current" ? block.currentLines.join("") : block.incomingLines.join("");
  editor.value?.accept(block.id, side, content);
}

function selectBlock(id: string | null) {
  if (!id) return;
  selectedConflictId.value = id;
  editor.value?.focusBlock(id);
}

function updateResult(value: string) {
  result.value = value;
  emit("update:result", value);
}

function updateRanges(value: ResultRange[]) {
  ranges.value = value;
  emit("update:unresolved", value.filter(range => range.resolution === "unresolved").length);
}

watch(
  () => [props.base, props.current, props.incoming],
  () => reset(),
);

watch(selectedConflictId, () => {
  showBase.value = false;
  void nextTick(() => selectedConflictId.value && editor.value?.focusBlock(selectedConflictId.value));
});

onMounted(() => {
  emit("update:result", result.value);
  emit("update:unresolved", unresolvedCount.value);
});
</script>

<template>
  <div class="gl-merge-shell">
    <div class="gl-merge-grid">
      <section class="gl-merge-pane">
        <header class="gl-merge-pane-header">{{ currentLabel }}</header>
        <div class="gl-merge-source" aria-label="Current version">
          <div v-for="(row, index) in currentRows" :key="`current:${index}:${row.conflictId ?? ''}`"
               class="gl-merge-source-row"
               :class="{
                 'is-conflict': row.conflictId,
                 'is-selected': row.conflictId === selectedConflictId,
                 'is-resolved': row.conflictId && resolutionFor(row.conflictId) !== 'unresolved',
               }"
               @click="selectBlock(row.conflictId)">
            <span class="gl-merge-line-no">{{ row.lineNumber ?? "" }}</span>
            <span class="gl-merge-code" :class="{ 'is-placeholder': row.placeholder }">{{ displayLine(row.content) }}</span>
            <button v-if="row.firstInConflict" class="gl-icon-btn gl-merge-accept"
                    :title="`Use ${currentLabel} for this conflict`"
                    @click.stop="accept(row, 'current')">
              <ArrowRight :size="13" />
            </button>
          </div>
        </div>
      </section>

      <section class="gl-merge-pane gl-merge-result">
        <header class="gl-merge-pane-header">
          <span>Result</span>
          <span v-if="unresolvedCount" class="gl-badge ml-auto">{{ unresolvedCount }} unresolved</span>
          <button v-if="selectedConflict" class="gl-command-btn h-7 px-2"
                  :class="{ 'gl-btn-primary': showBase }"
                  title="Show common base for the selected conflict"
                  @click="showBase = !showBase">
            <GitCompareArrows :size="13" />
            Base
          </button>
        </header>
        <div v-if="showBase && selectedConflict" class="gl-merge-base">
          <div class="gl-merge-base-label">Common base</div>
          <pre>{{ selectedConflict.baseLines.join("") || "(empty)" }}</pre>
        </div>
        <div class="gl-merge-editor-wrap">
          <MergeResultEditor ref="editor"
                             :model-value="result"
                             :ranges="ranges"
                             @update:model-value="updateResult"
                             @update:ranges="updateRanges" />
        </div>
      </section>

      <section class="gl-merge-pane">
        <header class="gl-merge-pane-header">{{ incomingLabel }}</header>
        <div class="gl-merge-source" aria-label="Incoming version">
          <div v-for="(row, index) in incomingRows" :key="`incoming:${index}:${row.conflictId ?? ''}`"
               class="gl-merge-source-row"
               :class="{
                 'is-conflict': row.conflictId,
                 'is-selected': row.conflictId === selectedConflictId,
                 'is-resolved': row.conflictId && resolutionFor(row.conflictId) !== 'unresolved',
               }"
               @click="selectBlock(row.conflictId)">
            <span class="gl-merge-line-no">{{ row.lineNumber ?? "" }}</span>
            <span class="gl-merge-code" :class="{ 'is-placeholder': row.placeholder }">{{ displayLine(row.content) }}</span>
            <button v-if="row.firstInConflict" class="gl-icon-btn gl-merge-accept"
                    :title="`Use ${incomingLabel} for this conflict`"
                    @click.stop="accept(row, 'incoming')">
              <ArrowLeft :size="13" />
            </button>
          </div>
        </div>
      </section>
    </div>
  </div>
</template>

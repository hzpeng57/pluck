<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from "vue";
import {
  Annotation,
  Compartment,
  EditorState,
  StateEffect,
  StateField,
} from "@codemirror/state";
import {
  Decoration,
  EditorView,
  WidgetType,
  type DecorationSet,
  type ViewUpdate,
} from "@codemirror/view";
import { basicSetup } from "codemirror";
import {
  replaceConflictBlock,
  type ResultRange,
} from "../lib/mergeResultRanges";

const props = defineProps<{
  modelValue: string;
  ranges: ResultRange[];
  readOnly?: boolean;
}>();
const emit = defineEmits<{
  "update:modelValue": [value: string];
  "update:ranges": [value: ResultRange[]];
}>();

const host = ref<HTMLElement | null>(null);
let view: EditorView | null = null;
let currentRanges = props.ranges.map(range => ({ ...range }));
const readOnlyCompartment = new Compartment();
const programmaticUpdate = Annotation.define<boolean>();
const setDecorations = StateEffect.define<DecorationSet>();

class EmptyConflictMarker extends WidgetType {
  toDOM(): HTMLElement {
    const marker = document.createElement("span");
    marker.className = "cm-merge-empty-conflict";
    marker.setAttribute("aria-hidden", "true");
    return marker;
  }
}

function buildDecorations(ranges: ResultRange[], documentLength: number): DecorationSet {
  const decorations = ranges
    .filter(range => range.resolution === "unresolved")
    .map(range => {
      const from = Math.max(0, Math.min(range.from, documentLength));
      const to = Math.max(from, Math.min(range.to, documentLength));
      if (from === to) {
        return Decoration.widget({ widget: new EmptyConflictMarker(), side: 1 }).range(from);
      }
      return Decoration.mark({ class: "cm-merge-unresolved" }).range(from, to);
    });
  return Decoration.set(decorations, true);
}

const decorationField = StateField.define<DecorationSet>({
  create: state => buildDecorations(currentRanges, state.doc.length),
  update(value, transaction) {
    let next = value.map(transaction.changes);
    for (const effect of transaction.effects) {
      if (effect.is(setDecorations)) next = effect.value;
    }
    return next;
  },
  provide: field => EditorView.decorations.from(field),
});

const editorTheme = EditorView.theme({
  "&": {
    height: "100%",
    color: "var(--fg)",
    backgroundColor: "var(--bg)",
    fontFamily: "var(--mono)",
    fontSize: "12.5px",
  },
  ".cm-scroller": { overflow: "auto", fontFamily: "var(--mono)" },
  ".cm-content": { padding: "10px 12px", minHeight: "100%", caretColor: "var(--accent)" },
  ".cm-gutters": { backgroundColor: "var(--diff-gutter-bg)", color: "var(--fg-3)", border: "none" },
  ".cm-activeLine, .cm-activeLineGutter": { backgroundColor: "var(--hover)" },
  ".cm-selectionBackground, ::selection": { backgroundColor: "var(--accent-soft)" },
  ".cm-merge-unresolved": {
    backgroundColor: "var(--warning-soft-weak)",
    boxShadow: "inset 3px 0 0 var(--warning)",
  },
  ".cm-merge-empty-conflict": {
    display: "inline-block",
    width: "3px",
    height: "1.35em",
    backgroundColor: "var(--warning)",
    verticalAlign: "text-bottom",
  },
});

function rangeTouched(update: ViewUpdate, range: ResultRange): boolean {
  let touched = false;
  update.changes.iterChanges((fromA, toA) => {
    if (touched) return;
    touched = fromA === toA
      ? fromA >= range.from && fromA <= range.to
      : fromA < range.to && toA > range.from;
  });
  return touched;
}

function mapUserRanges(update: ViewUpdate): ResultRange[] {
  return currentRanges.map(range => ({
    ...range,
    from: update.changes.mapPos(range.from, -1),
    to: update.changes.mapPos(range.to, 1),
    resolution: rangeTouched(update, range) ? "manual" : range.resolution,
  }));
}

function refreshDecorations() {
  if (!view) return;
  view.dispatch({
    effects: setDecorations.of(buildDecorations(currentRanges, view.state.doc.length)),
  });
}

function accept(id: string, side: "current" | "incoming", content: string) {
  if (!view) return;
  const target = currentRanges.find(range => range.id === id);
  if (!target) return;
  const next = replaceConflictBlock(
    view.state.doc.toString(),
    currentRanges,
    id,
    content,
    side,
  );
  currentRanges = next.ranges;
  view.dispatch({
    changes: { from: target.from, to: target.to, insert: content },
    annotations: programmaticUpdate.of(true),
    effects: EditorView.scrollIntoView(target.from, { y: "center" }),
  });
  emit("update:modelValue", next.document);
  emit("update:ranges", next.ranges);
}

function focusBlock(id: string) {
  if (!view) return;
  const target = currentRanges.find(range => range.id === id);
  if (target) view.dispatch({ effects: EditorView.scrollIntoView(target.from, { y: "center" }) });
}

defineExpose({ accept, focusBlock });

onMounted(() => {
  if (!host.value) return;
  const state = EditorState.create({
    doc: props.modelValue,
    extensions: [
      basicSetup,
      EditorState.tabSize.of(2),
      EditorView.lineWrapping,
      editorTheme,
      decorationField,
      readOnlyCompartment.of([
        EditorState.readOnly.of(!!props.readOnly),
        EditorView.editable.of(!props.readOnly),
      ]),
      EditorView.updateListener.of(update => {
        if (!update.docChanged) return;
        const isProgrammatic = update.transactions.some(transaction => (
          transaction.annotation(programmaticUpdate)
        ));
        if (isProgrammatic) return;
        currentRanges = mapUserRanges(update);
        emit("update:modelValue", update.state.doc.toString());
        emit("update:ranges", currentRanges);
      }),
    ],
  });
  view = new EditorView({ state, parent: host.value });
});

watch(() => props.modelValue, value => {
  if (!view || value === view.state.doc.toString()) return;
  view.dispatch({
    changes: { from: 0, to: view.state.doc.length, insert: value },
    annotations: programmaticUpdate.of(true),
  });
});

watch(() => props.ranges, value => {
  currentRanges = value.map(range => ({ ...range }));
  refreshDecorations();
}, { deep: true });

watch(() => props.readOnly, value => {
  view?.dispatch({
    effects: readOnlyCompartment.reconfigure([
      EditorState.readOnly.of(!!value),
      EditorView.editable.of(!value),
    ]),
  });
});

onBeforeUnmount(() => {
  view?.destroy();
  view = null;
});
</script>

<template>
  <div ref="host" class="gl-merge-editor" :class="{ 'is-read-only': readOnly }" />
</template>

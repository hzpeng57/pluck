<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from "vue";
import {
  Annotation,
  Compartment,
  EditorState,
  Transaction,
} from "@codemirror/state";
import { isolateHistory } from "@codemirror/commands";
import {
  Decoration,
  EditorView,
  WidgetType,
  type DecorationSet,
} from "@codemirror/view";
import { basicSetup } from "codemirror";
import {
  applyConflictSide,
  isResultRangeResolved,
  type ConflictBlockContents,
  type MergeSide,
  type MergeSideAction,
  type ResultRange,
} from "../lib/mergeResultRanges";
import {
  getResultRanges,
  resultRangeHistory,
  resultRangesField,
  setResultRanges,
} from "../lib/mergeResultHistory";

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
const readOnlyCompartment = new Compartment();
const externalUpdate = Annotation.define<boolean>();

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
    .filter(range => !isResultRangeResolved(range))
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

const resultDecorations = EditorView.decorations.compute(
  [resultRangesField],
  state => buildDecorations([...getResultRanges(state)], state.doc.length),
);

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
  ".cm-cursor, .cm-dropCursor": { borderLeftColor: "var(--accent) !important" },
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

function applySide(
  id: string,
  side: MergeSide,
  action: MergeSideAction,
  contents: ConflictBlockContents,
) {
  if (!view) return;
  const ranges = getResultRanges(view.state);
  const target = ranges.find(range => range.id === id);
  if (!target) return;
  const next = applyConflictSide(
    view.state.doc.toString(),
    [...ranges],
    id,
    side,
    action,
    contents,
  );
  const nextTarget = next.ranges.find(range => range.id === id);
  view.dispatch({
    changes: {
      from: target.from,
      to: target.to,
      insert: next.document.slice(target.from, nextTarget?.to ?? target.from),
    },
    annotations: isolateHistory.of("full"),
    effects: [
      setResultRanges.of(next.ranges),
      EditorView.scrollIntoView(target.from, { y: "center" }),
    ],
  });
  view.focus();
}

function focusBlock(id: string) {
  if (!view) return;
  const target = getResultRanges(view.state).find(range => range.id === id);
  if (target) view.dispatch({ effects: EditorView.scrollIntoView(target.from, { y: "center" }) });
}

defineExpose({ applySide, focusBlock });

onMounted(() => {
  if (!host.value) return;
  const state = EditorState.create({
    doc: props.modelValue,
    extensions: [
      basicSetup,
      EditorState.tabSize.of(2),
      EditorView.lineWrapping,
      editorTheme,
      resultRangeHistory(props.ranges),
      resultDecorations,
      readOnlyCompartment.of([
        EditorState.readOnly.of(!!props.readOnly),
        EditorView.editable.of(!props.readOnly),
      ]),
      EditorView.updateListener.of(update => {
        const rangesChanged = update.transactions.some(transaction => (
          transaction.effects.some(effect => effect.is(setResultRanges))
        ));
        if (!update.docChanged && !rangesChanged) return;
        const isExternal = update.transactions.some(transaction => (
          transaction.annotation(externalUpdate)
        ));
        if (isExternal) return;
        if (update.docChanged) emit("update:modelValue", update.state.doc.toString());
        emit("update:ranges", [...getResultRanges(update.state)]);
      }),
    ],
  });
  view = new EditorView({ state, parent: host.value });
});

watch(() => props.modelValue, value => {
  if (!view || value === view.state.doc.toString()) return;
  view.dispatch({
    changes: { from: 0, to: view.state.doc.length, insert: value },
    annotations: [externalUpdate.of(true), Transaction.addToHistory.of(false)],
    effects: setResultRanges.of(props.ranges),
  });
});

watch(() => props.ranges, value => {
  if (!view) return;
  const current = getResultRanges(view.state);
  const unchanged = current.length === value.length && current.every((range, index) => (
    range.id === value[index].id
    && range.from === value[index].from
    && range.to === value[index].to
    && range.current === value[index].current
    && range.incoming === value[index].incoming
    && range.manual === value[index].manual
  ));
  if (unchanged) return;
  view.dispatch({
    annotations: [externalUpdate.of(true), Transaction.addToHistory.of(false)],
    effects: setResultRanges.of(value),
  });
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

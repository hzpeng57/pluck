<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from "vue";
import { EditorState, Compartment } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { basicSetup } from "codemirror";

const props = defineProps<{ modelValue: string; readOnly?: boolean }>();
const emit = defineEmits<{ "update:modelValue": [value: string] }>();

const host = ref<HTMLElement | null>(null);
let view: EditorView | null = null;
const readOnlyCompartment = new Compartment();

const editorTheme = EditorView.theme({
  "&": {
    height: "100%",
    color: "var(--fg)",
    backgroundColor: "var(--bg)",
    fontFamily: "var(--mono)",
    fontSize: "12.5px",
  },
  ".cm-scroller": { overflow: "auto", fontFamily: "var(--mono)" },
  ".cm-content": { padding: "12px", minHeight: "100%", caretColor: "var(--accent)" },
  ".cm-gutters": { backgroundColor: "var(--diff-gutter-bg)", color: "var(--fg-3)", border: "none" },
  ".cm-activeLine, .cm-activeLineGutter": { backgroundColor: "var(--hover)" },
  ".cm-selectionBackground, ::selection": { backgroundColor: "var(--accent-soft)" },
});

onMounted(() => {
  if (!host.value) return;
  const state = EditorState.create({
    doc: props.modelValue,
    extensions: [
      basicSetup,
      EditorState.tabSize.of(2),
      EditorView.lineWrapping,
      editorTheme,
      readOnlyCompartment.of([
        EditorState.readOnly.of(!!props.readOnly),
        EditorView.editable.of(!props.readOnly),
      ]),
      EditorView.updateListener.of(update => {
        if (!update.docChanged) return;
        const userEdit = update.transactions.some(transaction =>
          transaction.isUserEvent("input") || transaction.isUserEvent("delete"),
        );
        if (userEdit) emit("update:modelValue", update.state.doc.toString());
      }),
    ],
  });
  view = new EditorView({ state, parent: host.value });
});

watch(() => props.modelValue, value => {
  if (!view || value === view.state.doc.toString()) return;
  view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: value } });
});

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
  <div ref="host" class="gl-conflict-editor" :class="{ 'is-read-only': readOnly }" />
</template>

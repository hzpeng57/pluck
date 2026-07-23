import { invertedEffects } from "@codemirror/commands";
import {
  Facet,
  StateEffect,
  StateField,
  type ChangeSet,
  type EditorState,
  type Extension,
} from "@codemirror/state";
import type { ResultRange } from "./mergeResultRanges";

const initialResultRanges = Facet.define<readonly ResultRange[], readonly ResultRange[]>({
  combine: values => values[values.length - 1] ?? [],
});

function cloneRanges(ranges: readonly ResultRange[]): ResultRange[] {
  return ranges.map(range => ({ ...range }));
}

function mapRanges(ranges: readonly ResultRange[], changes: ChangeSet): ResultRange[] {
  return ranges.map(range => {
    let touched = false;
    changes.iterChanges((fromA, toA) => {
      if (touched) return;
      touched = fromA === toA
        ? fromA >= range.from && fromA <= range.to
        : fromA < range.to && toA > range.from;
    });
    return {
      ...range,
      from: changes.mapPos(range.from, -1),
      to: changes.mapPos(range.to, 1),
      manual: range.manual || touched,
    };
  });
}

export const setResultRanges = StateEffect.define<readonly ResultRange[]>({
  map: (ranges, changes) => ranges.map(range => ({
    ...range,
    from: changes.mapPos(range.from, -1),
    to: changes.mapPos(range.to, 1),
  })),
});

export const resultRangesField = StateField.define<readonly ResultRange[]>({
  create: state => cloneRanges(state.facet(initialResultRanges)),
  update(value, transaction) {
    let explicit: readonly ResultRange[] | null = null;
    for (const effect of transaction.effects) {
      if (effect.is(setResultRanges)) explicit = effect.value;
    }
    if (explicit) return cloneRanges(explicit);
    return transaction.docChanged ? mapRanges(value, transaction.changes) : value;
  },
});

const resultRangeInversions = invertedEffects.of(transaction => {
  const changesRanges = transaction.docChanged
    || transaction.effects.some(effect => effect.is(setResultRanges));
  if (!changesRanges) return [];
  const previous = transaction.startState.field(resultRangesField, false);
  return previous ? [setResultRanges.of(cloneRanges(previous))] : [];
});

export function resultRangeHistory(initial: readonly ResultRange[]): Extension {
  return [
    initialResultRanges.of(cloneRanges(initial)),
    resultRangesField,
    resultRangeInversions,
  ];
}

export function getResultRanges(state: EditorState): readonly ResultRange[] {
  return state.field(resultRangesField);
}

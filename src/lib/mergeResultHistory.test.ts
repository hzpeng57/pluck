import { history, isolateHistory, redo, undo } from "@codemirror/commands";
import { EditorState, type Transaction, type TransactionSpec } from "@codemirror/state";
import { describe, expect, it } from "vitest";
import {
  applyConflictSide,
  type ConflictBlockContents,
  type MergeSide,
  type MergeSideAction,
  type ResultRange,
} from "./mergeResultRanges";
import {
  getResultRanges,
  resultRangeHistory,
  setResultRanges,
} from "./mergeResultHistory";

const contents: ConflictBlockContents = {
  base: "base\n",
  current: "current\n",
  incoming: "incoming\n",
};

function unresolvedRange(blockContents = contents): ResultRange {
  return {
    id: "conflict-0",
    from: 0,
    to: blockContents.base.length,
    current: "pending",
    incoming: "pending",
    manual: false,
  };
}

function createHarness(blockContents = contents) {
  let state = EditorState.create({
    doc: blockContents.base,
    extensions: [history(), resultRangeHistory([unresolvedRange(blockContents)])],
  });

  function dispatch(spec: TransactionSpec) {
    state = state.update(spec).state;
  }

  function run(command: typeof undo) {
    return command({
      state,
      dispatch: (transaction: Transaction) => { state = transaction.state; },
    });
  }

  function apply(side: MergeSide, action: MergeSideAction = "accept") {
    const ranges = getResultRanges(state);
    const target = ranges[0];
    const next = applyConflictSide(
      state.doc.toString(),
      [...ranges],
      target.id,
      side,
      action,
      blockContents,
    );
    const nextTarget = next.ranges[0];
    dispatch({
      changes: {
        from: target.from,
        to: target.to,
        insert: next.document.slice(target.from, nextTarget.to),
      },
      effects: setResultRanges.of(next.ranges),
      annotations: isolateHistory.of("full"),
    });
  }

  return {
    apply,
    dispatch,
    ranges: () => getResultRanges(state),
    redo: () => run(redo),
    text: () => state.doc.toString(),
    undo: () => run(undo),
  };
}

describe("merge result history", () => {
  it("undoes and redoes an accepted side with its range state", () => {
    const harness = createHarness();
    harness.apply("current");

    expect(harness.text()).toBe(contents.current);
    expect(harness.ranges()[0].current).toBe("accepted");

    expect(harness.undo()).toBe(true);
    expect(harness.text()).toBe(contents.base);
    expect(harness.ranges()[0]).toEqual(unresolvedRange());

    expect(harness.redo()).toBe(true);
    expect(harness.text()).toBe(contents.current);
    expect(harness.ranges()[0].current).toBe("accepted");
  });

  it("keeps consecutive side actions as separate undo steps", () => {
    const harness = createHarness();
    harness.apply("current");
    harness.apply("incoming");

    expect(harness.text()).toBe(contents.current + contents.incoming);
    expect(harness.undo()).toBe(true);
    expect(harness.text()).toBe(contents.current);
    expect(harness.ranges()[0]).toMatchObject({ current: "accepted", incoming: "pending" });

    expect(harness.undo()).toBe(true);
    expect(harness.text()).toBe(contents.base);
    expect(harness.ranges()[0]).toEqual(unresolvedRange());
  });

  it("restores pending controls when a manual edit is undone", () => {
    const harness = createHarness();
    harness.dispatch({
      changes: { from: 0, to: 0, insert: "edited " },
      userEvent: "input.type",
    });

    expect(harness.ranges()[0].manual).toBe(true);
    expect(harness.undo()).toBe(true);
    expect(harness.text()).toBe(contents.base);
    expect(harness.ranges()[0]).toEqual(unresolvedRange());
  });

  it("tracks an ignored side even when the document does not change", () => {
    const emptyContents: ConflictBlockContents = {
      base: "",
      current: "current\n",
      incoming: "incoming\n",
    };
    const harness = createHarness(emptyContents);
    harness.apply("current", "ignore");

    expect(harness.text()).toBe("");
    expect(harness.ranges()[0].current).toBe("ignored");

    expect(harness.undo()).toBe(true);
    expect(harness.text()).toBe("");
    expect(harness.ranges()[0]).toEqual(unresolvedRange(emptyContents));

    expect(harness.redo()).toBe(true);
    expect(harness.ranges()[0].current).toBe("ignored");
  });
});

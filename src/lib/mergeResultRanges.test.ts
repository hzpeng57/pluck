import { describe, expect, it } from "vitest";
import {
  applyConflictSide,
  applyDocumentChange,
  isResultRangeResolved,
  type ConflictBlockContents,
  type ResultRange,
} from "./mergeResultRanges";

const contents: ConflictBlockContents = {
  base: "base\n",
  current: "current\n",
  incoming: "incoming\n",
};

function unresolvedRange(from = 0, to = 5): ResultRange {
  return {
    id: "conflict-0",
    from,
    to,
    current: "pending",
    incoming: "pending",
    manual: false,
  };
}

describe("merge result ranges", () => {
  it("moves a later block when an earlier block grows", () => {
    const ranges: ResultRange[] = [
      unresolvedRange(2, 6),
      { ...unresolvedRange(10, 14), id: "conflict-1" },
    ];

    expect(applyDocumentChange(ranges, {
      from: 2,
      to: 6,
      insert: "current\nvalue\n",
    })[1]).toMatchObject({ from: 20, to: 24 });
  });

  it("marks only the edited block as manually resolved", () => {
    const ranges: ResultRange[] = [
      unresolvedRange(0, 4),
      { ...unresolvedRange(8, 12), id: "conflict-1" },
    ];

    const next = applyDocumentChange(ranges, { from: 1, to: 2, insert: "X" });

    expect(next[0].manual).toBe(true);
    expect(isResultRangeResolved(next[0])).toBe(true);
    expect(next[1].manual).toBe(false);
    expect(isResultRangeResolved(next[1])).toBe(false);
  });

  it("accepts Current while leaving Incoming pending", () => {
    const next = applyConflictSide(
      "base\n",
      [unresolvedRange()],
      "conflict-0",
      "current",
      "accept",
      contents,
    );

    expect(next.document).toBe("current\n");
    expect(next.ranges[0]).toMatchObject({
      current: "accepted",
      incoming: "pending",
      manual: false,
    });
    expect(isResultRangeResolved(next.ranges[0])).toBe(false);
  });

  it("keeps both sides when Incoming is accepted after Current", () => {
    const afterCurrent = applyConflictSide(
      "base\n",
      [unresolvedRange()],
      "conflict-0",
      "current",
      "accept",
      contents,
    );
    const afterIncoming = applyConflictSide(
      afterCurrent.document,
      afterCurrent.ranges,
      "conflict-0",
      "incoming",
      "accept",
      contents,
    );

    expect(afterIncoming.document).toBe("current\nincoming\n");
    expect(afterIncoming.ranges[0]).toMatchObject({
      current: "accepted",
      incoming: "accepted",
    });
    expect(isResultRangeResolved(afterIncoming.ranges[0])).toBe(true);
  });

  it("keeps deterministic Current then Incoming order regardless of click order", () => {
    const afterIncoming = applyConflictSide(
      "base\n",
      [unresolvedRange()],
      "conflict-0",
      "incoming",
      "accept",
      contents,
    );
    const afterCurrent = applyConflictSide(
      afterIncoming.document,
      afterIncoming.ranges,
      "conflict-0",
      "current",
      "accept",
      contents,
    );

    expect(afterCurrent.document).toBe("current\nincoming\n");
    expect(isResultRangeResolved(afterCurrent.ranges[0])).toBe(true);
  });

  it("ignores one side without applying it", () => {
    const ignoredCurrent = applyConflictSide(
      "base\n",
      [unresolvedRange()],
      "conflict-0",
      "current",
      "ignore",
      contents,
    );
    const acceptedIncoming = applyConflictSide(
      ignoredCurrent.document,
      ignoredCurrent.ranges,
      "conflict-0",
      "incoming",
      "accept",
      contents,
    );

    expect(acceptedIncoming.document).toBe("incoming\n");
    expect(acceptedIncoming.ranges[0]).toMatchObject({
      current: "ignored",
      incoming: "accepted",
    });
    expect(isResultRangeResolved(acceptedIncoming.ranges[0])).toBe(true);
  });

  it("keeps the base when both sides are ignored", () => {
    const ignoredCurrent = applyConflictSide(
      "base\n",
      [unresolvedRange()],
      "conflict-0",
      "current",
      "ignore",
      contents,
    );
    const ignoredBoth = applyConflictSide(
      ignoredCurrent.document,
      ignoredCurrent.ranges,
      "conflict-0",
      "incoming",
      "ignore",
      contents,
    );

    expect(ignoredBoth.document).toBe("base\n");
    expect(ignoredBoth.ranges[0]).toMatchObject({
      current: "ignored",
      incoming: "ignored",
    });
    expect(isResultRangeResolved(ignoredBoth.ranges[0])).toBe(true);
  });

  it("leaves the document unchanged for an unknown block", () => {
    const ranges = [unresolvedRange()];

    expect(applyConflictSide(
      "base\n",
      ranges,
      "missing",
      "current",
      "accept",
      contents,
    )).toEqual({ document: "base\n", ranges });
  });
});

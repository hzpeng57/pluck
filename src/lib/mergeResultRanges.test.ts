import { describe, expect, it } from "vitest";
import {
  applyDocumentChange,
  replaceConflictBlock,
  type ResultRange,
} from "./mergeResultRanges";

describe("merge result ranges", () => {
  it("moves a later block when an earlier block grows", () => {
    const ranges: ResultRange[] = [
      { id: "0", from: 2, to: 6, resolution: "unresolved" },
      { id: "1", from: 10, to: 14, resolution: "unresolved" },
    ];

    expect(applyDocumentChange(ranges, {
      from: 2,
      to: 6,
      insert: "current\nvalue\n",
    })[1]).toMatchObject({ from: 20, to: 24 });
  });

  it("marks only the intersected block as manually resolved", () => {
    const ranges: ResultRange[] = [
      { id: "0", from: 0, to: 4, resolution: "unresolved" },
      { id: "1", from: 8, to: 12, resolution: "incoming" },
    ];

    expect(applyDocumentChange(ranges, {
      from: 1,
      to: 2,
      insert: "X",
    })).toEqual([
      { id: "0", from: 0, to: 4, resolution: "manual" },
      { id: "1", from: 8, to: 12, resolution: "incoming" },
    ]);
  });

  it("captures inserted text for an empty deletion-conflict range", () => {
    const ranges: ResultRange[] = [
      { id: "0", from: 2, to: 2, resolution: "unresolved" },
    ];

    expect(applyDocumentChange(ranges, {
      from: 2,
      to: 2,
      insert: "restored\n",
    })).toEqual([
      { id: "0", from: 2, to: 11, resolution: "manual" },
    ]);
  });

  it("replaces only the requested mapped block", () => {
    const document = "a\nbase\nb\nbase\n";
    const ranges: ResultRange[] = [
      { id: "first", from: 2, to: 7, resolution: "unresolved" },
      { id: "second", from: 9, to: 14, resolution: "unresolved" },
    ];

    expect(replaceConflictBlock(
      document,
      ranges,
      "second",
      "incoming\n",
      "incoming",
    )).toEqual({
      document: "a\nbase\nb\nincoming\n",
      ranges: [
        { id: "first", from: 2, to: 7, resolution: "unresolved" },
        { id: "second", from: 9, to: 18, resolution: "incoming" },
      ],
    });
  });

  it("leaves the document unchanged for an unknown block", () => {
    const ranges: ResultRange[] = [
      { id: "first", from: 0, to: 4, resolution: "unresolved" },
    ];

    expect(replaceConflictBlock("base", ranges, "missing", "x", "current")).toEqual({
      document: "base",
      ranges,
    });
  });
});

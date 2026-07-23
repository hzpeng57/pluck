import { describe, expect, it } from "vitest";
import { createThreeWayMerge } from "./threeWayMerge";

describe("createThreeWayMerge", () => {
  it("pre-applies independent edits separated by stable context", () => {
    const merge = createThreeWayMerge(
      "one\ncurrent\nmiddle\nthree\n",
      "one\ntwo\nmiddle\nthree\n",
      "one\ntwo\nmiddle\nincoming\n",
    );

    expect(merge.conflicts).toEqual([]);
    expect(merge.initialResult).toBe("one\ncurrent\nmiddle\nincoming\n");
  });

  it("keeps base text for an overlapping conflict", () => {
    const merge = createThreeWayMerge(
      "one\ncurrent\n",
      "one\nbase\n",
      "one\nincoming\n",
    );

    expect(merge.conflicts).toHaveLength(1);
    expect(merge.conflicts[0]).toMatchObject({
      id: "conflict-0",
      baseLines: ["base\n"],
      currentLines: ["current\n"],
      incomingLines: ["incoming\n"],
      baseStartLine: 1,
      currentStartLine: 1,
      incomingStartLine: 1,
      resolution: "unresolved",
    });
    expect(merge.initialResult).toBe("one\nbase\n");
  });

  it("does not flag an identical change on both sides", () => {
    const merge = createThreeWayMerge(
      "one\nsame\n",
      "one\nbase\n",
      "one\nsame\n",
    );

    expect(merge.conflicts).toEqual([]);
    expect(merge.initialResult).toBe("one\nsame\n");
  });

  it("exposes deletion content and preserves the base newline", () => {
    const merge = createThreeWayMerge("", "delete-me\n", "incoming\n");

    expect(merge.conflicts[0]).toMatchObject({
      baseLines: ["delete-me\n"],
      currentLines: [],
      incomingLines: ["incoming\n"],
      currentStartLine: 0,
      incomingStartLine: 0,
    });
    expect(merge.initialResult).toBe("delete-me\n");
  });

  it("preserves content without a final newline", () => {
    const merge = createThreeWayMerge("current", "base", "incoming");

    expect(merge.conflicts[0].baseLines).toEqual(["base"]);
    expect(merge.initialResult).toBe("base");
  });

  it("preserves CRLF line endings in conflict content", () => {
    const merge = createThreeWayMerge("current\r\n", "base\r\n", "incoming\r\n");

    expect(merge.conflicts[0].baseLines).toEqual(["base\r\n"]);
    expect(merge.initialResult).toBe("base\r\n");
  });
});

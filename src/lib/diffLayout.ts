import type { DiffHunk, DiffLine, FileDiff } from "../types/git";

export type DiffCellKind = "context" | "added" | "deleted" | "empty" | "notice";

export interface DiffTextSegment {
  text: string;
  changed: boolean;
}

export interface SplitDiffCell {
  kind: DiffCellKind;
  number: number | null;
  content: string;
  segments: DiffTextSegment[];
}

export interface SplitDiffRow {
  left: SplitDiffCell;
  right: SplitDiffCell;
  pairedChange: boolean;
}

export interface SplitDiffHunk {
  header: string;
  rows: SplitDiffRow[];
}

const EMPTY_CELL: SplitDiffCell = {
  kind: "empty",
  number: null,
  content: "",
  segments: [],
};

function emptyCell(): SplitDiffCell {
  return { ...EMPTY_CELL, segments: [] };
}

function textSegments(content: string): DiffTextSegment[] {
  return content ? [{ text: content, changed: false }] : [];
}

function cellFromLine(line: DiffLine): SplitDiffCell {
  if (line.kind === "context") {
    return {
      kind: "context",
      number: line.oldNumber ?? line.newNumber,
      content: line.content,
      segments: textSegments(line.content),
    };
  }
  if (line.kind === "deleted") {
    return {
      kind: "deleted",
      number: line.oldNumber,
      content: line.content,
      segments: textSegments(line.content),
    };
  }
  if (line.kind === "added") {
    return {
      kind: "added",
      number: line.newNumber,
      content: line.content,
      segments: textSegments(line.content),
    };
  }
  return {
    kind: "notice",
    number: null,
    content: line.content,
    segments: textSegments(line.content),
  };
}

function changedSegments(left: string, right: string): [DiffTextSegment[], DiffTextSegment[]] {
  let prefix = 0;
  const maxPrefix = Math.min(left.length, right.length);
  while (prefix < maxPrefix && left[prefix] === right[prefix]) prefix++;

  let leftEnd = left.length;
  let rightEnd = right.length;
  while (leftEnd > prefix && rightEnd > prefix && left[leftEnd - 1] === right[rightEnd - 1]) {
    leftEnd--;
    rightEnd--;
  }

  const build = (value: string, end: number): DiffTextSegment[] => {
    const segments: DiffTextSegment[] = [];
    if (prefix > 0) segments.push({ text: value.slice(0, prefix), changed: false });
    if (end > prefix) segments.push({ text: value.slice(prefix, end), changed: true });
    if (end < value.length) segments.push({ text: value.slice(end), changed: false });
    return segments;
  };

  return [build(left, leftEnd), build(right, rightEnd)];
}

function pushChangeBlock(rows: SplitDiffRow[], block: DiffLine[]) {
  const deleted = block.filter(line => line.kind === "deleted");
  const added = block.filter(line => line.kind === "added");
  const count = Math.max(deleted.length, added.length);

  for (let i = 0; i < count; i++) {
    const left = deleted[i] ? cellFromLine(deleted[i]) : emptyCell();
    const right = added[i] ? cellFromLine(added[i]) : emptyCell();
    const pairedChange = left.kind === "deleted" && right.kind === "added";

    if (pairedChange) {
      const [leftSegments, rightSegments] = changedSegments(left.content, right.content);
      left.segments = leftSegments;
      right.segments = rightSegments;
    }

    rows.push({ left, right, pairedChange });
  }
}

function splitHunk(hunk: DiffHunk): SplitDiffHunk {
  const rows: SplitDiffRow[] = [];
  let i = 0;

  while (i < hunk.lines.length) {
    const line = hunk.lines[i];
    if (line.kind === "context") {
      const left = cellFromLine(line);
      const right = { ...left, number: line.newNumber, segments: [...left.segments] };
      rows.push({
        left: { ...left, number: line.oldNumber },
        right,
        pairedChange: false,
      });
      i++;
      continue;
    }

    if (line.kind === "noNewline") {
      const cell = cellFromLine(line);
      rows.push({
        left: cell,
        right: { ...cell, segments: [...cell.segments] },
        pairedChange: false,
      });
      i++;
      continue;
    }

    const block: DiffLine[] = [];
    while (i < hunk.lines.length && (hunk.lines[i].kind === "deleted" || hunk.lines[i].kind === "added")) {
      block.push(hunk.lines[i]);
      i++;
    }
    pushChangeBlock(rows, block);
  }

  return { header: hunk.header, rows };
}

export function toSplitDiffHunks(diff: FileDiff | null): SplitDiffHunk[] {
  if (!diff || diff.binary) return [];
  return diff.hunks.map(splitHunk);
}

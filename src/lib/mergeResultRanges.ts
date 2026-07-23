import type { MergeResolution } from "./threeWayMerge";

export interface ResultRange {
  id: string;
  from: number;
  to: number;
  resolution: MergeResolution;
}

export interface TextChange {
  from: number;
  to: number;
  insert: string;
}

function intersects(range: ResultRange, change: TextChange): boolean {
  if (change.from === change.to) {
    return change.from >= range.from && change.from <= range.to;
  }
  return change.from < range.to && change.to > range.from;
}

function mapPosition(position: number, change: TextChange, association: -1 | 1): number {
  const delta = change.insert.length - (change.to - change.from);
  if (position < change.from || (position === change.from && association < 0)) {
    return position;
  }
  if (position > change.to || (position === change.to && association > 0)) {
    return position + delta;
  }
  return association < 0 ? change.from : change.from + change.insert.length;
}

export function applyDocumentChange(
  ranges: ResultRange[],
  change: TextChange,
  markManual = true,
): ResultRange[] {
  return ranges.map(range => ({
    ...range,
    from: mapPosition(range.from, change, -1),
    to: mapPosition(range.to, change, 1),
    resolution: markManual && intersects(range, change) ? "manual" : range.resolution,
  }));
}

export function replaceConflictBlock(
  document: string,
  ranges: ResultRange[],
  id: string,
  insert: string,
  resolution: Extract<MergeResolution, "current" | "incoming">,
): { document: string; ranges: ResultRange[] } {
  const target = ranges.find(range => range.id === id);
  if (!target) return { document, ranges };

  const change = { from: target.from, to: target.to, insert };
  const nextRanges = applyDocumentChange(ranges, change, false).map(range => (
    range.id === id ? { ...range, resolution } : range
  ));

  return {
    document: document.slice(0, target.from) + insert + document.slice(target.to),
    ranges: nextRanges,
  };
}

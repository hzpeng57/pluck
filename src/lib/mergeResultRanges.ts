export type MergeSide = "current" | "incoming";
export type MergeSideAction = "accept" | "ignore";
export type MergeSideState = "pending" | "accepted" | "ignored";

export interface ResultRange {
  id: string;
  from: number;
  to: number;
  current: MergeSideState;
  incoming: MergeSideState;
  manual: boolean;
}

export interface ConflictBlockContents {
  base: string;
  current: string;
  incoming: string;
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

export function isResultRangeResolved(range: ResultRange): boolean {
  return range.manual || (range.current !== "pending" && range.incoming !== "pending");
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
    manual: range.manual || (markManual && intersects(range, change)),
  }));
}

function resultForStates(
  current: MergeSideState,
  incoming: MergeSideState,
  contents: ConflictBlockContents,
): string {
  const accepted: string[] = [];
  if (current === "accepted") accepted.push(contents.current);
  if (incoming === "accepted") accepted.push(contents.incoming);
  return accepted.length > 0 ? accepted.join("") : contents.base;
}

export function applyConflictSide(
  document: string,
  ranges: ResultRange[],
  id: string,
  side: MergeSide,
  action: MergeSideAction,
  contents: ConflictBlockContents,
): { document: string; ranges: ResultRange[] } {
  const target = ranges.find(range => range.id === id);
  if (!target) return { document, ranges };

  const nextState: MergeSideState = action === "accept" ? "accepted" : "ignored";
  const nextCurrent = side === "current" ? nextState : target.current;
  const nextIncoming = side === "incoming" ? nextState : target.incoming;
  const insert = resultForStates(nextCurrent, nextIncoming, contents);
  const change = { from: target.from, to: target.to, insert };
  const nextRanges = applyDocumentChange(ranges, change, false).map(range => (
    range.id === id
      ? {
          ...range,
          current: nextCurrent,
          incoming: nextIncoming,
          manual: false,
        }
      : range
  ));

  return {
    document: document.slice(0, target.from) + insert + document.slice(target.to),
    ranges: nextRanges,
  };
}

import { diff3Merge } from "node-diff3";

export type MergeResolution = "unresolved" | "current" | "incoming" | "manual";

export interface MergeConflictBlock {
  id: string;
  baseLines: string[];
  currentLines: string[];
  incomingLines: string[];
  baseStartLine: number;
  currentStartLine: number;
  incomingStartLine: number;
  resultStart: number;
  resultEnd: number;
  resolution: MergeResolution;
}

export interface ThreeWayMergeModel {
  currentLines: string[];
  incomingLines: string[];
  initialResult: string;
  conflicts: MergeConflictBlock[];
}

export type MergeSourceSide = "current" | "incoming";

export interface MergeSourceRow {
  lineNumber: number | null;
  content: string;
  conflictId: string | null;
  firstInConflict: boolean;
  placeholder: boolean;
}

export function splitTextLines(text: string): string[] {
  return text.match(/[^\r\n]*(?:\r\n|\r|\n)|[^\r\n]+$/g) ?? [];
}

export function createThreeWayMerge(
  currentText: string,
  baseText: string,
  incomingText: string,
): ThreeWayMergeModel {
  const currentLines = splitTextLines(currentText);
  const baseLines = splitTextLines(baseText);
  const incomingLines = splitTextLines(incomingText);
  const resultParts: string[] = [];
  const conflicts: MergeConflictBlock[] = [];
  let resultOffset = 0;

  const regions = diff3Merge(currentLines, baseLines, incomingLines, {
    excludeFalseConflicts: true,
  });

  for (const region of regions) {
    if (region.ok) {
      const text = region.ok.join("");
      resultParts.push(text);
      resultOffset += text.length;
      continue;
    }

    if (!region.conflict) continue;
    const baseTextForBlock = region.conflict.o.join("");
    conflicts.push({
      id: `conflict-${conflicts.length}`,
      baseLines: [...region.conflict.o],
      currentLines: [...region.conflict.a],
      incomingLines: [...region.conflict.b],
      baseStartLine: region.conflict.oIndex,
      currentStartLine: region.conflict.aIndex,
      incomingStartLine: region.conflict.bIndex,
      resultStart: resultOffset,
      resultEnd: resultOffset + baseTextForBlock.length,
      resolution: "unresolved",
    });
    resultParts.push(baseTextForBlock);
    resultOffset += baseTextForBlock.length;
  }

  return {
    currentLines,
    incomingLines,
    initialResult: resultParts.join(""),
    conflicts,
  };
}

export function buildSourceRows(
  lines: string[],
  conflicts: MergeConflictBlock[],
  side: MergeSourceSide,
): MergeSourceRow[] {
  const starts = new Map<number, MergeConflictBlock[]>();
  const lineOwners = new Map<number, { block: MergeConflictBlock; first: boolean }>();

  for (const block of conflicts) {
    const start = side === "current" ? block.currentStartLine : block.incomingStartLine;
    const blockLines = side === "current" ? block.currentLines : block.incomingLines;
    if (blockLines.length === 0) {
      starts.set(start, [...(starts.get(start) ?? []), block]);
      continue;
    }
    for (let offset = 0; offset < blockLines.length; offset++) {
      lineOwners.set(start + offset, { block, first: offset === 0 });
    }
  }

  const rows: MergeSourceRow[] = [];
  for (let index = 0; index <= lines.length; index++) {
    for (const block of starts.get(index) ?? []) {
      rows.push({
        lineNumber: null,
        content: "",
        conflictId: block.id,
        firstInConflict: true,
        placeholder: true,
      });
    }
    if (index === lines.length) break;
    const owner = lineOwners.get(index);
    rows.push({
      lineNumber: index + 1,
      content: lines[index],
      conflictId: owner?.block.id ?? null,
      firstInConflict: owner?.first ?? false,
      placeholder: false,
    });
  }
  return rows;
}

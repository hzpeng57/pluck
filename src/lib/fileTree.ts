import type { ChangedFile } from "../types/git";

interface FileTrieNode {
  segment: string;
  fullPath: string;
  file?: ChangedFile;
  children: Map<string, FileTrieNode>;
}

export type FileTreeEntry =
  | {
      kind: "folder";
      depth: number;
      prefix: string;
      label: string;
      fileCount: number;
      collapsed: boolean;
    }
  | {
      kind: "file";
      depth: number;
      file: ChangedFile;
      displayLabel: string;
    };

function insert(root: FileTrieNode, f: ChangedFile) {
  const parts = f.path.split("/");
  let node = root;
  let path = "";
  for (const seg of parts) {
    path = path ? `${path}/${seg}` : seg;
    let child = node.children.get(seg);
    if (!child) {
      child = { segment: seg, fullPath: path, children: new Map() };
      node.children.set(seg, child);
    }
    node = child;
  }
  node.file = f;
}

function fileCount(node: FileTrieNode): number {
  let n = node.file ? 1 : 0;
  for (const c of node.children.values()) n += fileCount(c);
  return n;
}

function onlyFile(node: FileTrieNode): ChangedFile | null {
  if (node.file) return node.file;
  for (const c of node.children.values()) {
    const f = onlyFile(c);
    if (f) return f;
  }
  return null;
}

function append(node: FileTrieNode, depth: number, out: FileTreeEntry[], collapsed: (p: string) => boolean) {
  const count = fileCount(node);

  // Single-file chain → flat leaf with remaining path as label.
  if (count === 1) {
    const file = onlyFile(node);
    if (!file) return;
    const displayLabel = file.path.split("/").slice(depth).join("/");
    out.push({ kind: "file", depth, file, displayLabel });
    return;
  }

  const isClosed = collapsed(node.fullPath);
  out.push({
    kind: "folder",
    depth,
    prefix: node.fullPath,
    label: node.segment,
    fileCount: count,
    collapsed: isClosed,
  });
  if (!isClosed) {
    // Folders first, then files. Within each, alphabetical.
    const kids = [...node.children.values()];
    kids.sort((a, b) => {
      const ad = a.children.size > 0 || fileCount(a) > 1 ? 0 : 1;
      const bd = b.children.size > 0 || fileCount(b) > 1 ? 0 : 1;
      if (ad !== bd) return ad - bd;
      return a.segment.localeCompare(b.segment);
    });
    for (const c of kids) append(c, depth + 1, out, collapsed);
  }
}

export function buildFileTree(files: ChangedFile[], collapsed: (p: string) => boolean): FileTreeEntry[] {
  const root: FileTrieNode = { segment: "", fullPath: "", children: new Map() };
  for (const f of files) insert(root, f);
  const out: FileTreeEntry[] = [];
  const kids = [...root.children.values()];
  kids.sort((a, b) => {
    const ad = a.children.size > 0 || fileCount(a) > 1 ? 0 : 1;
    const bd = b.children.size > 0 || fileCount(b) > 1 ? 0 : 1;
    if (ad !== bd) return ad - bd;
    return a.segment.localeCompare(b.segment);
  });
  for (const c of kids) append(c, 0, out, collapsed);
  return out;
}

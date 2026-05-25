import type { Branch } from "../types/git";

interface TrieNode {
  segment: string;
  fullPath: string;
  branch?: Branch;
  children: Map<string, TrieNode>;
}

export type TreeEntry =
  | {
      kind: "folder";
      depth: number;
      prefix: string;
      label: string;
      childCount: number;
      ahead: number;
      behind: number;
      collapsed: boolean;
      selfBranch?: Branch;
    }
  | {
      kind: "branch";
      depth: number;
      branch: Branch;
      displayLabel: string;
    };

function insert(root: TrieNode, b: Branch) {
  const parts = b.name.split("/");
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
  node.branch = b;
}

function descendantCount(node: TrieNode): number {
  let n = node.branch ? 1 : 0;
  for (const c of node.children.values()) n += descendantCount(c);
  return n;
}

function aggregate(node: TrieNode): { ahead: number; behind: number } {
  let ahead = node.branch?.ahead ?? 0;
  let behind = node.branch?.behind ?? 0;
  for (const c of node.children.values()) {
    const s = aggregate(c);
    ahead += s.ahead; behind += s.behind;
  }
  return { ahead, behind };
}

function onlyBranch(node: TrieNode): Branch | null {
  if (node.branch) return node.branch;
  for (const c of node.children.values()) {
    const b = onlyBranch(c);
    if (b) return b;
  }
  return null;
}

function append(node: TrieNode, depth: number, out: TreeEntry[], collapsed: (p: string) => boolean) {
  const count = descendantCount(node);

  // Single-branch chain → flatten to one leaf with its remaining segments as label.
  if (count === 1) {
    const branch = onlyBranch(node);
    if (!branch) return;
    const remaining = branch.name.split("/").slice(depth).join("/");
    out.push({ kind: "branch", depth, branch, displayLabel: remaining });
    return;
  }

  // 2+ branches under this node → render as folder; recurse when expanded.
  const stats = aggregate(node);
  const isClosed = collapsed(node.fullPath);
  out.push({
    kind: "folder",
    depth,
    prefix: node.fullPath,
    label: node.segment,
    childCount: count,
    ahead: stats.ahead,
    behind: stats.behind,
    collapsed: isClosed,
    selfBranch: node.branch,
  });
  if (!isClosed) {
    const sorted = [...node.children.values()].sort((a, b) => a.segment.localeCompare(b.segment));
    for (const c of sorted) append(c, depth + 1, out, collapsed);
  }
}

export function buildTree(branches: Branch[], collapsed: (p: string) => boolean): TreeEntry[] {
  const root: TrieNode = { segment: "", fullPath: "", children: new Map() };
  for (const b of branches) insert(root, b);
  const out: TreeEntry[] = [];
  const sorted = [...root.children.values()].sort((a, b) => a.segment.localeCompare(b.segment));
  for (const c of sorted) append(c, 0, out, collapsed);
  return out;
}

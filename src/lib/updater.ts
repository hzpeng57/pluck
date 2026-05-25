import { ref } from "vue";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

export type UpdaterState =
  | { kind: "idle" }
  | { kind: "checking" }
  | { kind: "uptodate" }
  | { kind: "available"; version: string; notes?: string }
  | { kind: "downloading"; percent: number }
  | { kind: "ready"; version: string }
  | { kind: "error"; message: string };

export const updaterState = ref<UpdaterState>({ kind: "idle" });

let pending: Update | null = null;

export async function checkForUpdates(showUpToDate = false): Promise<void> {
  updaterState.value = { kind: "checking" };
  try {
    const update = await check();
    if (!update) {
      pending = null;
      updaterState.value = showUpToDate ? { kind: "uptodate" } : { kind: "idle" };
      return;
    }
    pending = update;
    updaterState.value = {
      kind: "available",
      version: update.version,
      notes: update.body || undefined,
    };
  } catch (e: any) {
    updaterState.value = { kind: "error", message: e?.message ?? String(e) };
  }
}

export async function downloadAndInstall(): Promise<void> {
  if (!pending) return;
  const version = pending.version;
  let downloaded = 0;
  let total = 0;
  try {
    await pending.downloadAndInstall(evt => {
      switch (evt.event) {
        case "Started":
          total = evt.data.contentLength ?? 0;
          updaterState.value = { kind: "downloading", percent: 0 };
          break;
        case "Progress":
          downloaded += evt.data.chunkLength;
          updaterState.value = {
            kind: "downloading",
            percent: total > 0 ? Math.min(1, downloaded / total) : 0,
          };
          break;
        case "Finished":
          updaterState.value = { kind: "ready", version };
          break;
      }
    });
  } catch (e: any) {
    updaterState.value = { kind: "error", message: e?.message ?? String(e) };
  }
}

export async function restart(): Promise<void> {
  await relaunch();
}

export function dismiss(): void {
  pending = null;
  updaterState.value = { kind: "idle" };
}

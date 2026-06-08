export function formatErr(e: unknown): string {
  const err = e as any;
  if (err?.kind === "GitExit") return err.data?.friendly ?? err.data?.stderr ?? "git failed";
  if (typeof err?.data === "string") return err.data;
  if (typeof e === "string") return e;
  if (typeof err?.message === "string") return err.message;
  try {
    return JSON.stringify(e);
  } catch {
    return String(e);
  }
}

import {
  writeText,
  clear,
  readText,
} from "@tauri-apps/plugin-clipboard-manager";
import { isTauri } from "./utils";

let clearTimer: number | null = null;

/**
 * Copies `value` to the clipboard and auto-clears it after `ttlSeconds`
 * (only if the clipboard still holds the copied value).
 */
export async function copyWithAutoClear(value: string, ttlSeconds: number) {
  try {
    if (isTauri()) {
      await writeText(value);
    } else {
      await navigator.clipboard.writeText(value);
    }
  } catch (err) {
    console.error("clipboard write failed", err);
    return false;
  }

  if (clearTimer) window.clearTimeout(clearTimer);
  if (ttlSeconds <= 0) return true;

  clearTimer = window.setTimeout(async () => {
    try {
      if (isTauri()) {
        const current = await readText().catch(() => null);
        if (current === value) await clear();
      } else {
        const current = await navigator.clipboard.readText().catch(() => null);
        if (current === value) await navigator.clipboard.writeText("");
      }
    } catch {
      /* ignore */
    }
  }, ttlSeconds * 1000);
  return true;
}

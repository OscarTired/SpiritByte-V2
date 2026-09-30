import { convertFileSrc } from "@tauri-apps/api/core";
import { isTauri } from "@/lib/utils";

export function backgroundSource(value: string): string {
  return isTauri() && !value.startsWith("data:")
    ? convertFileSrc(value.replace(/\\/g, "/"))
    : value;
}

/** Check the actual decoder before replacing the user's working background. */
export function checkBackground(value: string): Promise<void> {
  return new Promise((resolve, reject) => {
    const image = new Image();
    const timeout = window.setTimeout(() => finish(false), 15000);
    function finish(ok: boolean) {
      window.clearTimeout(timeout);
      image.onload = image.onerror = null;
      image.removeAttribute("src");
      if (ok) resolve();
      else reject(new Error("wallpaper-decode"));
    }
    image.onload = () => finish(true);
    image.onerror = () => finish(false);
    image.src = backgroundSource(value);
  });
}

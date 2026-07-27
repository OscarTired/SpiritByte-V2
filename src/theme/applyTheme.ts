import type { Palette } from "./palettes";
import type { Settings } from "./settings";
import { isTauri } from "@/lib/utils";
import { convertFileSrc } from "@tauri-apps/api/core";

const VAR_MAP: Record<keyof Palette, string> = {
  bg: "--sb-bg",
  surface: "--sb-surface",
  surface2: "--sb-surface-2",
  border: "--sb-border",
  text: "--sb-text",
  textDim: "--sb-text-dim",
  primary: "--sb-primary",
  accent: "--sb-accent",
  danger: "--sb-danger",
  success: "--sb-success",
  warning: "--sb-warning",
};

export function applyTheme(settings: Settings) {
  const root = document.documentElement;
  const { palette } = settings;
  (Object.keys(VAR_MAP) as (keyof Palette)[]).forEach((key) => {
    root.style.setProperty(VAR_MAP[key], palette[key]);
  });

  root.classList.toggle("fx-scanlines", settings.scanlines);
  root.classList.toggle("fx-glow", settings.glow);
  root.classList.toggle("fx-flicker", settings.flicker);

  root.dataset.font = settings.font;

  root.style.setProperty("--sb-panel-opacity", String(settings.panelOpacity));
  root.style.setProperty("--sb-font-size", `${settings.fontSize}px`);

  // Background layer applied to <body>.
  const body = document.body;
  const { background } = settings;
  if (background.type === "image" && background.value) {
    let imgSrc = background.value;
    if (isTauri() && !imgSrc.startsWith("data:")) {
      imgSrc = convertFileSrc(imgSrc.replace(/\\/g, "/"));
      imgSrc += `?t=${Date.now()}`;
    }
    body.style.backgroundImage = `url("${imgSrc}")`;
    body.style.backgroundSize = "cover";
    body.style.backgroundPosition = "center";
  } else if (background.type === "gradient" && background.value.includes("|")) {
    const [a, b] = background.value.split("|");
    body.style.backgroundImage = `linear-gradient(135deg, ${a}, ${b})`;
    body.style.backgroundSize = "cover";
  } else {
    body.style.backgroundImage = "";
  }
}

/** Returns a CSS rgb() string for a palette channel. */
export function rgb(channel: string, alpha = 1): string {
  return alpha === 1 ? `rgb(${channel})` : `rgb(${channel} / ${alpha})`;
}

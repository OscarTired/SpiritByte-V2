import { DEFAULT_PRESET, type Palette } from "./palettes";

export type FontMode =
  | "mixed"
  | "pixel"
  | "mono"
  | "geist-square"
  | "geist-grid"
  | "geist-circle"
  | "geist-triangle"
  | "geist-line";

export interface BackgroundConfig {
  type: "solid" | "gradient" | "image";
  /** For solid: unused (uses palette bg). For gradient: "colorA|colorB". For image: file path (Tauri) or data URL (browser). */
  value: string;
}

export interface Settings {
  paletteId: string;
  palette: Palette;
  font: FontMode;
  scanlines: boolean;
  glow: boolean;
  flicker: boolean;
  ditherIntensity: number; // 1..4 pixel scale for the splash dither
  background: BackgroundConfig;
  panelOpacity: number; // 0..1 opacity for .panel elements
  panelBlur: number; // 0..20 px blur for .panel elements
  autoLockMinutes: number;
  clipboardClearSeconds: number;
  showSplash: boolean;
}

export const DEFAULT_SETTINGS: Settings = {
  paletteId: DEFAULT_PRESET.id,
  palette: DEFAULT_PRESET.palette,
  font: "mixed",
  scanlines: true,
  glow: true,
  flicker: true,
  ditherIntensity: 2,
  background: { type: "solid", value: "" },
  panelOpacity: 1,
  panelBlur: 0,
  autoLockMinutes: 5,
  clipboardClearSeconds: 20,
  showSplash: true,
};

export function mergeSettings(partial: unknown): Settings {
  if (!partial || typeof partial !== "object") return { ...DEFAULT_SETTINGS };
  const p = partial as Partial<Settings>;
  return {
    ...DEFAULT_SETTINGS,
    ...p,
    palette: { ...DEFAULT_SETTINGS.palette, ...(p.palette ?? {}) },
    background: { ...DEFAULT_SETTINGS.background, ...(p.background ?? {}) },
  };
}

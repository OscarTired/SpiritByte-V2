import { create } from "zustand";
import { api } from "@/lib/api";
import { isTauri } from "@/lib/utils";
import { applyTheme } from "@/theme/applyTheme";
import { PRESETS } from "@/theme/palettes";
import {
  DEFAULT_SETTINGS,
  mergeSettings,
  type Settings,
} from "@/theme/settings";
import { useI18n } from "@/lib/i18n";

const LS_KEY = "spiritbyte.settings";

interface SettingsState {
  settings: Settings;
  loaded: boolean;
  load: () => Promise<void>;
  update: (patch: Partial<Settings>) => void;
  setPaletteById: (id: string) => void;
  reset: () => void;
}

async function persist(settings: Settings) {
  try {
    if (isTauri()) {
      await api.saveSettings(settings);
    } else {
      localStorage.setItem(LS_KEY, JSON.stringify(settings));
    }
  } catch (err) {
    console.error("Failed to persist settings", err);
  }
}

export const useSettings = create<SettingsState>((set, get) => ({
  settings: DEFAULT_SETTINGS,
  loaded: false,
  load: async () => {
    let raw: unknown = null;
    try {
      raw = isTauri()
        ? await api.getSettings()
        : JSON.parse(localStorage.getItem(LS_KEY) || "null");
    } catch (err) {
      console.error("Failed to load settings", err);
    }
    const settings = mergeSettings(raw);
    applyTheme(settings);
    useI18n.getState().setLang(settings.language);
    set({ settings, loaded: true });
  },
  update: (patch) => {
    const settings = mergeSettings({ ...get().settings, ...patch });
    applyTheme(settings);
    if (patch.language) useI18n.getState().setLang(patch.language);
    set({ settings });
    void persist(settings);
  },
  setPaletteById: (id) => {
    const preset = PRESETS.find((p) => p.id === id);
    if (!preset) return;
    get().update({ paletteId: id, palette: preset.palette });
  },
  reset: () => {
    applyTheme(DEFAULT_SETTINGS);
    useI18n.getState().setLang(DEFAULT_SETTINGS.language);
    set({ settings: DEFAULT_SETTINGS });
    void persist(DEFAULT_SETTINGS);
  },
}));

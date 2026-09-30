import { BackupPanel } from "./BackupPanel";
import { useRef, useCallback, useState } from "react";
import { useSettings } from "@/store/useSettings";
import { useI18n, type TranslationKey } from "@/lib/i18n";
import { api } from "@/lib/api";
import { isTauri } from "@/lib/utils";
import { PRESETS, type Palette } from "@/theme/palettes";
import type { FontMode, Language } from "@/theme/settings";
import { checkBackground } from "@/theme/background";
import { Modal } from "@/components/ui/Modal";
import { Button } from "@/components/ui/Button";
import { Toggle } from "@/components/ui/Toggle";
import { Field, Input } from "@/components/ui/Input";

const CSS_VAR: Partial<Record<keyof Palette, string>> = {
  bg: "--sb-bg",
  surface: "--sb-surface",
  primary: "--sb-primary",
  accent: "--sb-accent",
  text: "--sb-text",
  danger: "--sb-danger",
};

const PALETTE_LABELS: { key: keyof Palette; labelKey: TranslationKey }[] = [
  { key: "bg", labelKey: "palette.bg" },
  { key: "surface", labelKey: "palette.surface" },
  { key: "primary", labelKey: "palette.primary" },
  { key: "accent", labelKey: "palette.accent" },
  { key: "text", labelKey: "palette.text" },
  { key: "danger", labelKey: "palette.danger" },
];

function channelToHex(channel: string): string {
  const [r, g, b] = channel.trim().split(/\s+/).map((n) => parseInt(n, 10));
  return (
    "#" +
    [r, g, b].map((c) => (c || 0).toString(16).padStart(2, "0")).join("")
  );
}
function hexToChannel(hex: string): string {
  const v = hex.replace("#", "");
  const r = parseInt(v.slice(0, 2), 16);
  const g = parseInt(v.slice(2, 4), 16);
  const b = parseInt(v.slice(4, 6), 16);
  return `${r} ${g} ${b}`;
}

const FONTS: { id: FontMode; labelKey: TranslationKey }[] = [
  { id: "mixed", labelKey: "settings.mixed" },
  { id: "mono", labelKey: "mono" as TranslationKey },
  { id: "pixel", labelKey: "pixel" as TranslationKey },
  { id: "geist-square", labelKey: "geist-square" as TranslationKey },
  { id: "geist-grid", labelKey: "geist-grid" as TranslationKey },
  { id: "geist-circle", labelKey: "geist-circle" as TranslationKey },
  { id: "geist-triangle", labelKey: "geist-triangle" as TranslationKey },
  { id: "geist-line", labelKey: "geist-line" as TranslationKey },
];

const FONT_LABELS: Record<string, string> = {
  mono: "VT323",
  pixel: "Press Start",
  "geist-square": "GP Square",
  "geist-grid": "GP Grid",
  "geist-circle": "GP Circle",
  "geist-triangle": "GP Triangle",
  "geist-line": "GP Line",
};

const LANGS: { id: Language; labelKey: TranslationKey }[] = [
  { id: "es", labelKey: "settings.spanish" },
  { id: "en", labelKey: "settings.english" },
];

export function SettingsPanel({ open, onClose }: { open: boolean; onClose: () => void }) {
  const { settings, update, setPaletteById, reset } = useSettings();
  const t = useI18n((s) => s.t);
  const fileRef = useRef<HTMLInputElement>(null);
  const colorDebounce = useRef<ReturnType<typeof setTimeout> | null>(null);
  const [imageBusy, setImageBusy] = useState(false);
  const [imageError, setImageError] = useState("");

  const setPaletteColor = useCallback(
    (key: keyof Palette, hex: string) => {
      if (colorDebounce.current) clearTimeout(colorDebounce.current);
      colorDebounce.current = setTimeout(() => {
        update({
          paletteId: "custom",
          palette: { ...useSettings.getState().settings.palette, [key]: hexToChannel(hex) },
        });
      }, 250);
    },
    [update],
  );

  function livePreviewColor(key: keyof Palette, hex: string) {
    const cssVar = CSS_VAR[key];
    if (cssVar) document.documentElement.style.setProperty(cssVar, hexToChannel(hex));
  }

  async function pickImage(file?: File) {
    setImageBusy(true);
    setImageError("");
    try {
      let value: string | null;
      if (isTauri()) {
        value = await api.pickWallpaper();
      } else {
        if (!file) return;
        if (file.size > 32 * 1024 * 1024) throw new Error("wallpaper-too-large");
        value = await new Promise<string>((resolve, reject) => {
          const reader = new FileReader();
          reader.onload = () => resolve(String(reader.result));
          reader.onerror = () => reject(reader.error);
          reader.readAsDataURL(file);
        });
      }
      if (!value) return; // Native dialog cancelled.
      await checkBackground(value);
      update({ background: { type: "image", value } });
    } catch (error) {
      const reason = String(error);
      setImageError(t(reason.includes("wallpaper-too-large")
        ? "settings.imageTooLarge"
        : "settings.imageFailed"));
    } finally {
      setImageBusy(false);
      if (fileRef.current) fileRef.current.value = "";
    }
  }

  return (
    <Modal open={open} onClose={onClose} title={t("settings.title")} width={620}>
      <div className="space-y-6">
        <BackupPanel />
        <section>
          <h3 className="label mb-2">{t("settings.palettes")}</h3>
          <div className="grid grid-cols-5 gap-2">
            {PRESETS.map((p) => (
              <button
                key={p.id}
                onClick={() => setPaletteById(p.id)}
                className={`flex flex-col items-center gap-1 border-2 p-2 ${
                  settings.paletteId === p.id ? "border-primary" : "border-border"
                }`}
              >
                <span className="flex gap-1">
                  {["bg", "surface", "primary", "accent"].map((k) => (
                    <span
                      key={k}
                      className="h-3 w-3 border border-black/40"
                      style={{ background: `rgb(${p.palette[k as keyof Palette]})` }}
                    />
                  ))}
                </span>
                <span className="text-xs text-text-dim text-center leading-tight">
                  {p.name}
                </span>
              </button>
            ))}
          </div>
        </section>

        <section>
          <h3 className="label mb-2">{t("settings.customColors")}</h3>
          <div className="grid grid-cols-3 gap-3">
            {PALETTE_LABELS.map(({ key, labelKey }) => (
              <label key={key} className="flex items-center gap-2">
                <input
                  type="color"
                  value={channelToHex(settings.palette[key])}
                  onInput={(e) => {
                    livePreviewColor(key, e.currentTarget.value);
                    setPaletteColor(key, e.currentTarget.value);
                  }}
                  className="h-8 w-8 cursor-pointer border-2 border-border bg-transparent"
                />
                <span className="text-term text-text-dim">{t(labelKey)}</span>
              </label>
            ))}
          </div>
        </section>

        <section>
          <h3 className="label mb-2">{t("settings.typography")}</h3>
          <div className="flex flex-wrap gap-2">
            {FONTS.map((f) => (
              <Button
                key={f.id}
                variant={settings.font === f.id ? "primary" : "default"}
                onClick={() => update({ font: f.id })}
              >
                {f.id === "mixed" ? t(f.labelKey) : FONT_LABELS[f.id]}
              </Button>
            ))}
          </div>
          <div className="flex items-center gap-3 mt-3">
            <span className="text-text-dim text-sm w-24">{t("settings.fontSize")}</span>
            <input
              type="range"
              min={10}
              max={28}
              step={1}
              value={settings.fontSize}
              onChange={(e) => update({ fontSize: Number(e.target.value) })}
              className="flex-1 accent-[rgb(var(--sb-primary))]"
            />
            <span className="text-text-dim text-sm w-10 text-right">
              {settings.fontSize}px
            </span>
          </div>
        </section>

        <section>
          <h3 className="label mb-2">{t("settings.language")}</h3>
          <div className="flex gap-2">
            {LANGS.map((l) => (
              <Button
                key={l.id}
                variant={settings.language === l.id ? "primary" : "default"}
                onClick={() => update({ language: l.id })}
              >
                {t(l.labelKey)}
              </Button>
            ))}
          </div>
        </section>

        <section>
          <h3 className="label mb-2">{t("settings.panels")}</h3>
          <div className="space-y-3">
            <label className="flex items-center gap-3">
              <span className="text-text-dim text-sm w-20">{t("settings.opacity")}</span>
              <input
                type="range"
                min={0}
                max={1}
                step={0.05}
                value={settings.panelOpacity}
                onChange={(e) => update({ panelOpacity: Number(e.target.value) })}
                className="flex-1 accent-[rgb(var(--sb-primary))]"
              />
              <span className="text-text-dim text-sm w-10 text-right">
                {Math.round(settings.panelOpacity * 100)}%
              </span>
            </label>
          </div>
        </section>

        <section className="grid grid-cols-2 gap-3">
          <Toggle
            label={t("settings.lowPower")}
            checked={settings.lowPowerMode}
            onChange={(v) => update({ lowPowerMode: v })}
          />
          <Toggle
            label={t("settings.scanlines")}
            checked={settings.scanlines}
            onChange={(v) => update({ scanlines: v })}
          />
          <Toggle label={t("settings.glow")} checked={settings.glow} onChange={(v) => update({ glow: v })} />
          <Toggle
            label={t("settings.flicker")}
            checked={settings.flicker && !settings.lowPowerMode}
            disabled={settings.lowPowerMode}
            onChange={(v) => update({ flicker: v })}
          />
          <Toggle
            label={t("settings.splashOnStart")}
            checked={settings.showSplash}
            onChange={(v) => update({ showSplash: v })}
          />
        </section>
        <p className="text-sm text-text-dim">{t("settings.lowPowerHint")}</p>

        <section>
          <h3 className="label mb-2">{t("settings.background")}</h3>
          <div className="flex gap-2 mb-2">
            <Button
              variant={settings.background.type === "solid" ? "primary" : "default"}
              disabled={imageBusy}
              onClick={() => {
                setImageError("");
                update({ background: { type: "solid", value: "" } });
              }}
            >
              {t("settings.solid")}
            </Button>
            <Button
              variant={settings.background.type === "gradient" ? "primary" : "default"}
              disabled={imageBusy}
              onClick={() => {
                setImageError("");
                update({
                  background: { type: "gradient", value: "#1a1712|#0d0c0a" },
                });
              }}
            >
              {t("settings.gradient")}
            </Button>
            <Button
              variant={settings.background.type === "image" ? "primary" : "default"}
              disabled={imageBusy}
              onClick={() => isTauri() ? void pickImage() : fileRef.current?.click()}
            >
              {t("settings.image")}
            </Button>
            <input
              ref={fileRef}
              type="file"
              accept=".png,.jpg,.jpeg,.gif,.webp,.bmp,image/png,image/jpeg,image/gif,image/webp,image/bmp"
              className="hidden"
              onChange={(e) => {
                const file = e.currentTarget.files?.[0];
                if (file) void pickImage(file);
              }}
            />
          </div>
          {imageError && <p role="alert" className="text-sm text-danger">{imageError}</p>}
          {settings.background.type === "gradient" && (
            <div className="flex gap-2">
              {settings.background.value.split("|").map((c, i) => (
                <input
                  key={i}
                  type="color"
                  value={c}
                  onInput={(e) => {
                    const parts = settings.background.value.split("|");
                    parts[i] = e.currentTarget.value;
                    document.body.style.backgroundImage = `linear-gradient(135deg, ${parts[0]}, ${parts[1]})`;
                    if (colorDebounce.current) clearTimeout(colorDebounce.current);
                    colorDebounce.current = setTimeout(() => {
                      update({
                        background: { type: "gradient", value: parts.join("|") },
                      });
                    }, 250);
                  }}
                  className="h-8 w-12 border-2 border-border bg-transparent"
                />
              ))}
            </div>
          )}
        </section>

        <section className="grid grid-cols-2 gap-3">
          <Field label={t("settings.autoLock")}>
            <Input
              type="number"
              min={0}
              value={settings.autoLockMinutes}
              onChange={(e) => update({ autoLockMinutes: Number(e.target.value) })}
            />
          </Field>
          <Field label={t("settings.clipboardClear")}>
            <Input
              type="number"
              min={0}
              value={settings.clipboardClearSeconds}
              onChange={(e) =>
                update({ clipboardClearSeconds: Number(e.target.value) })
              }
            />
          </Field>
          <Field label={t("settings.dither")}>
            <Input
              type="number"
              min={1}
              max={4}
              value={settings.ditherIntensity}
              onChange={(e) => update({ ditherIntensity: Number(e.target.value) })}
            />
          </Field>
        </section>

        <div className="flex justify-end">
          <Button variant="danger" onClick={reset}>
            {t("settings.reset")}
          </Button>
        </div>
      </div>
    </Modal>
  );
}

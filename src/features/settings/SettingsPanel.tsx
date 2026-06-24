import { useRef, useCallback } from "react";
import { useSettings } from "@/store/useSettings";
import { api } from "@/lib/api";
import { isTauri } from "@/lib/utils";
import { PRESETS, type Palette } from "@/theme/palettes";
import type { FontMode } from "@/theme/settings";
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

const PALETTE_LABELS: { key: keyof Palette; label: string }[] = [
  { key: "bg", label: "Fondo" },
  { key: "surface", label: "Panel" },
  { key: "primary", label: "Primario" },
  { key: "accent", label: "Acento" },
  { key: "text", label: "Texto" },
  { key: "danger", label: "Peligro" },
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

const FONTS: { id: FontMode; label: string }[] = [
  { id: "mixed", label: "Mixta" },
  { id: "mono", label: "VT323" },
  { id: "pixel", label: "Press Start" },
  { id: "geist-square", label: "GP Square" },
  { id: "geist-grid", label: "GP Grid" },
  { id: "geist-circle", label: "GP Circle" },
  { id: "geist-triangle", label: "GP Triangle" },
  { id: "geist-line", label: "GP Line" },
];

export function SettingsPanel({ open, onClose }: { open: boolean; onClose: () => void }) {
  const { settings, update, setPaletteById, reset } = useSettings();
  const fileRef = useRef<HTMLInputElement>(null);
  const colorDebounce = useRef<ReturnType<typeof setTimeout> | null>(null);

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

  async function onPickImage(e: React.ChangeEvent<HTMLInputElement>) {
    const file = e.target.files?.[0];
    if (!file) return;
    const ext = file.name.split(".").pop()?.toLowerCase() || "png";
    if (isTauri()) {
      const bytes = new Uint8Array(await file.arrayBuffer());
      const path = await api.saveWallpaper(Array.from(bytes), ext);
      update({ background: { type: "image", value: path } });
    } else {
      const reader = new FileReader();
      reader.onload = () =>
        update({ background: { type: "image", value: String(reader.result) } });
      reader.readAsDataURL(file);
    }
  }

  return (
    <Modal open={open} onClose={onClose} title="Personalización" width={620}>
      <div className="space-y-6">
        <section>
          <h3 className="label mb-2">Paletas</h3>
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
                <span className="text-[10px] text-text-dim text-center leading-tight">
                  {p.name}
                </span>
              </button>
            ))}
          </div>
        </section>

        <section>
          <h3 className="label mb-2">Colores personalizados</h3>
          <div className="grid grid-cols-3 gap-3">
            {PALETTE_LABELS.map(({ key, label }) => (
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
                <span className="text-term text-text-dim">{label}</span>
              </label>
            ))}
          </div>
        </section>

        <section>
          <h3 className="label mb-2">Tipografía</h3>
          <div className="flex flex-wrap gap-2">
            {FONTS.map((f) => (
              <Button
                key={f.id}
                variant={settings.font === f.id ? "primary" : "default"}
                onClick={() => update({ font: f.id })}
              >
                {f.label}
              </Button>
            ))}
          </div>
        </section>

        <section>
          <h3 className="label mb-2">Paneles</h3>
          <div className="space-y-3">
            <label className="flex items-center gap-3">
              <span className="text-text-dim text-sm w-20">Opacidad</span>
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
            <label className="flex items-center gap-3">
              <span className="text-text-dim text-sm w-20">Desenfoque</span>
              <input
                type="range"
                min={0}
                max={20}
                step={1}
                value={settings.panelBlur}
                onChange={(e) => update({ panelBlur: Number(e.target.value) })}
                className="flex-1 accent-[rgb(var(--sb-primary))]"
              />
              <span className="text-text-dim text-sm w-10 text-right">
                {settings.panelBlur}px
              </span>
            </label>
          </div>
        </section>

        <section className="grid grid-cols-2 gap-3">
          <Toggle
            label="Scanlines"
            checked={settings.scanlines}
            onChange={(v) => update({ scanlines: v })}
          />
          <Toggle label="Glow" checked={settings.glow} onChange={(v) => update({ glow: v })} />
          <Toggle
            label="Flicker"
            checked={settings.flicker}
            onChange={(v) => update({ flicker: v })}
          />
          <Toggle
            label="Splash al iniciar"
            checked={settings.showSplash}
            onChange={(v) => update({ showSplash: v })}
          />
        </section>

        <section>
          <h3 className="label mb-2">Fondo</h3>
          <div className="flex gap-2 mb-2">
            <Button
              variant={settings.background.type === "solid" ? "primary" : "default"}
              onClick={() => {
                if (isTauri()) void api.deleteWallpaper();
                update({ background: { type: "solid", value: "" } });
              }}
            >
              Sólido
            </Button>
            <Button
              variant={settings.background.type === "gradient" ? "primary" : "default"}
              onClick={() => {
                if (isTauri()) void api.deleteWallpaper();
                update({
                  background: { type: "gradient", value: "#1a1712|#0d0c0a" },
                });
              }}
            >
              Gradiente
            </Button>
            <Button
              variant={settings.background.type === "image" ? "primary" : "default"}
              onClick={() => fileRef.current?.click()}
            >
              Imagen
            </Button>
            <input
              ref={fileRef}
              type="file"
              accept="image/*"
              className="hidden"
              onChange={onPickImage}
            />
          </div>
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
          <Field label="Auto-bloqueo (min, 0=off)">
            <Input
              type="number"
              min={0}
              value={settings.autoLockMinutes}
              onChange={(e) => update({ autoLockMinutes: Number(e.target.value) })}
            />
          </Field>
          <Field label="Limpiar portapapeles (seg)">
            <Input
              type="number"
              min={0}
              value={settings.clipboardClearSeconds}
              onChange={(e) =>
                update({ clipboardClearSeconds: Number(e.target.value) })
              }
            />
          </Field>
          <Field label="Dither del zorro (1-4)">
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
            Restablecer
          </Button>
        </div>
      </div>
    </Modal>
  );
}

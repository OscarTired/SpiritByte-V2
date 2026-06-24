import { useEffect, useState } from "react";
import { Check, Copy, RefreshCw } from "lucide-react";
import { api } from "@/lib/api";
import type { GenOptions } from "@/lib/types";
import { Button } from "@/components/ui/Button";
import { Toggle } from "@/components/ui/Toggle";
import { StrengthMeter } from "@/components/StrengthMeter";
import { copyWithAutoClear } from "@/lib/clipboard";
import { useSettings } from "@/store/useSettings";

const DEFAULTS: GenOptions = {
  length: 20,
  lowercase: true,
  uppercase: true,
  digits: true,
  symbols: true,
  avoidAmbiguous: false,
};

interface Props {
  onUse?: (password: string) => void;
}

export function GeneratorPanel({ onUse }: Props) {
  const clipboardTtl = useSettings((s) => s.settings.clipboardClearSeconds);
  const [opts, setOpts] = useState<GenOptions>(DEFAULTS);
  const [value, setValue] = useState("");
  const [copied, setCopied] = useState(false);

  async function regen(o = opts) {
    const pw = await api.generatePassword(o);
    setValue(pw);
  }

  useEffect(() => {
    void regen(opts);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [opts]);

  function set<K extends keyof GenOptions>(key: K, v: GenOptions[K]) {
    setOpts((o) => ({ ...o, [key]: v }));
  }

  async function copy() {
    await copyWithAutoClear(value, clipboardTtl);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center gap-2 border-2 border-border bg-bg p-3">
        <span className="flex-1 break-all text-term text-primary glow-text">
          {value || "—"}
        </span>
        <Button variant="ghost" className="px-2 py-1" onClick={() => void regen()}>
          <RefreshCw size={16} />
        </Button>
        <Button variant="ghost" className="px-2 py-1" onClick={copy}>
          {copied ? <Check size={16} /> : <Copy size={16} />}
        </Button>
      </div>

      <StrengthMeter password={value} />

      <div>
        <div className="flex justify-between label">
          <span>Longitud</span>
          <span className="text-primary">{opts.length}</span>
        </div>
        <input
          type="range"
          min={8}
          max={64}
          value={opts.length}
          onChange={(e) => set("length", Number(e.target.value))}
          className="w-full accent-[rgb(var(--sb-primary))]"
        />
      </div>

      <div className="grid grid-cols-2 gap-3">
        <Toggle label="a-z" checked={opts.lowercase} onChange={(v) => set("lowercase", v)} />
        <Toggle label="A-Z" checked={opts.uppercase} onChange={(v) => set("uppercase", v)} />
        <Toggle label="0-9" checked={opts.digits} onChange={(v) => set("digits", v)} />
        <Toggle label="!@#$" checked={opts.symbols} onChange={(v) => set("symbols", v)} />
        <Toggle
          label="Sin ambiguos"
          checked={opts.avoidAmbiguous}
          onChange={(v) => set("avoidAmbiguous", v)}
          className="col-span-2"
        />
      </div>

      {onUse && (
        <Button variant="primary" className="w-full" onClick={() => onUse(value)}>
          Usar esta contraseña
        </Button>
      )}
    </div>
  );
}

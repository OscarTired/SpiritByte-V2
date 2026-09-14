import { useEffect, useState } from "react";
import { api } from "@/lib/api";
import { useI18n } from "@/lib/i18n";
import type { TranslationKey } from "@/lib/i18n";
import type { Strength } from "@/lib/types";
import { cn } from "@/lib/utils";

const COLORS = [
  "bg-danger",
  "bg-danger",
  "bg-warning",
  "bg-accent",
  "bg-success",
];

const LABEL_KEYS: TranslationKey[] = [
  "strength.veryWeak",
  "strength.weak",
  "strength.fair",
  "strength.strong",
  "strength.veryStrong",
];

export function StrengthMeter({ password }: { password: string }) {
  const t = useI18n((s) => s.t);
  const [s, setS] = useState<Strength | null>(null);

  useEffect(() => {
    let active = true;
    if (!password) {
      setS(null);
      return;
    }
    const timer = window.setTimeout(() => {
      api.passwordStrength(password).then((r) => active && setS(r)).catch(() => {});
    }, 150);
    return () => {
      active = false;
      window.clearTimeout(timer);
    };
  }, [password]);

  const score = s?.score ?? 0;

  return (
    <div className="mt-2">
      <div className="flex gap-1">
        {[0, 1, 2, 3, 4].map((i) => (
          <div
            key={i}
            className={cn(
              "h-2 flex-1 border border-border transition-colors duration-150",
              password && i <= score ? COLORS[score] : "bg-bg",
            )}
          />
        ))}
      </div>
      {password && (
        <div className="flex justify-between mt-1 text-term text-text-dim">
          <span className="uppercase">{s ? t(LABEL_KEYS[score]) : "\u00A0"}</span>
          <span>{s ? `${Math.round(s.entropyBits)} bits` : "\u00A0"}</span>
        </div>
      )}
    </div>
  );
}

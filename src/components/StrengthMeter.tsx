import { useEffect, useState } from "react";
import { api } from "@/lib/api";
import type { Strength } from "@/lib/types";
import { cn } from "@/lib/utils";

const COLORS = [
  "bg-danger",
  "bg-danger",
  "bg-warning",
  "bg-accent",
  "bg-success",
];

export function StrengthMeter({ password }: { password: string }) {
  const [s, setS] = useState<Strength | null>(null);

  useEffect(() => {
    let active = true;
    if (!password) {
      setS(null);
      return;
    }
    api
      .passwordStrength(password)
      .then((r) => active && setS(r))
      .catch(() => {});
    return () => {
      active = false;
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
              "h-2 flex-1 border border-border transition-colors",
              password && i <= score ? COLORS[score] : "bg-bg",
            )}
          />
        ))}
      </div>
      {s && (
        <div className="flex justify-between mt-1 text-term text-text-dim">
          <span className="uppercase">{s.label}</span>
          <span>{Math.round(s.entropyBits)} bits</span>
        </div>
      )}
    </div>
  );
}

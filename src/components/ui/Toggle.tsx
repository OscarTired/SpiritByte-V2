import { cn } from "@/lib/utils";

interface ToggleProps {
  checked: boolean;
  onChange: (v: boolean) => void;
  label?: string;
  className?: string;
}

export function Toggle({ checked, onChange, label, className }: ToggleProps) {
  return (
    <button
      type="button"
      onClick={() => onChange(!checked)}
      className={cn("flex items-center gap-2 select-none", className)}
    >
      <span
        className={cn(
          "relative inline-block h-5 w-9 border-2 transition-colors",
          checked ? "border-primary bg-primary/20" : "border-border bg-bg",
        )}
      >
        <span
          className={cn(
            "absolute top-0 h-[14px] w-[14px] transition-all",
            checked ? "left-[18px] bg-primary" : "left-0 bg-text-dim",
          )}
        />
      </span>
      {label && (
        <span className="font-pixel text-[9px] uppercase tracking-wider text-text">
          {label}
        </span>
      )}
    </button>
  );
}

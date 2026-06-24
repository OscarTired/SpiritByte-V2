import { type ReactNode, useEffect } from "react";
import { X } from "lucide-react";
import { Button } from "./Button";

interface ModalProps {
  open: boolean;
  onClose: () => void;
  title: string;
  children: ReactNode;
  footer?: ReactNode;
  width?: number;
}

export function Modal({ open, onClose, title, children, footer, width = 520 }: ModalProps) {
  useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open, onClose]);

  if (!open) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-4"
      onMouseDown={onClose}
    >
      <div
        className="panel pixel-border w-full max-h-[88vh] flex flex-col"
        style={{ maxWidth: width }}
        onMouseDown={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between border-b-2 border-border px-4 py-3 bg-surface-2">
          <h2 className="font-pixel text-[12px] text-primary glow-text uppercase tracking-wider">
            {title}
          </h2>
          <Button variant="ghost" className="px-2 py-1" onClick={onClose}>
            <X size={16} />
          </Button>
        </div>
        <div className="overflow-y-auto p-4 flex-1">{children}</div>
        {footer && (
          <div className="border-t-2 border-border px-4 py-3 flex justify-end gap-2 bg-surface-2">
            {footer}
          </div>
        )}
      </div>
    </div>
  );
}

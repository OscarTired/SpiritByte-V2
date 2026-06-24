import { useCallback, useEffect, useRef } from "react";

interface ResizeHandleProps {
  /** Current width in pixels */
  width: number;
  /** Minimum width in pixels */
  min: number;
  /** Maximum width in pixels */
  max: number;
  /** Callback when width changes during drag */
  onResize: (width: number) => void;
}

export function ResizeHandle({ width, min, max, onResize }: ResizeHandleProps) {
  const dragging = useRef(false);
  const startX = useRef(0);
  const startWidth = useRef(0);

  const onMouseDown = useCallback(
    (e: React.MouseEvent) => {
      e.preventDefault();
      dragging.current = true;
      startX.current = e.clientX;
      startWidth.current = width;
      document.body.style.cursor = "col-resize";
      document.body.style.userSelect = "none";
    },
    [width],
  );

  useEffect(() => {
    function onMouseMove(e: MouseEvent) {
      if (!dragging.current) return;
      const delta = e.clientX - startX.current;
      const next = Math.max(min, Math.min(max, startWidth.current + delta));
      onResize(next);
    }
    function onMouseUp() {
      if (!dragging.current) return;
      dragging.current = false;
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
    }
    window.addEventListener("mousemove", onMouseMove);
    window.addEventListener("mouseup", onMouseUp);
    return () => {
      window.removeEventListener("mousemove", onMouseMove);
      window.removeEventListener("mouseup", onMouseUp);
    };
  }, [min, max, onResize]);

  return (
    <div
      onMouseDown={onMouseDown}
      className="shrink-0 w-1 cursor-col-resize bg-transparent hover:bg-primary/40 transition-colors relative group"
      style={{ zIndex: 10 }}
    >
      <div className="absolute inset-y-0 -left-1 -right-1" />
    </div>
  );
}

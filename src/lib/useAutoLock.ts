import { useEffect, useRef } from "react";
import { useVault } from "@/store/useVault";
import { useSettings } from "@/store/useSettings";

/**
 * Locks the vault after `autoLockMinutes` of user inactivity.
 * Activity = mouse / keyboard / touch events.
 */
export function useAutoLock() {
  const screen = useVault((s) => s.screen);
  const lock = useVault((s) => s.lock);
  const minutes = useSettings((s) => s.settings.autoLockMinutes);
  const timer = useRef<number | null>(null);

  useEffect(() => {
    if (screen !== "unlocked" || minutes <= 0) return;

    const timeout = minutes * 60 * 1000;
    let lastActivity = Date.now();
    const reset = () => { lastActivity = Date.now(); };
    const check = () => {
      const remaining = timeout - (Date.now() - lastActivity);
      if (remaining <= 0) void lock();
      else timer.current = window.setTimeout(check, remaining);
    };
    timer.current = window.setTimeout(check, timeout);

    const events = ["mousemove", "mousedown", "keydown", "wheel", "touchstart"];
    events.forEach((e) => window.addEventListener(e, reset, { passive: true }));
    reset();

    return () => {
      events.forEach((e) => window.removeEventListener(e, reset));
      if (timer.current) window.clearTimeout(timer.current);
    };
  }, [screen, minutes, lock]);
}

import { lazy, Suspense, useEffect, useState } from "react";
import { useVault } from "@/store/useVault";
import { useSettings } from "@/store/useSettings";
import { useAutoLock } from "@/lib/useAutoLock";

const SplashFox = lazy(() => import("@/components/SplashFox").then((m) => ({ default: m.SplashFox })));
const Onboarding = lazy(() => import("@/screens/Onboarding").then((m) => ({ default: m.Onboarding })));
const Unlock = lazy(() => import("@/screens/Unlock").then((m) => ({ default: m.Unlock })));
const VaultApp = lazy(() => import("@/screens/VaultApp").then((m) => ({ default: m.VaultApp })));

export default function App() {
  const screen = useVault((s) => s.screen);
  const booted = useVault((s) => s.booted);
  const init = useVault((s) => s.init);
  const finishBoot = useVault((s) => s.finishBoot);
  const loadSettings = useSettings((s) => s.load);
  const showSplash = useSettings((s) => s.settings.showSplash);
  const settingsLoaded = useSettings((s) => s.loaded);

  const [ready, setReady] = useState(false);

  useAutoLock();

  useEffect(() => {
    (async () => {
      await loadSettings();
      await init();
      setReady(true);
    })();
  }, [init, loadSettings]);

  // Skip splash entirely if disabled in settings.
  useEffect(() => {
    if (ready && settingsLoaded && !showSplash && !booted) {
      void finishBoot();
    }
  }, [ready, settingsLoaded, showSplash, booted, finishBoot]);

  const splashVisible = ready && settingsLoaded && showSplash && !booted;

  return (
    <div className="h-full">
      <Suspense fallback={<div className="h-full" />}>
        {splashVisible && <SplashFox onDone={() => void finishBoot()} />}

        {booted && (
          <>
            {screen === "onboarding" && <Onboarding />}
            {screen === "locked" && <Unlock />}
            {screen === "unlocked" && <VaultApp />}
          </>
        )}

        {!booted && !splashVisible && <div className="h-full bg-bg" />}
      </Suspense>
    </div>
  );
}

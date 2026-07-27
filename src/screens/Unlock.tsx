import { useState } from "react";
import { Eye, EyeOff, KeyRound, LifeBuoy, Lock } from "lucide-react";
import { useVault } from "@/store/useVault";
import { useI18n } from "@/lib/i18n";
import { Button } from "@/components/ui/Button";
import { Input, Field } from "@/components/ui/Input";
import { StrengthMeter } from "@/components/StrengthMeter";

type Mode = "password" | "recovery";

export function Unlock() {
  const unlock = useVault((s) => s.unlock);
  const recoverAndReset = useVault((s) => s.recoverAndReset);
  const t = useI18n((s) => s.t);

  const [mode, setMode] = useState<Mode>("password");
  const [password, setPassword] = useState("");
  const [show, setShow] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Recovery flow
  const [phrase, setPhrase] = useState("");
  const [newPw, setNewPw] = useState("");
  const [newPw2, setNewPw2] = useState("");

  async function handleUnlock() {
    setError(null);
    setBusy(true);
    try {
      await unlock(password);
    } catch {
      setError(t("unlock.errWrongPassword"));
    } finally {
      setBusy(false);
    }
  }

  async function handleRecover() {
    setError(null);
    if (phrase.trim().split(/\s+/).length !== 12) {
      setError(t("unlock.errPhraseLength"));
      return;
    }
    if (newPw.length < 8) {
      setError(t("unlock.errNewPasswordShort"));
      return;
    }
    if (newPw !== newPw2) {
      setError(t("unlock.errPasswordMismatch"));
      return;
    }
    setBusy(true);
    try {
      await recoverAndReset(phrase, newPw);
    } catch {
      setError(t("unlock.errInvalidPhrase"));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="flex h-full items-center justify-center p-6">
      <div className="panel pixel-border w-full max-w-md p-6">
        <div className="flex items-center gap-3 mb-5">
          <Lock className="text-primary" size={26} />
          <div>
            <h1 className="font-pixel text-base text-primary glow-text">SPIRITBYTE</h1>
            <p className="text-term text-text-dim">
              {mode === "password" ? t("unlock.locked") : t("unlock.recover")}
            </p>
          </div>
        </div>

        {mode === "password" ? (
          <div className="space-y-4">
            <Field label={t("unlock.masterPassword")}>
              <div className="relative">
                <Input
                  type={show ? "text" : "password"}
                  value={password}
                  autoFocus
                  onChange={(e) => setPassword(e.target.value)}
                  onKeyDown={(e) => e.key === "Enter" && handleUnlock()}
                  placeholder="********"
                />
                <button
                  type="button"
                  className="absolute right-2 top-1/2 -translate-y-1/2 text-text-dim hover:text-primary"
                  onClick={() => setShow((v) => !v)}
                >
                  {show ? <EyeOff size={18} /> : <Eye size={18} />}
                </button>
              </div>
            </Field>
            {error && <p className="text-danger text-term">{error}</p>}
            <Button
              variant="primary"
              className="w-full"
              disabled={busy}
              onClick={handleUnlock}
            >
              <KeyRound size={16} />
              {busy ? t("unlock.decrypting") : t("unlock.unlock")}
            </Button>
            <button
              className="w-full text-center text-term text-text-dim hover:text-primary"
              onClick={() => {
                setMode("recovery");
                setError(null);
              }}
            >
              {t("unlock.forgotPassword")}
            </button>
          </div>
        ) : (
          <div className="space-y-4">
            <div className="flex items-start gap-2 border-2 border-border bg-bg p-3">
              <LifeBuoy className="text-accent shrink-0" size={18} />
              <p className="text-term text-text-dim">
                {t("unlock.recoveryIntro")}
              </p>
            </div>
            <Field label={t("unlock.recoveryPhrase")}>
              <textarea
                className="field resize-none h-20"
                value={phrase}
                onChange={(e) => setPhrase(e.target.value)}
                placeholder="palabra1 palabra2 ..."
              />
            </Field>
            <Field label={t("unlock.newPassword")}>
              <Input
                type="password"
                value={newPw}
                onChange={(e) => setNewPw(e.target.value)}
              />
              <StrengthMeter password={newPw} />
            </Field>
            <Field label={t("unlock.confirmNewPassword")}>
              <Input
                type="password"
                value={newPw2}
                onChange={(e) => setNewPw2(e.target.value)}
              />
            </Field>
            {error && <p className="text-danger text-term">{error}</p>}
            <div className="flex gap-2">
              <Button
                className="flex-1"
                onClick={() => {
                  setMode("password");
                  setError(null);
                }}
              >
                {t("unlock.back")}
              </Button>
              <Button
                variant="primary"
                className="flex-1"
                disabled={busy}
                onClick={handleRecover}
              >
                {busy ? t("unlock.recovering") : t("unlock.recover")}
              </Button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

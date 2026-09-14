import { useState } from "react";
import { AlertTriangle, Check, Copy, Globe, Heart, ShieldCheck, type LucideProps } from "lucide-react";
import { useVault } from "@/store/useVault";
import { useI18n } from "@/lib/i18n";
import { useSettings } from "@/store/useSettings";
import { Button } from "@/components/ui/Button";
import { Input, Field } from "@/components/ui/Input";
import { StrengthMeter } from "@/components/StrengthMeter";
import { copyWithAutoClear } from "@/lib/clipboard";
import { isTauri } from "@/lib/utils";
import { openUrl } from "@tauri-apps/plugin-opener";

function GithubIcon({ size = 24, ...props }: LucideProps) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={2}
      strokeLinecap="round"
      strokeLinejoin="round"
      {...props}
    >
      <path d="M15 22v-4a4.8 4.8 0 0 0-1-3.5c3 0 6-2 6-5.5.08-1.25-.27-2.48-1-3.5.28-1.15.28-2.35 0-3.5 0 0-1 0-3 1.5-2.64-.5-5.36-.5-8 0C6 2 5 2 5 2c-.3 1.15-.3 2.35 0 3.5A5.4 5.4 0 0 0 4 9c0 3.5 3 5.5 6 5.5-.39.49-.66 1.05-.79 1.65-.13.6-.13 1.23 0 1.85v4" />
      <path d="M9 18c-4.51 2-5-2-7-2" />
    </svg>
  );
}

const LINKS = [
  { url: "https://paypal.me/octacodec", icon: Heart, label: "PayPal" },
  { url: "https://github.com/OscarTired", icon: GithubIcon, label: "GitHub" },
  { url: "https://octa-dev.com/", icon: Globe, label: "Web" },
] as const;

async function openExternal(url: string) {
  if (isTauri()) {
    try { await openUrl(url); } catch { window.open(url, "_blank"); }
  } else {
    window.open(url, "_blank");
  }
}

type Step = "set-password" | "show-phrase" | "confirm";

export function Onboarding() {
  const createVault = useVault((s) => s.createVault);
  const finishBoot = useVault((s) => s.finishBoot);
  const t = useI18n((s) => s.t);
  const lang = useI18n((s) => s.lang);
  const setLang = useI18n((s) => s.setLang);
  const update = useSettings((s) => s.update);

  const [step, setStep] = useState<Step>("set-password");
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const [phrase, setPhrase] = useState("");
  const [acknowledged, setAcknowledged] = useState(false);
  const [copied, setCopied] = useState(false);

  const words = phrase ? phrase.split(/\s+/) : [];

  async function handleCreate() {
    setError(null);
    if (password.length < 8) {
      setError(t("onboarding.errShortPassword"));
      return;
    }
    if (password !== confirm) {
      setError(t("onboarding.errPasswordMismatch"));
      return;
    }
    setBusy(true);
    try {
      const recovery = await createVault(password);
      setPhrase(recovery);
      setStep("show-phrase");
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }

  async function copyPhrase() {
    if (!await copyWithAutoClear(phrase, 30)) return;
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  }

  return (
    <div className="flex h-full items-center justify-center p-6">
      <div className="panel pixel-border w-full max-w-xl p-6">
        <div className="flex items-center gap-3 mb-5">
          <ShieldCheck className="text-primary" size={28} />
          <div className="flex-1">
            <h1 className="font-pixel text-base text-primary glow-text">SPIRITBYTE</h1>
            <p className="text-term text-text-dim">{t("onboarding.subtitle")}</p>
          </div>
          <div className="flex items-center gap-2">
            <div className="flex border-2 border-border">
              <button
                onClick={() => { setLang("es"); update({ language: "es" }); }}
                className={`px-2 py-1 text-xs font-pixel ${lang === "es" ? "bg-primary text-bg" : "text-text-dim hover:text-primary"}`}
              >
                ES
              </button>
              <button
                onClick={() => { setLang("en"); update({ language: "en" }); }}
                className={`px-2 py-1 text-xs font-pixel ${lang === "en" ? "bg-primary text-bg" : "text-text-dim hover:text-primary"}`}
              >
                EN
              </button>
            </div>
            <div className="flex items-center gap-1">
              {LINKS.map(({ url, icon: Icon, label }) => (
                <button
                  key={label}
                  onClick={() => void openExternal(url)}
                  title={label}
                  className="flex h-7 w-7 items-center justify-center border-2 border-border text-text-dim hover:text-primary hover:border-primary transition-colors"
                >
                  <Icon size={14} />
                </button>
              ))}
            </div>
          </div>
        </div>

        {step === "set-password" && (
          <div className="space-y-4">
            <p className="text-term text-text-dim">
              {t("onboarding.description")}
            </p>
            <Field label={t("onboarding.masterPassword")}>
              <Input
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                autoFocus
                placeholder="********"
              />
              <StrengthMeter password={password} />
            </Field>
            <Field label={t("onboarding.confirmPassword")}>
              <Input
                type="password"
                value={confirm}
                onChange={(e) => setConfirm(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && handleCreate()}
                placeholder="********"
              />
            </Field>
            {error && <p className="text-danger text-term">{error}</p>}
            <Button
              variant="primary"
              className="w-full"
              disabled={busy}
              onClick={handleCreate}
            >
              {busy ? t("onboarding.creating") : t("onboarding.createVault")}
            </Button>
          </div>
        )}

        {step === "show-phrase" && (
          <div className="space-y-4">
            <div className="flex items-start gap-2 border-2 border-warning bg-warning/10 p-3">
              <AlertTriangle className="text-warning shrink-0" size={20} />
              <p className="text-term text-text">
                {t("onboarding.warning")}
              </p>
            </div>

            <div className="grid grid-cols-3 gap-2">
              {words.map((w, i) => (
                <div
                  key={i}
                  className="flex items-center gap-2 border-2 border-border bg-bg px-2 py-1"
                >
                  <span className="font-pixel text-xs text-text-dim">
                    {String(i + 1).padStart(2, "0")}
                  </span>
                  <span className="text-term text-primary glow-text">{w}</span>
                </div>
              ))}
            </div>

            <div className="flex gap-2">
              <Button onClick={copyPhrase} className="flex-1">
                {copied ? <Check size={16} /> : <Copy size={16} />}
                {copied ? t("onboarding.copied") : t("onboarding.copy")}
              </Button>
              <Button variant="primary" className="flex-1" onClick={() => setStep("confirm")}>
                {t("onboarding.noted")}
              </Button>
            </div>
          </div>
        )}

        {step === "confirm" && (
          <div className="space-y-4">
            <p className="text-term text-text">
              {t("onboarding.confirmText")}
            </p>
            <button
              className="flex items-center gap-2 text-left"
              onClick={() => setAcknowledged((v) => !v)}
            >
              <span
                className={`flex h-5 w-5 items-center justify-center border-2 ${
                  acknowledged ? "border-primary bg-primary/20" : "border-border"
                }`}
              >
                {acknowledged && <Check size={14} className="text-primary" />}
              </span>
              <span className="text-term text-text">
                {t("onboarding.understand")}
              </span>
            </button>
            <div className="flex gap-2">
              <Button onClick={() => setStep("show-phrase")} className="flex-1">
                {t("onboarding.viewPhrase")}
              </Button>
              <Button
                variant="primary"
                className="flex-1"
                disabled={!acknowledged}
                onClick={() => void finishBoot()}
              >
                {t("onboarding.enterVault")}
              </Button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

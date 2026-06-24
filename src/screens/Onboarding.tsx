import { useState } from "react";
import { AlertTriangle, Check, Copy, ShieldCheck } from "lucide-react";
import { useVault } from "@/store/useVault";
import { Button } from "@/components/ui/Button";
import { Input, Field } from "@/components/ui/Input";
import { StrengthMeter } from "@/components/StrengthMeter";
import { copyWithAutoClear } from "@/lib/clipboard";

type Step = "set-password" | "show-phrase" | "confirm";

export function Onboarding() {
  const createVault = useVault((s) => s.createVault);
  const finishBoot = useVault((s) => s.finishBoot);

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
      setError("La contraseña maestra debe tener al menos 8 caracteres.");
      return;
    }
    if (password !== confirm) {
      setError("Las contraseñas no coinciden.");
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
    await copyWithAutoClear(phrase, 30);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  }

  return (
    <div className="flex h-full items-center justify-center p-6">
      <div className="panel pixel-border w-full max-w-xl p-6">
        <div className="flex items-center gap-3 mb-5">
          <ShieldCheck className="text-primary" size={28} />
          <div>
            <h1 className="font-pixel text-base text-primary glow-text">SPIRITBYTE</h1>
            <p className="text-term text-text-dim">Inicializar bóveda segura</p>
          </div>
        </div>

        {step === "set-password" && (
          <div className="space-y-4">
            <p className="text-term text-text-dim">
              Crea tu contraseña maestra. Cifra tu bóveda con Argon2id +
              XChaCha20-Poly1305. <span className="text-warning">Nunca se almacena.</span>
            </p>
            <Field label="Contraseña maestra">
              <Input
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                autoFocus
                placeholder="********"
              />
              <StrengthMeter password={password} />
            </Field>
            <Field label="Confirmar contraseña">
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
              {busy ? "Generando..." : "Crear bóveda"}
            </Button>
          </div>
        )}

        {step === "show-phrase" && (
          <div className="space-y-4">
            <div className="flex items-start gap-2 border-2 border-warning bg-warning/10 p-3">
              <AlertTriangle className="text-warning shrink-0" size={20} />
              <p className="text-term text-text">
                Estas <span className="text-warning">12 palabras</span> son tu única
                forma de recuperar la bóveda si olvidas la contraseña. Anótalas y
                guárdalas offline. <span className="text-danger">No se mostrarán de nuevo.</span>
              </p>
            </div>

            <div className="grid grid-cols-3 gap-2">
              {words.map((w, i) => (
                <div
                  key={i}
                  className="flex items-center gap-2 border-2 border-border bg-bg px-2 py-1"
                >
                  <span className="font-pixel text-[9px] text-text-dim">
                    {String(i + 1).padStart(2, "0")}
                  </span>
                  <span className="text-term text-primary glow-text">{w}</span>
                </div>
              ))}
            </div>

            <div className="flex gap-2">
              <Button onClick={copyPhrase} className="flex-1">
                {copied ? <Check size={16} /> : <Copy size={16} />}
                {copied ? "Copiado" : "Copiar"}
              </Button>
              <Button variant="primary" className="flex-1" onClick={() => setStep("confirm")}>
                Ya las anoté
              </Button>
            </div>
          </div>
        )}

        {step === "confirm" && (
          <div className="space-y-4">
            <p className="text-term text-text">
              Confirma que has guardado tu frase de recuperación en un lugar seguro.
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
                Entiendo que perder ambas (contraseña y frase) hace la bóveda
                irrecuperable.
              </span>
            </button>
            <div className="flex gap-2">
              <Button onClick={() => setStep("show-phrase")} className="flex-1">
                Ver frase
              </Button>
              <Button
                variant="primary"
                className="flex-1"
                disabled={!acknowledged}
                onClick={() => void finishBoot()}
              >
                Entrar a la bóveda
              </Button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

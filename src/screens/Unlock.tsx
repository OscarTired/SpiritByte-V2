import { useState } from "react";
import { Eye, EyeOff, KeyRound, LifeBuoy, Lock } from "lucide-react";
import { useVault } from "@/store/useVault";
import { Button } from "@/components/ui/Button";
import { Input, Field } from "@/components/ui/Input";
import { StrengthMeter } from "@/components/StrengthMeter";

type Mode = "password" | "recovery";

export function Unlock() {
  const unlock = useVault((s) => s.unlock);
  const recoverAndReset = useVault((s) => s.recoverAndReset);

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
      setError("Contraseña incorrecta.");
    } finally {
      setBusy(false);
    }
  }

  async function handleRecover() {
    setError(null);
    if (phrase.trim().split(/\s+/).length !== 12) {
      setError("La frase de recuperación debe tener 12 palabras.");
      return;
    }
    if (newPw.length < 8) {
      setError("La nueva contraseña debe tener al menos 8 caracteres.");
      return;
    }
    if (newPw !== newPw2) {
      setError("Las contraseñas no coinciden.");
      return;
    }
    setBusy(true);
    try {
      await recoverAndReset(phrase, newPw);
    } catch {
      setError("Frase de recuperación inválida.");
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
              {mode === "password" ? "Bóveda bloqueada" : "Recuperar acceso"}
            </p>
          </div>
        </div>

        {mode === "password" ? (
          <div className="space-y-4">
            <Field label="Contraseña maestra">
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
              {busy ? "Descifrando..." : "Desbloquear"}
            </Button>
            <button
              className="w-full text-center text-term text-text-dim hover:text-primary"
              onClick={() => {
                setMode("recovery");
                setError(null);
              }}
            >
              ¿Olvidaste tu contraseña? Usar frase de recuperación
            </button>
          </div>
        ) : (
          <div className="space-y-4">
            <div className="flex items-start gap-2 border-2 border-border bg-bg p-3">
              <LifeBuoy className="text-accent shrink-0" size={18} />
              <p className="text-term text-text-dim">
                Introduce tus 12 palabras y define una nueva contraseña maestra.
              </p>
            </div>
            <Field label="Frase de recuperación (12 palabras)">
              <textarea
                className="field resize-none h-20"
                value={phrase}
                onChange={(e) => setPhrase(e.target.value)}
                placeholder="palabra1 palabra2 ..."
              />
            </Field>
            <Field label="Nueva contraseña maestra">
              <Input
                type="password"
                value={newPw}
                onChange={(e) => setNewPw(e.target.value)}
              />
              <StrengthMeter password={newPw} />
            </Field>
            <Field label="Confirmar nueva contraseña">
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
                Volver
              </Button>
              <Button
                variant="primary"
                className="flex-1"
                disabled={busy}
                onClick={handleRecover}
              >
                {busy ? "Recuperando..." : "Recuperar"}
              </Button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

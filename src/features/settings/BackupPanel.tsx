import { useRef, useState } from "react";
import { Download, Upload } from "lucide-react";
import { api } from "@/lib/api";
import { useI18n } from "@/lib/i18n";
import { useVault } from "@/store/useVault";
import { StrengthMeter } from "@/components/StrengthMeter";
import { Button } from "@/components/ui/Button";
import { Field, Input } from "@/components/ui/Input";
import type { ExportSelection } from "@/lib/types";
import { BackupSelection } from "./BackupSelection";

export function BackupPanel() {
  const lang = useI18n((s) => s.lang);
  const es = lang === "es";
  const entries = useVault((s) => s.entries);
  const folders = useVault((s) => s.folders);
  const [selection, setSelection] = useState<ExportSelection | null>(null);
  const hasSelection = selection === null || selection.entryIds.length > 0 || selection.folderIds.length > 0;
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const [file, setFile] = useState<File | null>(null);
  const passwordLength = [...password].length;
  const lengthAccepted = passwordLength >= 12;
  const confirmationAccepted = confirm.length > 0 && password === confirm;
  const [busy, setBusy] = useState(false);
  const pending = useRef(false);
  const [message, setMessage] = useState("");
  const [error, setError] = useState(false);
  const fileInput = useRef<HTMLInputElement>(null);

  async function run(importing: boolean) {
    if (pending.current) return;
    pending.current = true;
    setBusy(true); setMessage(""); setError(false);
    try {
      if (importing) {
        if (!file || file.size > 32 * 1024 * 1024) throw new Error("size");
        await api.importBackup(await file.text(), password);
        // Auto-lock can occur while Argon2 runs. Never refill a locked UI.
        if (useVault.getState().screen === "unlocked") await useVault.getState().refresh();
        setFile(null);
        if (fileInput.current) fileInput.current.value = "";
        setMessage(es ? "Importación completada. Se añadieron las entradas y carpetas." : "Import complete. Entries and folders were added.");
      } else {
        const saved = await api.exportBackup(password, selection);
        setMessage(saved
          ? (es ? "Respaldo cifrado guardado correctamente." : "Encrypted backup saved successfully.")
          : (es ? "Exportación cancelada." : "Export cancelled."));
      }
      setPassword(""); setConfirm("");
    } catch {
      setError(true);
      setMessage(es
        ? "No se completó la operación. Comprueba la contraseña, el archivo (máximo 32 MiB), elige un nombre nuevo al exportar, el espacio disponible y que el vault esté desbloqueado."
        : "Operation failed. Check the password, file (32 MiB maximum), a new export filename, available disk space and that the vault is unlocked.");
    } finally { pending.current = false; setBusy(false); }
  }

  return <section className="space-y-3 border-b-2 border-border pb-5">
    <h3 className="label">{es ? "Importar / Exportar vault" : "Import / Export vault"}</h3>
    <p className="text-text-dim text-sm">{es
      ? "Respalda toda la bóveda o una selección con Argon2id y XChaCha20-Poly1305. Usa una contraseña larga y exclusiva; sin ella no podrás recuperar el respaldo. No incluye ajustes visuales ni la clave o frase de recuperación del vault."
      : "Back up the entire vault or a selection with Argon2id and XChaCha20-Poly1305. Use a long, unique password; the backup cannot be recovered without it. Visual settings and vault credentials/recovery phrase are excluded."}</p>
    <BackupSelection entries={entries} folders={folders} selection={selection} change={setSelection} busy={busy} es={es} />
    <Field label={es ? "Contraseña del respaldo" : "Backup password"}>
      <Input type="password" autoComplete="new-password" value={password} disabled={busy}
        aria-describedby="backup-password-requirements"
        onChange={(e) => setPassword(e.target.value)} />
    </Field>
    <StrengthMeter password={password} />
    <p id="backup-password-requirements" role="status" className={`text-sm ${lengthAccepted ? "text-success" : "text-text-dim"}`}>
      {es
        ? `${passwordLength}/12 caracteres mínimos. ${lengthAccepted ? "Longitud aceptada." : "Necesitas al menos 12 caracteres."} La fuerza es orientativa.`
        : `${passwordLength}/12 minimum characters. ${lengthAccepted ? "Length accepted." : "At least 12 characters required."} Strength is advisory.`}
    </p>
    <Field label={es ? "Confirmar contraseña para exportar (mínimo 12 caracteres)" : "Confirm export password (at least 12 characters)"}>
      <Input type="password" autoComplete="new-password" value={confirm} disabled={busy}
        onChange={(e) => setConfirm(e.target.value)} />
    </Field>
    <p role="status" className={`text-sm ${confirmationAccepted ? "text-success" : "text-text-dim"}`}>
      {es
        ? (confirmationAccepted ? "Las contraseñas coinciden." : confirm ? "Las contraseñas no coinciden." : "Confirma la contraseña para habilitar la exportación.")
        : (confirmationAccepted ? "Passwords match." : confirm ? "Passwords do not match." : "Confirm the password to enable export.")}
    </p>
    <Button disabled={busy || !lengthAccepted || !confirmationAccepted || !hasSelection} onClick={() => void run(false)}>
      <Download size={16} />{selection === null ? (es ? "Exportar todo cifrado" : "Export all encrypted") : (es ? "Exportar selección cifrada" : "Export selection encrypted")}
    </Button>
    <p className="text-text-dim text-sm">{es
      ? "Para restaurar en otro equipo, crea o desbloquea un vault e importa aquí. Se añadirán los datos sin reemplazar los actuales; importar de nuevo creará duplicados. Usa la contraseña que elegiste al exportar."
      : "To restore on another device, create or unlock a vault and import here. Data is added without replacing existing records; repeated imports create duplicates. Use the password chosen when exporting."}</p>
    <input ref={fileInput} type="file" accept=".spiritbyte" disabled={busy}
      aria-label={es ? "Archivo de respaldo cifrado" : "Encrypted backup file"}
      className="block w-full text-sm" onChange={(e) => { setFile(e.target.files?.[0] ?? null); setMessage(""); }} />
    <Button disabled={busy || !file || file.size > 32 * 1024 * 1024 || !password}
      onClick={() => void run(true)}><Upload size={16} />{es ? "Importar y añadir" : "Import and add"}</Button>
    {busy && <p role="status">{es ? "Procesando…" : "Processing…"}</p>}
    {message && <p role={error ? "alert" : "status"} className={error ? "text-danger text-sm" : "text-success text-sm"}>{message}</p>}
  </section>;
}

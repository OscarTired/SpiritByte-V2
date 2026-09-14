import { useEffect, useState } from "react";
import { Dices, Eye, EyeOff } from "lucide-react";
import type { Entry, Folder } from "@/lib/types";
import { useI18n } from "@/lib/i18n";
import { Modal } from "@/components/ui/Modal";
import { Button } from "@/components/ui/Button";
import { Input, Textarea, Field } from "@/components/ui/Input";
import { StrengthMeter } from "@/components/StrengthMeter";
import { GeneratorPanel } from "@/features/generator/GeneratorPanel";
import { uid, now } from "@/lib/utils";

interface Props {
  open: boolean;
  entry: Entry | null;
  folders: Folder[];
  defaultFolderId: string | null;
  onClose: () => void;
  onSave: (entry: Entry) => void;
}

function blankEntry(folderId: string | null): Entry {
  return {
    id: uid(),
    title: "",
    username: "",
    password: "",
    url: "",
    notes: "",
    folderId,
    iconId: null,
    favorite: false,
    createdAt: now(),
    updatedAt: now(),
  };
}

export function EntryEditor({
  open,
  entry,
  folders,
  defaultFolderId,
  onClose,
  onSave,
}: Props) {
  const [draft, setDraft] = useState<Entry>(() => blankEntry(defaultFolderId));
  const [show, setShow] = useState(false);
  const [genOpen, setGenOpen] = useState(false);
  const t = useI18n((s) => s.t);

  useEffect(() => {
    if (open) {
      setDraft(entry ? { ...entry } : blankEntry(defaultFolderId));
      setShow(false);
      setGenOpen(false);
    }
  }, [open, entry, defaultFolderId]);

  function field<K extends keyof Entry>(key: K, value: Entry[K]) {
    setDraft((d) => ({ ...d, [key]: value }));
  }

  function save() {
    if (!draft.title.trim()) return;
    onSave({ ...draft, updatedAt: now() });
  }

  return (
    <Modal
      open={open}
      onClose={onClose}
      title={entry ? t("editor.editEntry") : t("editor.newEntry")}
      footer={
        <>
          <Button onClick={onClose}>{t("vault.cancel")}</Button>
          <Button variant="primary" onClick={save} disabled={!draft.title.trim()}>
            {t("vault.save")}
          </Button>
        </>
      }
    >
      <div className="space-y-3">
        <Field label={t("editor.title")}>
          <Input
            value={draft.title}
            autoFocus
            onChange={(e) => field("title", e.target.value)}
            placeholder="GitHub, Gmail, ..."
          />
        </Field>
        <Field label={t("editor.usernameEmail")}>
          <Input
            value={draft.username}
            onChange={(e) => field("username", e.target.value)}
          />
        </Field>
        <Field label={t("editor.password")}>
          <div className="relative">
            <Input
              type={show ? "text" : "password"}
              value={draft.password}
              onChange={(e) => field("password", e.target.value)}
              className="pr-20"
            />
            <div className="absolute right-2 top-1/2 -translate-y-1/2 flex gap-1">
              <button
                type="button"
                className="text-text-dim hover:text-primary"
                onClick={() => setShow((v) => !v)}
              >
                {show ? <EyeOff size={18} /> : <Eye size={18} />}
              </button>
              <button
                type="button"
                className="text-text-dim hover:text-primary"
                onClick={() => setGenOpen((v) => !v)}
                title={t("editor.generate")}
              >
                <Dices size={18} />
              </button>
            </div>
          </div>
          <StrengthMeter password={draft.password} />
        </Field>

        {genOpen && (
          <div className="panel p-3">
            <GeneratorPanel
              onUse={(pw) => {
                field("password", pw);
                setGenOpen(false);
              }}
            />
          </div>
        )}

        <Field label={t("editor.url")}>
          <Input
            value={draft.url}
            onChange={(e) => field("url", e.target.value)}
            placeholder="https://"
          />
        </Field>
        <Field label={t("editor.folder")}>
          <select
            className="field"
            value={draft.folderId ?? ""}
            onChange={(e) => field("folderId", e.target.value || null)}
          >
            <option value="">{t("editor.noFolder")}</option>
            {folders.map((f) => (
              <option key={f.id} value={f.id}>
                {f.name}
              </option>
            ))}
          </select>
        </Field>
        <Field label={t("editor.notes")}>
          <Textarea
            value={draft.notes}
            onChange={(e) => field("notes", e.target.value)}
            className="h-20"
          />
        </Field>
      </div>
    </Modal>
  );
}

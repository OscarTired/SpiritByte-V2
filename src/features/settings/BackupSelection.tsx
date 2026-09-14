import { useEffect, useMemo, useRef } from "react";
import { Folder as FolderIcon } from "lucide-react";
import { Button } from "@/components/ui/Button";
import type { Entry, Folder, ExportSelection } from "@/lib/types";
import { folderSubtree, orderedFolders, toggleFolderSelection } from "./exportSelection";

function Check({ checked, mixed = false, disabled, onChange, label }: {
  checked: boolean; mixed?: boolean; disabled: boolean; onChange: (checked: boolean) => void; label: string;
}) {
  const ref = useRef<HTMLInputElement>(null);
  useEffect(() => { if (ref.current) ref.current.indeterminate = mixed; }, [mixed]);
  return <input ref={ref} type="checkbox" checked={checked} disabled={disabled}
    aria-label={label} aria-checked={mixed ? "mixed" : checked}
    onChange={(event) => onChange(event.target.checked)} className="accent-primary shrink-0" />;
}

export function BackupSelection({ entries, folders, selection, change, busy, es }: {
  entries: Entry[]; folders: Folder[]; selection: ExportSelection | null;
  change: (value: ExportSelection | null) => void; busy: boolean; es: boolean;
}) {
  const ordered = useMemo(() => orderedFolders(folders), [folders]);
  const selected = new Set(selection?.entryIds ?? []);
  const selectedFolders = new Set(selection?.folderIds ?? []);
  function toggleEntry(id: string, checked: boolean) {
    if (!selection) return;
    const ids = new Set(selection.entryIds);
    checked ? ids.add(id) : ids.delete(id);
    change({ ...selection, entryIds: [...ids] });
  }
  function entryRow(entry: Entry) {
    return <label key={entry.id} className="flex gap-2 items-center py-1 pl-6 text-sm min-w-0">
      <Check checked={selected.has(entry.id)} disabled={busy} label={entry.title}
        onChange={(checked) => toggleEntry(entry.id, checked)} />
      <span className="truncate">{entry.title}<span className="text-text-dim">{entry.username ? ` · ${entry.username}` : ""}</span></span>
    </label>;
  }
  return <fieldset disabled={busy} className="border border-border p-3 space-y-3">
    <legend className="label px-1">{es ? "Contenido a exportar" : "Export contents"}</legend>
    <div className="flex flex-wrap gap-2">
      <Button type="button" aria-pressed={selection === null} variant={selection === null ? "primary" : "default"}
        onClick={() => change(null)}>{es ? "All · Todo" : "All"}</Button>
      <Button type="button" aria-pressed={selection !== null} variant={selection !== null ? "primary" : "default"}
        onClick={() => change(selection ?? { entryIds: [], folderIds: [] })}>{es ? "Seleccionar" : "Select"}</Button>
      {selection && <Button type="button" onClick={() => change({ entryIds: [], folderIds: [] })}>{es ? "Limpiar" : "Clear"}</Button>}
    </div>
    {selection === null ? <p className="text-sm text-text-dim">{es
      ? `Se exportará toda la bóveda: ${entries.length} credenciales y ${folders.length} carpetas.`
      : `Export the entire vault: ${entries.length} entries and ${folders.length} folders.`}</p>
      : <>
        <p className="text-sm text-text-dim">{es
          ? "Marca una carpeta para incluir su contenido y subcarpetas; desmarca cualquier credencial que quieras excluir. Las carpetas padre necesarias se conservan automáticamente."
          : "Check a folder to include its contents and subfolders; uncheck any entries to exclude. Required parent folders are preserved automatically."}</p>
        <div className="max-h-72 overflow-auto border border-border p-2 space-y-1">
          {ordered.map(({ folder, depth }) => {
            const subtree = folderSubtree(folders, folder.id);
            const nested = entries.filter((entry) => entry.folderId && subtree.has(entry.folderId));
            const checked = [...subtree].every((id) => selectedFolders.has(id)) && nested.every((entry) => selected.has(entry.id));
            const mixed = !checked && ([...subtree].some((id) => selectedFolders.has(id)) || nested.some((entry) => selected.has(entry.id)));
            return <div key={folder.id} style={{ marginLeft: `${Math.min(depth, 6) * 12}px` }}>
              <label className="flex items-center gap-2 py-1 text-sm">
                <Check checked={checked} mixed={mixed} disabled={busy} label={folder.name}
                  onChange={(value) => change(toggleFolderSelection(selection, entries, folders, folder.id, value))} />
                <FolderIcon size={15} style={{ color: folder.color ?? undefined }} />
                <span>{folder.name}</span><span className="text-text-dim">({nested.filter((e) => selected.has(e.id)).length}/{nested.length})</span>
              </label>
              {entries.filter((entry) => entry.folderId === folder.id).map(entryRow)}
            </div>;
          })}
          {entries.some((entry) => !entry.folderId) && <div><p className="text-sm text-text-dim py-1">{es ? "Sin carpeta" : "No folder"}</p>
            {entries.filter((entry) => !entry.folderId).map(entryRow)}</div>}
          {!entries.length && !folders.length && <p className="text-sm">{es ? "La bóveda está vacía." : "The vault is empty."}</p>}
        </div>
        <p role="status" className="text-sm">{es ? `${selected.size} credenciales · ${selectedFolders.size} carpetas seleccionadas` : `${selected.size} entries · ${selectedFolders.size} selected folders`}</p>
      </>}
  </fieldset>;
}

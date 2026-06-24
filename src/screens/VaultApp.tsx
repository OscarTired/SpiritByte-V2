import { useCallback, useMemo, useState } from "react";
import {
  Check,
  Copy,
  Dices,
  Folder as FolderIcon,
  FolderPlus,
  Globe,
  Gamepad2,
  KeySquare,
  Lock,
  Mail,
  Pencil,
  Plus,
  Search,
  Settings,
  Shield,
  Star,
  Trash2,
  User,
  Wallet,
  Briefcase,
  Cloud,
  Heart,
  Home,
  Music,
  Camera,
  Code,
  Book,
  ShoppingCart,
  Server,
  Smartphone,
  CreditCard,
  Plane,
  Gift,
  type LucideIcon,
} from "lucide-react";
import { useVault } from "@/store/useVault";
import type { Entry, Folder } from "@/lib/types";
import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";
import { Modal } from "@/components/ui/Modal";
import { EntryEditor } from "@/features/entries/EntryEditor";
import { GeneratorPanel } from "@/features/generator/GeneratorPanel";
import { SettingsPanel } from "@/features/settings/SettingsPanel";
import { copyWithAutoClear } from "@/lib/clipboard";
import { useSettings } from "@/store/useSettings";
import { uid, isTauri } from "@/lib/utils";
import { ResizeHandle } from "@/components/ResizeHandle";
import { openUrl } from "@tauri-apps/plugin-opener";

const ALL = "__all__";
const FAV = "__fav__";

const FOLDER_ICONS: { id: string; icon: LucideIcon }[] = [
  { id: "folder", icon: FolderIcon },
  { id: "globe", icon: Globe },
  { id: "mail", icon: Mail },
  { id: "shield", icon: Shield },
  { id: "user", icon: User },
  { id: "wallet", icon: Wallet },
  { id: "briefcase", icon: Briefcase },
  { id: "cloud", icon: Cloud },
  { id: "gamepad", icon: Gamepad2 },
  { id: "key", icon: KeySquare },
  { id: "heart", icon: Heart },
  { id: "home", icon: Home },
  { id: "music", icon: Music },
  { id: "camera", icon: Camera },
  { id: "code", icon: Code },
  { id: "book", icon: Book },
  { id: "cart", icon: ShoppingCart },
  { id: "server", icon: Server },
  { id: "phone", icon: Smartphone },
  { id: "card", icon: CreditCard },
  { id: "plane", icon: Plane },
  { id: "gift", icon: Gift },
];

const FOLDER_COLORS = [
  "#ff6b00", "#57ff78", "#ff40a0", "#78c8ff",
  "#ffd500", "#ff5a5a", "#78ffd4", "#b478ff",
];

function folderIconComponent(iconId?: string | null): LucideIcon {
  return FOLDER_ICONS.find((f) => f.id === iconId)?.icon ?? FolderIcon;
}

export function VaultApp() {
  const {
    entries,
    folders,
    query,
    setQuery,
    selectedFolderId,
    selectFolder,
    selectedEntryId,
    selectEntry,
    saveEntry,
    deleteEntry,
    saveFolder,
    deleteFolder,
    lock,
  } = useVault();
  const clipboardTtl = useSettings((s) => s.settings.clipboardClearSeconds);

  const [editorOpen, setEditorOpen] = useState(false);
  const [editing, setEditing] = useState<Entry | null>(null);
  const [genOpen, setGenOpen] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [folderModal, setFolderModal] = useState(false);
  const [editingFolder, setEditingFolder] = useState<Folder | null>(null);
  const [folderName, setFolderName] = useState("");
  const [folderIcon, setFolderIcon] = useState("folder");
  const [folderColor, setFolderColor] = useState<string | null>(null);
  const [copiedField, setCopiedField] = useState<string | null>(null);
  const [sidebarWidth, setSidebarWidth] = useState(224);
  const [listWidth, setListWidth] = useState(288);

  const activeFolder = selectedFolderId ?? ALL;

  const openExternalUrl = useCallback(async (url: string) => {
    if (!url) return;
    const fullUrl = url.match(/^https?:\/\//i) ? url : `https://${url}`;
    if (isTauri()) {
      try {
        await openUrl(fullUrl);
      } catch {
        window.open(fullUrl, "_blank");
      }
    } else {
      window.open(fullUrl, "_blank");
    }
  }, []);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    return entries
      .filter((e) => {
        if (activeFolder === ALL) return true;
        if (activeFolder === FAV) return e.favorite;
        return e.folderId === activeFolder;
      })
      .filter(
        (e) =>
          !q ||
          e.title.toLowerCase().includes(q) ||
          e.username.toLowerCase().includes(q) ||
          e.url.toLowerCase().includes(q),
      )
      .sort((a, b) => a.title.localeCompare(b.title));
  }, [entries, query, activeFolder]);

  const selected = entries.find((e) => e.id === selectedEntryId) ?? null;

  async function copyField(field: string, value: string) {
    if (!value) return;
    await copyWithAutoClear(value, clipboardTtl);
    setCopiedField(field);
    setTimeout(() => setCopiedField(null), 1500);
  }

  function newEntry() {
    setEditing(null);
    setEditorOpen(true);
  }
  function editEntry(e: Entry) {
    setEditing(e);
    setEditorOpen(true);
  }
  async function onSaveEntry(e: Entry) {
    await saveEntry(e);
    setEditorOpen(false);
  }
  function openNewFolder() {
    setEditingFolder(null);
    setFolderName("");
    setFolderIcon("folder");
    setFolderColor(null);
    setFolderModal(true);
  }

  function openEditFolder(f: Folder) {
    setEditingFolder(f);
    setFolderName(f.name);
    setFolderIcon(f.icon ?? "folder");
    setFolderColor(f.color ?? null);
    setFolderModal(true);
  }

  async function saveFolderModal() {
    if (!folderName.trim()) return;
    if (editingFolder) {
      await saveFolder({
        ...editingFolder,
        name: folderName.trim(),
        icon: folderIcon,
        color: folderColor,
      });
    } else {
      await saveFolder({
        id: uid(),
        name: folderName.trim(),
        icon: folderIcon,
        color: folderColor,
        parentId: null,
      });
    }
    setFolderName("");
    setFolderIcon("folder");
    setFolderColor(null);
    setEditingFolder(null);
    setFolderModal(false);
  }

  return (
    <div className="flex h-full">
      {/* Sidebar */}
      <aside
        className="shrink-0 panel border-r-2 border-t-0 border-l-0 border-b-0 flex flex-col"
        style={{ width: sidebarWidth }}
      >
        <div className="px-3 py-3 border-b-2 border-border flex items-center gap-2">
          <KeySquare className="text-primary" size={18} />
          <span className="font-pixel text-[11px] text-primary glow-text">SPIRITBYTE</span>
        </div>

        <nav className="flex-1 overflow-y-auto p-2 space-y-1">
          <FolderItem
            active={activeFolder === ALL}
            icon={<KeySquare size={15} />}
            label="Todas"
            count={entries.length}
            onClick={() => selectFolder(null)}
          />
          <FolderItem
            active={activeFolder === FAV}
            icon={<Star size={15} />}
            label="Favoritas"
            count={entries.filter((e) => e.favorite).length}
            onClick={() => selectFolder(FAV)}
          />
          <div className="flex items-center justify-between px-2 pt-3 pb-1">
            <span className="label !mb-0">Carpetas</span>
            <button
              className="text-text-dim hover:text-primary"
              onClick={openNewFolder}
            >
              <FolderPlus size={15} />
            </button>
          </div>
          {folders.map((f) => {
            const Icon = folderIconComponent(f.icon);
            return (
              <FolderItem
                key={f.id}
                active={activeFolder === f.id}
                icon={<Icon size={15} style={f.color ? { color: f.color } : undefined} />}
                label={f.name}
                count={entries.filter((e) => e.folderId === f.id).length}
                onClick={() => selectFolder(f.id)}
                onEdit={() => openEditFolder(f)}
                onDelete={() => void deleteFolder(f.id)}
              />
            );
          })}
        </nav>

        <div className="p-2 border-t-2 border-border flex gap-1">
          <Button variant="ghost" className="flex-1 px-2 py-2" onClick={() => setGenOpen(true)}>
            <Dices size={16} />
          </Button>
          <Button
            variant="ghost"
            className="flex-1 px-2 py-2"
            onClick={() => setSettingsOpen(true)}
          >
            <Settings size={16} />
          </Button>
          <Button variant="ghost" className="flex-1 px-2 py-2" onClick={() => void lock()}>
            <Lock size={16} />
          </Button>
        </div>
      </aside>
      <ResizeHandle
        width={sidebarWidth}
        min={160}
        max={400}
        onResize={setSidebarWidth}
      />

      {/* List */}
      <section
        className="shrink-0 panel border-r-2 border-t-0 border-l-0 border-b-0 flex flex-col"
        style={{ width: listWidth }}
      >
        <div className="p-2 border-b-2 border-border flex items-center gap-2">
          <div className="relative flex-1">
            <Search
              size={15}
              className="absolute left-2 top-1/2 -translate-y-1/2 text-text-dim"
            />
            <Input
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="Buscar..."
              className="pl-7 py-1"
            />
          </div>
          <Button variant="primary" className="px-2 py-2" onClick={newEntry}>
            <Plus size={16} />
          </Button>
        </div>
        <div className="flex-1 overflow-y-auto">
          {filtered.length === 0 ? (
            <p className="p-4 text-term text-text-dim text-center">Sin entradas.</p>
          ) : (
            filtered.map((e) => (
              <button
                key={e.id}
                onClick={() => selectEntry(e.id)}
                className={`w-full text-left px-3 py-2 border-b border-border/50 transition-colors ${
                  selectedEntryId === e.id
                    ? "bg-primary/10 border-l-2 border-l-primary"
                    : "hover:bg-surface-2"
                }`}
              >
                <div className="flex items-center gap-2">
                  {e.favorite && <Star size={12} className="text-accent shrink-0" />}
                  <span className="text-term text-text truncate flex-1">
                    {e.title || "(sin título)"}
                  </span>
                </div>
                <span className="text-term text-text-dim text-sm truncate block">
                  {e.username}
                </span>
              </button>
            ))
          )}
        </div>
      </section>
      <ResizeHandle
        width={listWidth}
        min={200}
        max={500}
        onResize={setListWidth}
      />

      {/* Detail */}
      <main
        className="flex-1 overflow-y-auto p-6"
        style={{ minWidth: 0 }}
      >
        {selected ? (
          <EntryDetail
            entry={selected}
            copiedField={copiedField}
            onCopy={copyField}
            onEdit={() => editEntry(selected)}
            onDelete={() => void deleteEntry(selected.id)}
            onToggleFav={() => void saveEntry({ ...selected, favorite: !selected.favorite })}
            onOpenUrl={openExternalUrl}
          />
        ) : (
          <div className="h-full flex flex-col items-center justify-center text-text-dim">
            <KeySquare size={48} className="opacity-40 mb-3" />
            <p className="text-term">Selecciona o crea una entrada.</p>
          </div>
        )}
      </main>

      <EntryEditor
        open={editorOpen}
        entry={editing}
        folders={folders}
        defaultFolderId={
          activeFolder === ALL || activeFolder === FAV ? null : activeFolder
        }
        onClose={() => setEditorOpen(false)}
        onSave={onSaveEntry}
      />

      <Modal open={genOpen} onClose={() => setGenOpen(false)} title="Generador">
        <GeneratorPanel />
      </Modal>

      <SettingsPanel open={settingsOpen} onClose={() => setSettingsOpen(false)} />

      <Modal
        open={folderModal}
        onClose={() => setFolderModal(false)}
        title={editingFolder ? "Editar carpeta" : "Nueva carpeta"}
        footer={
          <>
            <Button onClick={() => setFolderModal(false)}>Cancelar</Button>
            <Button variant="primary" onClick={saveFolderModal}>
              {editingFolder ? "Guardar" : "Crear"}
            </Button>
          </>
        }
      >
        <div className="space-y-4">
          <Input
            value={folderName}
            autoFocus
            placeholder="Nombre de la carpeta"
            onChange={(e) => setFolderName(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && saveFolderModal()}
          />

          <div>
            <span className="label mb-2 block">Icono</span>
            <div className="grid grid-cols-5 gap-2">
              {FOLDER_ICONS.map(({ id, icon: Icon }) => (
                <button
                  key={id}
                  onClick={() => setFolderIcon(id)}
                  className={`flex items-center justify-center h-9 border-2 transition-colors ${
                    folderIcon === id
                      ? "border-primary text-primary"
                      : "border-border text-text-dim hover:text-text"
                  }`}
                >
                  <Icon size={16} />
                </button>
              ))}
            </div>
          </div>

          <div>
            <span className="label mb-2 block">Color</span>
            <div className="flex items-center gap-2 flex-wrap">
              <button
                onClick={() => setFolderColor(null)}
                className={`h-7 w-7 border-2 flex items-center justify-center text-[10px] ${
                  folderColor === null ? "border-primary" : "border-border"
                }`}
                title="Sin color"
              >
                ✕
              </button>
              {FOLDER_COLORS.map((c) => (
                <button
                  key={c}
                  onClick={() => setFolderColor(c)}
                  className={`h-7 w-7 border-2 ${
                    folderColor === c ? "border-primary" : "border-border"
                  }`}
                  style={{ background: c }}
                />
              ))}
            </div>
          </div>
        </div>
      </Modal>
    </div>
  );
}

function FolderItem({
  active,
  icon,
  label,
  count,
  onClick,
  onEdit,
  onDelete,
}: {
  active: boolean;
  icon: React.ReactNode;
  label: string;
  count: number;
  onClick: () => void;
  onEdit?: () => void;
  onDelete?: () => void;
}) {
  return (
    <div
      className={`group flex items-center gap-2 px-2 py-1.5 cursor-pointer transition-colors ${
        active ? "bg-primary/10 text-primary" : "text-text hover:bg-surface-2"
      }`}
      onClick={onClick}
    >
      <span className="shrink-0">{icon}</span>
      <span className="text-term flex-1 truncate">{label}</span>
      <span className="text-term text-text-dim text-sm">{count}</span>
      {onEdit && (
        <button
          className="opacity-0 group-hover:opacity-100 text-text-dim hover:text-primary"
          onClick={(e) => {
            e.stopPropagation();
            onEdit();
          }}
        >
          <Pencil size={13} />
        </button>
      )}
      {onDelete && (
        <button
          className="opacity-0 group-hover:opacity-100 text-text-dim hover:text-danger"
          onClick={(e) => {
            e.stopPropagation();
            onDelete();
          }}
        >
          <Trash2 size={13} />
        </button>
      )}
    </div>
  );
}

function EntryDetail({
  entry,
  copiedField,
  onCopy,
  onEdit,
  onDelete,
  onToggleFav,
  onOpenUrl,
}: {
  entry: Entry;
  copiedField: string | null;
  onCopy: (field: string, value: string) => void;
  onEdit: () => void;
  onDelete: () => void;
  onToggleFav: () => void;
  onOpenUrl: (url: string) => void;
}) {
  const [revealed, setRevealed] = useState(false);

  return (
    <div className="max-w-2xl">
      <div className="flex items-start justify-between mb-6">
        <h1 className="font-pixel text-lg text-primary glow-text break-all">
          {entry.title}
        </h1>
        <div className="flex gap-2 shrink-0">
          <Button variant="ghost" className="px-2 py-2" onClick={onToggleFav}>
            <Star
              size={16}
              className={entry.favorite ? "text-accent fill-[rgb(var(--sb-accent))]" : ""}
            />
          </Button>
          <Button variant="ghost" className="px-2 py-2" onClick={onEdit}>
            <Pencil size={16} />
          </Button>
          <Button variant="ghost" className="px-2 py-2 hover:text-danger" onClick={onDelete}>
            <Trash2 size={16} />
          </Button>
        </div>
      </div>

      <div className="space-y-3">
        <DetailRow
          label="Usuario"
          value={entry.username}
          copied={copiedField === "username"}
          onCopy={() => onCopy("username", entry.username)}
        />
        <div className="panel p-3">
          <div className="flex items-center justify-between">
            <span className="label !mb-0">Contraseña</span>
            <div className="flex gap-2">
              <button
                className="text-text-dim hover:text-primary text-term"
                onClick={() => setRevealed((v) => !v)}
              >
                {revealed ? "Ocultar" : "Mostrar"}
              </button>
              <button
                className="text-text-dim hover:text-primary"
                onClick={() => onCopy("password", entry.password)}
              >
                {copiedField === "password" ? <Check size={15} /> : <Copy size={15} />}
              </button>
            </div>
          </div>
          <div className="text-term text-primary glow-text break-all mt-1">
            {entry.password
              ? revealed
                ? entry.password
                : "•".repeat(Math.min(entry.password.length, 24))
              : "—"}
          </div>
        </div>
        <DetailRow
          label="URL"
          value={entry.url}
          copied={copiedField === "url"}
          onCopy={() => onCopy("url", entry.url)}
          link
          onOpenUrl={onOpenUrl}
        />
        {entry.notes && (
          <div className="panel p-3">
            <span className="label">Notas</span>
            <p className="text-term text-text whitespace-pre-wrap break-words">
              {entry.notes}
            </p>
          </div>
        )}
      </div>
    </div>
  );
}

function DetailRow({
  label,
  value,
  copied,
  onCopy,
  link,
  onOpenUrl,
}: {
  label: string;
  value: string;
  copied: boolean;
  onCopy: () => void;
  link?: boolean;
  onOpenUrl?: (url: string) => void;
}) {
  return (
    <div className="panel p-3 flex items-center justify-between gap-3">
      <div className="min-w-0">
        <span className="label !mb-0">{label}</span>
        <div className="text-term text-text break-all">
          {value ? (
            link ? (
              <a
                href="#"
                onClick={(e) => {
                  e.preventDefault();
                  onOpenUrl?.(value);
                }}
                className="text-accent glow-accent hover:underline cursor-pointer"
              >
                {value}
              </a>
            ) : (
              value
            )
          ) : (
            "—"
          )}
        </div>
      </div>
      {value && (
        <button className="text-text-dim hover:text-primary shrink-0" onClick={onCopy}>
          {copied ? <Check size={15} /> : <Copy size={15} />}
        </button>
      )}
    </div>
  );
}

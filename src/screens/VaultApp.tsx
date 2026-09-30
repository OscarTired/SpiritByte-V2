import { lazy, Suspense, memo, useCallback, useMemo, useState, useEffect, useRef } from "react";
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
  type LucideProps,
} from "lucide-react";
import { useVault } from "@/store/useVault";
import { useI18n } from "@/lib/i18n";
import type { Entry, Folder } from "@/lib/types";
import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";
import { Modal } from "@/components/ui/Modal";
import { copyWithAutoClear } from "@/lib/clipboard";
import { useSettings } from "@/store/useSettings";
import { uid, isTauri } from "@/lib/utils";
import { ResizeHandle } from "@/components/ResizeHandle";
import { openUrl } from "@tauri-apps/plugin-opener";

const EntryEditor = lazy(() => import("@/features/entries/EntryEditor").then((m) => ({ default: m.EntryEditor })));
const GeneratorPanel = lazy(() => import("@/features/generator/GeneratorPanel").then((m) => ({ default: m.GeneratorPanel })));
const SettingsPanel = lazy(() => import("@/features/settings/SettingsPanel").then((m) => ({ default: m.SettingsPanel })));

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

const SOCIAL_LINKS = [
  { url: "https://paypal.me/octacodec", icon: Heart, label: "PayPal" },
  { url: "https://github.com/OscarTired", icon: GithubIcon, label: "GitHub" },
  { url: "https://octa-dev.com/", icon: Globe, label: "Web" },
] as const;

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
  const entries = useVault((s) => s.entries);
  const folders = useVault((s) => s.folders);
  const query = useVault((s) => s.query);
  const setQuery = useVault((s) => s.setQuery);
  const selectedFolderId = useVault((s) => s.selectedFolderId);
  const selectFolder = useVault((s) => s.selectFolder);
  const selectedEntryId = useVault((s) => s.selectedEntryId);
  const selectEntry = useVault((s) => s.selectEntry);
  const saveEntry = useVault((s) => s.saveEntry);
  const deleteEntry = useVault((s) => s.deleteEntry);
  const saveFolder = useVault((s) => s.saveFolder);
  const deleteFolder = useVault((s) => s.deleteFolder);
  const lock = useVault((s) => s.lock);
  const clipboardTtl = useSettings((s) => s.settings.clipboardClearSeconds);
  const t = useI18n((s) => s.t);

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
  const copyTimer = useRef<ReturnType<typeof setTimeout>>();
  useEffect(() => {
    setCopiedField(null);
    return () => clearTimeout(copyTimer.current);
  }, [selectedEntryId]);
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

  const searchable = useMemo(() => [...entries]
    .sort((a, b) => a.title.localeCompare(b.title))
    .map((entry) => ({ entry, text: [entry.title, entry.username, entry.url].map((v) => v.toLowerCase()) })), [entries]);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    return searchable.filter(({ entry: e, text }) =>
      (activeFolder === ALL || (activeFolder === FAV ? e.favorite : e.folderId === activeFolder)) &&
      (!q || text.some((value) => value.includes(q)))
    ).map(({ entry }) => entry);
  }, [searchable, query, activeFolder]);

  // Counts computed in a single pass instead of re-filtering per folder on every render.
  const { favCount, folderCounts } = useMemo(() => {
    const counts = new Map<string, number>();
    let fav = 0;
    for (const e of entries) {
      if (e.favorite) fav++;
      if (e.folderId) counts.set(e.folderId, (counts.get(e.folderId) ?? 0) + 1);
    }
    return { favCount: fav, folderCounts: counts };
  }, [entries]);

  const selected = useMemo(
    () => entries.find((e) => e.id === selectedEntryId) ?? null,
    [entries, selectedEntryId],
  );

  async function copyField(field: string, value: string) {
    if (!value) return;
    if (!await copyWithAutoClear(value, clipboardTtl)) return;
    clearTimeout(copyTimer.current);
    setCopiedField(field);
    copyTimer.current = setTimeout(() => setCopiedField(null), 1500);
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
    <div className="relative flex h-full">
      {/* Sidebar */}
      <aside
        className="shrink-0 panel border-r-2 border-t-0 border-l-0 border-b-0 flex flex-col"
        style={{ width: sidebarWidth }}
      >
        <div className="px-3 py-3 border-b-2 border-border flex items-center gap-2">
          <KeySquare className="text-primary" size={18} />
          <span className="font-pixel text-xs text-primary glow-text">SPIRITBYTE</span>
        </div>

        <nav className="flex-1 overflow-y-auto p-2 space-y-1">
          <FolderItem
            active={activeFolder === ALL}
            icon={<KeySquare size={15} />}
            label={t("vault.all")}
            count={entries.length}
            onClick={() => selectFolder(null)}
          />
          <FolderItem
            active={activeFolder === FAV}
            icon={<Star size={15} />}
            label={t("vault.favorites")}
            count={favCount}
            onClick={() => selectFolder(FAV)}
          />
          <div className="flex items-center justify-between px-2 pt-3 pb-1">
            <span className="label !mb-0">{t("vault.folders")}</span>
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
                count={folderCounts.get(f.id) ?? 0}
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
              placeholder={t("vault.search")}
              className="pl-7 py-1"
            />
          </div>
          <Button variant="primary" className="px-2 py-2" onClick={newEntry}>
            <Plus size={16} />
          </Button>
        </div>
        <div className="flex-1 overflow-y-auto">
          {filtered.length === 0 ? (
            <p className="p-4 text-term text-text-dim text-center">{t("vault.noEntries")}</p>
          ) : (
            filtered.map((e) => (
              <EntryListItem key={e.id} entry={e} selected={selectedEntryId === e.id}
                onSelect={selectEntry} untitled={t("vault.untitled")} />
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
            key={selected.id}
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
            <p className="text-term">{t("vault.selectEntry")}</p>
          </div>
        )}
      </main>

      <div className="absolute bottom-3 right-3 flex gap-1">
        {SOCIAL_LINKS.map(({ url, icon: Icon, label }) => (
          <button
            key={label}
            onClick={() => void openExternalUrl(url)}
            title={label}
            className="flex h-7 w-7 items-center justify-center text-text-dim/60 hover:text-primary transition-colors"
          >
            <Icon size={15} />
          </button>
        ))}
      </div>

      <Suspense fallback={null}>
        {editorOpen && (
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
        )}

        {genOpen && (
          <Modal open onClose={() => setGenOpen(false)} title={t("vault.generator")}>
            <GeneratorPanel />
          </Modal>
        )}

        {settingsOpen && <SettingsPanel open onClose={() => setSettingsOpen(false)} />}
      </Suspense>

      <Modal
        open={folderModal}
        onClose={() => setFolderModal(false)}
        title={editingFolder ? t("vault.editFolder") : t("vault.newFolder")}
        footer={
          <>
            <Button onClick={() => setFolderModal(false)}>{t("vault.cancel")}</Button>
            <Button variant="primary" onClick={saveFolderModal}>
              {editingFolder ? t("vault.save") : t("vault.create")}
            </Button>
          </>
        }
      >
        <div className="space-y-4">
          <Input
            value={folderName}
            autoFocus
            placeholder={t("vault.folderName")}
            onChange={(e) => setFolderName(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && saveFolderModal()}
          />

          <div>
            <span className="label mb-2 block">{t("vault.icon")}</span>
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
            <span className="label mb-2 block">{t("vault.color")}</span>
            <div className="flex items-center gap-2 flex-wrap">
              <button
                onClick={() => setFolderColor(null)}
                className={`h-7 w-7 border-2 flex items-center justify-center text-xs ${
                  folderColor === null ? "border-primary" : "border-border"
                }`}
                title={t("vault.noColor")}
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

const EntryListItem = memo(function EntryListItem({ entry, selected, onSelect, untitled }: {
  entry: Entry; selected: boolean; onSelect: (id: string) => void; untitled: string;
}) {
  return <button onClick={() => onSelect(entry.id)}
    className={`w-full text-left px-3 py-2 border-b border-border/50 transition-colors ${
      selected ? "bg-primary/10 border-l-2 border-l-primary" : "hover:bg-surface-2"
    }`}>
    <div className="flex items-center gap-2">
      {entry.favorite && <Star size={12} className="text-accent shrink-0" />}
      <span className="text-term text-text truncate flex-1">{entry.title || untitled}</span>
    </div>
    <span className="text-sm text-text-dim truncate block">{entry.username}</span>
  </button>;
});

const FolderItem = memo(function FolderItem({
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
      <span className="text-sm text-text-dim">{count}</span>
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
});

const EntryDetail = memo(function EntryDetail({
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
  const t = useI18n((s) => s.t);

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
          label={t("entry.username")}
          value={entry.username}
          copied={copiedField === "username"}
          onCopy={() => onCopy("username", entry.username)}
        />
        <div className="panel p-3">
          <div className="flex items-center justify-between">
            <span className="label !mb-0">{t("entry.password")}</span>
            <div className="flex gap-2">
              <button
                className="text-text-dim hover:text-primary text-term"
                onClick={() => setRevealed((v) => !v)}
              >
                {revealed ? t("entry.hide") : t("entry.show")}
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
          label={t("entry.url")}
          value={entry.url}
          copied={copiedField === "url"}
          onCopy={() => onCopy("url", entry.url)}
          link
          onOpenUrl={onOpenUrl}
        />
        {entry.notes && (
          <div className="panel p-3">
            <div className="flex items-center justify-between gap-3">
              <span className="label">{t("entry.notes")}</span>
              <button type="button" className="text-text-dim hover:text-primary"
                title={t("entry.copyNotes")} aria-label={t("entry.copyNotes")}
                onClick={() => onCopy("notes", entry.notes)}>
                {copiedField === "notes" ? <Check size={15} /> : <Copy size={15} />}
              </button>
            </div>
            <p tabIndex={0} className="select-text text-term text-text whitespace-pre-wrap break-words">
              {entry.notes}
            </p>
          </div>
        )}
      </div>
    </div>
  );
});

const DetailRow = memo(function DetailRow({
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
});

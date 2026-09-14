import { create } from "zustand";
import { api } from "@/lib/api";
import type { Entry, Folder } from "@/lib/types";

export type Screen = "boot" | "onboarding" | "locked" | "unlocked";

interface VaultState {
  screen: Screen;
  booted: boolean;
  entries: Entry[];
  folders: Folder[];
  selectedFolderId: string | null;
  selectedEntryId: string | null;
  query: string;
  error: string | null;

  init: () => Promise<void>;
  finishBoot: () => Promise<void>;
  refresh: () => Promise<void>;

  createVault: (password: string) => Promise<string>;
  unlock: (password: string) => Promise<void>;
  unlockWithRecovery: (phrase: string) => Promise<void>;
  recoverAndReset: (phrase: string, newPassword: string) => Promise<void>;
  lock: () => Promise<void>;

  saveEntry: (entry: Entry) => Promise<void>;
  deleteEntry: (id: string) => Promise<void>;
  saveFolder: (folder: Folder) => Promise<void>;
  deleteFolder: (id: string) => Promise<void>;

  setQuery: (q: string) => void;
  selectFolder: (id: string | null) => void;
  selectEntry: (id: string | null) => void;
  setError: (e: string | null) => void;
}

let sessionEpoch = 0;

export const useVault = create<VaultState>((set, get) => ({
  screen: "boot",
  booted: false,
  entries: [],
  folders: [],
  selectedFolderId: null,
  selectedEntryId: null,
  query: "",
  error: null,

  init: async () => {
    const status = await api.vaultStatus();
    const target: Screen = !status.exists
      ? "onboarding"
      : status.unlocked
        ? "unlocked"
        : "locked";
    if (status.unlocked) await get().refresh();
    // Stay on boot screen until the splash finishes; remember the target.
    set({ screen: get().booted ? target : "boot" });
    (get() as VaultState & { _target?: Screen })._target = target;
  },

  finishBoot: async () => {
    const status = await api.vaultStatus();
    const target: Screen = !status.exists
      ? "onboarding"
      : status.unlocked
        ? "unlocked"
        : "locked";
    set({ booted: true, screen: target });
  },

  refresh: async () => {
    const epoch = sessionEpoch;
    const data = await api.getVault();
    if (epoch === sessionEpoch) set({ entries: data.entries, folders: data.folders });
  },

  createVault: async (password) => {
    const phrase = await api.createVault(password);
    await get().refresh();
    return phrase;
  },

  unlock: async (password) => {
    set({ error: null });
    await api.unlock(password);
    await get().refresh();
    set({ screen: "unlocked" });
  },

  unlockWithRecovery: async (phrase) => {
    set({ error: null });
    await api.unlockWithRecovery(phrase);
    await get().refresh();
    set({ screen: "unlocked" });
  },

  recoverAndReset: async (phrase, newPassword) => {
    set({ error: null });
    await api.recoverAndReset(phrase, newPassword);
    await get().refresh();
    set({ screen: "unlocked" });
  },

  lock: async () => {
    sessionEpoch++;
    await api.lock();
    sessionEpoch++;
    set({
      screen: "locked",
      entries: [],
      folders: [],
      selectedEntryId: null,
      selectedFolderId: null,
      query: "",
    });
  },

  saveEntry: async (entry) => {
    const epoch = sessionEpoch;
    const stored = await api.upsertEntry(entry);
    if (epoch !== sessionEpoch) return;
    set((s) => ({ entries: s.entries.some((e) => e.id === stored.id)
      ? s.entries.map((e) => e.id === stored.id ? stored : e)
      : [...s.entries, stored], selectedEntryId: stored.id }));
  },

  deleteEntry: async (id) => {
    const epoch = sessionEpoch;
    await api.deleteEntry(id);
    if (epoch !== sessionEpoch) return;
    set((s) => ({ entries: s.entries.filter((e) => e.id !== id),
      selectedEntryId: s.selectedEntryId === id ? null : s.selectedEntryId }));
  },

  saveFolder: async (folder) => {
    const epoch = sessionEpoch;
    const stored = await api.upsertFolder(folder);
    if (epoch !== sessionEpoch) return;
    set((s) => ({ folders: s.folders.some((f) => f.id === stored.id)
      ? s.folders.map((f) => f.id === stored.id ? stored : f) : [...s.folders, stored] }));
  },

  deleteFolder: async (id) => {
    await api.deleteFolder(id);
    await get().refresh();
    if (get().selectedFolderId === id) set({ selectedFolderId: null });
  },

  setQuery: (q) => set({ query: q }),
  selectFolder: (id) => set({ selectedFolderId: id, selectedEntryId: null }),
  selectEntry: (id) => set({ selectedEntryId: id }),
  setError: (e) => set({ error: e }),
}));

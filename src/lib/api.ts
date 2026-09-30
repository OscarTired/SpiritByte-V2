import { invoke } from "@tauri-apps/api/core";
import type {
  Entry,
  Folder,
  GenOptions,
  Strength,
  VaultData,
  VaultStatus,
  ExportSelection,
} from "./types";

/**
 * Typed wrappers around the Rust command surface.
 * Tauri converts camelCase JS args to snake_case Rust parameters automatically.
 */
export const api = {
  exportBackup: (password: string, selection: ExportSelection | null = null) =>
    invoke<boolean>("export_backup", { password, selection }),
  importBackup: (contents: string, password: string) =>
    invoke<void>("import_backup", { contents, password }),
  vaultStatus: () => invoke<VaultStatus>("vault_status"),
  createVault: (password: string) => invoke<string>("create_vault", { password }),
  unlock: (password: string) => invoke<void>("unlock", { password }),
  unlockWithRecovery: (phrase: string) =>
    invoke<void>("unlock_with_recovery", { phrase }),
  lock: () => invoke<void>("lock"),
  changeMasterPassword: (newPassword: string) =>
    invoke<void>("change_master_password", { newPassword }),
  recoverAndReset: (phrase: string, newPassword: string) =>
    invoke<void>("recover_and_reset", { phrase, newPassword }),

  listEntries: () => invoke<Entry[]>("list_entries"),
  listFolders: () => invoke<Folder[]>("list_folders"),
  getVault: () => invoke<VaultData>("get_vault"),
  upsertEntry: (entry: Entry) => invoke<Entry>("upsert_entry", { entry }),
  deleteEntry: (id: string) => invoke<void>("delete_entry", { id }),
  upsertFolder: (folder: Folder) => invoke<Folder>("upsert_folder", { folder }),
  deleteFolder: (id: string) => invoke<void>("delete_folder", { id }),

  generatePassword: (options: GenOptions) =>
    invoke<string>("generate_password", { options }),
  passwordStrength: (password: string) =>
    invoke<Strength>("password_strength", { password }),

  getSettings: () => invoke<unknown>("get_settings"),
  saveSettings: (settings: unknown) =>
    invoke<void>("save_settings", { settings }),

  pickWallpaper: () => invoke<string | null>("pick_wallpaper"),
};

export interface Entry {
  id: string;
  title: string;
  username: string;
  password: string;
  url: string;
  notes: string;
  folderId?: string | null;
  iconId?: string | null;
  favorite: boolean;
  createdAt: number;
  updatedAt: number;
}

export interface Folder {
  id: string;
  name: string;
  icon?: string | null;
  color?: string | null;
  parentId?: string | null;
}

export interface VaultData {
  entries: Entry[];
  folders: Folder[];
}

export interface ExportSelection {
  entryIds: string[];
  folderIds: string[];
}

export interface VaultStatus {
  exists: boolean;
  unlocked: boolean;
}

export interface GenOptions {
  length: number;
  lowercase: boolean;
  uppercase: boolean;
  digits: boolean;
  symbols: boolean;
  avoidAmbiguous: boolean;
}

export interface Strength {
  score: number;
  entropyBits: number;
  label: string;
}

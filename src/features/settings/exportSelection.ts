import type { Entry, Folder, ExportSelection } from "@/lib/types";

export function folderSubtree(folders: Folder[], id: string): Set<string> {
  const children = new Map<string, string[]>();
  for (const folder of folders) if (folder.parentId) {
    children.set(folder.parentId, [...(children.get(folder.parentId) ?? []), folder.id]);
  }
  const result = new Set<string>();
  const pending = [id];
  while (pending.length) {
    const next = pending.pop()!;
    if (result.has(next)) continue;
    result.add(next);
    pending.push(...(children.get(next) ?? []));
  }
  return result;
}

export function toggleFolderSelection(selection: ExportSelection, entries: Entry[], folders: Folder[], id: string, checked: boolean): ExportSelection {
  const subtree = folderSubtree(folders, id);
  const entryIds = new Set(selection.entryIds);
  const folderIds = new Set(selection.folderIds);
  for (const folderId of subtree) checked ? folderIds.add(folderId) : folderIds.delete(folderId);
  for (const entry of entries) if (entry.folderId && subtree.has(entry.folderId)) {
    checked ? entryIds.add(entry.id) : entryIds.delete(entry.id);
  }
  return { entryIds: [...entryIds], folderIds: [...folderIds] };
}

export function orderedFolders(folders: Folder[]): { folder: Folder; depth: number }[] {
  const known = new Set(folders.map((f) => f.id));
  const children = new Map<string | null, Folder[]>();
  for (const folder of folders) {
    const parent = folder.parentId && known.has(folder.parentId) ? folder.parentId : null;
    children.set(parent, [...(children.get(parent) ?? []), folder]);
  }
  const result: { folder: Folder; depth: number }[] = [];
  const pending = [...(children.get(null) ?? [])].reverse().map((folder) => ({ folder, depth: 0 }));
  const visited = new Set<string>();
  while (pending.length) {
    const item = pending.pop()!;
    if (visited.has(item.folder.id)) continue;
    visited.add(item.folder.id); result.push(item);
    pending.push(...[...(children.get(item.folder.id) ?? [])].reverse().map((folder) => ({ folder, depth: item.depth + 1 })));
  }
  return result;
}

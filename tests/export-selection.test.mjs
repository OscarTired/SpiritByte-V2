import { test } from 'node:test';
import assert from 'node:assert/strict';
import ts from 'typescript';
import { readFile } from 'node:fs/promises';
const source = await readFile(new URL('../src/features/settings/exportSelection.ts', import.meta.url), 'utf8');
const { outputText } = ts.transpileModule(source, { compilerOptions: { module: ts.ModuleKind.ESNext, target: ts.ScriptTarget.ES2020 } });
const { toggleFolderSelection, orderedFolders } = await import(`data:text/javascript;base64,${Buffer.from(outputText).toString('base64')}`);
const folders = [{ id:'parent', name:'Root' }, { id:'child', name:'Child', parentId:'parent' }, { id:'other', name:'Other' }];
const entries = [{ id:'a', folderId:'parent' }, { id:'b', folderId:'child' }, { id:'c', folderId:'child' }, { id:'d', folderId:'other' }];

test('a folder selects its subtree; excluding one credential remains explicit', () => {
  const selected = toggleFolderSelection({ entryIds:[], folderIds:[] }, entries, folders, 'parent', true);
  assert.deepEqual(new Set(selected.entryIds), new Set(['a','b','c']));
  assert.deepEqual(new Set(selected.folderIds), new Set(['parent','child']));
  selected.entryIds = selected.entryIds.filter((id) => id !== 'c');
  const extra = toggleFolderSelection(selected, entries, folders, 'other', true);
  assert.deepEqual(new Set(extra.entryIds), new Set(['a','b','d']));
  const removeChild = toggleFolderSelection(extra, entries, folders, 'child', false);
  assert.deepEqual(new Set(removeChild.entryIds), new Set(['a','d']));
  assert.deepEqual(new Set(removeChild.folderIds), new Set(['parent','other']));
});

test('nested folders keep their ordering and empty folders can be selected', () => {
  assert.deepEqual(orderedFolders([folders[1],folders[0],folders[2]]).map(({folder,depth}) => [folder.id,depth]), [['parent',0],['child',1],['other',0]]);
  assert.deepEqual(toggleFolderSelection({entryIds:[],folderIds:[]},[],folders,'child',true), { entryIds:[],folderIds:['child'] });
});

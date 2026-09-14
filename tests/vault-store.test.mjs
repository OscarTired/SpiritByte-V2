import { test } from "node:test";
import assert from "node:assert/strict";
import ts from "typescript";
import { readFile } from "node:fs/promises";

const source = await readFile(new URL("../src/store/useVault.ts", import.meta.url), "utf8");
const { outputText } = ts.transpileModule(source, {
  compilerOptions: { module: ts.ModuleKind.ESNext, target: ts.ScriptTarget.ES2020 },
});
const code = outputText.replace('import { api } from "@/lib/api";', 'const api = globalThis.testApi;')
  .replace('from "zustand"', `from "${import.meta.resolve("zustand")}"`);
let instance = 0;
async function store(api) {
  globalThis.testApi = api;
  const mod = await import(`data:text/javascript;base64,${Buffer.from(code).toString("base64")}#${instance++}`);
  return mod.useVault;
}

test("saving an entry updates only that record without requesting the entire vault", async () => {
  const first = { id: "first", title: "Before" };
  const second = { id: "second", title: "Unchanged" };
  const vault = await store({ upsertEntry: async (entry) => entry,
    getVault: () => { throw new Error("unexpected full reload"); } });
  vault.setState({ entries: [first, second], screen: "unlocked" });
  await vault.getState().saveEntry({ ...first, title: "After" });
  assert.equal(vault.getState().entries[0].title, "After");
  assert.equal(vault.getState().entries[1], second);
  assert.equal(vault.getState().selectedEntryId, "first");
});

test("failed save retains the visible data", async () => {
  const entry = { id: "first", title: "Before" };
  const vault = await store({ upsertEntry: async () => { throw new Error("disk full"); } });
  vault.setState({ entries: [entry], screen: "unlocked" });
  await assert.rejects(vault.getState().saveEntry({ ...entry, title: "After" }));
  assert.equal(vault.getState().entries[0], entry);
});

test("a refresh finishing after lock never restores secrets to the UI", async () => {
  let resolve;
  const vault = await store({ getVault: () => new Promise((done) => { resolve = done; }), lock: async () => {} });
  vault.setState({ screen: "unlocked" });
  const refresh = vault.getState().refresh();
  await vault.getState().lock();
  resolve({ entries: [{ id: "secret" }], folders: [] });
  await refresh;
  assert.equal(vault.getState().screen, "locked");
  assert.deepEqual(vault.getState().entries, []);
});

test("a save finishing after lock never restores secrets to the UI", async () => {
  let resolve;
  const vault = await store({ upsertEntry: () => new Promise((done) => { resolve = done; }), lock: async () => {} });
  vault.setState({ screen: "unlocked" });
  const save = vault.getState().saveEntry({ id: "secret" });
  await vault.getState().lock();
  resolve({ id: "secret" });
  await save;
  assert.deepEqual(vault.getState().entries, []);
  assert.equal(vault.getState().selectedEntryId, null);
});

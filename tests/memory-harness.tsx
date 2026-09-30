// Browser-only UI fixture: no native IPC and no access to a real vault.
import React from "react";
import ReactDOM from "react-dom/client";
import "@fontsource/press-start-2p/index.css";
import "@fontsource/vt323/index.css";
import "../src/index.css";
import App from "../src/App";
import { api } from "../src/lib/api";
import type { Entry } from "../src/lib/types";
import { DEFAULT_SETTINGS } from "../src/theme/settings";

let unlocked = false;
let entries: Entry[] = [{ id: "demo", title: "Demo fixture", username: "example", password: "demo-only", url: "example.invalid", notes: "Local UI fixture", folderId: null, iconId: null, favorite: false, createdAt: 1, updatedAt: 1 }];
api.vaultStatus = async () => ({ exists: true, unlocked });
api.unlock = async () => { unlocked = true; };
api.lock = async () => { unlocked = false; };
api.getVault = async () => ({ entries, folders: [] });
api.generatePassword = async () => "UI-demo-password-123!";
api.passwordStrength = async () => ({ score: 4, entropyBits: 90, label: "Strong" });
api.upsertEntry = async (entry) => {
  entries = [entry, ...entries.filter((e) => e.id !== entry.id)];
  return entry;
};
localStorage.setItem("spiritbyte.settings", JSON.stringify({ ...DEFAULT_SETTINGS, scanlines: true, glow: true, showSplash: true, lowPowerMode: false }));
ReactDOM.createRoot(document.getElementById("root")!).render(<React.StrictMode><App /></React.StrictMode>);

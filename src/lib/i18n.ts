import { create } from "zustand";
import type { Language } from "@/theme/settings";

export type TranslationKey =
  | "entry.copyNotes"
  // Splash
  | "splash.tagline"
  // VaultApp
  | "vault.all"
  | "vault.favorites"
  | "vault.folders"
  | "vault.search"
  | "vault.noEntries"
  | "vault.untitled"
  | "vault.selectEntry"
  | "vault.generator"
  | "vault.editFolder"
  | "vault.newFolder"
  | "vault.cancel"
  | "vault.save"
  | "vault.create"
  | "vault.folderName"
  | "vault.icon"
  | "vault.color"
  | "vault.noColor"
  // EntryDetail
  | "entry.username"
  | "entry.password"
  | "entry.show"
  | "entry.hide"
  | "entry.url"
  | "entry.notes"
  // Onboarding
  | "onboarding.title"
  | "onboarding.subtitle"
  | "onboarding.description"
  | "onboarding.masterPassword"
  | "onboarding.confirmPassword"
  | "onboarding.creating"
  | "onboarding.createVault"
  | "onboarding.warning"
  | "onboarding.copy"
  | "onboarding.copied"
  | "onboarding.noted"
  | "onboarding.confirmText"
  | "onboarding.understand"
  | "onboarding.viewPhrase"
  | "onboarding.enterVault"
  | "onboarding.errShortPassword"
  | "onboarding.errPasswordMismatch"
  // Unlock
  | "unlock.locked"
  | "unlock.recover"
  | "unlock.masterPassword"
  | "unlock.decrypting"
  | "unlock.unlock"
  | "unlock.forgotPassword"
  | "unlock.recoveryIntro"
  | "unlock.recoveryPhrase"
  | "unlock.newPassword"
  | "unlock.confirmNewPassword"
  | "unlock.back"
  | "unlock.recovering"
  | "unlock.errWrongPassword"
  | "unlock.errPhraseLength"
  | "unlock.errNewPasswordShort"
  | "unlock.errPasswordMismatch"
  | "unlock.errInvalidPhrase"
  // EntryEditor
  | "editor.editEntry"
  | "editor.newEntry"
  | "editor.title"
  | "editor.usernameEmail"
  | "editor.password"
  | "editor.generate"
  | "editor.url"
  | "editor.folder"
  | "editor.noFolder"
  | "editor.notes"
  // GeneratorPanel
  | "generator.length"
  | "generator.noAmbiguous"
  | "generator.usePassword"
  // StrengthMeter
  | "strength.veryWeak"
  | "strength.weak"
  | "strength.fair"
  | "strength.strong"
  | "strength.veryStrong"
  // SettingsPanel
  | "settings.title"
  | "settings.palettes"
  | "settings.customColors"
  | "settings.typography"
  | "settings.fontSize"
  | "settings.language"
  | "settings.panels"
  | "settings.opacity"
  | "settings.scanlines"
  | "settings.glow"
  | "settings.flicker"
  | "settings.splashOnStart"
  | "settings.background"
  | "settings.solid"
  | "settings.gradient"
  | "settings.image"
  | "settings.autoLock"
  | "settings.clipboardClear"
  | "settings.dither"
  | "settings.reset"
  | "settings.mixed"
  | "settings.spanish"
  | "settings.english"
  // Palette labels
  | "palette.bg"
  | "palette.surface"
  | "palette.primary"
  | "palette.accent"
  | "palette.text"
  | "palette.danger";

type TranslationDict = Record<TranslationKey, string>;

const es: TranslationDict = {
  "entry.copyNotes": "Copiar todas las notas",
  "splash.tagline": "PHOSPHOR FOX // SECURE VAULT",
  "vault.all": "Todas",
  "vault.favorites": "Favoritas",
  "vault.folders": "Carpetas",
  "vault.search": "Buscar...",
  "vault.noEntries": "Sin entradas.",
  "vault.untitled": "(sin título)",
  "vault.selectEntry": "Selecciona o crea una entrada.",
  "vault.generator": "Generador",
  "vault.editFolder": "Editar carpeta",
  "vault.newFolder": "Nueva carpeta",
  "vault.cancel": "Cancelar",
  "vault.save": "Guardar",
  "vault.create": "Crear",
  "vault.folderName": "Nombre de la carpeta",
  "vault.icon": "Icono",
  "vault.color": "Color",
  "vault.noColor": "Sin color",
  "entry.username": "Usuario",
  "entry.password": "Contraseña",
  "entry.show": "Mostrar",
  "entry.hide": "Ocultar",
  "entry.url": "URL",
  "entry.notes": "Notas",
  "onboarding.title": "SPIRITBYTE",
  "onboarding.subtitle": "Inicializar bóveda segura",
  "onboarding.description":
    "Crea tu contraseña maestra. Cifra tu bóveda con Argon2id + XChaCha20-Poly1305.",
  "onboarding.masterPassword": "Contraseña maestra",
  "onboarding.confirmPassword": "Confirmar contraseña",
  "onboarding.creating": "Generando...",
  "onboarding.createVault": "Crear bóveda",
  "onboarding.warning":
    "Estas 12 palabras son tu única forma de recuperar la bóveda si olvidas la contraseña. Anótalas y guárdalas offline. No se mostrarán de nuevo.",
  "onboarding.copy": "Copiar",
  "onboarding.copied": "Copiado",
  "onboarding.noted": "Ya las anoté",
  "onboarding.confirmText":
    "Confirma que has guardado tu frase de recuperación en un lugar seguro.",
  "onboarding.understand":
    "Entiendo que perder ambas (contraseña y frase) hace la bóveda irrecuperable.",
  "onboarding.viewPhrase": "Ver frase",
  "onboarding.enterVault": "Entrar a la bóveda",
  "onboarding.errShortPassword":
    "La contraseña maestra debe tener al menos 8 caracteres.",
  "onboarding.errPasswordMismatch": "Las contraseñas no coinciden.",
  "unlock.locked": "Bóveda bloqueada",
  "unlock.recover": "Recuperar acceso",
  "unlock.masterPassword": "Contraseña maestra",
  "unlock.decrypting": "Descifrando...",
  "unlock.unlock": "Desbloquear",
  "unlock.forgotPassword":
    "¿Olvidaste tu contraseña? Usar frase de recuperación",
  "unlock.recoveryIntro":
    "Introduce tus 12 palabras y define una nueva contraseña maestra.",
  "unlock.recoveryPhrase": "Frase de recuperación (12 palabras)",
  "unlock.newPassword": "Nueva contraseña maestra",
  "unlock.confirmNewPassword": "Confirmar nueva contraseña",
  "unlock.back": "Volver",
  "unlock.recovering": "Recuperando...",
  "unlock.errWrongPassword": "Contraseña incorrecta.",
  "unlock.errPhraseLength":
    "La frase de recuperación debe tener 12 palabras.",
  "unlock.errNewPasswordShort":
    "La nueva contraseña debe tener al menos 8 caracteres.",
  "unlock.errPasswordMismatch": "Las contraseñas no coinciden.",
  "unlock.errInvalidPhrase": "Frase de recuperación inválida.",
  "editor.editEntry": "Editar entrada",
  "editor.newEntry": "Nueva entrada",
  "editor.title": "Título",
  "editor.usernameEmail": "Usuario / Email",
  "editor.password": "Contraseña",
  "editor.generate": "Generar",
  "editor.url": "URL",
  "editor.folder": "Carpeta",
  "editor.noFolder": "Sin carpeta",
  "editor.notes": "Notas",
  "generator.length": "Longitud",
  "generator.noAmbiguous": "Sin ambiguos",
  "generator.usePassword": "Usar esta contraseña",
  "strength.veryWeak": "Muy débil",
  "strength.weak": "Débil",
  "strength.fair": "Regular",
  "strength.strong": "Fuerte",
  "strength.veryStrong": "Muy fuerte",
  "settings.title": "Personalización",
  "settings.palettes": "Paletas",
  "settings.customColors": "Colores personalizados",
  "settings.typography": "Tipografía",
  "settings.fontSize": "Tamaño de fuente",
  "settings.language": "Idioma",
  "settings.panels": "Paneles",
  "settings.opacity": "Opacidad",
  "settings.scanlines": "Scanlines",
  "settings.glow": "Glow",
  "settings.flicker": "Flicker",
  "settings.splashOnStart": "Splash al iniciar",
  "settings.background": "Fondo",
  "settings.solid": "Sólido",
  "settings.gradient": "Gradiente",
  "settings.image": "Imagen",
  "settings.autoLock": "Auto-bloqueo (min, 0=off)",
  "settings.clipboardClear": "Limpiar portapapeles (seg)",
  "settings.dither": "Dither del zorro (1-4)",
  "settings.reset": "Restablecer",
  "settings.mixed": "Mixta",
  "settings.spanish": "Español",
  "settings.english": "English",
  "palette.bg": "Fondo",
  "palette.surface": "Panel",
  "palette.primary": "Primario",
  "palette.accent": "Acento",
  "palette.text": "Texto",
  "palette.danger": "Peligro",
};

const en: TranslationDict = {
  "entry.copyNotes": "Copy all notes",
  "splash.tagline": "PHOSPHOR FOX // SECURE VAULT",
  "vault.all": "All",
  "vault.favorites": "Favorites",
  "vault.folders": "Folders",
  "vault.search": "Search...",
  "vault.noEntries": "No entries.",
  "vault.untitled": "(untitled)",
  "vault.selectEntry": "Select or create an entry.",
  "vault.generator": "Generator",
  "vault.editFolder": "Edit folder",
  "vault.newFolder": "New folder",
  "vault.cancel": "Cancel",
  "vault.save": "Save",
  "vault.create": "Create",
  "vault.folderName": "Folder name",
  "vault.icon": "Icon",
  "vault.color": "Color",
  "vault.noColor": "No color",
  "entry.username": "Username",
  "entry.password": "Password",
  "entry.show": "Show",
  "entry.hide": "Hide",
  "entry.url": "URL",
  "entry.notes": "Notes",
  "onboarding.title": "SPIRITBYTE",
  "onboarding.subtitle": "Initialize secure vault",
  "onboarding.description":
    "Create your master password. Your vault is encrypted with Argon2id + XChaCha20-Poly1305.",
  "onboarding.masterPassword": "Master password",
  "onboarding.confirmPassword": "Confirm password",
  "onboarding.creating": "Generating...",
  "onboarding.createVault": "Create vault",
  "onboarding.warning":
    "These 12 words are your only way to recover the vault if you forget your password. Write them down and store offline. They will not be shown again.",
  "onboarding.copy": "Copy",
  "onboarding.copied": "Copied",
  "onboarding.noted": "I've written them down",
  "onboarding.confirmText":
    "Confirm you have saved your recovery phrase in a secure place.",
  "onboarding.understand":
    "I understand that losing both (password and phrase) makes the vault unrecoverable.",
  "onboarding.viewPhrase": "View phrase",
  "onboarding.enterVault": "Enter vault",
  "onboarding.errShortPassword":
    "Master password must be at least 8 characters.",
  "onboarding.errPasswordMismatch": "Passwords do not match.",
  "unlock.locked": "Vault locked",
  "unlock.recover": "Recover access",
  "unlock.masterPassword": "Master password",
  "unlock.decrypting": "Decrypting...",
  "unlock.unlock": "Unlock",
  "unlock.forgotPassword":
    "Forgot your password? Use recovery phrase",
  "unlock.recoveryIntro":
    "Enter your 12 words and set a new master password.",
  "unlock.recoveryPhrase": "Recovery phrase (12 words)",
  "unlock.newPassword": "New master password",
  "unlock.confirmNewPassword": "Confirm new password",
  "unlock.back": "Back",
  "unlock.recovering": "Recovering...",
  "unlock.errWrongPassword": "Incorrect password.",
  "unlock.errPhraseLength":
    "Recovery phrase must have 12 words.",
  "unlock.errNewPasswordShort":
    "New password must be at least 8 characters.",
  "unlock.errPasswordMismatch": "Passwords do not match.",
  "unlock.errInvalidPhrase": "Invalid recovery phrase.",
  "editor.editEntry": "Edit entry",
  "editor.newEntry": "New entry",
  "editor.title": "Title",
  "editor.usernameEmail": "Username / Email",
  "editor.password": "Password",
  "editor.generate": "Generate",
  "editor.url": "URL",
  "editor.folder": "Folder",
  "editor.noFolder": "No folder",
  "editor.notes": "Notes",
  "generator.length": "Length",
  "generator.noAmbiguous": "No ambiguous",
  "generator.usePassword": "Use this password",
  "strength.veryWeak": "Very weak",
  "strength.weak": "Weak",
  "strength.fair": "Fair",
  "strength.strong": "Strong",
  "strength.veryStrong": "Very strong",
  "settings.title": "Customization",
  "settings.palettes": "Palettes",
  "settings.customColors": "Custom colors",
  "settings.typography": "Typography",
  "settings.fontSize": "Font size",
  "settings.language": "Language",
  "settings.panels": "Panels",
  "settings.opacity": "Opacity",
  "settings.scanlines": "Scanlines",
  "settings.glow": "Glow",
  "settings.flicker": "Flicker",
  "settings.splashOnStart": "Splash on start",
  "settings.background": "Background",
  "settings.solid": "Solid",
  "settings.gradient": "Gradient",
  "settings.image": "Image",
  "settings.autoLock": "Auto-lock (min, 0=off)",
  "settings.clipboardClear": "Clipboard clear (sec)",
  "settings.dither": "Fox dither (1-4)",
  "settings.reset": "Reset",
  "settings.mixed": "Mixed",
  "settings.spanish": "Español",
  "settings.english": "English",
  "palette.bg": "Background",
  "palette.surface": "Panel",
  "palette.primary": "Primary",
  "palette.accent": "Accent",
  "palette.text": "Text",
  "palette.danger": "Danger",
};

const DICTS: Record<Language, TranslationDict> = { es, en };

interface I18nState {
  lang: Language;
  setLang: (lang: Language) => void;
  t: (key: TranslationKey) => string;
}

export const useI18n = create<I18nState>((set, get) => ({
  lang: "es",
  setLang: (lang) => set({ lang }),
  t: (key) => DICTS[get().lang][key] ?? key,
}));

//! Linux-only GTK client. Windows keeps the existing Tauri/React interface.
#[cfg(not(target_os = "linux"))]
compile_error!("Use the Tauri project on Windows; this client is Linux-only.");
mod background;
mod folder_style;
mod fonts;
mod gif_stream;
mod intro;
mod model;
mod ui;
#[cfg(feature = "ui-check")]
mod ui_check;
#[path = "../../src-tauri/src/wallpaper.rs"]
mod wallpaper;

use gtk::{gdk, gio, prelude::*};
use serde_json::{json, Value};
use spiritbyte_core::{
    backup, generator,
    vault::{self, Entry, Folder, Session, VaultData, VaultPaths},
};
use std::{
    cell::{Cell, RefCell},
    path::{Path, PathBuf},
    rc::Rc,
    sync::mpsc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use zeroize::Zeroizing;

struct App {
    window: gtk::ApplicationWindow,
    content: gtk::Box,
    background: Rc<background::Background>,
    effects: gtk::Overlay,
    css: gtk::CssProvider,
    paths: VaultPaths,
    settings: RefCell<Value>,
    session: RefCell<Option<Session>>,
    epoch: Cell<u64>,
    busy: Cell<bool>,
    activity: Rc<Cell<Instant>>,
    clipboard: RefCell<Option<Rc<Cell<bool>>>>,
    clipboard_serial: Cell<u64>,
    filter: RefCell<String>,
    query: RefCell<String>,
    list: RefCell<Option<gtk::ListStore>>,
    tree: RefCell<Option<gtk::TreeView>>,
    status: RefCell<Option<gtk::Label>>,
    navigation: RefCell<Option<gtk::ListBox>>,
    detail: RefCell<Option<gtk::Box>>,
    entry_stack: RefCell<Option<gtk::Stack>>,
    refresh_guard: Cell<bool>,
    effect_timer: RefCell<Option<glib::SourceId>>,
    intro_timer: RefCell<Option<glib::SourceId>>,
    #[cfg(feature = "ui-check")]
    external_url: RefCell<Option<String>>,
}

struct ClipboardLease {
    owned: Rc<Cell<bool>>,
    secret: Zeroizing<String>,
}
impl Drop for ClipboardLease {
    fn drop(&mut self) {
        self.owned.set(false);
    }
}

fn main() {
    let _fonts = fonts::register().unwrap_or_else(|error| {
        eprintln!("{error}");
        std::process::exit(1)
    });
    let application = gtk::Application::new(
        Some("com.spiritbyte.app"),
        gio::ApplicationFlags::NON_UNIQUE,
    );
    application.connect_activate(|application| {
        let app = App::new(application);
        #[cfg(feature = "ui-check")]
        if let Some(path) = std::env::var_os("SPIRITBYTE_NATIVE_CHECK") {
            ui_check::run(app.clone(), PathBuf::from(path));
        }
        #[cfg(feature = "ui-check")]
        if let Some(path) = std::env::var_os("SPIRITBYTE_POINTER_CHECK") {
            ui_check::pointer_fixture(app, PathBuf::from(path));
        }
        #[cfg(not(feature = "ui-check"))]
        let _ = app;
    });
    application.run_with_args(&std::env::args().collect::<Vec<_>>());
}

fn box_v(spacing: i32) -> gtk::Box {
    gtk::Box::new(gtk::Orientation::Vertical, spacing)
}
fn box_h(spacing: i32) -> gtk::Box {
    gtk::Box::new(gtk::Orientation::Horizontal, spacing)
}
fn label(text: &str) -> gtk::Label {
    let l = gtk::Label::new(Some(text));
    l.set_xalign(0.0);
    l.set_line_wrap(true);
    l
}
fn input(text: &str, secret: bool) -> gtk::Entry {
    let e = gtk::Entry::new();
    e.set_text(text);
    e.set_visibility(!secret);
    e.set_hexpand(true);
    e
}
fn pack(container: &gtk::Box, widget: &impl IsA<gtk::Widget>) {
    container.pack_start(widget, false, false, 0);
}
fn button(container: &gtk::Box, text: &str, action: impl Fn() + 'static) -> gtk::Button {
    let b = gtk::Button::with_label(text);
    b.connect_clicked(move |_| action());
    pack(container, &b);
    b
}
fn field(container: &gtk::Box, title: &str, text: &str, secret: bool) -> gtk::Entry {
    pack(container, &label(title));
    let e = input(text, secret);
    pack(container, &e);
    e
}
fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

impl App {
    fn new(application: &gtk::Application) -> Rc<Self> {
        let window = gtk::ApplicationWindow::new(application);
        window.set_title("SpiritByte");
        window.set_default_size(1100, 720);
        window.set_size_request(880, 600);
        window.set_position(gtk::WindowPosition::Center);
        window.set_icon(Some(&gdk_pixbuf::Pixbuf::from_bytes(
            &glib::Bytes::from_static(include_bytes!("../assets/window-icon.rgba")),
            gdk_pixbuf::Colorspace::Rgb,
            true,
            8,
            128,
            128,
            128 * 4,
        )));
        let overlay = gtk::Overlay::new();
        let background = background::Background::new();
        overlay.add(&background.area);
        let content = box_v(0);
        overlay.add_overlay(&content);
        // Draw CRT effects after the content, on the existing container. A
        // DrawingArea overlay has its own GdkWindow, which can intercept
        // input even when the overlay wrapper passes input through.
        let effects = overlay.clone();
        window.add(&overlay);
        let paths = VaultPaths::new(model::data_dir());
        let settings = model::settings(&paths);
        let this = Rc::new(Self {
            window,
            content,
            background,
            effects,
            css: gtk::CssProvider::new(),
            paths,
            settings: RefCell::new(settings),
            session: RefCell::new(None),
            epoch: Cell::new(0),
            busy: Cell::new(false),
            activity: Rc::new(Cell::new(Instant::now())),
            clipboard: RefCell::new(None),
            clipboard_serial: Cell::new(0),
            filter: RefCell::new("all".into()),
            query: RefCell::new(String::new()),
            list: RefCell::new(None),
            tree: RefCell::new(None),
            status: RefCell::new(None),
            navigation: RefCell::new(None),
            detail: RefCell::new(None),
            entry_stack: RefCell::new(None),
            refresh_guard: Cell::new(false),
            effect_timer: RefCell::new(None),
            intro_timer: RefCell::new(None),
            #[cfg(feature = "ui-check")]
            external_url: RefCell::new(None),
        });
        gtk::StyleContext::add_provider_for_screen(
            &gdk::Screen::default().unwrap(),
            &this.css,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        let weak = Rc::downgrade(&this);
        this.effects.connect_local("draw", true, move |values| {
            let area = values[0].get::<gtk::Overlay>().unwrap();
            let cr = values[1].get::<gtk::cairo::Context>().unwrap();
            if let Some(app) = weak.upgrade() {
                let settings = app.settings.borrow();
                if settings["scanlines"].as_bool().unwrap_or(false) {
                    cr.set_source_rgba(0.0, 0.0, 0.0, 0.13);
                    let width = area.allocated_width() as f64;
                    for y in (0..area.allocated_height()).step_by(4) {
                        cr.rectangle(0.0, y as f64, width, 1.0);
                    }
                    let _ = cr.fill();
                }
                if settings["flicker"].as_bool().unwrap_or(false)
                    && (!settings["lowPowerMode"].as_bool().unwrap_or(true)
                        || app.window.is_active())
                {
                    cr.set_source_rgba(
                        0.0,
                        0.0,
                        0.0,
                        if (now() / 500) % 4 == 0 { 0.06 } else { 0.0 },
                    );
                    let _ = cr.paint();
                }
            }
            Some(false.to_value())
        });
        let app = this.clone();
        this.window.connect_is_active_notify(move |_| app.animate());
        let app = this.clone();
        this.window.connect_map(move |_| app.animate());
        let app = this.clone();
        this.window.connect_unmap(move |_| app.animate());
        let app = this.clone();
        this.window.connect_delete_event(move |_, _| {
            app.lock();
            glib::Propagation::Proceed
        });
        this.window.add_events(
            gdk::EventMask::POINTER_MOTION_MASK
                | gdk::EventMask::BUTTON_PRESS_MASK
                | gdk::EventMask::KEY_PRESS_MASK
                | gdk::EventMask::SCROLL_MASK,
        );
        let app = this.clone();
        this.window.connect_event(move |_, event| {
            if matches!(
                event.event_type(),
                gdk::EventType::KeyPress
                    | gdk::EventType::ButtonPress
                    | gdk::EventType::MotionNotify
                    | gdk::EventType::Scroll
            ) {
                app.activity.set(Instant::now());
            }
            glib::Propagation::Proceed
        });
        // Events in modal native forms also count as activity.
        let app = this.clone();
        glib::timeout_add_seconds_local(1, move || {
            app.check_inactivity();
            glib::ControlFlow::Continue
        });
        if let Err(e) = this.apply_settings() {
            eprintln!("Wallpaper: {e}");
        }
        if this.settings.borrow()["showSplash"]
            .as_bool()
            .unwrap_or(true)
        {
            this.show_intro();
        } else {
            this.show_locked();
        }
        this.window.show_all();
        this.animate();
        // The window's callbacks keep the client alive until its application exits.
        this
    }
    fn tr<'a>(&self, es: &'a str, en: &'a str) -> &'a str {
        if self.settings.borrow()["language"] == "en" {
            en
        } else {
            es
        }
    }
    fn clean_content(&self) {
        if let Some(timer) = self.intro_timer.borrow_mut().take() {
            timer.remove();
        }
        for child in self.content.children() {
            wipe(&child);
            self.content.remove(&child);
        }
        self.list.borrow_mut().take();
        self.tree.borrow_mut().take();
        self.status.borrow_mut().take();
        self.navigation.borrow_mut().take();
        self.detail.borrow_mut().take();
        self.entry_stack.borrow_mut().take();
    }
    fn show_locked(self: &Rc<Self>) {
        self.show_locked_values("", "");
    }
    fn show_locked_values(self: &Rc<Self>, password_value: &str, confirmation: &str) {
        self.clean_content();
        let card = box_v(12);
        card.style_context().add_class("panel");
        card.set_valign(gtk::Align::Center);
        card.set_halign(gtk::Align::Center);
        card.set_size_request(480, -1);
        card.style_context().add_class("unlock-card");
        let brand = box_h(12);
        pack(&brand, &ui::icon("key", 24));
        let name = label("SPIRITBYTE");
        name.style_context().add_class("brand");
        brand.pack_start(&name, true, true, 0);
        let mut languages = vec![];
        for (code, title) in [("es", "Español"), ("en", "English")] {
            let b = gtk::Button::with_label(&code.to_uppercase());
            b.set_widget_name(&format!("language-{code}"));
            b.set_tooltip_text(Some(title));
            b.style_context().add_class("language-button");
            if self.settings.borrow()["language"] == code {
                b.style_context().add_class("primary");
            }
            pack(&brand, &b);
            languages.push((code, b));
        }
        let app = self.clone();
        ui::icon_button(
            &brand,
            "settings",
            self.tr("Ajustes", "Settings"),
            move || app.settings_dialog(),
        );
        pack(&card, &brand);
        let exists = vault::vault_exists(&self.paths);
        let title = label(if exists {
            self.tr("DESBLOQUEAR BÓVEDA", "UNLOCK VAULT")
        } else {
            self.tr("CREAR BÓVEDA", "CREATE VAULT")
        });
        title.style_context().add_class("title");
        pack(&card, &title);
        let password = ui::password_field(
            &card,
            self.tr("Contraseña maestra", "Master password"),
            "auth-password",
            self.tr("Mostrar contraseña", "Show password"),
            self.tr("Ocultar contraseña", "Hide password"),
        );
        password.set_text(password_value);
        let confirm = if exists {
            None
        } else {
            let c = ui::password_field(
                &card,
                self.tr("Repite la contraseña", "Confirm password"),
                "auth-confirm",
                self.tr("Mostrar contraseña", "Show password"),
                self.tr("Ocultar contraseña", "Hide password"),
            );
            c.set_text(confirmation);
            Some(c)
        };
        for (language, b) in languages {
            let app = self.clone();
            let p = password.clone();
            let c = confirm.clone();
            b.connect_clicked(move |_| {
                if app.settings.borrow()["language"] == language {
                    return;
                }
                let mut settings = app.settings.borrow().clone();
                settings["language"] = json!(language);
                if let Err(e) = model::save_settings(&app.paths, &settings) {
                    app.message(&e);
                    return;
                }
                let master = Zeroizing::new(p.text().to_string());
                let confirmation =
                    Zeroizing::new(c.as_ref().map(|c| c.text().to_string()).unwrap_or_default());
                *app.settings.borrow_mut() = settings;
                app.show_locked_values(&master, &confirmation);
            });
        }
        let app = self.clone();
        let p = password.clone();
        let c = confirm.clone();
        let submit = button(
            &card,
            if exists {
                self.tr("Desbloquear", "Unlock")
            } else {
                self.tr("Crear bóveda", "Create vault")
            },
            move || {
                let password = Zeroizing::new(p.text().to_string());
                p.set_text("");
                if let Some(c) = &c {
                    let matched = c.text().as_str() == password.as_str();
                    c.set_text("");
                    if !matched || password.chars().count() < 12 {
                        app.message(app.tr(
                            "Usa al menos 12 caracteres y confirma la misma contraseña.",
                            "Use at least 12 characters and a matching confirmation.",
                        ));
                        return;
                    }
                }
                let paths = app.paths.clone();
                app.work(
                    move || {
                        let result = if exists {
                            vault::unlock(&paths, &password).map(|s| (s, None))
                        } else {
                            vault::create_vault(&paths, &password)
                                .map(|(s, p)| (s, Some(Zeroizing::new(p))))
                        };
                        result
                            .and_then(|(s, p)| {
                                backup::validate(&s.data)?;
                                Ok((s, p))
                            })
                            .map_err(|e| e.to_string())
                    },
                    move |app, (session, phrase)| {
                        *app.session.borrow_mut() = Some(session);
                        app.activity.set(Instant::now());
                        app.show_vault();
                        if let Some(phrase) = phrase {
                            app.recovery_notice(&phrase);
                        }
                    },
                );
            },
        );
        if let Some(c) = &confirm {
            let submit = submit.downgrade();
            c.connect_activate(move |_| {
                if let Some(submit) = submit.upgrade() {
                    submit.clicked();
                }
            });
        }
        // The submit handler owns the entry; a strong reference back from the
        // entry would retain the entire form after switching language or locking.
        let submit = submit.downgrade();
        password.connect_activate(move |_| {
            if let Some(submit) = submit.upgrade() {
                submit.clicked();
            }
        });
        if exists {
            let app = self.clone();
            button(
                &card,
                self.tr(
                    "Recuperar con las 12 palabras",
                    "Recover using the 12 words",
                ),
                move || app.recover(),
            );
        }
        let links = box_h(4);
        links.set_halign(gtk::Align::End);
        self.social_links(&links);
        pack(&card, &links);
        self.content.pack_start(&card, true, true, 0);
        self.content.show_all();
        password.grab_focus();
    }
    fn work<T: Send + 'static>(
        self: &Rc<Self>,
        job: impl FnOnce() -> Result<T, String> + Send + 'static,
        done: impl FnOnce(&Rc<Self>, T) + 'static,
    ) {
        if self.busy.replace(true) {
            return;
        }
        self.window.set_sensitive(false);
        self.window
            .set_title(self.tr("SpiritByte — trabajando…", "SpiritByte — working…"));
        let epoch = self.epoch.get();
        let (tx, rx) = mpsc::sync_channel(1);
        std::thread::spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(job))
                .unwrap_or_else(|_| Err("Operation failed".into()));
            let _ = tx.send(result);
        });
        let weak = Rc::downgrade(self);
        let mut done = Some(done);
        glib::timeout_add_local(Duration::from_millis(30), move || {
            let Some(app) = weak.upgrade() else {
                return glib::ControlFlow::Break;
            };
            match rx.try_recv() {
                Ok(result) => {
                    app.busy.set(false);
                    app.window.set_sensitive(true);
                    app.window.set_title("SpiritByte");
                    if app.epoch.get() == epoch {
                        match result {
                            Ok(value) => done.take().unwrap()(&app, value),
                            Err(e) => app.message(&e),
                        }
                    }
                    glib::ControlFlow::Break
                }
                Err(mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
                Err(mpsc::TryRecvError::Disconnected) => {
                    app.busy.set(false);
                    app.window.set_sensitive(true);
                    app.window.set_title("SpiritByte");
                    app.message("Worker stopped");
                    glib::ControlFlow::Break
                }
            }
        });
    }
    fn dialog(&self, title: &str) -> gtk::Dialog {
        let dialog = gtk::Dialog::with_buttons(
            Some(title),
            Some(&self.window),
            gtk::DialogFlags::MODAL | gtk::DialogFlags::DESTROY_WITH_PARENT,
            &[
                (self.tr("Cancelar", "Cancel"), gtk::ResponseType::Cancel),
                (self.tr("Guardar", "Save"), gtk::ResponseType::Ok),
            ],
        );
        dialog.set_default_size(520, -1);
        dialog.set_default_response(gtk::ResponseType::Ok);
        if let Some(save) = dialog.widget_for_response(gtk::ResponseType::Ok) {
            save.set_widget_name("dialog-save");
        }
        let area = dialog.content_area();
        area.set_spacing(8);
        area.set_margin_start(20);
        area.set_margin_end(20);
        area.set_margin_top(16);
        area.set_margin_bottom(16);
        let activity = self.activity.clone();
        dialog.connect_event(move |_, event| {
            if matches!(
                event.event_type(),
                gdk::EventType::KeyPress
                    | gdk::EventType::ButtonPress
                    | gdk::EventType::MotionNotify
                    | gdk::EventType::Scroll
            ) {
                activity.set(Instant::now());
            }
            glib::Propagation::Proceed
        });
        dialog
    }
    fn message(&self, text: &str) {
        let d = gtk::MessageDialog::new(
            Some(&self.window),
            gtk::DialogFlags::MODAL | gtk::DialogFlags::DESTROY_WITH_PARENT,
            gtk::MessageType::Info,
            gtk::ButtonsType::Ok,
            text,
        );
        d.run();
        wipe(&d.clone().upcast());
        d.hide();
        d.close();
    }
    fn confirm(&self, text: &str) -> bool {
        let d = gtk::MessageDialog::new(
            Some(&self.window),
            gtk::DialogFlags::MODAL | gtk::DialogFlags::DESTROY_WITH_PARENT,
            gtk::MessageType::Question,
            gtk::ButtonsType::OkCancel,
            text,
        );
        if let Some(accept) = d.widget_for_response(gtk::ResponseType::Ok) {
            accept.set_widget_name("confirmation-accept");
        }
        let response = d.run() == gtk::ResponseType::Ok;
        d.hide();
        d.close();
        response
    }
    fn recovery_notice(&self, phrase: &str) {
        let d = self.dialog(self.tr(
            "Guarda tu frase de recuperación",
            "Save your recovery phrase",
        ));
        d.set_response_sensitive(gtk::ResponseType::Cancel, false);
        pack(&d.content_area(),&label(self.tr("Anota estas 12 palabras en un lugar seguro. Permiten recuperar tu bóveda si olvidas la contraseña.","Write these 12 words in a safe place. They recover your vault if you forget the password.")));
        let words = input(phrase, false);
        words.set_editable(false);
        pack(&d.content_area(), &words);
        d.show_all();
        d.run();
        words.set_text("");
        d.hide();
        d.close();
    }
    fn recover(self: &Rc<Self>) {
        let d = self.dialog(self.tr("Recuperar bóveda", "Recover vault"));
        let area = d.content_area();
        let phrase = field(
            &area,
            self.tr("12 palabras de recuperación", "12 recovery words"),
            "",
            true,
        );
        let pw = field(
            &area,
            self.tr(
                "Nueva contraseña (mínimo 12 caracteres)",
                "New password (at least 12 characters)",
            ),
            "",
            true,
        );
        let repeat = field(
            &area,
            self.tr("Repite la contraseña", "Confirm password"),
            "",
            true,
        );
        d.show_all();
        let response = d.run();
        let phrase_value = Zeroizing::new(phrase.text().to_string());
        let password = Zeroizing::new(pw.text().to_string());
        let matched = repeat.text().as_str() == password.as_str();
        wipe(&d.clone().upcast());
        d.hide();
        d.close();
        if response != gtk::ResponseType::Ok {
            return;
        }
        if password.chars().count() < 12 || !matched {
            self.message(self.tr(
                "Contraseña demasiado corta o confirmación distinta.",
                "Password too short or confirmation differs.",
            ));
            return;
        }
        let paths = self.paths.clone();
        self.work(
            move || {
                vault::recover_and_reset(&paths, &phrase_value, &password)
                    .map_err(|e| e.to_string())
            },
            |app, s| {
                *app.session.borrow_mut() = Some(s);
                app.activity.set(Instant::now());
                app.show_vault();
            },
        );
    }
    fn check_inactivity(self: &Rc<Self>) {
        let minutes = self.settings.borrow()["autoLockMinutes"]
            .as_u64()
            .unwrap_or(5);
        if minutes > 0
            && self.session.borrow().is_some()
            && self.activity.get().elapsed().as_secs() >= minutes.saturating_mul(60)
        {
            self.lock();
        }
    }
    fn lock(self: &Rc<Self>) {
        #[cfg(feature = "ui-check")]
        eprintln!("native-check: locking");
        self.epoch.set(self.epoch.get().wrapping_add(1));
        self.clear_clipboard();
        self.session.borrow_mut().take();
        for window in gtk::Window::list_toplevels() {
            let Ok(window) = window.downcast::<gtk::Window>() else {
                continue;
            };
            if window != self.window.clone().upcast::<gtk::Window>()
                && window.transient_for().is_some()
            {
                wipe(&window.clone().upcast());
                if let Some(dialog) = window.downcast_ref::<gtk::Dialog>() {
                    dialog.response(gtk::ResponseType::Cancel);
                }
                window.hide();
                window.close();
            }
        }
        self.filter.replace("all".into());
        self.query.borrow_mut().clear();
        self.show_locked();
    }
    fn copy(self: &Rc<Self>, secret: String) {
        self.clear_clipboard();
        let clipboard = gtk::Clipboard::get(&gdk::SELECTION_CLIPBOARD);
        let owned = Rc::new(Cell::new(true));
        let lease = ClipboardLease {
            owned: owned.clone(),
            secret: Zeroizing::new(secret),
        };
        let targets = ["UTF8_STRING", "TEXT", "STRING", "text/plain;charset=utf-8"]
            .map(|name| gtk::TargetEntry::new(name, gtk::TargetFlags::empty(), 0));
        if !clipboard.set_with_data(&targets, move |_, selection, _| {
            if lease.owned.get() {
                selection.set_text(&lease.secret);
            }
        }) {
            return;
        }
        *self.clipboard.borrow_mut() = Some(owned);
        let serial = self.clipboard_serial.get();
        let seconds = self.settings.borrow()["clipboardClearSeconds"]
            .as_u64()
            .unwrap_or(20)
            .clamp(1, 3600);
        let weak = Rc::downgrade(self);
        glib::timeout_add_seconds_local_once(seconds as u32, move || {
            if let Some(app) = weak.upgrade() {
                if serial == app.clipboard_serial.get() {
                    app.clear_clipboard();
                }
            }
        });
    }
    fn clear_clipboard(&self) {
        self.clipboard_serial
            .set(self.clipboard_serial.get().wrapping_add(1));
        if let Some(owned) = self.clipboard.borrow_mut().take() {
            // GTK drops ClipboardLease when ownership changes. Never synchronously
            // request another application's clipboard on Wayland, and never erase
            // text the user copied elsewhere after copying a vault credential.
            if owned.get() {
                gtk::Clipboard::get(&gdk::SELECTION_CLIPBOARD).clear();
            }
        }
    }
    fn change(self: &Rc<Self>, update: impl FnOnce(&mut VaultData)) -> bool {
        let result = {
            let mut session = self.session.borrow_mut();
            if let Some(s) = session.as_mut() {
                let mut data = s.data.clone();
                update(&mut data);
                model::commit(&self.paths, s, data)
            } else {
                Err("Vault locked".into())
            }
        };
        match result {
            Ok(()) => {
                self.refresh_navigation();
                self.refresh();
                true
            }
            Err(e) => {
                self.message(&e);
                false
            }
        }
    }
    fn folder_path(data: &VaultData, id: &str) -> String {
        let mut names = vec![];
        let mut current = Some(id);
        let mut seen = std::collections::HashSet::new();
        while let Some(id) = current {
            if !seen.insert(id) {
                break;
            }
            let Some(f) = data.folders.iter().find(|f| f.id == id) else {
                break;
            };
            names.push(f.name.clone());
            current = f.parent_id.as_deref();
        }
        names.reverse();
        names.join(" / ")
    }
    fn folders(&self, combo: &gtk::ComboBoxText, all: bool) {
        combo.remove_all();
        combo.append(
            Some(if all { "all" } else { "" }),
            self.tr(
                if all {
                    "Todas las carpetas"
                } else {
                    "Sin carpeta"
                },
                if all { "All folders" } else { "No folder" },
            ),
        );
        let session = self.session.borrow();
        if let Some(s) = session.as_ref() {
            let mut folders: Vec<_> = s
                .data
                .folders
                .iter()
                .map(|f| (&f.id, Self::folder_path(&s.data, &f.id)))
                .collect();
            folders.sort_by(|a, b| a.1.to_lowercase().cmp(&b.1.to_lowercase()));
            for (id, path) in folders {
                combo.append(Some(id), &path);
            }
        }
    }
    fn selected(&self) -> Option<String> {
        self.selected_ids().into_iter().next()
    }
    fn selected_ids(&self) -> Vec<String> {
        let tree = self.tree.borrow();
        let Some(tree) = tree.as_ref() else {
            return vec![];
        };
        let (paths, model) = tree.selection().selected_rows();
        paths
            .iter()
            .filter_map(|path| {
                model
                    .iter(path)
                    .and_then(|iter| model.value(&iter, 0).get::<String>().ok())
            })
            .collect()
    }
    fn in_folder(data: &VaultData, folder: &str, ancestor: &str) -> bool {
        let mut current = Some(folder);
        let mut seen = std::collections::HashSet::new();
        while let Some(id) = current {
            if id == ancestor {
                return true;
            }
            if !seen.insert(id) {
                break;
            }
            current = data
                .folders
                .iter()
                .find(|f| f.id == id)
                .and_then(|f| f.parent_id.as_deref());
        }
        false
    }
    fn refresh(self: &Rc<Self>) {
        if self.refresh_guard.replace(true) {
            return;
        }
        if let (Some(list), Some(session)) =
            (self.list.borrow().as_ref(), self.session.borrow().as_ref())
        {
            let selected = self.selected_ids();
            list.clear();
            let filter = self.filter.borrow();
            let query = self.query.borrow().to_lowercase();
            let mut count = 0;
            let mut entries: Vec<_> = session
                .data
                .entries
                .iter()
                .filter(|e| {
                    let scope = match filter.as_str() {
                        "all" => true,
                        "star" => e.favorite,
                        "none" => e.folder_id.is_none(),
                        id => e
                            .folder_id
                            .as_deref()
                            .is_some_and(|folder| Self::in_folder(&session.data, folder, id)),
                    };
                    scope
                        && (query.is_empty()
                            || [&e.title, &e.username, &e.url, &e.notes]
                                .iter()
                                .any(|v| v.to_lowercase().contains(&query)))
                })
                .collect();
            entries.sort_by(|a, b| {
                b.favorite
                    .cmp(&a.favorite)
                    .then_with(|| a.title.to_lowercase().cmp(&b.title.to_lowercase()))
            });
            for e in entries {
                let title = format!("{}{}", if e.favorite { "★ " } else { "" }, e.title);
                let folder = e
                    .folder_id
                    .as_deref()
                    .map(|id| Self::folder_path(&session.data, id))
                    .unwrap_or_default();
                let iter = list.insert_with_values(
                    None,
                    &[
                        (0, &e.id),
                        (1, &title),
                        (2, &e.username),
                        (3, &folder),
                        (
                            4,
                            &format!(
                                "{}\n<small>{}</small>",
                                glib::markup_escape_text(&title),
                                glib::markup_escape_text(&e.username)
                            ),
                        ),
                    ],
                );
                if selected.contains(&e.id) {
                    if let Some(tree) = self.tree.borrow().as_ref() {
                        tree.selection().select_iter(&iter);
                    }
                }
                count += 1;
            }
            if let Some(stack) = self.entry_stack.borrow().as_ref() {
                if let Some(empty) = stack
                    .child_by_name("empty")
                    .and_then(|w| w.downcast::<gtk::Label>().ok())
                {
                    empty.set_text(if query.is_empty() {
                        self.tr("Sin entradas.", "No entries.")
                    } else {
                        self.tr("Sin resultados.", "No results.")
                    });
                }
                stack.set_visible_child_name(if count == 0 { "empty" } else { "entries" });
            }
            if let Some(l) = self.status.borrow().as_ref() {
                l.set_text(&format!("{count} / {}", session.data.entries.len()));
            }
        }
        self.refresh_guard.set(false);
        self.show_detail();
    }
    fn edit(self: &Rc<Self>, id: Option<String>) {
        let existing = self.session.borrow().as_ref().and_then(|s| {
            s.data
                .entries
                .iter()
                .find(|e| Some(&e.id) == id.as_ref())
                .cloned()
        });
        if self.session.borrow().is_none() {
            return;
        }
        let d = self.dialog(self.tr("Entrada de la bóveda", "Vault entry"));
        d.set_default_size(520, 580);
        let area = box_v(8);
        let scroll = gtk::ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
        scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
        scroll.set_min_content_height(320);
        scroll.set_max_content_height(460);
        scroll.set_propagate_natural_height(true);
        scroll.add(&area);
        d.content_area().pack_start(&scroll, true, true, 0);
        let title = field(
            &area,
            self.tr("Título", "Title"),
            existing.as_ref().map_or("", |e| &e.title),
            false,
        );
        let username = field(
            &area,
            self.tr("Usuario", "Username"),
            existing.as_ref().map_or("", |e| &e.username),
            false,
        );
        let password = field(
            &area,
            self.tr("Contraseña", "Password"),
            existing.as_ref().map_or("", |e| &e.password),
            true,
        );
        let tools = box_h(8);
        let reveal = gtk::CheckButton::with_label(self.tr("Mostrar contraseña", "Show password"));
        let p = password.clone();
        reveal.connect_toggled(move |b| p.set_visibility(b.is_active()));
        pack(&tools, &reveal);
        let app = self.clone();
        let p = password.clone();
        button(&tools, self.tr("Generar", "Generate"), move || {
            app.generator(Some(p.clone()))
        });
        pack(&area, &tools);
        let strength = label("");
        let l = strength.clone();
        password.connect_changed(move |p| {
            let s = generator::strength(p.text().as_str());
            l.set_text(&format!("{} / 4 · {:.0} bits", s.score, s.entropy_bits));
        });
        pack(&area, &strength);
        let url = field(
            &area,
            "URL",
            existing.as_ref().map_or("", |e| &e.url),
            false,
        );
        pack(&area, &label(self.tr("Notas", "Notes")));
        let notes = gtk::TextView::new();
        notes.set_wrap_mode(gtk::WrapMode::WordChar);
        notes.set_size_request(-1, 90);
        notes
            .buffer()
            .unwrap()
            .set_text(existing.as_ref().map_or("", |e| &e.notes));
        let scroll = gtk::ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
        scroll.set_min_content_height(90);
        scroll.add(&notes);
        pack(&area, &scroll);
        let folders = gtk::ComboBoxText::new();
        self.folders(&folders, false);
        folders.set_active_id(Some(
            existing
                .as_ref()
                .and_then(|e| e.folder_id.as_deref())
                .unwrap_or(""),
        ));
        pack(&area, &folders);
        let favorite = gtk::CheckButton::with_label(self.tr("★ Favorito", "★ Favorite"));
        favorite.set_active(existing.as_ref().is_some_and(|e| e.favorite));
        pack(&area, &favorite);
        d.show_all();
        let response = d.run();
        if response == gtk::ResponseType::Ok && self.session.borrow().is_some() {
            let buffer = notes.buffer().unwrap();
            let folder = folders
                .active_id()
                .filter(|v| !v.is_empty())
                .map(|v| v.to_string());
            let mut e = existing.unwrap_or_else(|| Entry {
                id: uuid::Uuid::new_v4().to_string(),
                title: String::new(),
                username: String::new(),
                password: String::new(),
                url: String::new(),
                notes: String::new(),
                folder_id: None,
                icon_id: None,
                favorite: false,
                created_at: now(),
                updated_at: now(),
            });
            e.title = title.text().trim().to_string();
            e.username = username.text().to_string();
            e.password = password.text().to_string();
            e.url = url.text().to_string();
            e.notes = buffer
                .text(&buffer.start_iter(), &buffer.end_iter(), false)
                .map(|v| v.to_string())
                .unwrap_or_default();
            e.folder_id = folder;
            e.favorite = favorite.is_active();
            e.updated_at = now();
            if e.title.is_empty() {
                self.message(self.tr("El título es obligatorio.", "Title is required."));
            } else {
                self.change(move |data| {
                    if let Some(old) = data.entries.iter_mut().find(|old| old.id == e.id) {
                        *old = e;
                    } else {
                        data.entries.push(e);
                    }
                });
            }
        }
        wipe(&d.clone().upcast());
        d.hide();
        d.close();
    }
    fn manage_folders(self: &Rc<Self>) {
        self.folder_dialog(None);
    }
    fn folder_dialog(self: &Rc<Self>, id: Option<String>) {
        let existing = self.session.borrow().as_ref().and_then(|s| {
            s.data
                .folders
                .iter()
                .find(|f| Some(&f.id) == id.as_ref())
                .cloned()
        });
        if self.session.borrow().is_none() || (id.is_some() && existing.is_none()) {
            return;
        }
        let d = self.dialog(if existing.is_some() {
            self.tr("Editar carpeta", "Edit folder")
        } else {
            self.tr("Nueva carpeta", "New folder")
        });
        d.set_default_size(560, 600);
        let area = box_v(8);
        let scroll = gtk::ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
        scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
        scroll.set_min_content_height(320);
        scroll.add(&area);
        d.content_area().pack_start(&scroll, true, true, 0);
        let name = field(
            &area,
            self.tr("Nombre", "Name"),
            existing.as_ref().map_or("", |f| &f.name),
            false,
        );
        name.set_widget_name("folder-name");
        let parent = gtk::ComboBoxText::new();
        parent.set_widget_name("folder-parent");
        parent.append(Some(""), self.tr("Nivel principal", "Top level"));
        if let Some(session) = self.session.borrow().as_ref() {
            let mut folders: Vec<_> = session
                .data
                .folders
                .iter()
                .filter(|f| {
                    !id.as_ref()
                        .is_some_and(|id| Self::in_folder(&session.data, &f.id, id))
                })
                .collect();
            folders.sort_by_key(|f| Self::folder_path(&session.data, &f.id).to_lowercase());
            for f in folders {
                parent.append(Some(&f.id), &Self::folder_path(&session.data, &f.id));
            }
        }
        parent.set_active_id(Some(
            existing
                .as_ref()
                .and_then(|f| f.parent_id.as_deref())
                .unwrap_or(""),
        ));
        pack(&area, &label(self.tr("Carpeta superior", "Parent folder")));
        pack(&area, &parent);
        let appearance = folder_style::Picker::new(self, &area);
        if let Some(f) = &existing {
            appearance.set(f.icon.as_deref(), f.color.as_deref());
        }
        if let Some(id) = id.clone() {
            let app = self.clone();
            let dialog = d.downgrade();
            let delete = button(
                &area,
                self.tr(
                    "Eliminar carpeta (conserva sus entradas)",
                    "Delete folder (keeps its entries)",
                ),
                move || {
                    if app.confirm(app.tr(
                        "¿Eliminar esta carpeta? Sus entradas y subcarpetas se conservarán.",
                        "Delete this folder? Its entries and subfolders will be kept.",
                    )) {
                        if app.change(|data| {
                            data.folders.retain(|f| f.id != id);
                            for f in &mut data.folders {
                                if f.parent_id.as_ref() == Some(&id) {
                                    f.parent_id = None;
                                }
                            }
                            for e in &mut data.entries {
                                if e.folder_id.as_ref() == Some(&id) {
                                    e.folder_id = None;
                                }
                            }
                        }) {
                            if let Some(dialog) = dialog.upgrade() {
                                dialog.response(gtk::ResponseType::Cancel);
                            }
                            if *app.filter.borrow() == id {
                                app.filter.replace("all".into());
                            }
                            app.show_vault();
                        }
                    }
                },
            );
            delete.set_widget_name("delete-folder");
            delete.style_context().add_class("danger");
        }
        d.show_all();
        name.grab_focus();
        let response = d.run();
        if response == gtk::ResponseType::Ok && self.session.borrow().is_some() {
            let name = name.text().trim().to_string();
            let id = id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
            let parent = parent
                .active_id()
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string());
            if !name.is_empty() {
                let valid = {
                    let s = self.session.borrow();
                    s.as_ref()
                        .map(|s| {
                            let mut data = s.data.clone();
                            if let Some(f) = data.folders.iter_mut().find(|f| f.id == id) {
                                f.name = name.clone();
                                f.icon = appearance.icon.borrow().clone();
                                f.color = appearance.color.borrow().clone();
                                f.parent_id = parent.clone();
                            } else {
                                data.folders.push(Folder {
                                    id: id.clone(),
                                    name: name.clone(),
                                    icon: appearance.icon.borrow().clone(),
                                    color: appearance.color.borrow().clone(),
                                    parent_id: parent.clone(),
                                });
                            }
                            backup::validate(&data).map(|_| data)
                        })
                        .unwrap()
                };
                match valid {
                    Ok(data) => {
                        self.change(|d| *d = data);
                        self.show_vault();
                    }
                    Err(e) => self.message(&e.to_string()),
                }
            }
        }
        d.hide();
        d.close();
    }
    fn generator(self: &Rc<Self>, target: Option<gtk::Entry>) {
        let d = self.dialog(self.tr("Generador de contraseñas", "Password generator"));
        let area = d.content_area();
        let length = gtk::SpinButton::with_range(4.0, 256.0, 1.0);
        length.set_value(20.0);
        pack(&area, &label(self.tr("Longitud", "Length")));
        pack(&area, &length);
        let checks: Vec<_> = [
            ("Minúsculas", "Lowercase", true),
            ("Mayúsculas", "Uppercase", true),
            ("Números", "Digits", true),
            ("Símbolos", "Symbols", true),
            (
                "Evitar caracteres ambiguos",
                "Avoid ambiguous characters",
                false,
            ),
        ]
        .iter()
        .map(|(es, en, active)| {
            let b = gtk::CheckButton::with_label(self.tr(es, en));
            b.set_active(*active);
            pack(&area, &b);
            b
        })
        .collect();
        let result = field(
            &area,
            self.tr("Contraseña generada", "Generated password"),
            "",
            true,
        );
        let reveal = gtk::CheckButton::with_label(self.tr("Mostrar", "Show"));
        let r = result.clone();
        reveal.connect_toggled(move |b| r.set_visibility(b.is_active()));
        pack(&area, &reveal);
        let generate = {
            let l = length.clone();
            let c = checks.clone();
            let r = result.clone();
            move || {
                let options = generator::GenOptions {
                    length: l.value_as_int() as usize,
                    lowercase: c[0].is_active(),
                    uppercase: c[1].is_active(),
                    digits: c[2].is_active(),
                    symbols: c[3].is_active(),
                    avoid_ambiguous: c[4].is_active(),
                };
                let password = Zeroizing::new(generator::generate(&options));
                r.set_text(&password);
            }
        };
        generate();
        button(&area, self.tr("Generar otra", "Generate another"), generate);
        let app = self.clone();
        let r = result.clone();
        button(&area, self.tr("Copiar", "Copy"), move || {
            app.copy(r.text().to_string())
        });
        d.show_all();
        if d.run() == gtk::ResponseType::Ok {
            if let Some(target) = target {
                if self.session.borrow().is_some() {
                    target.set_text(result.text().as_str());
                }
            }
        }
        wipe(&d.clone().upcast());
        d.hide();
        d.close();
    }
    fn choose(&self, save: bool, images: bool) -> Option<PathBuf> {
        let d = gtk::FileChooserDialog::new(
            Some(self.tr(
                if images {
                    "Seleccionar fondo"
                } else if save {
                    "Guardar copia cifrada"
                } else {
                    "Importar copia cifrada"
                },
                if images {
                    "Choose wallpaper"
                } else if save {
                    "Save encrypted backup"
                } else {
                    "Import encrypted backup"
                },
            )),
            Some(&self.window),
            if save {
                gtk::FileChooserAction::Save
            } else {
                gtk::FileChooserAction::Open
            },
        );
        d.add_buttons(&[
            (self.tr("Cancelar", "Cancel"), gtk::ResponseType::Cancel),
            (self.tr("Seleccionar", "Select"), gtk::ResponseType::Ok),
        ]);
        let filter = gtk::FileFilter::new();
        if images {
            filter.set_name(Some("PNG · JPEG · GIF · WebP · BMP"));
            for mime in [
                "image/png",
                "image/jpeg",
                "image/gif",
                "image/webp",
                "image/bmp",
            ] {
                filter.add_mime_type(mime);
            }
        } else {
            filter.set_name(Some("SpiritByte (*.spiritbyte)"));
            filter.add_pattern("*.spiritbyte");
        }
        d.add_filter(filter);
        if save {
            d.set_current_name("backup.spiritbyte");
            d.set_do_overwrite_confirmation(true);
        }
        d.show_all();
        let result = if d.run() == gtk::ResponseType::Ok {
            d.filename()
        } else {
            None
        };
        d.hide();
        d.close();
        result
    }
    fn backup_password(&self, export: bool) -> Option<(Zeroizing<String>, u32)> {
        let d = self.dialog(self.tr(
            "Contraseña de la copia cifrada",
            "Encrypted backup password",
        ));
        let area = d.content_area();
        let pw = field(
            &area,
            self.tr(
                "Contraseña de la copia (mínimo 12 caracteres)",
                "Backup password (at least 12 characters)",
            ),
            "",
            true,
        );
        let confirm = if export {
            Some(field(
                &area,
                self.tr("Repite la contraseña", "Confirm password"),
                "",
                true,
            ))
        } else {
            None
        };
        let scope = gtk::ComboBoxText::new();
        if export {
            for (i, es, en) in [
                ("0", "Toda la bóveda", "Entire vault"),
                ("1", "Carpeta / filtro actual", "Current folder / filter"),
                (
                    "2",
                    "Entradas seleccionadas (Ctrl / Shift)",
                    "Selected entries (Ctrl / Shift)",
                ),
            ] {
                scope.append(Some(i), self.tr(es, en));
            }
            scope.set_active(Some(0));
            pack(&area, &scope);
        }
        d.show_all();
        let response = d.run();
        let password = Zeroizing::new(pw.text().to_string());
        let matched = confirm
            .as_ref()
            .map_or(true, |c| c.text().as_str() == password.as_str());
        let choice = scope.active().unwrap_or(0);
        wipe(&d.clone().upcast());
        d.hide();
        d.close();
        if response != gtk::ResponseType::Ok {
            return None;
        }
        if export && (password.chars().count() < 12 || !matched) {
            self.message(self.tr(
                "Contraseña demasiado corta o confirmación distinta.",
                "Password too short or confirmation differs.",
            ));
            return None;
        }
        Some((password, choice))
    }
    fn export(self: &Rc<Self>) {
        let Some((password, scope)) = self.backup_password(true) else {
            return;
        };
        let Some(path) = self.choose(true, false) else {
            return;
        };
        let snapshot = {
            let s = self.session.borrow();
            let Some(s) = s.as_ref() else {
                return;
            };
            let selection = if scope == 0 {
                None
            } else {
                let ids = if scope == 2 {
                    self.selected_ids()
                } else {
                    let mut ids = vec![];
                    if let Some(list) = self.list.borrow().as_ref() {
                        if let Some(i) = list.iter_first() {
                            loop {
                                if let Ok(id) = list.value(&i, 0).get::<String>() {
                                    ids.push(id);
                                }
                                if !list.iter_next(&i) {
                                    break;
                                }
                            }
                        }
                    }
                    ids
                };
                Some(backup::ExportSelection {
                    entry_ids: ids,
                    folder_ids: if scope == 1
                        && self.query.borrow().is_empty()
                        && s.data.folders.iter().any(|f| f.id == *self.filter.borrow())
                    {
                        vec![self.filter.borrow().clone()]
                    } else {
                        vec![]
                    },
                })
            };
            backup::select(&s.data, selection.as_ref())
        };
        match snapshot {
            Err(e) => self.message(&e.to_string()),
            Ok(data) => self.work(
                move || {
                    let blob = backup::export(&data, &password).map_err(|e| e.to_string())?;
                    backup::save_new(&path, &blob).map_err(|e| e.to_string())
                },
                |app, ()| app.message(app.tr("Copia cifrada guardada.", "Encrypted backup saved.")),
            ),
        }
    }
    fn import(self: &Rc<Self>) {
        let Some(path) = self.choose(false, false) else {
            return;
        };
        let Some((password, _)) = self.backup_password(false) else {
            return;
        };
        if self.session.borrow().is_none() {
            return;
        }
        self.work(
            move || {
                use std::io::Read;
                let f = std::fs::File::open(path).map_err(|e| e.to_string())?;
                let mut bytes = Vec::new();
                f.take(backup::MAX_BACKUP_BYTES as u64 + 1)
                    .read_to_end(&mut bytes)
                    .map_err(|e| e.to_string())?;
                if bytes.len() > backup::MAX_BACKUP_BYTES {
                    return Err("Backup exceeds 32 MiB".into());
                }
                let text = String::from_utf8(bytes).map_err(|e| e.to_string())?;
                backup::decrypt(&text, &password).map_err(|e| e.to_string())
            },
            |app, imported| {
                if app.confirm(app.tr(
                    "¿Añadir las entradas de esta copia a tu bóveda? Las existentes se conservan.",
                    "Add the backup entries to your vault? Existing entries are kept.",
                )) {
                    app.change(|data| *data = backup::merge(data, imported));
                    app.show_vault();
                }
            },
        );
    }
    fn change_password(self: &Rc<Self>) {
        let d = self.dialog(self.tr("Cambiar contraseña maestra", "Change master password"));
        let area = d.content_area();
        let pw = field(
            &area,
            self.tr(
                "Nueva contraseña (mínimo 12 caracteres)",
                "New password (at least 12 characters)",
            ),
            "",
            true,
        );
        let repeat = field(
            &area,
            self.tr("Repite la contraseña", "Confirm password"),
            "",
            true,
        );
        d.show_all();
        let response = d.run();
        let password = Zeroizing::new(pw.text().to_string());
        let matched = repeat.text().as_str() == password.as_str();
        wipe(&d.clone().upcast());
        d.hide();
        d.close();
        if response != gtk::ResponseType::Ok {
            return;
        }
        if !matched || password.chars().count() < 12 {
            self.message(self.tr(
                "Contraseña demasiado corta o confirmación distinta.",
                "Password too short or confirmation differs.",
            ));
            return;
        }
        let dek = self.session.borrow().as_ref().map(|s| s.dek.clone());
        if let Some(dek) = dek {
            let paths = self.paths.clone();
            self.work(
                move || vault::change_password(&paths, &dek, &password).map_err(|e| e.to_string()),
                |app, ()| {
                    app.message(app.tr(
                        "Contraseña maestra actualizada.",
                        "Master password updated.",
                    ))
                },
            );
        }
    }
    fn settings_dialog(self: &Rc<Self>) {
        let d = self.dialog(self.tr("Ajustes de SpiritByte", "SpiritByte settings"));
        d.set_default_size(580, 680);
        let tabs = gtk::Notebook::new();
        d.content_area().pack_start(&tabs, true, true, 0);
        let security = box_v(12);
        let backups = box_v(12);
        let area = box_v(8);
        let scroll = gtk::ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
        scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
        scroll.set_min_content_height(480);
        scroll.add(&area);
        tabs.append_page(&scroll, Some(&label(self.tr("Apariencia", "Appearance"))));
        tabs.append_page(&security, Some(&label(self.tr("Seguridad", "Security"))));
        tabs.append_page(&backups, Some(&label(self.tr("Bóveda", "Vault"))));
        for page in [&area, &security, &backups] {
            page.set_margin_top(12);
            page.set_margin_bottom(12);
            page.set_margin_start(12);
            page.set_margin_end(12);
        }
        let app = self.clone();
        let change = button(
            &security,
            self.tr("Cambiar contraseña maestra", "Change master password"),
            move || app.change_password(),
        );
        change.set_sensitive(self.session.borrow().is_some());
        pack(
            &backups,
            &label(self.tr("Copias cifradas de tu bóveda", "Encrypted vault backups")),
        );
        let app = self.clone();
        let export = button(
            &backups,
            self.tr("Exportar bóveda…", "Export vault…"),
            move || app.export(),
        );
        let app = self.clone();
        let import = button(
            &backups,
            self.tr("Importar bóveda…", "Import vault…"),
            move || app.import(),
        );
        export.set_sensitive(self.session.borrow().is_some());
        import.set_sensitive(self.session.borrow().is_some());
        pack(
            &backups,
            &label(self.tr(
                "Desbloquea la bóveda para importar o exportar.",
                "Unlock the vault to import or export.",
            )),
        );
        let mut draft = self.settings.borrow().clone();
        let presets = model::presets();
        pack(&area, &label(self.tr("Paleta", "Palette")));
        let palette = gtk::ComboBoxText::new();
        for p in presets.as_array().unwrap() {
            palette.append(p["id"].as_str(), p["name"].as_str().unwrap());
        }
        palette.append(Some("custom"), self.tr("Personalizada", "Custom"));
        palette.set_active_id(draft["paletteId"].as_str());
        pack(&area, &palette);
        let custom = gtk::Expander::new(Some(self.tr("Editar colores", "Edit colors")));
        let colors = box_v(4);
        let color_inputs: Vec<_> = [
            "bg", "surface", "surface2", "border", "text", "textDim", "primary", "accent",
            "danger", "success", "warning",
        ]
        .iter()
        .map(|key| {
            (
                *key,
                field(
                    &colors,
                    key,
                    draft["palette"][*key].as_str().unwrap_or("13 12 10"),
                    false,
                ),
            )
        })
        .collect();
        custom.add(&colors);
        pack(&area, &custom);
        let fields = color_inputs.clone();
        let p = presets.clone();
        palette.connect_changed(move |combo| {
            if let Some(id) = combo.active_id() {
                if let Some(p) = p
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|p| p["id"] == id.as_str())
                {
                    for (key, entry) in &fields {
                        entry.set_text(p["palette"][*key].as_str().unwrap_or("13 12 10"));
                    }
                }
            }
        });
        pack(&area, &label(self.tr("Fuente", "Font")));
        let font = gtk::ComboBoxText::new();
        for (id, name) in [
            ("mixed", "Retro"),
            ("pixel", "Pixel"),
            ("mono", "Monospace"),
            ("geist-square", "Geist Square"),
            ("geist-grid", "Geist Grid"),
            ("geist-circle", "Geist Circle"),
            ("geist-triangle", "Geist Triangle"),
            ("geist-line", "Geist Line"),
        ] {
            font.append(Some(id), name);
        }
        font.set_active_id(draft["font"].as_str());
        pack(&area, &font);
        let size = gtk::SpinButton::with_range(10.0, 24.0, 1.0);
        size.set_value(draft["fontSize"].as_f64().unwrap_or(18.0));
        pack(&area, &label(self.tr("Tamaño de fuente", "Font size")));
        pack(&area, &size);
        let language = gtk::ComboBoxText::new();
        language.append(Some("es"), "Español");
        language.append(Some("en"), "English");
        language.set_active_id(draft["language"].as_str());
        pack(&area, &language);
        let checks: Vec<_> = [
            ("scanlines", "Líneas CRT", "CRT scanlines"),
            (
                "showSplash",
                "Intro del zorro al iniciar",
                "Fox intro at startup",
            ),
            (
                "showUnfiled",
                "Mostrar «Sin carpeta» en la barra lateral",
                "Show “No folder” in the sidebar",
            ),
            ("glow", "Brillo de texto", "Text glow"),
            ("flicker", "Parpadeo CRT", "CRT flicker"),
            (
                "lowPowerMode",
                "Ahorro de energía: pausa animaciones sin foco",
                "Power saving: pause animations when unfocused",
            ),
        ]
        .iter()
        .map(|(key, es, en)| {
            let c = gtk::CheckButton::with_label(self.tr(es, en));
            c.set_widget_name(key);
            c.set_active(draft[*key].as_bool().unwrap_or(false));
            pack(&area, &c);
            (*key, c)
        })
        .collect();
        let opacity = gtk::SpinButton::with_range(0.0, 1.0, 0.05);
        opacity.set_digits(2);
        opacity.set_value(draft["panelOpacity"].as_f64().unwrap_or(1.0));
        pack(
            &area,
            &label(self.tr("Opacidad de paneles", "Panel opacity")),
        );
        pack(&area, &opacity);
        let lock = gtk::SpinButton::with_range(0.0, 120.0, 1.0);
        lock.set_value(draft["autoLockMinutes"].as_f64().unwrap_or(5.0));
        pack(
            &security,
            &label(self.tr(
                "Bloqueo por inactividad (minutos; 0 desactiva)",
                "Idle lock (minutes; 0 disables)",
            )),
        );
        pack(&security, &lock);
        let clipboard = gtk::SpinButton::with_range(1.0, 3600.0, 1.0);
        clipboard.set_value(draft["clipboardClearSeconds"].as_f64().unwrap_or(20.0));
        pack(
            &security,
            &label(self.tr(
                "Limpiar portapapeles (segundos)",
                "Clear clipboard (seconds)",
            )),
        );
        pack(&security, &clipboard);
        let bgtype = gtk::ComboBoxText::new();
        for (id, es, en) in [
            ("solid", "Color sólido", "Solid color"),
            ("gradient", "Degradado", "Gradient"),
            ("image", "Foto / GIF", "Photo / GIF"),
        ] {
            bgtype.append(Some(id), self.tr(es, en));
        }
        bgtype.set_active_id(draft["background"]["type"].as_str());
        pack(&area, &label(self.tr("Fondo", "Background")));
        pack(&area, &bgtype);
        let value = field(
            &area,
            self.tr(
                "Ruta de imagen o colores: #0d0c0a|#302018",
                "Image path or colors: #0d0c0a|#302018",
            ),
            draft["background"]["value"].as_str().unwrap_or(""),
            false,
        );
        let app = self.clone();
        let v = value.clone();
        let t = bgtype.clone();
        button(
            &area,
            self.tr("Seleccionar foto o GIF…", "Choose photo or GIF…"),
            move || {
                if let Some(path) = app.choose(false, true) {
                    v.set_text(&path.to_string_lossy());
                    t.set_active_id(Some("image"));
                }
            },
        );
        d.show_all();
        let response = d.run();
        if response == gtk::ResponseType::Ok {
            if let Some(id) = palette.active_id() {
                if let Some(p) = presets
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|p| p["id"] == id.as_str())
                {
                    if draft["paletteId"] != id.as_str() {
                        draft["palette"] = p["palette"].clone();
                        draft["paletteId"] = json!(id.as_str());
                    }
                }
            }
            for (key, entry) in color_inputs {
                let rgb = background::rgb(entry.text().as_str());
                draft["palette"][key] = json!(format!(
                    "{} {} {}",
                    (rgb[0] * 255.0).round() as u32,
                    (rgb[1] * 255.0).round() as u32,
                    (rgb[2] * 255.0).round() as u32
                ));
            }
            if let Some(id) = palette.active_id() {
                draft["paletteId"] = json!(id.as_str());
            }
            draft["font"] = json!(font
                .active_id()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "mixed".into()));
            draft["fontSize"] = json!(size.value_as_int());
            draft["language"] = json!(language
                .active_id()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "es".into()));
            for (key, c) in checks {
                draft[key] = json!(c.is_active());
            }
            draft["panelOpacity"] = json!(opacity.value());
            draft["autoLockMinutes"] = json!(lock.value_as_int());
            draft["clipboardClearSeconds"] = json!(clipboard.value_as_int());
            let kind = bgtype
                .active_id()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "solid".into());
            let selected = value.text().to_string();
            let mut imported = None;
            let result = (|| -> Result<(), String> {
                let selected = if kind == "image" {
                    let source = Path::new(&selected);
                    background::load(source)?;
                    let base = self.paths.settings.parent().unwrap();
                    if source.parent() == Some(base)
                        && source
                            .file_name()
                            .is_some_and(|n| n.to_string_lossy().starts_with("wallpaper."))
                    {
                        selected.clone()
                    } else {
                        let path =
                            wallpaper::import(source, base, &uuid::Uuid::new_v4().to_string())?;
                        let result = path.to_string_lossy().into_owned();
                        imported = Some(path);
                        background::load(imported.as_ref().unwrap())?;
                        result
                    }
                } else {
                    selected.clone()
                };
                draft["background"] = json!({"type":kind,"value":selected});
                model::save_settings(&self.paths, &draft)?;
                Ok(())
            })();
            match result {
                Ok(()) => {
                    *self.settings.borrow_mut() = draft;
                    if let Err(e) = self.apply_settings() {
                        self.message(&e);
                    }
                    if let Some(path) = imported.as_ref() {
                        wallpaper::cleanup(self.paths.settings.parent().unwrap(), &[path]);
                    }
                    if self.session.borrow().is_some() {
                        self.show_vault();
                    } else {
                        self.show_locked();
                    }
                }
                Err(e) => {
                    if let Some(path) = imported {
                        let _ = std::fs::remove_file(path);
                    }
                    self.message(&e);
                }
            }
        }
        d.hide();
        d.close();
    }
    fn apply_settings(self: &Rc<Self>) -> Result<(), String> {
        let s = self.settings.borrow();
        let color = |key: &str| {
            let c = background::rgb(s["palette"][key].as_str().unwrap_or("13 12 10"));
            format!(
                "rgb({},{},{})",
                (c[0] * 255.0).round() as u32,
                (c[1] * 255.0).round() as u32,
                (c[2] * 255.0).round() as u32
            )
        };
        let c = background::rgb(s["palette"]["surface"].as_str().unwrap_or("26 23 18"));
        let opacity = s["panelOpacity"].as_f64().unwrap_or(1.0).clamp(0.0, 1.0);
        let panel = format!(
            "rgba({},{},{},{})",
            (c[0] * 255.0).round() as u32,
            (c[1] * 255.0).round() as u32,
            (c[2] * 255.0).round() as u32,
            opacity
        );
        let font = match s["font"].as_str().unwrap_or("mixed") {
            "geist-square" => "Geist Pixel Square",
            "geist-grid" => "Geist Pixel Grid",
            "geist-circle" => "Geist Pixel Circle",
            "geist-triangle" => "Geist Pixel Triangle",
            "geist-line" => "Geist Pixel Line",
            "pixel" => "Press Start 2P",
            "mono" => "VT323",
            _ => "VT323",
        };
        let size = s["fontSize"].as_u64().unwrap_or(18).clamp(10, 24);
        let glow = if s["glow"].as_bool().unwrap_or(true) {
            format!("0 0 3px {}", color("primary"))
        } else {
            "none".into()
        };
        let display_font = if s["font"] == "mixed" || s["font"] == "pixel" {
            "Press Start 2P"
        } else {
            font
        };
        let body_size = match s["font"].as_str().unwrap_or("mixed") {
            "pixel" => (size as f64 * 0.67).round() as u64,
            mode if mode.starts_with("geist-") => (size as f64 * 0.78).round() as u64,
            _ => size,
        };
        let css = format!(
            r#"
            * {{ font-family: "{font}", monospace; font-size: {body_size}px; color: {text}; }}
            window {{ background-color: {bg}; }}
            .intro-screen {{ background-color: {bg}; }}
            .boot-log {{ font-family: monospace; font-size: 13px; color: {primary}; }}
            dialog, messagedialog {{ background-color: {surface}; }}
            .brand {{ font-family: "{display_font}", monospace; font-size: 12px; color: {primary}; text-shadow: {glow}; }}
            .title {{ font-family: "{display_font}", monospace; font-size: 15px; color: {primary}; text-shadow: {glow}; }}
            .panel {{ background-color: {panel}; border: 2px solid {border}; border-radius: 0; padding: 24px; }}
            .unlock-card {{ margin: 24px; }}
            .sidebar, .entry-pane {{ background-color: {panel}; }}
            .pane-header {{ padding: 8px; min-height: 30px; border-bottom: 2px solid {border}; }}
            .pane-footer {{ padding: 8px; border-top: 2px solid {border}; }}
            paned > separator {{ min-width: 4px; background-image: none; background-color: {border}; }}
            paned > separator:hover {{ background-color: {primary}; }}
            .navigation {{ background-color: transparent; padding: 8px; }}
            .navigation row {{ padding: 8px; border: 0; border-radius: 0; }}
            .navigation row:hover {{ background-color: {surface2}; }}
            .navigation row:selected {{ background-color: {selected}; }}
            .navigation row:selected label, .navigation row:selected drawingarea {{ color: {primary}; }}
            .navigation row.folder-section {{ padding: 0; }}
            .folder-heading {{ padding-top: 12px; padding-bottom: 4px; }}
            .field-label {{ font-family: "{display_font}", monospace; font-size: 10px; color: {dim}; }}
            .count, .muted, .list-status, .empty-state * {{ color: {dim}; }}
            .count {{ font-size: 14px; }}
            .list-status {{ font-size: 14px; padding: 8px 12px; border-top: 1px solid {border}; }}
            .detail {{ padding: 24px; }}
            .detail-field {{ padding: 12px; background-color: {panel}; border: 2px solid {border}; }}
            entry, textview, textview text, spinbutton {{ background-color: {bg}; color: {text}; caret-color: {primary}; }}
            entry, spinbutton {{ min-height: 28px; padding: 4px 10px; border: 2px solid {border}; border-radius: 0; box-shadow: none; }}
            textview {{ padding: 8px; border: 2px solid {border}; }}
            entry:focus, textview:focus, spinbutton:focus {{ border-color: {primary}; }}
            .secret-password {{ padding: 0; border: 0; min-height: 24px; background-color: transparent; color: {primary}; }}
            button {{ font-family: "{display_font}", monospace; font-size: {button_size}px; min-height: 28px; background-image: none; background-color: {surface2}; padding: 6px 10px; border: 2px solid {border}; border-radius: 0; box-shadow: none; text-shadow: none; }}
            button label {{ font-family: inherit; font-size: inherit; }}
            button:hover {{ background-color: {surface2}; border-color: {primary}; }}
            button:hover label, button:hover drawingarea {{ color: {primary}; }}
            button:active {{ background-color: {selected}; }}
            button:focus {{ outline-color: {primary}; outline-style: solid; outline-width: 1px; outline-offset: -4px; }}
            button:disabled {{ opacity: 0.4; }}
            button:checked {{ background-color: {selected}; border-color: {primary}; }}
            .language-button {{ min-height: 20px; padding: 4px 6px; }}
            .icon-button {{ min-width: 20px; min-height: 20px; padding: 4px; background-color: transparent; border-color: transparent; }}
            .primary, .primary * {{ color: {primary}; }}
            button.primary {{ background-color: {selected}; border-color: {primary}; }}
            button.danger:hover, button.danger:hover * {{ color: {danger}; border-color: {danger}; }}
            combobox button {{ background-color: {bg}; border-color: {border}; }}
            treeview, treeview.view {{ background-color: {panel}; color: {text}; -GtkTreeView-horizontal-separator: 0; -GtkTreeView-vertical-separator: 0; }}
            treeview:selected {{ background-color: {selected}; color: {primary}; }}
            treeview:hover {{ background-color: {surface2}; }}
            treeview:selected:hover {{ background-color: {selected}; }}
            selection {{ background-color: {selected}; color: {primary}; }}
            checkbutton check {{ min-width: 14px; min-height: 14px; border: 2px solid {border}; border-radius: 0; background-image: none; background-color: {bg}; }}
            checkbutton:checked check {{ color: {primary}; border-color: {primary}; }}
            scale trough {{ background-color: {border}; }}
            scale highlight {{ background-color: {primary}; }}
            scrollbar {{ background-color: {surface}; }}
            scrollbar slider {{ min-width: 6px; min-height: 6px; border: 0; border-radius: 0; background-color: {border}; }}
            scrollbar slider:hover {{ background-color: {primary}; }}
            notebook, notebook > stack {{ background-color: {surface}; }}
            viewport {{ background-color: transparent; }}
            notebook > header {{ background-color: {bg}; border-bottom: 2px solid {border}; }}
            notebook tab {{ padding: 10px 12px; border-radius: 0; }}
            notebook tab:checked {{ background-color: {surface2}; border-bottom: 2px solid {primary}; box-shadow: none; }}
        "#,
            text = color("text"),
            bg = color("bg"),
            surface = color("surface"),
            surface2 = color("surface2"),
            border = color("border"),
            primary = color("primary"),
            dim = color("textDim"),
            danger = color("danger"),
            selected = {
                let c = background::rgb(s["palette"]["primary"].as_str().unwrap_or("255 176 0"));
                format!(
                    "rgba({},{},{},0.12)",
                    (c[0] * 255.0).round() as u32,
                    (c[1] * 255.0).round() as u32,
                    (c[2] * 255.0).round() as u32
                )
            },
            button_size = if display_font == "Press Start 2P" {
                (size as f64 * 0.61).round() as u64
            } else {
                body_size
            }
        );
        self.css
            .load_from_data(css.as_bytes())
            .map_err(|e| e.to_string())?;
        let bg = &s["background"];
        self.background.gradient.set(bg["type"] == "gradient");
        let colors = if bg["type"] == "gradient" {
            let v = bg["value"].as_str().unwrap_or("#0d0c0a|#302018");
            let (a, b) = v.split_once('|').unwrap_or((v, v));
            (background::rgb(a), background::rgb(b))
        } else {
            let c = background::rgb(s["palette"]["bg"].as_str().unwrap_or("13 12 10"));
            (c, c)
        };
        self.background.colors.set(colors);
        let result = if bg["type"] == "image" {
            background::load(Path::new(bg["value"].as_str().unwrap_or(""))).map(Some)
        } else {
            Ok(None)
        };
        drop(s);
        match result {
            Ok(image) => self.background.set_image(image),
            Err(e) => {
                self.background.set_image(None);
                self.animate();
                return Err(e);
            }
        }
        self.effects.queue_draw();
        self.animate();
        Ok(())
    }
    fn animate(self: &Rc<Self>) {
        let s = self.settings.borrow();
        let run = self.window.is_mapped()
            && self.intro_timer.borrow().is_none()
            && (!s["lowPowerMode"].as_bool().unwrap_or(true) || self.window.is_active());
        self.background.resume(run);
        if let Some(id) = self.effect_timer.borrow_mut().take() {
            id.remove();
        }
        // GIFs invalidate only changed rectangles. Repaint the whole window
        // only when the global CRT pulse changes, independently of GIF frames.
        if run && s["flicker"].as_bool().unwrap_or(false) {
            let weak = Rc::downgrade(self);
            let mut previous = (now() / 500) % 4 == 0;
            let id = glib::timeout_add_local(Duration::from_millis(500), move || {
                if let Some(app) = weak.upgrade() {
                    let pulse = (now() / 500) % 4 == 0;
                    if pulse != previous {
                        app.effects.queue_draw();
                        previous = pulse;
                    }
                    glib::ControlFlow::Continue
                } else {
                    glib::ControlFlow::Break
                }
            });
            *self.effect_timer.borrow_mut() = Some(id);
        }
    }
}

/// Clear GTK's editable buffers before disposing forms or locking the vault.
fn wipe(widget: &gtk::Widget) {
    if let Some(label) = widget.downcast_ref::<gtk::Label>() {
        if label.style_context().has_class("secret-value") {
            label.set_text("");
        }
    }
    if let Some(entry) = widget.downcast_ref::<gtk::Entry>() {
        entry.set_text("");
    }
    if let Some(text) = widget.downcast_ref::<gtk::TextView>() {
        if let Some(buffer) = text.buffer() {
            buffer.set_text("");
        }
    }
    if let Some(container) = widget.downcast_ref::<gtk::Container>() {
        for child in container.children() {
            wipe(&child);
        }
    }
}

//! Opt-in release UI exercise, never included in the installed production binary.
use super::*;
fn dialog() -> gtk::Dialog {
    gtk::Window::list_toplevels()
        .into_iter()
        .find_map(|w| w.downcast::<gtk::Dialog>().ok().filter(|d| d.is_visible()))
        .expect("visible native dialog")
}
fn entries(widget: &gtk::Widget) -> Vec<gtk::Entry> {
    let mut out = vec![];
    if let Some(entry) = widget.downcast_ref::<gtk::Entry>() {
        out.push(entry.clone());
    }
    if let Some(container) = widget.downcast_ref::<gtk::Container>() {
        for child in container.children() {
            out.extend(entries(&child));
        }
    }
    out
}
fn count(app: &App) -> i32 {
    app.list.borrow().as_ref().unwrap().iter_n_children(None)
}

/// Only compiled with ui-check. Publishes geometry and state from a disposable
/// fixture so the external checker can send compositor pointer/keyboard events.
pub fn pointer_fixture(app: Rc<App>, report: PathBuf) {
    let base = app.paths.settings.parent().unwrap();
    assert!(
        base.starts_with("/tmp") && base.to_string_lossy().contains("/spiritbyte-native-check-")
    );
    assert!(report.starts_with(base));
    if std::env::var_os("SPIRITBYTE_TEST_LEGACY_OVERLAY").is_some() {
        let legacy = gtk::DrawingArea::new();
        legacy.set_hexpand(true);
        legacy.set_vexpand(true);
        app.effects.add_overlay(&legacy);
        app.effects.set_overlay_pass_through(&legacy, true);
        legacy.show();
    }
    if std::env::var_os("SPIRITBYTE_AUTH_ONBOARDING").is_some() {
        app.window.set_title("SpiritByte — pointer fixture");
        watch_pointer(app, report);
        return;
    }
    let paths = app.paths.clone();
    app.work(
        move || {
            let (mut s, _) = vault::create_vault(&paths, "pointer fixture master password")
                .map_err(|e| e.to_string())?;
            s.data.folders = vec![
                Folder {
                    id: "work".into(),
                    name: "Work".into(),
                    parent_id: None,
                    icon: None,
                    color: None,
                },
                Folder {
                    id: "child".into(),
                    name: "Nested".into(),
                    parent_id: Some("work".into()),
                    icon: None,
                    color: None,
                },
            ];
            for i in 0..24 {
                s.data.entries.push(Entry {
                    id: format!("pointer-{i}"),
                    title: format!("Example {i:02}"),
                    username: format!("user{i}@example.org"),
                    password: format!("fixture password {i}"),
                    url: "https://example.org".into(),
                    notes: "Synthetic fixture. No real credentials.".into(),
                    folder_id: Some(if i % 2 == 0 { "work" } else { "child" }.into()),
                    icon_id: None,
                    favorite: i % 4 == 0,
                    created_at: 1,
                    updated_at: 1,
                });
            }
            vault::write_data(&paths, &s).map_err(|e| e.to_string())?;
            Ok(s)
        },
        move |app, session| {
            *app.session.borrow_mut() = Some(session);
            app.settings.borrow_mut()["autoLockMinutes"] = json!(0);
            app.show_vault();
            app.window.set_title("SpiritByte — pointer fixture");
            if std::env::var_os("SPIRITBYTE_STRESS_CHECK").is_some() {
                stress(app.clone(), report);
            } else {
                watch_pointer(app.clone(), report);
            }
        },
    );
}

fn stress(app: Rc<App>, report: PathBuf) {
    fn memory() -> Value {
        let text = std::fs::read_to_string("/proc/self/smaps_rollup").unwrap();
        let mut out = serde_json::Map::new();
        for line in text.lines() {
            if let Some((key, value)) = line.split_once(':') {
                if matches!(key, "Rss" | "Pss" | "Private_Dirty") {
                    out.insert(
                        key.into(),
                        json!(value
                            .split_whitespace()
                            .next()
                            .unwrap()
                            .parse::<u64>()
                            .unwrap()),
                    );
                }
            }
        }
        Value::Object(out)
    }
    let mut samples = vec![json!({"round":0,"memory_kib":memory()})];
    let mut discarded = vec![];
    let mut round = 0;
    glib::timeout_add_local(Duration::from_millis(40), move || {
        app.show_locked();
        for e in entries(app.content.upcast_ref()) {
            discarded.push(e.downgrade());
        }
        app.show_vault();
        round += 1;
        if round % 50 == 0 {
            samples.push(json!({"round":round,"memory_kib":memory(),"retained_discarded_entries":discarded.iter().filter(|w|w.upgrade().is_some()).count()}));
        }
        if round == 200 {
            std::fs::write(
                &report,
                serde_json::to_vec_pretty(&json!({"samples":samples,"real_vault_accessed":false}))
                    .unwrap(),
            )
            .unwrap();
            app.window.application().unwrap().quit();
            glib::ControlFlow::Break
        } else {
            glib::ControlFlow::Continue
        }
    });
}

fn watch_pointer(app: Rc<App>, report: PathBuf) {
    glib::timeout_add_local(Duration::from_millis(100), move || {
        fn widgets(widget: &gtk::Widget, root: &gtk::Window, out: &mut Vec<Value>) {
            if !widget.is_visible() {
                return;
            }
            if widget.is::<gtk::Button>()
                || widget.is::<gtk::Entry>()
                || widget.is::<gtk::ListBoxRow>()
                || widget.is::<gtk::TreeView>()
                || widget.is::<gtk::TextView>()
                || widget.is::<gtk::ComboBoxText>()
            {
                if let Some((x, y)) = widget.translate_coordinates(root, 0, 0) {
                    let text = widget
                        .downcast_ref::<gtk::Entry>()
                        .filter(|e| gtk::prelude::EntryExt::is_visible(*e))
                        .map(|e| e.text().to_string());
                    let active = widget
                        .downcast_ref::<gtk::ToggleButton>()
                        .map(|b| b.is_active());
                    out.push(json!({"type":widget.type_().name(),"name":widget.widget_name().to_string(),"tooltip":widget.tooltip_text().map(|s|s.to_string()),"text":text,"active":active,"focus":widget.has_focus(),"x":x,"y":y,"w":widget.allocated_width(),"h":widget.allocated_height()}));
                }
            }
            if let Some(c) = widget.downcast_ref::<gtk::Container>() {
                for child in c.children() {
                    widgets(&child, root, out);
                }
            }
        }
        let windows: Vec<_> = gtk::Window::list_toplevels()
            .into_iter()
            .filter_map(|w| w.downcast::<gtk::Window>().ok())
            .filter(|w| w.is_visible())
            .map(|w| {
                let mut nodes = vec![];
                widgets(w.upcast_ref(), &w, &mut nodes);
                json!({"title":w.title().map(|s|s.to_string()),"widgets":nodes})
            })
            .collect();
        let folders: Vec<_> = app
            .session
            .borrow()
            .as_ref()
            .map(|s| {
                s.data
                    .folders
                    .iter()
                    .map(|f| json!({"id":f.id,"name":f.name,"icon":f.icon,"color":f.color}))
                    .collect()
            })
            .unwrap_or_default();
        let value = json!({"windows":windows,"query":*app.query.borrow(),"filter":*app.filter.borrow(),"selected":app.selected(),"visible_entries":app.list.borrow().as_ref().map(|l|l.iter_n_children(None)),"unlocked":app.session.borrow().is_some(),"language":app.settings.borrow()["language"],"last_external_url":*app.external_url.borrow(),"folders":folders,"intro_visible":app.intro_timer.borrow().is_some()});
        let temp = report.with_extension("tmp");
        std::fs::write(&temp, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
        std::fs::rename(temp, &report).unwrap();
        glib::ControlFlow::Continue
    });
}
pub fn run(app: Rc<App>, report: PathBuf) {
    let base = app.paths.settings.parent().unwrap().to_path_buf();
    assert!(
        base.starts_with("/tmp")
            && base.to_string_lossy().contains("/spiritbyte-native-check-")
            && report.starts_with(base.parent().unwrap().parent().unwrap()),
        "disposable XDG only"
    );
    app.window.set_title("SpiritByte — native UI check");
    eprintln!("native-check: starting");
    let paths = app.paths.clone();
    app.work(move||{
        let (mut session,_)=vault::create_vault(&paths,"native UI fixture password").map_err(|e|e.to_string())?;
        session.data.folders=vec![Folder{id:"work".into(),name:"Work".into(),icon:Some("folder".into()),color:Some("#ffb000".into()),parent_id:None},Folder{id:"child".into(),name:"Nested".into(),icon:None,color:None,parent_id:Some("work".into())}];
        for i in 0..1000 {session.data.entries.push(Entry{id:format!("fixture-{i}"),title:format!("Example {i:04}"),username:format!("user{i}"),password:format!("fixture secret {i}"),url:"https://example.org".into(),notes:"Fictitious UI record".into(),folder_id:Some(if i%2==0{"work"}else{"child"}.into()),icon_id:Some("key".into()),favorite:i%10==0,created_at:1,updated_at:1});}
        vault::write_data(&paths,&session).map_err(|e|e.to_string())?;drop(session);
        vault::unlock(&paths,"native UI fixture password").map_err(|e|e.to_string())
    },move|app,session|{
        eprintln!("native-check: unlocked fixture");*app.session.borrow_mut()=Some(session);app.show_vault();app.window.set_title("SpiritByte — native UI check");assert_eq!(count(app),1000);
        let start=Instant::now();app.query.replace("Example 0499".into());app.refresh();let search_ms=start.elapsed().as_secs_f64()*1000.0;assert_eq!(count(app),1);
        app.query.replace(String::new());app.filter.replace("work".into());app.refresh();assert_eq!(count(app),1000); // includes descendants
        app.filter.replace("child".into());app.refresh();assert_eq!(count(app),500);
        app.filter.replace("star".into());app.refresh();assert_eq!(count(app),100);
        app.filter.replace("all".into());app.refresh();
        glib::idle_add_local_once(||{
            let d=dialog();let e=entries(&d.clone().upcast());assert_eq!(e.len(),4);e[0].set_text("Native edited record");e[2].set_text("native edited secret");d.response(gtk::ResponseType::Ok);
        });
        app.edit(Some("fixture-499".into()));eprintln!("native-check: edited");
        {let s=app.session.borrow();let e=s.as_ref().unwrap().data.entries.iter().find(|e|e.id=="fixture-499").unwrap();assert_eq!(e.title,"Native edited record");assert_eq!(e.password,"native edited secret");assert_eq!(e.icon_id.as_deref(),Some("key"));}
        glib::idle_add_local_once(||{let d=dialog();let e=entries(&d.clone().upcast());e[0].set_text("New native folder");let mut pending=vec![d.clone().upcast::<gtk::Widget>()];while let Some(w)=pending.pop(){if w.widget_name()=="folder-icon-heart" || w.widget_name()=="folder-color-78c8ff" {w.downcast_ref::<gtk::ToggleButton>().unwrap().set_active(true);}if let Some(c)=w.downcast_ref::<gtk::Container>(){pending.extend(c.children());}}d.response(gtk::ResponseType::Ok);});
        app.manage_folders();eprintln!("native-check: folder saved");assert_eq!(app.session.borrow().as_ref().unwrap().data.folders.len(),3);
        app.settings.borrow_mut()["clipboardClearSeconds"]=json!(1);app.copy("UI clipboard fixture".into());let clipboard_lease=app.clipboard.borrow().as_ref().unwrap().clone();
        let app=app.clone();glib::timeout_add_seconds_local_once(3,move||{
            eprintln!("native-check: clipboard timer");assert!(app.clipboard.borrow().is_none());assert!(!clipboard_lease.get());eprintln!("native-check: clipboard ownership released");
            // Lock while the editor is open: its GTK buffers and live DEK disappear.
            let lock=app.clone();glib::timeout_add_local_once(Duration::from_millis(100),move||{lock.settings.borrow_mut()["autoLockMinutes"]=json!(1);lock.activity.set(Instant::now()-Duration::from_secs(61));lock.check_inactivity();eprintln!("native-check: aged activity");});app.edit(Some("fixture-499".into()));assert!(app.session.borrow().is_none());
            eprintln!("native-check: locked editor");
            // An in-flight worker result must not unlock after a lock/epoch change.
            let paths=app.paths.clone();app.work(move||{std::thread::sleep(Duration::from_millis(50));vault::unlock(&paths,"native UI fixture password").map_err(|e|e.to_string())},|app,s|{*app.session.borrow_mut()=Some(s);});app.lock();
            let app=app.clone();glib::timeout_add_seconds_local_once(2,move||{
                eprintln!("native-check: late unlock check");assert!(app.session.borrow().is_none());assert!(!app.busy.get());
                let s=vault::unlock(&app.paths,"native UI fixture password").unwrap();let e=s.data.entries.iter().find(|e|e.id=="fixture-499").unwrap();assert_eq!(e.password,"native edited secret");
                let f=s.data.folders.iter().find(|f|f.name=="New native folder").unwrap();assert_eq!(f.icon.as_deref(),Some("heart"));assert_eq!(f.color.as_deref(),Some("#78c8ff"));let blob=backup::export(&s.data,"native backup fixture password").unwrap();let restored=backup::decrypt(&blob,"native backup fixture password").unwrap();assert_eq!(restored.entries.len(),1000);*app.session.borrow_mut()=Some(s);app.activity.set(Instant::now());app.settings.borrow_mut()["autoLockMinutes"]=json!(5);app.show_vault();
                let mut settings=app.settings.borrow_mut();settings["background"]=json!({"type":"image","value":app.paths.settings.parent().unwrap().join("wallpaper.check.gif").to_string_lossy()});settings["lowPowerMode"]=json!(false);settings["panelOpacity"]=json!(0.8);settings["flicker"]=json!(true);drop(settings);
                app.apply_settings().unwrap();app.window.set_title("SpiritByte — native UI check");
                std::fs::write(&report,serde_json::to_string_pretty(&json!({"entries":1000,"search_ms":search_ms,"encrypted_edit_roundtrip":true,"nested_folder_filter":true,"folder_creation":true,"folder_icon_color_encrypted_roundtrip":true,"clipboard_expiry":true,"lock_closes_secret_editor":true,"idle_lock":true,"late_unlock_discarded":true,"encrypted_backup_roundtrip":true,"gif_loaded":true,"ready_for_frame_capture":true})).unwrap()).unwrap();
                let app=app.clone();glib::timeout_add_seconds_local_once(45,move||app.window.application().unwrap().quit());
            });
        });
    });
}

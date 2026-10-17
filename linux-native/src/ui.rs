//! Native widgets following the existing Windows three-pane vault layout.
//! No web runtime, animation loop, or extra decoded wallpaper surfaces.
use super::*;
use gtk::atk::prelude::*;

fn draw_extra_icon(name: &str, cr: &gtk::cairo::Context) -> bool {
    let line = |p: &[(f64, f64)]| {
        cr.move_to(p[0].0, p[0].1);
        for &(x, y) in &p[1..] {
            cr.line_to(x, y);
        }
        let _ = cr.stroke();
    };
    let rect = |x, y, w, h| {
        cr.rectangle(x, y, w, h);
        let _ = cr.stroke();
    };
    let circle = |x, y, r| {
        cr.new_sub_path();
        cr.arc(x, y, r, 0., std::f64::consts::TAU);
        let _ = cr.stroke();
    };
    match name {
        "mail" => {
            rect(3., 5., 18., 14.);
            line(&[(3., 5.), (12., 12.), (21., 5.)]);
        }
        "shield" => {
            line(&[
                (12., 3.),
                (21., 7.),
                (20., 15.),
                (16., 19.),
                (12., 22.),
                (8., 19.),
                (4., 15.),
                (3., 7.),
                (12., 3.),
            ]);
        }
        "user" => {
            circle(12., 7., 4.);
            cr.arc(12., 21., 7., std::f64::consts::PI, std::f64::consts::TAU);
            let _ = cr.stroke();
        }
        "wallet" => {
            rect(3., 5., 18., 15.);
            rect(14., 10., 7., 6.);
            circle(17., 13., 0.7);
        }
        "briefcase" => {
            rect(3., 7., 18., 14.);
            line(&[(8., 7.), (8., 3.), (16., 3.), (16., 7.)]);
            line(&[(3., 12.), (12., 15.), (21., 12.)]);
        }
        "cloud" => {
            cr.move_to(6., 19.);
            cr.curve_to(-1., 18., 1., 9., 7., 10.);
            cr.curve_to(8., 1., 20., 3., 19., 11.);
            cr.curve_to(25., 12., 23., 19., 18., 19.);
            cr.close_path();
            let _ = cr.stroke();
        }
        "gamepad" => {
            line(&[
                (6., 6.),
                (18., 6.),
                (21., 10.),
                (22., 18.),
                (18., 20.),
                (15., 16.),
                (9., 16.),
                (6., 20.),
                (2., 18.),
                (3., 10.),
                (6., 6.),
            ]);
            line(&[(5., 11.), (11., 11.)]);
            line(&[(8., 8.), (8., 14.)]);
            circle(16., 10., 0.7);
            circle(19., 13., 0.7);
        }
        "heart" => {
            cr.move_to(12., 21.);
            cr.curve_to(9., 18., 2., 13., 2., 8.);
            cr.curve_to(2., 1., 10., 1., 12., 6.);
            cr.curve_to(14., 1., 22., 1., 22., 8.);
            cr.curve_to(22., 13., 15., 18., 12., 21.);
            let _ = cr.stroke();
        }
        "home" => {
            line(&[(2., 11.), (12., 3.), (22., 11.)]);
            line(&[
                (5., 9.),
                (5., 21.),
                (10., 21.),
                (10., 14.),
                (14., 14.),
                (14., 21.),
                (19., 21.),
                (19., 9.),
            ]);
        }
        "music" => {
            line(&[(9., 18.), (9., 5.), (21., 3.), (21., 16.)]);
            circle(6., 19., 3.);
            circle(18., 17., 3.);
        }
        "camera" => {
            line(&[
                (3., 7.),
                (7., 7.),
                (9., 4.),
                (15., 4.),
                (17., 7.),
                (21., 7.),
                (21., 20.),
                (3., 20.),
                (3., 7.),
            ]);
            circle(12., 13., 4.);
        }
        "code" => {
            line(&[(8., 6.), (2., 12.), (8., 18.)]);
            line(&[(16., 6.), (22., 12.), (16., 18.)]);
            line(&[(14., 3.), (10., 21.)]);
        }
        "book" => {
            line(&[
                (12., 5.),
                (7., 3.),
                (2., 4.),
                (2., 20.),
                (7., 19.),
                (12., 21.),
                (17., 19.),
                (22., 20.),
                (22., 4.),
                (17., 3.),
                (12., 5.),
                (12., 21.),
            ]);
        }
        "cart" => {
            line(&[
                (2., 3.),
                (5., 3.),
                (8., 16.),
                (19., 16.),
                (22., 7.),
                (6., 7.),
            ]);
            circle(9., 21., 1.);
            circle(18., 21., 1.);
        }
        "server" => {
            rect(3., 3., 18., 7.);
            rect(3., 14., 18., 7.);
            circle(7., 6.5, 0.7);
            circle(7., 17.5, 0.7);
        }
        "phone" => {
            rect(6., 2., 12., 20.);
            line(&[(10., 18.), (14., 18.)]);
        }
        "card" => {
            rect(2., 4., 20., 16.);
            line(&[(2., 9.), (22., 9.)]);
            line(&[(6., 16.), (10., 16.)]);
        }
        "plane" => {
            line(&[
                (2., 9.),
                (10., 10.),
                (17., 3.),
                (20., 3.),
                (20., 6.),
                (13., 13.),
                (14., 21.),
                (11., 20.),
                (8., 16.),
                (4., 13.),
                (2., 9.),
            ]);
        }
        "gift" => {
            rect(3., 10., 18., 11.);
            rect(2., 6., 20., 4.);
            line(&[(12., 6.), (12., 21.)]);
            cr.move_to(12., 6.);
            cr.curve_to(2., 7., 5., -3., 12., 6.);
            cr.curve_to(19., -3., 22., 7., 12., 6.);
            let _ = cr.stroke();
        }
        "github" => {
            cr.move_to(9., 22.);
            cr.line_to(9., 18.);
            cr.curve_to(3., 19., 3., 16., 1., 16.);
            let _ = cr.stroke();
            cr.move_to(15., 22.);
            cr.line_to(15., 18.);
            cr.curve_to(15., 16., 14., 16., 14., 16.);
            cr.curve_to(23., 15., 23., 8., 20., 6.);
            cr.curve_to(20.5, 4., 20., 2., 20., 2.);
            cr.curve_to(18., 2., 16., 3., 15., 4.);
            cr.curve_to(13., 3.5, 11., 3.5, 9., 4.);
            cr.curve_to(7., 2., 4., 2., 4., 2.);
            cr.curve_to(3.5, 4., 4., 6., 4., 6.);
            cr.curve_to(1., 9., 2., 15., 10., 16.);
            cr.curve_to(9., 17., 9., 18., 9., 18.);
            let _ = cr.stroke();
        }
        "paypal" => {
            line(&[(5., 21.), (8., 3.), (15., 3.)]);
            cr.move_to(15., 3.);
            cr.curve_to(23., 3., 21., 14., 14., 14.);
            cr.line_to(9., 14.);
            let _ = cr.stroke();
            line(&[(10., 22.), (13., 7.), (17., 7.)]);
            cr.move_to(17., 7.);
            cr.curve_to(24., 7., 22., 18., 16., 18.);
            cr.line_to(11., 18.);
            let _ = cr.stroke();
        }
        _ => return false,
    }
    true
}

pub fn password_field(
    container: &gtk::Box,
    title: &str,
    name: &str,
    show: &str,
    hide: &str,
) -> gtk::Entry {
    pack(container, &label(title));
    let row = box_h(8);
    let entry = input("", true);
    entry.set_widget_name(name);
    row.pack_start(&entry, true, true, 0);
    let b = gtk::Button::new();
    b.style_context().add_class("icon-button");
    b.set_widget_name(&format!("{name}-eye"));
    b.set_tooltip_text(Some(show));
    if let Some(a) = b.accessible() {
        a.set_name(show);
    }
    b.add(&icon("eye", 18));
    let p = entry.clone();
    let show = show.to_string();
    let hide = hide.to_string();
    b.connect_clicked(move |b| {
        let visible = !gtk::prelude::EntryExt::is_visible(&p);
        p.set_visibility(visible);
        if let Some(child) = b.child() {
            b.remove(&child);
        }
        b.add(&icon(if visible { "eye-off" } else { "eye" }, 18));
        let title = if visible { &hide } else { &show };
        b.set_tooltip_text(Some(title));
        if let Some(a) = b.accessible() {
            a.set_name(title);
        }
        b.show_all();
    });
    pack(&row, &b);
    pack(container, &row);
    entry
}

impl App {
    pub(super) fn social_links(self: &Rc<Self>, row: &gtk::Box) {
        for (name, title, url) in [
            ("github", "GitHub", "https://github.com/OscarTired"),
            (
                "globe",
                self.tr("Portafolio", "Portfolio"),
                "https://octa-dev.com/",
            ),
            (
                "paypal",
                self.tr("Apoyar en PayPal", "Support on PayPal"),
                "https://www.paypal.com/paypalme/octacodec",
            ),
        ] {
            let app = self.clone();
            let b = icon_button(row, name, title, move || app.open_external(url));
            b.set_widget_name(if name == "globe" {
                "social-portfolio"
            } else if name == "github" {
                "social-github"
            } else {
                "social-paypal"
            });
        }
    }

    fn open_external(&self, url: &str) {
        #[cfg(feature = "ui-check")]
        if std::env::var_os("SPIRITBYTE_POINTER_CHECK").is_some() {
            *self.external_url.borrow_mut() = Some(url.to_string());
            return;
        }
        if let Err(e) = gio::AppInfo::launch_default_for_uri(url, None::<&gio::AppLaunchContext>) {
            self.message(&e.to_string());
        }
    }
}

pub fn icon(name: &str, size: i32) -> gtk::DrawingArea {
    let area = gtk::DrawingArea::new();
    area.set_size_request(size, size);
    area.set_halign(gtk::Align::Center);
    area.set_valign(gtk::Align::Center);
    // Windowless decoration cannot become an input target over its button.
    area.set_has_window(false);
    let name = name.to_string();
    area.connect_draw(move |area, cr| {
        let c = area.style_context().color(area.state_flags());
        cr.set_source_rgba(
            c.red().into(),
            c.green().into(),
            c.blue().into(),
            c.alpha().into(),
        );
        cr.scale(size as f64 / 24.0, size as f64 / 24.0);
        cr.set_line_width(1.7);
        cr.set_line_cap(gtk::cairo::LineCap::Round);
        cr.set_line_join(gtk::cairo::LineJoin::Round);
        let line = |points: &[(f64, f64)]| {
            if let Some((x, y)) = points.first() {
                cr.move_to(*x, *y);
            }
            for (x, y) in &points[1..] {
                cr.line_to(*x, *y);
            }
            let _ = cr.stroke();
        };
        match name.as_str() {
            "plus" => {
                line(&[(12., 5.), (12., 19.)]);
                line(&[(5., 12.), (19., 12.)]);
            }
            "star" => line(&[
                (12., 3.),
                (14.8, 8.7),
                (21., 9.6),
                (16.5, 14.),
                (17.6, 20.2),
                (12., 17.2),
                (6.4, 20.2),
                (7.5, 14.),
                (3., 9.6),
                (9.2, 8.7),
                (12., 3.),
            ]),
            "folder" => line(&[
                (3., 6.),
                (9., 6.),
                (11., 8.),
                (21., 8.),
                (21., 20.),
                (3., 20.),
                (3., 6.),
            ]),
            "lock" => {
                cr.rectangle(5., 10., 14., 11.);
                let _ = cr.stroke();
                cr.arc(12., 9., 4., std::f64::consts::PI, 2. * std::f64::consts::PI);
                let _ = cr.stroke();
                line(&[(12., 14.), (12., 17.)]);
            }
            "copy" => {
                cr.rectangle(8., 8., 12., 13.);
                let _ = cr.stroke();
                line(&[(16., 4.), (4., 4.), (4., 16.)]);
            }
            "edit" => {
                line(&[
                    (14., 5.),
                    (19., 10.),
                    (8., 21.),
                    (3., 21.),
                    (3., 16.),
                    (14., 5.),
                    (17., 2.),
                    (22., 7.),
                    (19., 10.),
                ]);
            }
            "trash" => {
                line(&[(3., 6.), (21., 6.)]);
                line(&[(6., 6.), (7., 21.), (17., 21.), (18., 6.)]);
                line(&[(9., 6.), (9., 3.), (15., 3.), (15., 6.)]);
                line(&[(10., 10.), (10., 17.)]);
                line(&[(14., 10.), (14., 17.)]);
            }
            "eye" | "eye-off" => {
                cr.save().ok();
                cr.translate(12., 12.);
                cr.scale(1., 0.6);
                cr.arc(0., 0., 9., 0., std::f64::consts::TAU);
                cr.restore().ok();
                let _ = cr.stroke();
                cr.arc(12., 12., 2.5, 0., std::f64::consts::TAU);
                let _ = cr.stroke();
                if name == "eye-off" {
                    line(&[(3., 3.), (21., 21.)]);
                }
            }
            "globe" => {
                cr.arc(12., 12., 9., 0., std::f64::consts::TAU);
                let _ = cr.stroke();
                line(&[(3., 12.), (21., 12.)]);
                cr.save().ok();
                cr.translate(12., 12.);
                cr.scale(0.45, 1.);
                cr.arc(0., 0., 9., 0., std::f64::consts::TAU);
                cr.restore().ok();
                let _ = cr.stroke();
            }
            "generator" => {
                cr.rectangle(4., 4., 16., 16.);
                let _ = cr.stroke();
                for (x, y) in [(8., 8.), (16., 8.), (12., 12.), (8., 16.), (16., 16.)] {
                    cr.arc(x, y, 0.9, 0., std::f64::consts::TAU);
                    let _ = cr.fill();
                }
            }
            "settings" => {
                for i in 0..8 {
                    let a = i as f64 * std::f64::consts::FRAC_PI_4;
                    line(&[
                        (12. + 7. * a.cos(), 12. + 7. * a.sin()),
                        (12. + 10. * a.cos(), 12. + 10. * a.sin()),
                    ]);
                }
                cr.arc(12., 12., 7., 0., std::f64::consts::TAU);
                let _ = cr.stroke();
                cr.arc(12., 12., 2.5, 0., std::f64::consts::TAU);
                let _ = cr.stroke();
            }
            _ if draw_extra_icon(&name, cr) => {}
            _ => {
                cr.arc(15., 8., 5., 0., std::f64::consts::TAU);
                let _ = cr.stroke();
                line(&[
                    (11., 12.),
                    (3., 20.),
                    (3., 16.),
                    (6., 16.),
                    (6., 13.),
                    (9., 13.),
                ]);
            }
        }
        glib::Propagation::Proceed
    });
    area
}

pub fn icon_button(
    container: &gtk::Box,
    name: &str,
    title: &str,
    action: impl Fn() + 'static,
) -> gtk::Button {
    let b = gtk::Button::new();
    b.style_context().add_class("icon-button");
    b.set_tooltip_text(Some(title));
    b.set_widget_name(name);
    if let Some(a) = b.accessible() {
        a.set_name(title);
    }
    b.add(&icon(name, 16));
    b.connect_clicked(move |_| action());
    pack(container, &b);
    b
}

fn scroller(child: &impl IsA<gtk::Widget>) -> gtk::ScrolledWindow {
    let scroll = gtk::ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
    scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scroll.add(child);
    scroll
}

impl App {
    pub(super) fn show_vault(self: &Rc<Self>) {
        if self.session.borrow().is_none() {
            self.show_locked();
            return;
        }
        self.clean_content();
        let sidebar = box_v(0);
        sidebar.style_context().add_class("sidebar");
        sidebar.set_size_request(192, -1);
        let brand = box_h(8);
        brand.style_context().add_class("pane-header");
        pack(&brand, &icon("key", 18));
        let name = label("SPIRITBYTE");
        name.style_context().add_class("brand");
        pack(&brand, &name);
        pack(&sidebar, &brand);
        let nav = gtk::ListBox::new();
        nav.set_selection_mode(gtk::SelectionMode::Single);
        nav.set_activate_on_single_click(true);
        nav.style_context().add_class("navigation");
        sidebar.pack_start(&scroller(&nav), true, true, 0);
        *self.navigation.borrow_mut() = Some(nav);
        let tools = box_h(4);
        tools.style_context().add_class("pane-footer");
        let app = self.clone();
        icon_button(
            &tools,
            "generator",
            self.tr("Generador", "Generator"),
            move || app.generator(None),
        );
        let app = self.clone();
        icon_button(
            &tools,
            "settings",
            self.tr("Ajustes", "Settings"),
            move || app.settings_dialog(),
        );
        let app = self.clone();
        icon_button(&tools, "lock", self.tr("Bloquear", "Lock"), move || {
            app.lock()
        });
        for child in tools.children() {
            child.set_hexpand(true);
        }
        pack(&sidebar, &tools);

        let center = box_v(0);
        center.style_context().add_class("entry-pane");
        center.set_size_request(240, -1);
        let searchbar = box_h(8);
        searchbar.style_context().add_class("pane-header");
        let search = gtk::SearchEntry::new();
        search.set_widget_name("vault-search");
        search.set_placeholder_text(Some(self.tr("Buscar…", "Search…")));
        search.set_text(&self.query.borrow());
        search.set_width_chars(12);
        searchbar.pack_start(&search, true, true, 0);
        let app = self.clone();
        let add = icon_button(
            &searchbar,
            "plus",
            self.tr("Nueva entrada", "New entry"),
            move || app.edit(None),
        );
        add.set_widget_name("new-entry");
        add.style_context().add_class("primary");
        pack(&center, &searchbar);
        let app = self.clone();
        search.connect_search_changed(move |s| {
            app.query.replace(s.text().to_string());
            app.refresh();
        });
        let list = gtk::ListStore::new(&[
            String::static_type(),
            String::static_type(),
            String::static_type(),
            String::static_type(),
            String::static_type(),
        ]);
        let tree = gtk::TreeView::with_model(&list);
        tree.set_widget_name("vault-entries");
        tree.selection().set_mode(gtk::SelectionMode::Multiple);
        tree.set_headers_visible(false);
        tree.set_search_column(-1);
        tree.set_enable_search(false);
        tree.set_enable_tree_lines(false);
        let renderer = gtk::CellRendererText::new();
        renderer.set_ellipsize(gtk::pango::EllipsizeMode::End);
        renderer.set_xpad(12);
        renderer.set_ypad(9);
        let column = gtk::TreeViewColumn::new();
        gtk::prelude::TreeViewColumnExt::pack_start(&column, &renderer, true);
        gtk::prelude::TreeViewColumnExt::add_attribute(&column, &renderer, "markup", 4);
        column.set_expand(true);
        tree.append_column(&column);
        let stack = gtk::Stack::new();
        stack.add_named(&scroller(&tree), "entries");
        let empty = label("");
        empty.set_halign(gtk::Align::Center);
        empty.set_valign(gtk::Align::Start);
        empty.set_margin_top(20);
        empty.style_context().add_class("muted");
        stack.add_named(&empty, "empty");
        center.pack_start(&stack, true, true, 0);
        *self.entry_stack.borrow_mut() = Some(stack);
        let app = self.clone();
        tree.selection().connect_changed(move |_| {
            if !app.refresh_guard.get() {
                app.show_detail();
            }
        });
        let app = self.clone();
        tree.connect_row_activated(move |tree, path, _| {
            let id = tree.model().and_then(|m| {
                m.iter(path)
                    .and_then(|i| m.value(&i, 0).get::<String>().ok())
            });
            if let Some(id) = id {
                app.edit(Some(id));
            }
        });
        *self.list.borrow_mut() = Some(list);
        *self.tree.borrow_mut() = Some(tree);
        let status = label("");
        status.style_context().add_class("list-status");
        pack(&center, &status);
        *self.status.borrow_mut() = Some(status);

        let detail = box_v(16);
        detail.set_widget_name("entry-detail");
        detail.style_context().add_class("detail");
        let detail_scroll = scroller(&detail);
        let detail_column = box_v(0);
        detail_column.pack_start(&detail_scroll, true, true, 0);
        let links = box_h(4);
        links.set_halign(gtk::Align::End);
        links.set_margin_end(12);
        links.set_margin_bottom(8);
        self.social_links(&links);
        pack(&detail_column, &links);
        *self.detail.borrow_mut() = Some(detail);
        let panes = gtk::Paned::new(gtk::Orientation::Horizontal);
        panes.pack1(&center, false, false);
        panes.pack2(&detail_column, true, false);
        panes.set_position(288);
        let shell = gtk::Paned::new(gtk::Orientation::Horizontal);
        shell.pack1(&sidebar, false, false);
        shell.pack2(&panes, true, false);
        shell.set_position(224);
        self.content.pack_start(&shell, true, true, 0);
        self.refresh_navigation();
        self.refresh();
        self.content.show_all();
        search.grab_focus();
    }

    pub(super) fn refresh_navigation(self: &Rc<Self>) {
        let nav = self.navigation.borrow().clone();
        let Some(nav) = nav else {
            return;
        };
        for child in nav.children() {
            nav.remove(&child);
        }
        let session = self.session.borrow();
        let Some(session) = session.as_ref() else {
            return;
        };
        let data = &session.data;
        let mut items = vec![
            (
                "all".to_string(),
                self.tr("Todas", "All").to_string(),
                "key",
                data.entries.len(),
                0,
                None,
            ),
            (
                "star".to_string(),
                self.tr("Favoritos", "Favorites").to_string(),
                "star",
                data.entries.iter().filter(|e| e.favorite).count(),
                0,
                None,
            ),
        ];
        // Keep the creation action available even with no folders or filters.
        items.push((String::new(), String::new(), "folder", 0, 0, None));
        let mut folders: Vec<_> = data.folders.iter().collect();
        folders.sort_by_key(|f| Self::folder_path(data, &f.id).to_lowercase());
        for f in folders {
            let path = Self::folder_path(data, &f.id);
            let count = data
                .entries
                .iter()
                .filter(|e| {
                    e.folder_id
                        .as_deref()
                        .is_some_and(|id| Self::in_folder(data, id, &f.id))
                })
                .count();
            items.push((
                f.id.clone(),
                f.name.clone(),
                f.icon.as_deref().unwrap_or("folder"),
                count,
                path.matches(" / ").count(),
                f.color.clone(),
            ));
        }
        let no_folder = data
            .entries
            .iter()
            .filter(|e| e.folder_id.is_none())
            .count();
        if self.settings.borrow()["showUnfiled"]
            .as_bool()
            .unwrap_or(true)
        {
            items.push((
                "none".into(),
                self.tr("Sin carpeta", "No folder").into(),
                "folder",
                no_folder,
                0,
                None,
            ));
        }
        for (i, (id, name, icon_name, count, depth, color)) in items.into_iter().enumerate() {
            if i == 2 {
                let section = box_h(4);
                section.style_context().add_class("folder-heading");
                let l = label(self.tr("CARPETAS", "FOLDERS"));
                l.style_context().add_class("field-label");
                section.pack_start(&l, true, true, 0);
                let app = self.clone();
                icon_button(
                    &section,
                    "plus",
                    self.tr("Nueva carpeta", "New folder"),
                    move || app.manage_folders(),
                );
                let row = gtk::ListBoxRow::new();
                row.style_context().add_class("folder-section");
                row.set_selectable(false);
                row.set_activatable(false);
                row.add(&section);
                nav.add(&row);
                continue;
            }
            let row = gtk::ListBoxRow::new();
            row.set_widget_name(&id);
            let line = box_h(8);
            line.set_margin_start((depth.min(5) * 12) as i32);
            let glyph = icon(icon_name, 16);
            if let Some(color) = color.filter(|c| {
                c.len() == 7 && c.starts_with('#') && c[1..].chars().all(|c| c.is_ascii_hexdigit())
            }) {
                let css = gtk::CssProvider::new();
                let _ = css.load_from_data(format!("* {{color:{color};}}").as_bytes());
                glyph
                    .style_context()
                    .add_provider(&css, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION + 1);
            }
            pack(&line, &glyph);
            let title = label(&name);
            title.set_ellipsize(gtk::pango::EllipsizeMode::End);
            title.set_max_width_chars(18);
            line.pack_start(&title, true, true, 0);
            let count = label(&count.to_string());
            count.style_context().add_class("count");
            pack(&line, &count);
            if data.folders.iter().any(|f| f.id == id) {
                let app = self.clone();
                let folder_id = id.clone();
                let edit = icon_button(
                    &line,
                    "edit",
                    self.tr("Editar carpeta", "Edit folder"),
                    move || {
                        app.folder_dialog(Some(folder_id.clone()));
                    },
                );
                edit.set_widget_name(&format!("edit-folder-{id}"));
            } else if id == "none" {
                let app = self.clone();
                let remove = icon_button(
                    &line,
                    "trash",
                    self.tr("Quitar de la barra lateral", "Remove from sidebar"),
                    move || {
                        if !app.confirm(app.tr("¿Quitar «Sin carpeta» de la barra lateral? Tus entradas seguirán en «Todas». Puedes volver a mostrarla en Ajustes.", "Remove “No folder” from the sidebar? Your entries remain in “All”. You can show it again in Settings.")) { return; }
                        let mut settings = app.settings.borrow().clone();
                        settings["showUnfiled"] = json!(false);
                        match model::save_settings(&app.paths, &settings) {
                            Ok(()) => {
                                *app.settings.borrow_mut() = settings;
                                if *app.filter.borrow() == "none" {
                                    app.filter.replace("all".into());
                                }
                                app.refresh_navigation();
                                app.refresh();
                            }
                            Err(error) => app.message(&error),
                        }
                    },
                );
                remove.set_widget_name("remove-unfiled");
            }
            row.add(&line);
            nav.add(&row);
            if id == *self.filter.borrow() {
                nav.select_row(Some(&row));
            }
        }
        // A rebuilt sidebar has a new list of rows, but only one signal handler.
        if nav.widget_name() != "vault-navigation" {
            nav.set_widget_name("vault-navigation");
            let app = self.clone();
            nav.connect_row_selected(move |_, row| {
                if let Some(row) = row {
                    app.filter.replace(row.widget_name().to_string());
                    app.refresh();
                }
            });
        }
        nav.show_all();
    }

    pub(super) fn show_detail(self: &Rc<Self>) {
        let detail = self.detail.borrow().clone();
        let Some(detail) = detail else {
            return;
        };
        for child in detail.children() {
            wipe(&child);
            detail.remove(&child);
        }
        let id = self.selected();
        let session = self.session.borrow();
        let entry = session
            .as_ref()
            .and_then(|s| s.data.entries.iter().find(|e| Some(&e.id) == id.as_ref()));
        let Some(entry) = entry else {
            let empty = box_v(16);
            empty.set_valign(gtk::Align::Center);
            empty.set_halign(gtk::Align::Center);
            empty.style_context().add_class("empty-state");
            pack(&empty, &icon("key", 48));
            let text = label(self.tr(
                "Selecciona o crea una entrada.",
                "Select or create an entry.",
            ));
            text.set_line_wrap(true);
            text.set_justify(gtk::Justification::Center);
            pack(&empty, &text);
            detail.pack_start(&empty, true, true, 0);
            detail.show_all();
            return;
        };
        let header = box_h(8);
        let title = label(if entry.title.is_empty() {
            self.tr("Sin título", "Untitled")
        } else {
            &entry.title
        });
        title.style_context().add_class("title");
        title.set_line_wrap(true);
        title.set_line_wrap_mode(gtk::pango::WrapMode::WordChar);
        title.set_max_width_chars(32);
        header.pack_start(&title, true, true, 0);
        let app = self.clone();
        for (name, title, action) in [
            ("star", self.tr("Favorito", "Favorite"), 3),
            ("edit", self.tr("Editar", "Edit"), 0),
            ("trash", self.tr("Eliminar", "Delete"), 5),
        ] {
            let a = app.clone();
            let b = icon_button(&header, name, title, move || a.entry_action(action));
            if action == 3 && entry.favorite {
                b.style_context().add_class("primary");
            }
            if action == 5 {
                b.style_context().add_class("danger");
            }
        }
        pack(&detail, &header);
        self.detail_field(
            &detail,
            &app,
            self.tr("USUARIO", "USERNAME"),
            &entry.username,
            1,
            false,
        );
        self.detail_field(
            &detail,
            &app,
            self.tr("CONTRASEÑA", "PASSWORD"),
            &entry.password,
            2,
            true,
        );
        self.detail_field(&detail, &app, "URL", &entry.url, 4, false);
        if !entry.notes.is_empty() {
            self.detail_field(
                &detail,
                &app,
                self.tr("NOTAS", "NOTES"),
                &entry.notes,
                6,
                false,
            );
        }
        if let Some(folder) = entry.folder_id.as_deref() {
            let folder = label(&Self::folder_path(&session.as_ref().unwrap().data, folder));
            folder.style_context().add_class("muted");
            pack(&detail, &folder);
        }
        detail.show_all();
    }

    fn detail_field(
        &self,
        parent: &gtk::Box,
        app: &Rc<Self>,
        name: &str,
        value: &str,
        action: u8,
        secret: bool,
    ) {
        let field = box_v(6);
        field.style_context().add_class("detail-field");
        let head = box_h(8);
        let caption = label(name);
        caption.style_context().add_class("field-label");
        head.pack_start(&caption, true, true, 0);
        let value_widget = gtk::Entry::new();
        if secret {
            value_widget.set_text(value);
            value_widget.set_editable(false);
            value_widget.set_visibility(false);
            value_widget.set_has_frame(false);
            value_widget.style_context().add_class("secret-password");
            let p = value_widget.clone();
            let title = self.tr("Mostrar / ocultar contraseña", "Show / hide password");
            icon_button(&head, "eye", title, move || {
                p.set_visibility(!gtk::prelude::EntryExt::is_visible(&p))
            });
        }
        if action == 4 && !value.is_empty() {
            let a = app.clone();
            icon_button(
                &head,
                "globe",
                self.tr("Abrir URL", "Open URL"),
                move || a.entry_action(4),
            );
        }
        let a = app.clone();
        let copy_action = if action == 4 { 7 } else { action };
        let copy = icon_button(&head, "copy", self.tr("Copiar", "Copy"), move || {
            a.entry_action(copy_action)
        });
        copy.set_sensitive(!value.is_empty());
        pack(&field, &head);
        if secret {
            pack(&field, &value_widget);
        } else {
            let v = label(if value.is_empty() { "—" } else { value });
            v.set_selectable(true);
            v.set_line_wrap(true);
            v.set_line_wrap_mode(gtk::pango::WrapMode::WordChar);
            v.set_max_width_chars(65);
            v.style_context().add_class("secret-value");
            pack(&field, &v);
        }
        pack(parent, &field);
    }

    fn entry_action(self: &Rc<Self>, action: u8) {
        let Some(id) = self.selected() else {
            return;
        };
        match action {
            0 => self.edit(Some(id)),
            1 | 2 | 6 | 7 => {
                let value = self
                    .session
                    .borrow()
                    .as_ref()
                    .and_then(|s| s.data.entries.iter().find(|e| e.id == id))
                    .map(|e| match action {
                        1 => e.username.clone(),
                        2 => e.password.clone(),
                        6 => e.notes.clone(),
                        _ => e.url.clone(),
                    });
                if let Some(v) = value.filter(|v| !v.is_empty()) {
                    self.copy(v);
                    if let Some(s) = self.status.borrow().as_ref() {
                        s.set_text(self.tr(
                            "Copiado · borrado automático activo",
                            "Copied · auto-clear active",
                        ));
                    }
                }
            }
            3 => {
                self.change(|data| {
                    if let Some(e) = data.entries.iter_mut().find(|e| e.id == id) {
                        e.favorite = !e.favorite;
                        e.updated_at = now();
                    }
                });
            }
            4 => {
                let url = self
                    .session
                    .borrow()
                    .as_ref()
                    .and_then(|s| s.data.entries.iter().find(|e| e.id == id))
                    .map(|e| e.url.clone())
                    .unwrap_or_default();
                if url.is_empty() {
                    return;
                }
                let url = if url.contains("://") {
                    url
                } else {
                    format!("https://{url}")
                };
                if url.starts_with("https://") || url.starts_with("http://") {
                    if let Err(e) =
                        gio::AppInfo::launch_default_for_uri(&url, None::<&gio::AppLaunchContext>)
                    {
                        self.message(&e.to_string());
                    }
                } else {
                    self.message(self.tr(
                        "Solo se permiten enlaces http y https.",
                        "Only http and https links are allowed.",
                    ));
                }
            }
            _ => {
                if self.confirm(self.tr(
                    "¿Eliminar esta entrada de la bóveda?",
                    "Delete this entry from the vault?",
                )) {
                    self.change(|data| data.entries.retain(|e| e.id != id));
                }
            }
        }
    }
}

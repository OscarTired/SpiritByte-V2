//! Folder metadata uses the existing cross-platform icon IDs and hex colors.
use super::*;
use gtk::atk::prelude::*;

const ICONS: &[(&str, &str, &str)] = &[
    ("folder", "Carpeta", "Folder"),
    ("globe", "Web", "Web"),
    ("mail", "Correo", "Mail"),
    ("shield", "Seguridad", "Security"),
    ("user", "Usuario", "User"),
    ("wallet", "Billetera", "Wallet"),
    ("briefcase", "Trabajo", "Work"),
    ("cloud", "Nube", "Cloud"),
    ("gamepad", "Juegos", "Games"),
    ("key", "Llave", "Key"),
    ("heart", "Corazón", "Heart"),
    ("home", "Casa", "Home"),
    ("music", "Música", "Music"),
    ("camera", "Cámara", "Camera"),
    ("code", "Código", "Code"),
    ("book", "Libro", "Book"),
    ("cart", "Compras", "Shopping"),
    ("server", "Servidor", "Server"),
    ("phone", "Teléfono", "Phone"),
    ("card", "Tarjeta", "Card"),
    ("plane", "Viajes", "Travel"),
    ("gift", "Regalo", "Gift"),
];
const COLORS: &[&str] = &[
    "#ff6b00", "#57ff78", "#ff40a0", "#78c8ff", "#ffd500", "#ff5a5a", "#78ffd4", "#b478ff",
];

#[derive(Clone)]
pub struct Picker {
    pub icon: Rc<RefCell<Option<String>>>,
    pub color: Rc<RefCell<Option<String>>>,
    icons: Vec<(String, gtk::RadioButton)>,
    colors: Vec<(String, gtk::RadioButton)>,
    automatic: gtk::RadioButton,
    custom: gtk::RadioButton,
    chooser: gtk::ColorButton,
}

fn color_hex(color: &gdk::RGBA) -> String {
    format!(
        "#{:02x}{:02x}{:02x}",
        (color.red() * 255.).round() as u8,
        (color.green() * 255.).round() as u8,
        (color.blue() * 255.).round() as u8
    )
}

impl Picker {
    pub fn new(app: &App, container: &gtk::Box) -> Self {
        pack(container, &label(app.tr("Icono", "Icon")));
        let icon = Rc::new(RefCell::new(Some("folder".to_string())));
        let color = Rc::new(RefCell::new(None));
        let grid = gtk::Grid::new();
        grid.set_row_spacing(4);
        grid.set_column_spacing(4);
        let mut icons: Vec<(String, gtk::RadioButton)> = vec![];
        for (i, (id, es, en)) in ICONS.iter().enumerate() {
            let b = if let Some((_, first)) = icons.first() {
                gtk::RadioButton::from_widget(first)
            } else {
                gtk::RadioButton::builder().build()
            };
            b.set_mode(false);
            b.set_widget_name(&format!("folder-icon-{id}"));
            b.style_context().add_class("icon-button");
            b.set_tooltip_text(Some(app.tr(es, en)));
            if let Some(a) = b.accessible() {
                a.set_name(app.tr(es, en));
            }
            b.add(&ui::icon(id, 18));
            let value = icon.clone();
            let id = id.to_string();
            b.connect_toggled(move |b| {
                if b.is_active() {
                    *value.borrow_mut() = Some(id.clone());
                }
            });
            grid.attach(&b, (i % 8) as i32, (i / 8) as i32, 1, 1);
            icons.push((ICONS[i].0.to_string(), b));
        }
        pack(container, &grid);
        pack(container, &label(app.tr("Color del icono", "Icon color")));
        let options = box_h(8);
        let automatic = gtk::RadioButton::builder()
            .label(app.tr("De la paleta", "From palette"))
            .build();
        automatic.set_widget_name("folder-color-auto");
        let value = color.clone();
        automatic.connect_toggled(move |b| {
            if b.is_active() {
                *value.borrow_mut() = None;
            }
        });
        pack(&options, &automatic);
        let custom =
            gtk::RadioButton::with_label_from_widget(&automatic, app.tr("Personalizado", "Custom"));
        custom.set_widget_name("folder-color-custom");
        pack(&options, &custom);
        let chooser = gtk::ColorButton::with_rgba(&gdk::RGBA::parse("#ffb000").unwrap());
        chooser.set_use_alpha(false);
        chooser.set_title(app.tr("Color de la carpeta", "Folder color"));
        chooser.set_widget_name("folder-custom-color");
        let value = color.clone();
        let c = chooser.downgrade();
        custom.connect_toggled(move |b| {
            if b.is_active() {
                if let Some(c) = c.upgrade() {
                    *value.borrow_mut() = Some(color_hex(&c.rgba()));
                }
            }
        });
        let value = color.clone();
        let choice = custom.clone();
        chooser.connect_color_set(move |c| {
            choice.set_active(true);
            *value.borrow_mut() = Some(color_hex(&c.rgba()));
        });
        pack(&options, &chooser);
        pack(container, &options);
        let swatches = box_h(4);
        let mut colors = vec![];
        for hex in COLORS {
            let b = gtk::RadioButton::from_widget(&automatic);
            b.set_mode(false);
            b.style_context().add_class("icon-button");
            b.set_widget_name(&format!("folder-color-{}", &hex[1..]));
            b.set_tooltip_text(Some(hex));
            if let Some(a) = b.accessible() {
                a.set_name(hex);
            }
            let swatch = gtk::DrawingArea::new();
            swatch.set_has_window(false);
            swatch.set_size_request(18, 18);
            let rgba = gdk::RGBA::parse(hex).unwrap();
            swatch.connect_draw(move |_, cr| {
                cr.set_source_rgba(
                    rgba.red().into(),
                    rgba.green().into(),
                    rgba.blue().into(),
                    1.,
                );
                cr.rectangle(1., 1., 16., 16.);
                let _ = cr.fill();
                glib::Propagation::Proceed
            });
            b.add(&swatch);
            let value = color.clone();
            let hex = hex.to_string();
            let selected_hex = hex.clone();
            b.connect_toggled(move |b| {
                if b.is_active() {
                    *value.borrow_mut() = Some(selected_hex.clone());
                }
            });
            pack(&swatches, &b);
            colors.push((hex.to_string(), b));
        }
        pack(container, &swatches);
        Self {
            icon,
            color,
            icons,
            colors,
            automatic,
            custom,
            chooser,
        }
    }

    pub fn set(&self, icon: Option<&str>, color: Option<&str>) {
        let choice = self
            .icons
            .iter()
            .find(|(id, _)| Some(id.as_str()) == icon)
            .unwrap_or(&self.icons[0]);
        choice.1.set_active(true);
        // Preserve unfamiliar metadata when editing only a folder's name/parent.
        *self.icon.borrow_mut() = icon.map(str::to_string);
        if let Some((_, b)) = self
            .colors
            .iter()
            .find(|(hex, _)| Some(hex.as_str()) == color)
        {
            b.set_active(true);
        } else if let Some(rgba) = color.and_then(|c| gdk::RGBA::parse(c).ok()) {
            self.chooser.set_rgba(&rgba);
            self.custom.set_active(true);
        } else {
            self.automatic.set_active(true);
        }
        *self.color.borrow_mut() = color.map(str::to_string);
    }
}

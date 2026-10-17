//! The Windows phosphor-fox opening, drawn once and revealed with native Cairo.
use super::*;

const BAYER: [[u8; 8]; 8] = [
    [0, 32, 8, 40, 2, 34, 10, 42],
    [48, 16, 56, 24, 50, 18, 58, 26],
    [12, 44, 4, 36, 14, 46, 6, 38],
    [60, 28, 52, 20, 62, 30, 54, 22],
    [3, 35, 11, 43, 1, 33, 9, 41],
    [51, 19, 59, 27, 49, 17, 57, 25],
    [15, 47, 7, 39, 13, 45, 5, 37],
    [63, 31, 55, 23, 61, 29, 53, 21],
];

fn artwork(
    primary: [f64; 3],
    accent: [f64; 3],
    block: usize,
) -> Result<gtk::cairo::ImageSurface, String> {
    // Rasterized from the Windows SVG during development. Raw bytes avoid
    // retaining an SVG loader subprocess after the 3.2-second intro has ended.
    let pixels: &[u8; 360 * 360 * 4] = include_bytes!("../assets/fox-source.rgba");
    let surface = gtk::cairo::ImageSurface::create(gtk::cairo::Format::ARgb32, 420, 420)
        .map_err(|e| e.to_string())?;
    let cr = gtk::cairo::Context::new(&surface).map_err(|e| e.to_string())?;
    for y in (0..420).step_by(block) {
        for x in (0..420).step_by(block) {
            let (mut intensity, mut peak, mut samples) = (0.0_f64, 0.0_f64, 0);
            for dy in 0..block.min(420 - y) {
                for dx in 0..block.min(420 - x) {
                    let (sx, sy) = ((x + dx) as i32 - 30, (y + dy) as i32 - 30);
                    let value = if sx >= 0 && sy >= 0 && sx < 360 && sy < 360 {
                        let i = (sy as usize * 360 + sx as usize) * 4;
                        let alpha = pixels[i + 3] as f64 / 255.0;
                        (pixels[i] as f64 * 0.299
                            + pixels[i + 1] as f64 * 0.587
                            + pixels[i + 2] as f64 * 0.114)
                            / 255.0
                            * alpha
                    } else {
                        0.0
                    };
                    intensity += value;
                    peak = peak.max(value);
                    samples += 1;
                }
            }
            intensity /= samples as f64;
            if intensity > 0.03
                && intensity > BAYER[(y / block) & 7][(x / block) & 7] as f64 / 64.0 * 0.85
            {
                let color = if peak > 0.78 { accent } else { primary };
                cr.set_source_rgba(
                    color[0],
                    color[1],
                    color[2],
                    if peak > 0.78 {
                        0.95
                    } else {
                        0.35 + intensity * 0.55
                    },
                );
                cr.rectangle(x as f64, y as f64, block as f64, block as f64);
                cr.fill().map_err(|e| e.to_string())?;
            }
        }
    }
    Ok(surface)
}

impl App {
    pub(super) fn show_intro(self: &Rc<Self>) {
        self.clean_content();
        let settings = self.settings.borrow();
        let primary = background::rgb(
            settings["palette"]["primary"]
                .as_str()
                .unwrap_or("255 176 0"),
        );
        let accent = background::rgb(
            settings["palette"]["accent"]
                .as_str()
                .unwrap_or("255 215 128"),
        );
        let block = settings["ditherIntensity"]
            .as_u64()
            .unwrap_or(2)
            .clamp(1, 4) as usize;
        let art = match artwork(primary, accent, block) {
            Ok(art) => art,
            Err(error) => {
                eprintln!("Intro: {error}");
                self.show_locked();
                return;
            }
        };
        drop(settings);
        let reduced = gtk::Settings::default().is_some_and(|s| !s.is_gtk_enable_animations());
        let started = Instant::now();
        let reveal = Rc::new(Cell::new(if reduced { 1.0 } else { 0.0 }));
        let stage = box_v(8);
        stage.set_halign(gtk::Align::Center);
        stage.set_valign(gtk::Align::Center);
        stage.set_widget_name("fox-intro");
        let fox = gtk::DrawingArea::new();
        fox.set_has_window(false);
        fox.set_size_request(320, 320);
        if let Some(a) = fox.accessible() {
            use gtk::atk::prelude::*;
            a.set_name(self.tr("Zorro de SpiritByte", "SpiritByte fox"));
        }
        let progress = reveal.clone();
        fox.connect_draw(move |area, cr| {
            let _ = cr.save();
            let size = area.allocated_width().min(area.allocated_height()) as f64;
            cr.translate((area.allocated_width() as f64 - size) / 2.0, 0.0);
            cr.scale(size / 420.0, size / 420.0);
            cr.set_source_rgba(primary[0], primary[1], primary[2], 0.16);
            for (a, b) in [(0.15, 1.30), (1.72, 2.98), (3.30, 4.50), (4.88, 6.10)] {
                cr.new_sub_path();
                cr.arc(210., 210., 190., a, b);
            }
            let _ = cr.stroke();
            cr.set_source_rgba(primary[0], primary[1], primary[2], 0.4);
            for (x, y, sx, sy) in [
                (37., 37., 1., 1.),
                (383., 37., -1., 1.),
                (37., 383., 1., -1.),
                (383., 383., -1., -1.),
            ] {
                cr.move_to(x, y + sy * 15.);
                cr.line_to(x, y);
                cr.line_to(x + sx * 15., y);
            }
            let _ = cr.stroke();
            let _ = cr.save();
            cr.rectangle(0., 0., 420., 420. * progress.get());
            cr.clip();
            let _ = cr.set_source_surface(&art, 0., 0.);
            let _ = cr.paint();
            let _ = cr.restore();
            if progress.get() < 1.0 {
                cr.set_source_rgba(accent[0], accent[1], accent[2], 0.65);
                cr.rectangle(35., 420. * progress.get(), 350., 1.);
                let _ = cr.fill();
            }
            let _ = cr.restore();
            glib::Propagation::Proceed
        });
        pack(&stage, &fox);
        let title = label("SPIRITBYTE");
        title.style_context().add_class("title");
        title.set_halign(gtk::Align::Center);
        pack(&stage, &title);
        let tagline = label("PHOSPHOR FOX // SECURE VAULT");
        tagline.style_context().add_class("muted");
        tagline.set_halign(gtk::Align::Center);
        pack(&stage, &tagline);
        let log = label("");
        log.set_size_request(420, 112);
        log.set_valign(gtk::Align::Start);
        log.set_yalign(0.0);
        log.style_context().add_class("boot-log");
        pack(&stage, &log);
        let screen = box_v(0);
        screen.style_context().add_class("intro-screen");
        screen.pack_start(&stage, true, true, 0);
        self.content.pack_start(&screen, true, true, 0);
        self.content.show_all();
        let lines = [
            format!("SPIRITBYTE SYSTEM v{}", env!("CARGO_PKG_VERSION")),
            "SECURITY CORE ........ READY".into(),
            "ENCRYPTION LAYER ..... ACTIVE".into(),
            "ZERO-KNOWLEDGE ....... VERIFIED".into(),
            "SECURE STORAGE ....... MOUNTED".into(),
            "VAULT SHIELD ......... ENGAGED".into(),
            "PHOSPHOR FOX ......... ONLINE".into(),
        ];
        let weak = Rc::downgrade(self);
        let mut last_line = usize::MAX;
        let timer = glib::timeout_add_local(Duration::from_millis(40), move || {
            let Some(app) = weak.upgrade() else {
                return glib::ControlFlow::Break;
            };
            let elapsed = started.elapsed().as_secs_f64();
            if elapsed >= 3.2 {
                app.intro_timer.borrow_mut().take();
                app.show_locked(); // Drops the artwork and stops this one-shot clock.
                app.animate();
                return glib::ControlFlow::Break;
            }
            let next = if reduced {
                1.0
            } else {
                (elapsed / (3.2 * 0.65)).min(1.0)
            };
            if reveal.replace(next) != next {
                fox.queue_draw();
            }
            let count = (elapsed / (3.2 / 9.0)).floor().min(7.0) as usize;
            if count != last_line {
                last_line = count;
                log.set_text(
                    &lines[..count]
                        .iter()
                        .map(|s| format!("> {s}"))
                        .collect::<Vec<_>>()
                        .join("\n"),
                );
            }
            if elapsed > 2.75 && !reduced {
                stage.set_opacity(((3.2 - elapsed) / 0.45).clamp(0.0, 1.0));
            }
            glib::ControlFlow::Continue
        });
        *self.intro_timer.borrow_mut() = Some(timer);
    }
}

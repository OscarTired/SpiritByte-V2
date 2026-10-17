use gdk_pixbuf::{PixbufAnimation, PixbufAnimationIter, PixbufLoader};
use gtk::prelude::*;
use std::{
    cell::{Cell, RefCell},
    io::Read,
    path::Path,
    rc::Rc,
    time::{Duration, Instant, SystemTime},
};

pub enum Wallpaper {
    Native(PixbufAnimation),
    Gif(super::gif_stream::GifStream),
}

impl Wallpaper {
    fn is_animated(&self) -> bool {
        match self {
            Self::Native(a) => !a.is_static_image(),
            Self::Gif(g) => g.is_animated(),
        }
    }
}

pub fn load(path: &Path) -> Result<Wallpaper, String> {
    if std::fs::metadata(path).map_err(|e| e.to_string())?.len() > 32 * 1024 * 1024 {
        return Err("Image exceeds 32 MiB".into());
    }
    let mut header = [0; 6];
    std::fs::File::open(path)
        .map_err(|e| e.to_string())?
        .read_exact(&mut header)
        .map_err(|e| e.to_string())?;
    if &header == b"GIF87a" || &header == b"GIF89a" {
        return super::gif_stream::GifStream::open(path).map(Wallpaper::Gif);
    }
    load_gdk(path)
        .or_else(|original| {
            // WebP is not a standard GdkPixbuf loader on all distributions. Decode
            // static WebP locally so users do not need an optional system plugin.
            let mut encoded = Vec::new();
            std::fs::File::open(path)
                .map_err(|_| original.clone())?
                .take(32 * 1024 * 1024 + 1)
                .read_to_end(&mut encoded)
                .map_err(|e| e.to_string())?;
            if encoded.len() > 32 * 1024 * 1024 {
                return Err("Image exceeds 32 MiB".into());
            }
            let mut reader = image::ImageReader::new(std::io::Cursor::new(encoded))
                .with_guessed_format()
                .map_err(|_| original.clone())?;
            if reader.format() != Some(image::ImageFormat::WebP) {
                return Err(original);
            }
            let mut limits = image::Limits::default();
            limits.max_alloc = Some(64 * 1024 * 1024);
            limits.max_image_width = Some(16384);
            limits.max_image_height = Some(16384);
            reader.limits(limits);
            let decoded = reader
                .decode()
                .map_err(|e| e.to_string())?
                .thumbnail(1600, 1000)
                .into_rgba8();
            let (w, h) = decoded.dimensions();
            let bytes = glib::Bytes::from_owned(decoded.into_raw());
            let pixbuf = gdk_pixbuf::Pixbuf::from_bytes(
                &bytes,
                gdk_pixbuf::Colorspace::Rgb,
                true,
                8,
                w as i32,
                h as i32,
                w as i32 * 4,
            );
            let animation = gdk_pixbuf::PixbufSimpleAnim::new(w as i32, h as i32, 1.0);
            animation.add_frame(&pixbuf);
            Ok(animation.upcast())
        })
        .map(Wallpaper::Native)
}

fn load_gdk(path: &Path) -> Result<PixbufAnimation, String> {
    let mut file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    if file.metadata().map_err(|e| e.to_string())?.len() > 32 * 1024 * 1024 {
        return Err("Image exceeds 32 MiB".into());
    }
    let loader = PixbufLoader::new();
    let invalid = Rc::new(Cell::new(false));
    let flag = invalid.clone();
    loader.connect_size_prepared(move |loader, w, h| {
        if w <= 0 || h <= 0 || w > 16384 || h > 16384 || i64::from(w) * i64::from(h) > 64_000_000 {
            flag.set(true);
            loader.set_size(1, 1);
            return;
        }
        // Decode to a bounded surface instead of retaining a full-resolution
        // photograph plus several canvas/GPU copies.
        let scale = (1600.0 / w as f64).min(1000.0 / h as f64).min(1.0);
        loader.set_size(
            (w as f64 * scale).round().max(1.0) as i32,
            (h as f64 * scale).round().max(1.0) as i32,
        );
    });
    let result: Result<(), String> = (|| {
        let mut buf = [0u8; 16384];
        let mut total = 0;
        loop {
            let n = file.read(&mut buf).map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }
            total += n;
            if total > 32 * 1024 * 1024 {
                return Err("Image exceeds 32 MiB".into());
            }
            loader.write(&buf[..n]).map_err(|e| e.to_string())?;
            if invalid.get() {
                return Err("Image dimensions are too large".into());
            }
        }
        Ok(())
    })();
    let closed = loader.close().map_err(|e| e.to_string());
    result?;
    closed?;
    loader.animation().ok_or_else(|| "Unsupported image".into())
}

pub struct Background {
    pub area: gtk::DrawingArea,
    animation: RefCell<Option<Wallpaper>>,
    iter: RefCell<Option<PixbufAnimationIter>>,
    timer: RefCell<Option<glib::SourceId>>,
    running: Cell<bool>,
    generation: Cell<u64>,
    rendered: RefCell<Option<(i32, i32, u64, gtk::cairo::ImageSurface)>>,
    pub colors: Cell<([f64; 3], [f64; 3])>,
    pub gradient: Cell<bool>,
}

impl Background {
    pub fn new() -> Rc<Self> {
        let area = gtk::DrawingArea::new();
        area.set_hexpand(true);
        area.set_vexpand(true);
        let this = Rc::new(Self {
            area,
            animation: RefCell::new(None),
            iter: RefCell::new(None),
            timer: RefCell::new(None),
            running: Cell::new(false),
            generation: Cell::new(0),
            rendered: RefCell::new(None),
            colors: Cell::new(([0.05; 3], [0.05; 3])),
            gradient: Cell::new(false),
        });
        let weak = Rc::downgrade(&this);
        this.area.connect_draw(move |area, cr| {
            if let Some(this) = weak.upgrade() {
                let w = area.allocated_width() as f64;
                let h = area.allocated_height() as f64;
                let (a, b) = this.colors.get();
                if this.gradient.get() {
                    let gradient = gtk::cairo::LinearGradient::new(0.0, 0.0, w, h);
                    gradient.add_color_stop_rgb(0.0, a[0], a[1], a[2]);
                    gradient.add_color_stop_rgb(1.0, b[0], b[1], b[2]);
                    let _ = cr.set_source(&gradient);
                } else {
                    cr.set_source_rgb(a[0], a[1], a[2]);
                }
                let _ = cr.paint();
                let pixbuf = match this.animation.borrow().as_ref() {
                    Some(Wallpaper::Gif(g)) => Some(g.frame.clone()),
                    _ => this.iter.borrow().as_ref().map(|i| i.pixbuf()),
                };
                if let Some(pixbuf) = pixbuf {
                    let pw = pixbuf.width() as f64;
                    let ph = pixbuf.height() as f64;
                    if pw == 1.0 && ph == 1.0 {
                        let bytes = pixbuf.read_pixel_bytes();
                        let p = bytes.as_ref();
                        cr.set_source_rgba(
                            p[0] as f64 / 255.0,
                            p[1] as f64 / 255.0,
                            p[2] as f64 / 255.0,
                            if pixbuf.has_alpha() {
                                p[3] as f64 / 255.0
                            } else {
                                1.0
                            },
                        );
                        let _ = cr.paint();
                    } else if w > 0.0 && h > 0.0 && w <= 4096.0 && h <= 4096.0 {
                        let generation = this.generation.get();
                        let mut cache = this.rendered.borrow_mut();
                        if cache.as_ref().map_or(true, |(cw, ch, cg, _)| {
                            *cw != w as i32 || *ch != h as i32 || *cg != generation
                        }) {
                            let scale = (w / pw).max(h / ph);
                            let sw = (w / scale).round().clamp(1.0, pw) as i32;
                            let sh = (h / scale).round().clamp(1.0, ph) as i32;
                            let cropped = pixbuf.new_subpixbuf(
                                (pixbuf.width() - sw) / 2,
                                (pixbuf.height() - sh) / 2,
                                sw,
                                sh,
                            );
                            if let (Some(scaled), Ok(surface)) = (
                                cropped.scale_simple(
                                    w as i32,
                                    h as i32,
                                    gdk_pixbuf::InterpType::Bilinear,
                                ),
                                gtk::cairo::ImageSurface::create(
                                    gtk::cairo::Format::ARgb32,
                                    w as i32,
                                    h as i32,
                                ),
                            ) {
                                if let Ok(context) = gtk::cairo::Context::new(&surface) {
                                    context.set_source_pixbuf(&scaled, 0.0, 0.0);
                                    let _ = context.paint();
                                }
                                *cache = Some((w as i32, h as i32, generation, surface));
                            }
                        }
                        // Reuse the scaled, converted surface for GTK exposes and
                        // CRT pulses. Retain just the current frame, not a window-
                        // sized copy of every GIF frame.
                        if let Some((_, _, _, surface)) = cache.as_ref() {
                            let _ = cr.set_source_surface(surface, 0.0, 0.0);
                            let _ = cr.paint();
                        }
                    }
                }
            }
            glib::Propagation::Proceed
        });
        this
    }

    pub fn set_image(self: &Rc<Self>, animation: Option<Wallpaper>) {
        self.stop();
        self.rendered.borrow_mut().take();
        self.generation.set(self.generation.get().wrapping_add(1));
        *self.iter.borrow_mut() = animation.as_ref().and_then(|a| match a {
            Wallpaper::Native(a) => Some(a.iter(Some(SystemTime::now()))),
            _ => None,
        });
        *self.animation.borrow_mut() = animation;
        self.area.queue_draw();
    }

    pub fn resume(self: &Rc<Self>, run: bool) {
        self.stop();
        if !run
            || self
                .animation
                .borrow()
                .as_ref()
                .map_or(true, |a| !a.is_animated())
        {
            return;
        }
        self.running.set(true);
        if let Some(Wallpaper::Gif(g)) = self.animation.borrow_mut().as_mut() {
            g.resume();
        }
        self.schedule();
    }

    fn schedule(self: &Rc<Self>) {
        let delay = if let Some(Wallpaper::Gif(g)) = self.animation.borrow().as_ref() {
            g.delay()
        } else {
            self.iter
                .borrow()
                .as_ref()
                .and_then(|i| i.delay_time())
                .unwrap_or(Duration::from_millis(100))
                .max(Duration::from_millis(50))
        };
        let weak = Rc::downgrade(self);
        let id = glib::timeout_add_local_once(delay, move || {
            if let Some(this) = weak.upgrade() {
                this.timer.borrow_mut().take();
                if this.running.get() {
                    let mut changed = false;
                    let mut damage = None;
                    if let Some(Wallpaper::Gif(g)) = this.animation.borrow_mut().as_mut() {
                        match g.advance(Instant::now()) {
                            Ok(value) => changed = value,
                            Err(error) => {
                                eprintln!("GIF: {error}");
                                this.running.set(false);
                            }
                        }
                        if let Some((x, y, dw, dh)) = g.damage {
                            let (pw, ph) = (g.frame.width() as f64, g.frame.height() as f64);
                            let (w, h) = (
                                this.area.allocated_width() as f64,
                                this.area.allocated_height() as f64,
                            );
                            let scale = (w / pw).max(h / ph);
                            let (sw, sh) = (
                                (w / scale).round().clamp(1.0, pw),
                                (h / scale).round().clamp(1.0, ph),
                            );
                            let (ox, oy) = (((pw - sw) / 2.0).floor(), ((ph - sh) / 2.0).floor());
                            let left =
                                (((x as f64 - ox) * w / sw).floor() - 2.0).clamp(0.0, w) as i32;
                            let top =
                                (((y as f64 - oy) * h / sh).floor() - 2.0).clamp(0.0, h) as i32;
                            let right = ((((x + dw) as f64 - ox) * w / sw).ceil() + 2.0)
                                .clamp(0.0, w) as i32;
                            let bottom = ((((y + dh) as f64 - oy) * h / sh).ceil() + 2.0)
                                .clamp(0.0, h) as i32;
                            damage = Some((left, top, right - left, bottom - top));
                        }
                        if !g.is_animated() {
                            this.running.set(false);
                        }
                    }
                    if let Some(iter) = this.iter.borrow().as_ref() {
                        if iter.advance(SystemTime::now()) {
                            changed = true;
                        }
                    }
                    if changed {
                        this.generation.set(this.generation.get().wrapping_add(1));
                        if let Some((x, y, w, h)) = damage {
                            if w > 0 && h > 0 {
                                this.area.queue_draw_area(x, y, w, h);
                            }
                        } else {
                            this.area.queue_draw();
                        }
                    }
                    if this.running.get() {
                        this.schedule();
                    }
                }
            }
        });
        *self.timer.borrow_mut() = Some(id);
    }

    fn stop(&self) {
        self.running.set(false);
        if let Some(id) = self.timer.borrow_mut().take() {
            id.remove();
        }
    }
}

pub fn rgb(value: &str) -> [f64; 3] {
    if let Some(hex) = value.strip_prefix('#') {
        if hex.len() == 6 {
            if let Ok(n) = u32::from_str_radix(hex, 16) {
                return [
                    ((n >> 16) & 255) as f64 / 255.0,
                    ((n >> 8) & 255) as f64 / 255.0,
                    (n & 255) as f64 / 255.0,
                ];
            }
        }
    }
    let values: Vec<f64> = value
        .split_whitespace()
        .filter_map(|v| v.parse::<f64>().ok())
        .collect();
    if values.len() == 3 {
        [
            values[0].clamp(0.0, 255.0) / 255.0,
            values[1].clamp(0.0, 255.0) / 255.0,
            values[2].clamp(0.0, 255.0) / 255.0,
        ]
    } else {
        [0.05; 3]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn supported_wallpapers_and_animation() {
        let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("../src-tauri/examples/fixtures");
        for ext in ["png", "jpg", "gif", "webp", "bmp"] {
            let files: Vec<_> = std::fs::read_dir(&base)
                .unwrap()
                .flatten()
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|s| s == ext))
                .collect();
            assert!(!files.is_empty());
            for path in files {
                assert!(load(&path).is_ok(), "{}", path.display());
            }
        }
        let Wallpaper::Gif(mut animation) = load(&base.join("wallpaper.animated.gif")).unwrap()
        else {
            panic!("GIF must use bounded decoder")
        };
        assert!(animation.is_animated());
        let first = animation.frame.read_pixel_bytes();
        assert!(animation
            .advance(Instant::now() + Duration::from_millis(150))
            .unwrap());
        assert_ne!(first.as_ref(), animation.frame.read_pixel_bytes().as_ref());
    }
}

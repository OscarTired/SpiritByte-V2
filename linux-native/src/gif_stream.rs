//! Incremental LZW decoding and native pixel compositing, with bounded storage.
use gdk_pixbuf::{Colorspace, InterpType, Pixbuf};
use std::{
    fs::File,
    io::{BufReader, Seek, SeekFrom},
    path::Path,
    time::{Duration, Instant},
};

fn decoder(file: File) -> Result<gif::Decoder<BufReader<File>>, String> {
    let mut options = gif::DecodeOptions::new();
    options.set_color_output(gif::ColorOutput::RGBA);
    options.set_memory_limit(gif::MemoryLimit::Bytes(
        (16 * 1024 * 1024).try_into().unwrap(),
    ));
    options.check_frame_consistency(true);
    let reader = options
        .read_info(BufReader::new(file))
        .map_err(|e| e.to_string())?;
    let (w, h) = (u32::from(reader.width()), u32::from(reader.height()));
    if w == 0 || h == 0 || w > 4096 || h > 4096 || w * h > 4_000_000 {
        return Err("GIF dimensions exceed the native animation memory limit".into());
    }
    Ok(reader)
}

type Rect = (i32, i32, i32, i32);
#[derive(Clone, Copy)]
struct Info {
    rect: Rect,
    delay: Duration,
    dispose: gif::DisposalMethod,
    transparent: bool,
}
impl Info {
    fn from(frame: &gif::Frame<'_>) -> Result<Self, String> {
        if frame.width == 0 || frame.height == 0 {
            return Err("Empty GIF frame".into());
        }
        Ok(Self {
            rect: (
                frame.left.into(),
                frame.top.into(),
                frame.width.into(),
                frame.height.into(),
            ),
            delay: Duration::from_millis(if frame.delay == 0 {
                100
            } else {
                u64::from(frame.delay) * 10
            }),
            dispose: frame.dispose,
            transparent: frame.transparent.is_some(),
        })
    }
}
fn union(a: Option<Rect>, b: Rect) -> Option<Rect> {
    Some(if let Some(a) = a {
        let (x, y) = (a.0.min(b.0), a.1.min(b.1));
        (
            x,
            y,
            (a.0 + a.2).max(b.0 + b.2) - x,
            (a.1 + a.3).max(b.1 + b.3) - y,
        )
    } else {
        b
    })
}

pub struct GifStream {
    source: File,
    reader: gif::Decoder<BufReader<File>>,
    pending: Option<Info>,
    previous: Option<Info>,
    restore: Option<Pixbuf>,
    pub frame: Pixbuf,
    pub damage: Option<Rect>,
    delay: Duration,
    next_at: Instant,
    animated: bool,
    remaining_loops: Option<u32>,
}
impl GifStream {
    pub fn open(path: &Path) -> Result<Self, String> {
        let source = File::open(path).map_err(|e| e.to_string())?;
        let reader = decoder(source.try_clone().map_err(|e| e.to_string())?)?;
        let frame = Pixbuf::new(
            Colorspace::Rgb,
            true,
            8,
            reader.width().into(),
            reader.height().into(),
        )
        .ok_or("Cannot allocate GIF canvas")?;
        frame.fill(0);
        let mut this = Self {
            source,
            reader,
            frame,
            pending: None,
            previous: None,
            restore: None,
            damage: None,
            delay: Duration::from_millis(100),
            next_at: Instant::now(),
            animated: true,
            remaining_loops: Some(1),
        };
        this.decode_next()?;
        this.pending = this
            .reader
            .next_frame_info()
            .map_err(|e| e.to_string())?
            .map(Info::from)
            .transpose()?;
        this.animated = this.pending.is_some();
        this.remaining_loops = match this.reader.repeat() {
            gif::Repeat::Infinite => None,
            gif::Repeat::Finite(n) => Some(u32::from(n) + 1),
        };
        this.next_at = Instant::now() + this.delay;
        Ok(this)
    }

    fn decode_next(&mut self) -> Result<bool, String> {
        let mut info = self.pending.take();
        if info.is_none() {
            info = self
                .reader
                .next_frame_info()
                .map_err(|e| e.to_string())?
                .map(Info::from)
                .transpose()?;
        }
        let mut reset = false;
        if info.is_none() {
            if let Some(count) = self.remaining_loops.as_mut() {
                if *count <= 1 {
                    self.animated = false;
                    return Ok(false);
                }
                *count -= 1;
            }
            self.source
                .seek(SeekFrom::Start(0))
                .map_err(|e| e.to_string())?;
            self.reader = decoder(self.source.try_clone().map_err(|e| e.to_string())?)?;
            info = self
                .reader
                .next_frame_info()
                .map_err(|e| e.to_string())?
                .map(Info::from)
                .transpose()?;
            reset = true;
        }
        let info = info.ok_or("Empty GIF")?;
        let (x, y, w, h) = info.rect;
        let mut buffer = vec![0; self.reader.buffer_size()];
        self.reader
            .read_into_buffer(&mut buffer)
            .map_err(|e| e.to_string())?;
        let bytes = glib::Bytes::from_owned(buffer);
        let pixels = Pixbuf::from_bytes(&bytes, Colorspace::Rgb, true, 8, w, h, w * 4);
        if reset {
            self.frame.fill(0);
            self.previous = None;
            self.restore = None;
            self.damage = Some((0, 0, self.frame.width(), self.frame.height()));
        }
        if let Some(previous) = self.previous {
            let (px, py, pw, ph) = previous.rect;
            match previous.dispose {
                gif::DisposalMethod::Background => {
                    self.frame.new_subpixbuf(px, py, pw, ph).fill(0);
                    self.damage = union(self.damage, previous.rect);
                }
                gif::DisposalMethod::Previous => {
                    if let Some(saved) = self.restore.take() {
                        saved.copy_area(0, 0, pw, ph, &self.frame, px, py);
                    }
                    self.damage = union(self.damage, previous.rect);
                }
                _ => {}
            }
        }
        self.restore = if info.dispose == gif::DisposalMethod::Previous {
            Some(
                self.frame
                    .new_subpixbuf(x, y, w, h)
                    .copy()
                    .ok_or("Cannot preserve GIF disposal rectangle")?,
            )
        } else {
            None
        };
        // Native row compositing avoids walking/copying the whole image for
        // every small GIF patch. Transparent pixels preserve the old canvas.
        if info.transparent {
            pixels.composite(
                &self.frame,
                x,
                y,
                w,
                h,
                x as f64,
                y as f64,
                1.,
                1.,
                InterpType::Nearest,
                255,
            );
        } else {
            pixels.copy_area(0, 0, w, h, &self.frame, x, y);
        }
        self.previous = Some(info);
        self.delay = info.delay;
        self.damage = union(self.damage, info.rect);
        Ok(true)
    }

    pub fn is_animated(&self) -> bool {
        self.animated
    }
    pub fn resume(&mut self) {
        self.next_at = Instant::now() + self.delay;
    }
    pub fn delay(&self) -> Duration {
        self.next_at
            .saturating_duration_since(Instant::now())
            .max(Duration::from_millis(50))
    }
    pub fn advance(&mut self, now: Instant) -> Result<bool, String> {
        if !self.animated || now < self.next_at {
            return Ok(false);
        }
        self.damage = None;
        let mut changed = false;
        for _ in 0..8 {
            if !self.decode_next()? {
                break;
            }
            changed = true;
            self.next_at += self.delay;
            if self.next_at > now {
                break;
            }
        }
        if changed && self.next_at <= now {
            self.next_at = now + self.delay;
        }
        Ok(changed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Cow;

    #[test]
    fn transparent_patches_restore_previous_and_clear_background() {
        for disposal in [
            gif::DisposalMethod::Previous,
            gif::DisposalMethod::Background,
        ] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("patches.gif");
            {
                let file = File::create(&path).unwrap();
                let mut encoder =
                    gif::Encoder::new(file, 4, 3, &[255, 0, 0, 0, 0, 255, 0, 255, 0, 0, 0, 0])
                        .unwrap();
                encoder.set_repeat(gif::Repeat::Infinite).unwrap();
                for (left, top, width, height, buffer, dispose, transparent) in [
                    (0, 0, 4, 3, vec![0; 12], gif::DisposalMethod::Keep, None),
                    (0, 0, 2, 2, vec![1, 3, 3, 1], disposal, Some(3)),
                    (3, 2, 1, 1, vec![2], gif::DisposalMethod::Keep, None),
                ] {
                    encoder
                        .write_frame(&gif::Frame {
                            left,
                            top,
                            width,
                            height,
                            buffer: Cow::Owned(buffer),
                            delay: 10,
                            dispose,
                            transparent,
                            ..Default::default()
                        })
                        .unwrap();
                }
            }
            let mut stream = GifStream::open(&path).unwrap();
            let start = Instant::now();
            stream.advance(start + Duration::from_millis(110)).unwrap();
            let bytes = stream.frame.read_pixel_bytes();
            assert_eq!(&bytes.as_ref()[0..4], &[0, 0, 255, 255]);
            assert_eq!(&bytes.as_ref()[4..8], &[255, 0, 0, 255]);
            stream.advance(start + Duration::from_millis(210)).unwrap();
            let bytes = stream.frame.read_pixel_bytes();
            let expected = if disposal == gif::DisposalMethod::Previous {
                [255, 0, 0, 255]
            } else {
                [0, 0, 0, 0]
            };
            assert_eq!(&bytes.as_ref()[0..4], &expected);
            let i = 2 * stream.frame.rowstride() as usize + 3 * 4;
            assert_eq!(&bytes.as_ref()[i..i + 4], &[0, 255, 0, 255]);
            stream.advance(start + Duration::from_millis(310)).unwrap();
            let bytes = stream.frame.read_pixel_bytes();
            assert_eq!(&bytes.as_ref()[i..i + 4], &[255, 0, 0, 255]); // loop starts cleanly
        }
    }
}

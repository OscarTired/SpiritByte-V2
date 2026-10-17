//! Isolate incremental GIF decoding from GTK drawing for CPU diagnosis.
#[path = "../src/gif_stream.rs"]
mod gif_stream;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
fn main() {
    let path = PathBuf::from(std::env::args_os().nth(1).expect("GIF path"));
    let mut gif = gif_stream::GifStream::open(&path).unwrap();
    gif.resume();
    let start = Instant::now();
    for i in 1..=200 {
        gif.advance(start + Duration::from_millis(i * 50)).unwrap();
    }
    println!(
        "{}",
        serde_json::json!({"simulated_ms":10000,"decode_ms":start.elapsed().as_secs_f64()*1000.0,"animated":gif.is_animated(),"delay_ms":gif.delay().as_millis()})
    );
}

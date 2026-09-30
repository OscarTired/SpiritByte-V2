//! Configurable password generator + strength estimation.

use rand::{rngs::OsRng, seq::SliceRandom, Rng, RngCore};
use serde::{Deserialize, Serialize};

const LOWER: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
const UPPER: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const DIGITS: &[u8] = b"0123456789";
const SYMBOLS: &[u8] = b"!@#$%^&*()-_=+[]{};:,.<>?/";
// Visually ambiguous characters removed when `avoid_ambiguous` is set.
const AMBIGUOUS: &[u8] = b"O0ol1I|`'\"{}[]";

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GenOptions {
    pub length: usize,
    pub lowercase: bool,
    pub uppercase: bool,
    pub digits: bool,
    pub symbols: bool,
    #[serde(default)]
    pub avoid_ambiguous: bool,
}

impl Default for GenOptions {
    fn default() -> Self {
        Self {
            length: 20,
            lowercase: true,
            uppercase: true,
            digits: true,
            symbols: true,
            avoid_ambiguous: false,
        }
    }
}

fn filter_ambiguous(set: &[u8], avoid: bool) -> Vec<u8> {
    if !avoid {
        return set.to_vec();
    }
    set.iter().copied().filter(|c| !AMBIGUOUS.contains(c)).collect()
}

/// Generates a random password honoring the selected character classes.
/// Guarantees at least one character from each enabled class.
pub fn generate(opts: &GenOptions) -> String {
    let length = opts.length.clamp(4, 256);
    let mut classes: Vec<Vec<u8>> = Vec::new();
    if opts.lowercase {
        classes.push(filter_ambiguous(LOWER, opts.avoid_ambiguous));
    }
    if opts.uppercase {
        classes.push(filter_ambiguous(UPPER, opts.avoid_ambiguous));
    }
    if opts.digits {
        classes.push(filter_ambiguous(DIGITS, opts.avoid_ambiguous));
    }
    if opts.symbols {
        classes.push(filter_ambiguous(SYMBOLS, opts.avoid_ambiguous));
    }
    if classes.is_empty() {
        classes.push(filter_ambiguous(LOWER, opts.avoid_ambiguous));
    }

    let pool: Vec<u8> = classes.iter().flatten().copied().collect();
    let mut out: Vec<u8> = Vec::with_capacity(length);

    // One guaranteed char per enabled class (up to length).
    for class in classes.iter().take(length) {
        let idx = (OsRng.next_u32() as usize) % class.len();
        out.push(class[idx]);
    }
    while out.len() < length {
        let idx = (OsRng.next_u32() as usize) % pool.len();
        out.push(pool[idx]);
    }
    out.shuffle(&mut OsRng);
    String::from_utf8(out).unwrap_or_default()
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Strength {
    /// 0..=4 (very weak .. very strong)
    pub score: u8,
    pub entropy_bits: f64,
    pub label: String,
}

/// Estimates password strength from character-pool entropy.
pub fn strength(password: &str) -> Strength {
    if password.is_empty() {
        return Strength {
            score: 0,
            entropy_bits: 0.0,
            label: "empty".into(),
        };
    }
    let mut pool = 0u32;
    if password.chars().any(|c| c.is_ascii_lowercase()) {
        pool += 26;
    }
    if password.chars().any(|c| c.is_ascii_uppercase()) {
        pool += 26;
    }
    if password.chars().any(|c| c.is_ascii_digit()) {
        pool += 10;
    }
    if password.chars().any(|c| !c.is_ascii_alphanumeric()) {
        pool += 33;
    }
    let pool = pool.max(1) as f64;
    let entropy_bits = (password.chars().count() as f64) * pool.log2();
    let (score, label) = match entropy_bits {
        b if b < 28.0 => (0u8, "very weak"),
        b if b < 40.0 => (1, "weak"),
        b if b < 60.0 => (2, "fair"),
        b if b < 80.0 => (3, "strong"),
        _ => (4, "very strong"),
    };
    Strength {
        score,
        entropy_bits,
        label: label.into(),
    }
}

#[allow(dead_code)]
fn _assert_rng_usable() {
    // Keeps `Rng` import meaningful across rand versions.
    let _ = OsRng.gen::<u8>();
}

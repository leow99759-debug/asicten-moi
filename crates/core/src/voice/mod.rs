//! Voice output level 1 (SPEC §6.1): phrase packs of recorded WAVs, picked by category.
//!
//! Layout: `<pack>/<lang>/{category}/*.wav` (+ optional `<pack>/voice.json`).
//! Flat files like Priler's `ru/ok1.wav` also work: category = stem without trailing digits.

#[cfg(windows)]
mod player;
#[cfg(windows)]
pub use player::Player;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::{Error, Result};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct VoiceMeta {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub author: String,
    /// Spoken text per file name (`ok/yes_sir.wav` → «Да, сэр»), for UI and STT checks.
    #[serde(default)]
    pub texts: BTreeMap<String, String>,
}

pub struct VoicePack {
    pub meta: VoiceMeta,
    clips: BTreeMap<String, Vec<PathBuf>>,
    /// Normalized spoken text → clip (from `voice.json` texts).
    by_text: BTreeMap<String, PathBuf>,
    last: Mutex<BTreeMap<String, usize>>,
    rng: Mutex<u64>,
}

fn is_wav(p: &Path) -> bool {
    p.extension().is_some_and(|e| e.eq_ignore_ascii_case("wav"))
}

fn sorted_entries(dir: &Path) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .collect();
    v.sort();
    v
}

impl VoicePack {
    /// Load `<root>/<lang>`; errors if it holds no clips.
    pub fn load(root: &Path, lang: &str) -> Result<Self> {
        let meta = std::fs::read_to_string(root.join("voice.json"))
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default();
        let mut clips: BTreeMap<String, Vec<PathBuf>> = BTreeMap::new();
        for p in sorted_entries(&root.join(lang)) {
            if p.is_dir() {
                let cat = p.file_name().map(|n| n.to_string_lossy().into_owned());
                let files: Vec<PathBuf> = sorted_entries(&p)
                    .into_iter()
                    .filter(|f| is_wav(f))
                    .collect();
                if let (Some(cat), false) = (cat, files.is_empty()) {
                    clips.entry(cat).or_default().extend(files);
                }
            } else if is_wav(&p) {
                let stem = p
                    .file_stem()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_default();
                let cat = stem
                    .trim_end_matches(|c: char| c.is_ascii_digit())
                    .to_owned();
                clips.entry(cat).or_default().push(p);
            }
        }
        if clips.is_empty() {
            return Err(Error::Audio(format!(
                "no voice clips in {}",
                root.join(lang).display()
            )));
        }
        let meta: VoiceMeta = meta;
        let by_text = meta
            .texts
            .iter()
            .map(|(file, text)| (norm_text(text), root.join(lang).join(file)))
            .filter(|(_, p)| p.exists())
            .collect();
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x9E37_79B9);
        Ok(Self {
            meta,
            clips,
            by_text,
            last: Mutex::default(),
            rng: Mutex::new(seed | 1),
        })
    }

    pub fn categories(&self) -> impl Iterator<Item = &str> {
        self.clips.keys().map(String::as_str)
    }

    /// Recording of exactly this text (pre-generated replies, §6.1), punctuation/case-insensitive.
    pub fn by_text(&self, text: &str) -> Option<&Path> {
        self.by_text.get(&norm_text(text)).map(PathBuf::as_path)
    }

    pub fn has(&self, category: &str) -> bool {
        self.clips.contains_key(category)
    }

    /// Random clip of the first category that exists, never the same file twice in a row.
    pub fn pick(&self, categories: &[impl AsRef<str>]) -> Option<&Path> {
        let (cat, files) = categories
            .iter()
            .find_map(|c| self.clips.get_key_value(c.as_ref()))?;
        let mut last = self.last.lock().unwrap_or_else(|e| e.into_inner());
        let prev = last.get(cat).copied();
        let mut i = self.next() as usize % files.len();
        if files.len() > 1 && Some(i) == prev {
            i = (i + 1) % files.len();
        }
        last.insert(cat.clone(), i);
        Some(&files[i])
    }

    fn next(&self) -> u64 {
        // xorshift64: plenty for picking a phrase
        let mut s = self.rng.lock().unwrap_or_else(|e| e.into_inner());
        *s ^= *s << 13;
        *s ^= *s >> 7;
        *s ^= *s << 17;
        *s
    }
}

/// Lowercase letters/digits only, single spaces, ё→е: the key for text lookups.
pub fn norm_text(text: &str) -> String {
    text.to_lowercase()
        .replace('ё', "е")
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Decode a WAV file to mono f32 and its sample rate.
pub fn read_wav(path: &Path) -> Result<(Vec<f32>, u32)> {
    let f = std::fs::File::open(path)?;
    decode_wav(std::io::BufReader::new(f))
        .map_err(|e| Error::Audio(format!("{}: {e}", path.display())))
}

/// Decode WAV bytes (files, Windows voice output) to mono f32 and sample rate.
pub fn decode_wav(reader: impl std::io::Read) -> Result<(Vec<f32>, u32)> {
    let mut r = hound::WavReader::new(reader).map_err(|e| Error::Audio(e.to_string()))?;
    let spec = r.spec();
    let ch = usize::from(spec.channels.max(1));
    let interleaved: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => r.samples::<f32>().filter_map(|s| s.ok()).collect(),
        hound::SampleFormat::Int => {
            let scale = (1_i64 << (spec.bits_per_sample.clamp(1, 32) - 1)) as f32;
            r.samples::<i32>()
                .filter_map(|s| s.ok())
                .map(|s| s as f32 / scale)
                .collect()
        }
    };
    let mono = interleaved
        .chunks_exact(ch)
        .map(|f| f.iter().sum::<f32>() / ch as f32)
        .collect();
    Ok((mono, spec.sample_rate))
}

/// Reply texts may hold variants «a|b|c»; pick one at random.
pub fn pick_variant(text: &str) -> &str {
    let parts: Vec<&str> = text
        .split('|')
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .collect();
    let n = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as usize)
        .unwrap_or(0);
    parts.get(n % parts.len().max(1)).copied().unwrap_or(text)
}

/// Words for a clip category, used when the category has no recording or the
/// Windows voice is selected (§6.1 categories).
pub fn category_text(category: &str) -> Option<&'static str> {
    Some(match category {
        "greet" | "run" => "Джарвис к вашим услугам, сэр",
        "reply" => "Да, сэр?",
        "ok" => "Да, сэр",
        "loading" => "Загружаю, сэр",
        "done" => "Запрос выполнен, сэр",
        "ready" => "Всегда к вашим услугам, сэр",
        "thanks" => "Всегда рад помочь, сэр",
        "status" => "Все системы работают нормально, сэр",
        "not_found" => "Простите, сэр, не понял команду",
        "no_internet" => crate::brain::NO_INTERNET_PHRASE,
        "cancel" => "Отменено, сэр",
        "error" => "Сэр, не удалось выполнить",
        "off" | "goodbye" => "До свидания, сэр",
        "game_mode" | "calibration" => "Начинаю калибровку, сэр",
        _ => return None,
    })
}

/// Linear resampler for playback (speech clips; quality is fine for voice).
pub fn resample(samples: &[f32], from: u32, to: u32) -> Vec<f32> {
    if from == to || samples.is_empty() || from == 0 || to == 0 {
        return samples.to_vec();
    }
    let step = f64::from(from) / f64::from(to);
    let n = ((samples.len() as f64) / step).floor() as usize;
    (0..n)
        .map(|i| {
            let pos = i as f64 * step;
            let j = pos as usize;
            let t = (pos - j as f64) as f32;
            let a = samples[j];
            let b = samples.get(j + 1).copied().unwrap_or(a);
            a + (b - a) * t
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wav(path: &Path, rate: u32, n: usize) {
        std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        let spec = hound::WavSpec {
            channels: 2,
            sample_rate: rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut w = hound::WavWriter::create(path, spec).expect("create");
        for i in 0..n {
            let s = if i % 2 == 0 { 16_384 } else { -16_384 };
            w.write_sample(s as i16).expect("l");
            w.write_sample(s as i16).expect("r");
        }
        w.finalize().expect("finalize");
    }

    #[test]
    fn loads_folders_and_flat_files_and_never_repeats() {
        let dir = tempfile::tempdir().expect("tmp");
        let ru = dir.path().join("ru");
        for f in [
            "ok/a.wav",
            "ok/b.wav",
            "ok/c.wav",
            "done/x.wav",
            "thanks1.wav",
            "thanks2.wav",
        ] {
            wav(&ru.join(f), 22_050, 10);
        }
        std::fs::write(ru.join("ok/readme.txt"), "x").expect("txt");
        std::fs::write(
            dir.path().join("voice.json"),
            r#"{"id":"jarvis","name":"Джарвис","texts":{"done/x.wav":"Запрос выполнен, сэр!","ok/gone.wav":"нет файла"}}"#,
        )
        .expect("json");
        let p = VoicePack::load(dir.path(), "ru").expect("load");
        assert_eq!(p.meta.name, "Джарвис");
        assert_eq!(
            p.categories().collect::<Vec<_>>(),
            vec!["done", "ok", "thanks"]
        );
        let mut prev = None;
        for _ in 0..50 {
            let c = p.pick(&["ok"]).expect("pick").to_path_buf();
            assert_ne!(Some(c.clone()), prev);
            prev = Some(c);
        }
        assert!(p
            .pick(&["missing", "done"])
            .expect("fallback")
            .ends_with("x.wav"));
        assert!(p.pick(&["missing"]).is_none());
        assert!(p
            .by_text("запрос выполнён сэр")
            .expect("text")
            .ends_with("x.wav"));
        assert!(p.by_text("нет файла").is_none());
        assert!(VoicePack::load(dir.path(), "en").is_err());
    }

    #[test]
    fn variants() {
        assert_eq!(pick_variant("да сэр"), "да сэр");
        assert!(["а", "б"].contains(&pick_variant("а | б")));
    }

    #[test]
    fn wav_decode_and_resample() {
        let dir = tempfile::tempdir().expect("tmp");
        let f = dir.path().join("a.wav");
        wav(&f, 22_050, 2205);
        let (s, rate) = read_wav(&f).expect("read");
        assert_eq!((s.len(), rate), (2205, 22_050));
        assert!((s[0] - 0.5).abs() < 1e-3);
        let up = resample(&s, 22_050, 48_000);
        assert!((up.len() as i64 - 4800).abs() <= 1);
        assert!(up.iter().all(|x| x.abs() <= 0.5 + 1e-3));
    }

    #[test]
    fn priler_pack_loads_if_present() {
        let root = crate::test_util::asset("voices-priler/jarvis-og");
        let Some(root) = root else { return };
        let p = VoicePack::load(&root, "ru").expect("load");
        assert!(p.has("ok") && p.has("not_found"));
        let (s, rate) = read_wav(p.pick(&["ok"]).expect("ok")).expect("wav");
        assert!(rate >= 16_000 && !s.is_empty());
    }

    /// Every fixed reply in the repo packs has a recording: category clip or exact text (T044).
    #[test]
    fn curated_pack_covers_repo_replies() {
        let Some(root) = crate::test_util::asset("voice-jarvis") else {
            return;
        };
        let p = VoicePack::load(&root, "ru").expect("load");
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packs");
        let mut missing = Vec::new();
        for pack in crate::commands::addons(&dir, &[])
            .into_iter()
            .map(|a| a.pack)
        {
            for c in pack.commands {
                let r = &c.reply;
                if r.clips.iter().any(|k| p.has(k)) {
                    continue;
                }
                for t in r.text.iter().flat_map(|t| t.split('|')) {
                    if p.by_text(t).is_none() {
                        missing.push(format!("{}: {t}", c.id));
                    }
                }
            }
        }
        assert!(missing.is_empty(), "{missing:#?}");
        for cat in [
            "reply",
            "ok",
            "loading",
            "done",
            "ready",
            "thanks",
            "not_found",
            "cancel",
            "error",
            "no_internet",
            "greet",
            "calibration",
        ] {
            assert!(p.has(cat), "{cat}");
        }
    }
}

"""Build the Jarvis voice pack (SPEC §6.1) → voices/jarvis/{voice.json, ru/<category>/*.wav}.

Usage (from repo root, after tools/fetch-assets.ps1):
  FISH_API_KEY=... python tools/voice/build_pack.py <fish_cache_dir> <out_dir>
1. Originals (tools/voice/originals.tsv) from assets/jarvis-sound + assets/voices-priler/jarvis-og.
2. Missing phrases via Fish Audio: fish_extra.tsv + every fixed reply text in packs/*.json
   that no original covers (text match ignoring case/punctuation). Cached in <fish_cache_dir>.
3. Every clip: 60 Hz high-pass, silence trimmed, peak -1 dBFS, 22.05 kHz mono 16-bit.
Needs ffmpeg in PATH. Stdlib only.
"""

import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
ASSETS = ROOT / "assets"
# detection=peak: ffmpeg 5.1 rms detection can eat the first word of a clip
CLEAN = (
    "highpass=f=60,"
    "silenceremove=start_periods=1:start_threshold=-50dB:detection=peak:stop_periods=-1:"
    "stop_threshold=-50dB:stop_duration=0.3,afade=t=in:d=0.01"
)


def norm(t: str) -> str:
    t = t.lower().replace("ё", "е")
    return " ".join(w for w in re.split(r"[^\w]+", t) if w)


def rows(path: Path):
    for line in path.read_text(encoding="utf-8").splitlines():
        if line.strip() and not line.startswith("#"):
            yield line.split("\t")


def source(ref: str) -> Path:
    kind, name = ref.split(":", 1)
    if kind == "js":
        return ASSETS / "jarvis-sound" / f"{name}.wav"
    if kind == "og":
        return ASSETS / "voices-priler" / "jarvis-og" / "ru" / f"{name}.wav"
    raise ValueError(ref)


def peak_db(path: Path) -> float:
    out = subprocess.run(
        ["ffmpeg", "-hide_banner", "-i", str(path), "-af", CLEAN + ",volumedetect", "-f", "null", "-"],
        capture_output=True, text=True, check=True,
    ).stderr
    m = re.search(r"max_volume: (-?[\d.]+) dB", out)
    return float(m.group(1)) if m else 0.0


def clean(src: Path, dst: Path) -> None:
    dst.parent.mkdir(parents=True, exist_ok=True)
    gain = -1.0 - peak_db(src)
    subprocess.run(
        ["ffmpeg", "-loglevel", "error", "-y", "-i", str(src), "-af", f"{CLEAN},volume={gain:.2f}dB",
         "-ar", "22050", "-ac", "1", "-sample_fmt", "s16", str(dst)],
        check=True,
    )


CONFIRM = {"System.Shutdown", "System.Restart", "System.Logoff"}


def pack_texts() -> list[tuple[str, str]]:
    """(category hint, text) for every fixed reply variant in packs/*.json (+ add-ons) and the
    confirm question of dangerous commands (brain.rs: «Сэр, выполнить «name»?»)."""
    out = []
    files = sorted((ROOT / "packs").glob("*.json")) + sorted((ROOT / "packs" / "addons").glob("*.json"))
    for f in files:
        for c in json.loads(f.read_text(encoding="utf-8"))["commands"]:
            r = c.get("reply") or {}
            for v in (r.get("text") or "").split("|"):
                if v.strip():
                    out.append(("phrases", v.strip()))
            if c.get("confirm") or any(a.get("type") in CONFIRM for a in c.get("actions", [])):
                out.append(("phrases", f"Сэр, выполнить «{c['name'].lower()}»?"))
    return out


def main() -> None:
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    cache, out = Path(sys.argv[1]), Path(sys.argv[2])
    ru = out / "ru"
    texts: dict[str, str] = {}
    covered = set()
    for ref, cat, text in rows(HERE / "originals.tsv"):
        stem = ref.replace(":", "_")
        rel = f"{cat}/{stem}.wav"
        clean(source(ref), ru / rel)
        texts[rel] = text
        covered.add(norm(text))
    wanted = [(c, s, t) for c, s, t in rows(HERE / "fish_extra.tsv")]
    covered |= {norm(t) for _, _, t in wanted}
    for cat, text in pack_texts():
        if norm(text) not in covered:
            covered.add(norm(text))
            stem = "t_" + hashlib.sha1(norm(text).encode()).hexdigest()[:10]
            wanted.append((cat, stem, text))
    tsv = cache / "wanted.tsv"
    cache.mkdir(parents=True, exist_ok=True)
    tsv.write_text("".join(f"{c}\t{s}\t{t}\n" for c, s, t in wanted), encoding="utf-8")
    subprocess.run([sys.executable, str(HERE / "fish_gen.py"), str(tsv), str(cache)], check=True)
    for cat, stem, text in wanted:
        rel = f"{cat}/{stem}.wav"
        clean(cache / rel, ru / rel)
        texts[rel] = text
    meta = {"id": "jarvis-film", "name": "Джарвис (фильм)", "author": "RU dub originals + Fish Audio", "texts": texts}
    (out / "voice.json").write_text(json.dumps(meta, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")
    print(f"{len(texts)} clips → {out}")


if __name__ == "__main__":
    main()

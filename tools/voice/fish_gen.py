"""Generate missing Jarvis phrases with Fish Audio (SPEC §6.1, build time only).

Usage: FISH_API_KEY=... [FISH_VOICE=id] python tools/voice/fish_gen.py phrases.tsv out_dir
phrases.tsv: `category<TAB>file_stem<TAB>text` per line. Existing files are skipped.
Originals (Jarvis-Sound, Priler, film cuts) always win; list only phrases they lack.
Stdlib only. Output: 24 kHz mono 16-bit WAV (the player resamples).
"""

import json
import os
import sys
import time
import urllib.request
import wave
from pathlib import Path

API = "https://api.fish.audio/v1/tts"
MODEL = "s2.1-pro-free"  # free under Fair Use, supports Russian
# «ДЖАРВИС» (ru) on fish.audio; FISH_VOICE = any other model id
VOICE = os.environ.get("FISH_VOICE", "4c3eaacc1a0545cdb0295bfddf3e3785")
RATE = 24_000


def synth(text: str, key: str) -> bytes:
    body = json.dumps(
        {"text": text, "reference_id": VOICE, "format": "wav", "sample_rate": RATE}
    ).encode()
    req = urllib.request.Request(
        API,
        data=body,
        headers={
            "Authorization": f"Bearer {key}",
            "Content-Type": "application/json",
            "model": MODEL,
        },
    )
    with urllib.request.urlopen(req, timeout=120) as r:
        data = r.read()
    # streamed WAV has a bogus length in its header: keep PCM after the 44-byte header
    return data[44:]


def main() -> None:
    key = os.environ.get("FISH_API_KEY")
    if not key or len(sys.argv) != 3:
        sys.exit(__doc__)
    out = Path(sys.argv[2])
    for line in Path(sys.argv[1]).read_text(encoding="utf-8").splitlines():
        if not line.strip() or line.startswith("#"):
            continue
        cat, stem, text = line.split("\t", 2)
        dst = out / cat / f"{stem}.wav"
        if dst.exists():
            continue
        dst.parent.mkdir(parents=True, exist_ok=True)
        for attempt in range(3):
            try:
                pcm = synth(text, key)
                break
            except Exception as e:  # network hiccup / rate limit
                print(f"retry {stem}: {e}", file=sys.stderr)
                time.sleep(2 + attempt * 3)
        else:
            sys.exit(f"failed: {stem}")
        with wave.open(str(dst), "wb") as w:
            w.setnchannels(1)
            w.setsampwidth(2)
            w.setframerate(RATE)
            w.writeframes(pcm)
        print(f"{cat}/{stem}.wav  {len(pcm) / 2 / RATE:.1f}s  {text}")


if __name__ == "__main__":
    main()

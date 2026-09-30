"""Voice pack re-spoken by another Fish Audio voice (SPEC §6.1).

Usage (repo root): FISH_API_KEY=... python tools/voice/clone_pack.py <src_pack> <out_dir> <fish_voice> <id> <name>
Every line of <src_pack>/voice.json plus fixed replies in packs/*.json it lacks, same file layout.
out_dir == src_pack: only the missing replies are added in <fish_voice>, existing clips stay.
Needs ffmpeg in PATH. Stdlib only.
"""

import hashlib
import json
import os
import subprocess
import sys
from pathlib import Path

from build_pack import clean, norm, pack_texts

HERE = Path(__file__).resolve().parent


def main() -> None:
    if len(sys.argv) != 6 or not os.environ.get("FISH_API_KEY"):
        sys.exit(__doc__)
    src, out, voice, pid, name = Path(sys.argv[1]), Path(sys.argv[2]), *sys.argv[3:]
    old = json.loads((src / "voice.json").read_text(encoding="utf-8"))
    same = src.resolve() == out.resolve()
    texts = dict(old["texts"]) if same else {}
    wanted = [] if same else [(rel, t) for rel, t in old["texts"].items()]
    covered = {norm(t) for t in old["texts"].values()}
    for cat, text in pack_texts():
        if norm(text) not in covered:
            covered.add(norm(text))
            wanted.append((f"{cat}/t_{hashlib.sha1(norm(text).encode()).hexdigest()[:10]}.wav", text))
    cache = out.parent / ".fish-cache" / voice
    cache.mkdir(parents=True, exist_ok=True)
    tsv = cache / "wanted.tsv"
    tsv.write_text(
        "".join(f"{Path(r).parent}\t{Path(r).stem}\t{t}\n" for r, t in wanted), encoding="utf-8"
    )
    env = dict(os.environ, FISH_VOICE=voice)
    subprocess.run([sys.executable, str(HERE / "fish_gen.py"), str(tsv), str(cache)], check=True, env=env)
    for rel, text in wanted:
        clean(cache / rel, out / "ru" / rel)
        texts[rel] = text
    meta = {"id": pid, "name": name, "author": old["author"] if same else "Fish Audio", "texts": texts}
    (out / "voice.json").write_text(json.dumps(meta, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")
    print(f"{len(texts)} clips ({len(wanted)} new) → {out}")


if __name__ == "__main__":
    main()

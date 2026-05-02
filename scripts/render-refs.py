"""
Decode reference-art canvas data URLs into PNG-32 files.

Usage:
    1. Open `scripts/render-refs-inline.html` in any modern browser (Chrome/Firefox).
       Wait until the on-page status badge turns green and reads "done: 10/10".
    2. In the browser devtools console, run:
           copy(JSON.stringify(Object.fromEntries(
               Object.entries(window.__refs).map(([k, v]) => [k, v.dataUrl || null])
           )))
       This copies the data-URL JSON to your clipboard.
    3. Paste into `scripts/.refs-snapshot.json` (gitignored), then run:
           python scripts/render-refs.py
       PNGs land in `design/art/refs/png/REF01.png` ... `REF10.png`.

This decoder has zero deps. The harness is the source of truth; rerun it any
time the JSX refs change. The PNGs in `design/art/refs/png/` are committed
artifacts the contractor consumes per `design/04b-art-deliverables.md`.
"""
from __future__ import annotations

import base64
import json
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
SNAPSHOT = REPO / "scripts" / ".refs-snapshot.json"
OUT_DIR = REPO / "design" / "art" / "refs" / "png"


def main() -> int:
    if not SNAPSHOT.exists():
        print(f"snapshot not found: {SNAPSHOT}", file=sys.stderr)
        print("see module docstring for the harness workflow.", file=sys.stderr)
        return 1

    obj = json.loads(SNAPSHOT.read_text(encoding="utf-8"))
    OUT_DIR.mkdir(parents=True, exist_ok=True)

    written = 0
    for ref_id, data_url in obj.items():
        if not data_url or not data_url.startswith("data:image/png;base64,"):
            print(f"  skip {ref_id}: not a png data url")
            continue
        png = base64.b64decode(data_url.split(",", 1)[1])
        out = OUT_DIR / f"{ref_id}.png"
        out.write_bytes(png)
        print(f"  wrote {out.relative_to(REPO)}  ({len(png)} bytes)")
        written += 1

    print(f"\n{written} PNG(s) written to {OUT_DIR.relative_to(REPO)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())

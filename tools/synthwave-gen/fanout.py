# SPDX-License-Identifier: MIT
"""Multi-GPU dispatcher for synthwave-gen.

Distributes manifest tracks across N CUDA devices by spawning N
parallel `generate.py` subprocesses, each pinned to one card via
--device. Each subprocess handles a slice of the manifest. The
dispatcher waits for all to finish, collects exit codes, and reports.

Why subprocess fanout instead of a single multi-GPU process:
- Stable Audio's pipeline is not designed for cross-device sharding.
- Three independent processes with one pipe each is simpler and more
  fault-tolerant than torch.distributed for this kind of workload.
- Each card holds its own model copy. ~6 GB per card; trivial on the
  P100 16 GB pile, fine on a single 3070 Ti 8 GB (which falls back to
  --device 0 only).

Usage:
    python tools/synthwave-gen/fanout.py             # auto-detect cards, fan out
    python tools/synthwave-gen/fanout.py --devices 0,1,2
    python tools/synthwave-gen/fanout.py --force --only title,village,epilogue
    python tools/synthwave-gen/fanout.py --dry-run   # show plan, run nothing

Single-GPU rigs see no benefit from this script — call generate.py
directly. The dispatcher is for the multi-card upgrade path.
"""
import argparse
import json
import logging
import os
import subprocess
import sys
import time
from pathlib import Path

sys.stdout.reconfigure(encoding="utf-8", errors="replace")
sys.stderr.reconfigure(encoding="utf-8", errors="replace")

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(levelname)s] fanout: %(message)s",
    datefmt="%H:%M:%S",
)
log = logging.getLogger("fanout")

REPO_ROOT = Path(__file__).resolve().parents[2]
TOOL_DIR = REPO_ROOT / "tools" / "synthwave-gen"
MANIFEST = TOOL_DIR / "manifest.json"
DRIVER = TOOL_DIR / "generate.py"


def detect_devices() -> list[int]:
    """Ask torch how many CUDA devices it sees. Imports torch lazily so
    --help and --dry-run don't pay the import cost."""
    import torch
    if not torch.cuda.is_available():
        log.error("no CUDA visible to torch; can't fan out")
        return []
    return list(range(torch.cuda.device_count()))


def split_round_robin(items: list[str], n: int) -> list[list[str]]:
    """Distribute `items` across `n` buckets round-robin so each card
    gets a similar wall-clock slice (assuming similar track durations).
    Tracks of wildly different durations would benefit from a
    longest-processing-time heuristic; defer that until track count
    grows past ~12."""
    buckets: list[list[str]] = [[] for _ in range(n)]
    for i, item in enumerate(items):
        buckets[i % n].append(item)
    return buckets


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--devices", default="",
                        help="comma-separated CUDA indices (default: all visible)")
    parser.add_argument("--force", action="store_true",
                        help="passed through: regenerate even if WAVs exist")
    parser.add_argument("--only", default="",
                        help="comma-separated track subset to dispatch (default: full manifest)")
    parser.add_argument("--steps", type=int, default=None,
                        help="passed through: override inference steps")
    parser.add_argument("--dry-run", action="store_true",
                        help="show the per-card plan, run no subprocesses")
    parser.add_argument("--python", default=sys.executable,
                        help="python interpreter to use for the subprocesses "
                             "(default: the one running this script — usually "
                             "the venv's python so the deps are visible)")
    args = parser.parse_args()

    # Resolve track set.
    with MANIFEST.open("r", encoding="utf-8") as f:
        manifest = json.load(f)
    all_track_names = [t["name"] for t in manifest["tracks"]]
    if args.only:
        wanted = [s.strip() for s in args.only.split(",") if s.strip()]
        unknown = [n for n in wanted if n not in all_track_names]
        if unknown:
            log.error("unknown track(s) in --only: %s", unknown)
            return 2
        track_names = wanted
    else:
        track_names = all_track_names

    # Resolve device set.
    if args.devices:
        device_ids = [int(s.strip()) for s in args.devices.split(",") if s.strip()]
    else:
        device_ids = detect_devices()
    if not device_ids:
        log.error("no devices to dispatch to")
        return 3

    log.info("dispatching %d track(s) across %d device(s): %s",
             len(track_names), len(device_ids), device_ids)
    buckets = split_round_robin(track_names, len(device_ids))
    plan = list(zip(device_ids, buckets))
    for dev, names in plan:
        log.info("  cuda:%d -> %s", dev, names or "(idle)")

    if args.dry_run:
        log.info("dry-run; no subprocesses spawned")
        return 0

    # Spawn one process per non-empty bucket. Each gets its own
    # CUDA_VISIBLE_DEVICES so it only sees the card it owns — that way
    # generate.py's --device 0 inside the process maps to whatever
    # physical card we assigned. Cleaner than relying on the inner
    # process to honor a different index than 0.
    procs = []
    started = time.time()
    for dev, names in plan:
        if not names:
            continue
        env = os.environ.copy()
        env["CUDA_VISIBLE_DEVICES"] = str(dev)
        cmd = [args.python, str(DRIVER), "--device", "0", "--only", ",".join(names)]
        if args.force:
            cmd.append("--force")
        if args.steps is not None:
            cmd.extend(["--steps", str(args.steps)])
        log.info("[cuda:%d] spawning: %s", dev, " ".join(cmd))
        # Inherit stdout/stderr so progress is visible in real time.
        # If we ever want per-card log files instead, redirect here.
        p = subprocess.Popen(cmd, env=env)
        procs.append((dev, p))

    failures: list[tuple[int, int]] = []
    for dev, p in procs:
        rc = p.wait()
        elapsed = time.time() - started
        if rc == 0:
            log.info("[cuda:%d] done (rc=0, total elapsed=%.1fs)", dev, elapsed)
        else:
            log.error("[cuda:%d] failed with rc=%d after %.1fs", dev, rc, elapsed)
            failures.append((dev, rc))

    if failures:
        log.error("%d subprocess(es) failed: %s", len(failures), failures)
        return 1
    log.info("all subprocesses complete in %.1fs total", time.time() - started)
    return 0


if __name__ == "__main__":
    sys.exit(main())

# SPDX-License-Identifier: MIT
"""Pledge & Crown synthwave score generator.

Reads tools/synthwave-gen/manifest.json and produces one WAV per track
under assets/audio/. Uses Stable Audio Open 1.0 via diffusers.

Usage:
    python tools/synthwave-gen/generate.py            # generate missing only
    python tools/synthwave-gen/generate.py --force    # regenerate everything
    python tools/synthwave-gen/generate.py --only title,epilogue
    python tools/synthwave-gen/generate.py --dry-run  # show plan, do nothing

Hardware: tested target is RTX 3070 Ti 8 GB. Uses fp16 to fit; ~6-7 GB
peak. P100 16 GB users can drop the dtype override and run fp32 for
slightly better numerical fidelity (Pascal has no fp16 acceleration so
the speed difference is small).

License note: Stable Audio Open 1.0 ships under the Stability AI
Community License - commercial use allowed under the threshold; well
within scope for a sole-operator indie game.
"""
import argparse
import json
import logging
import sys
import time
from pathlib import Path

# Force UTF-8 stdio per the global rule (Windows + emoji-in-logs trap).
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
sys.stderr.reconfigure(encoding="utf-8", errors="replace")

logging.basicConfig(
    level=logging.DEBUG,
    format="%(asctime)s [%(levelname)s] %(name)s: %(message)s",
    datefmt="%H:%M:%S",
)
log = logging.getLogger("synthwave-gen")

REPO_ROOT = Path(__file__).resolve().parents[2]
MANIFEST = REPO_ROOT / "tools" / "synthwave-gen" / "manifest.json"
# Bevy's AssetServer resolves "audio/x.wav" relative to <pkg>/assets,
# which on this project is game/assets. Match that.
ASSETS_DIR = REPO_ROOT / "game" / "assets" / "audio"
MODEL_ID = "stabilityai/stable-audio-open-1.0"


def load_manifest() -> dict:
    log.info("loading manifest from %s", MANIFEST)
    with MANIFEST.open("r", encoding="utf-8") as f:
        data = json.load(f)
    log.info(
        "manifest: %d tracks, default_steps=%s, default_cfg=%s",
        len(data["tracks"]),
        data.get("_default_steps"),
        data.get("_default_cfg"),
    )
    return data


def select_tracks(manifest: dict, only: list[str] | None) -> list[dict]:
    tracks = manifest["tracks"]
    if not only:
        return tracks
    by_name = {t["name"]: t for t in tracks}
    selected = []
    for name in only:
        if name not in by_name:
            log.error("unknown track name `%s`; available: %s", name, list(by_name))
            sys.exit(2)
        selected.append(by_name[name])
    return selected


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--force", action="store_true",
                        help="overwrite existing WAVs (default: skip if present)")
    parser.add_argument("--only", default="",
                        help="comma-separated track names to generate (default: all)")
    parser.add_argument("--dry-run", action="store_true",
                        help="show plan, load no model, write no files")
    parser.add_argument("--steps", type=int, default=None,
                        help="override inference steps (manifest default applies otherwise)")
    args = parser.parse_args()

    manifest = load_manifest()
    only_list = [s.strip() for s in args.only.split(",") if s.strip()]
    tracks = select_tracks(manifest, only_list)
    default_steps = args.steps or int(manifest.get("_default_steps", 200))
    default_cfg = float(manifest.get("_default_cfg", 7.0))
    negative = manifest.get("_negative_prompt", "")

    ASSETS_DIR.mkdir(parents=True, exist_ok=True)

    plan = []
    for t in tracks:
        out = ASSETS_DIR / f"{t['name']}.wav"
        action = "skip (exists)" if (out.exists() and not args.force) else "generate"
        plan.append((t, out, action))
        log.info(
            "plan: %s -> %s (%.1fs, seed=%s, steps=%s) [%s]",
            t["name"], out.relative_to(REPO_ROOT),
            t["duration_s"], t["seed"], default_steps, action,
        )

    if args.dry_run:
        log.info("dry-run complete; nothing written")
        return 0

    to_run = [(t, out) for t, out, action in plan if action == "generate"]
    if not to_run:
        log.info("nothing to do; pass --force to regenerate")
        return 0

    # Heavy imports only when we know we'll generate. Lets dry-run / help
    # work without the diffusers/torch wall-clock penalty.
    log.info("loading torch + diffusers (this is slow first time)...")
    import torch  # noqa: E402
    from diffusers import StableAudioPipeline  # noqa: E402
    import soundfile as sf  # noqa: E402

    if not torch.cuda.is_available():
        log.error("no CUDA device visible to torch; bailing (CPU inference would take hours)")
        return 3
    device = "cuda"
    log.info("using device=%s, torch=%s, cuda=%s",
             device, torch.__version__, torch.version.cuda)
    log.info("VRAM total=%.1f GiB, free=%.1f GiB",
             torch.cuda.get_device_properties(0).total_memory / 2**30,
             (torch.cuda.mem_get_info()[0]) / 2**30)

    log.info("loading model `%s` (fp16)", MODEL_ID)
    t0 = time.time()
    pipe = StableAudioPipeline.from_pretrained(MODEL_ID, torch_dtype=torch.float16)
    pipe = pipe.to(device)
    log.info("model loaded in %.1fs", time.time() - t0)

    sample_rate = pipe.vae.sampling_rate
    log.info("model sample_rate=%d Hz", sample_rate)

    for t, out in to_run:
        log.info("=== generating `%s` ===", t["name"])
        log.debug("prompt: %s", t["prompt"])
        gen = torch.Generator(device).manual_seed(int(t["seed"]))
        t0 = time.time()
        audio = pipe(
            prompt=t["prompt"],
            negative_prompt=negative,
            num_inference_steps=default_steps,
            audio_end_in_s=float(t["duration_s"]),
            num_waveforms_per_prompt=1,
            guidance_scale=default_cfg,
            generator=gen,
        ).audios
        elapsed = time.time() - t0

        # diffusers returns (batch, channels, samples) tensors. Take the
        # first (only) waveform, transpose to (samples, channels) for
        # soundfile, and force CPU-side float32 numpy.
        waveform = audio[0].T.float().cpu().numpy()
        sf.write(str(out), waveform, sample_rate)
        size_kb = out.stat().st_size / 1024
        log.info("wrote %s (%.1f KiB) in %.1fs",
                 out.relative_to(REPO_ROOT), size_kb, elapsed)

        # Free the per-generation working set so the next track has a
        # clean VRAM slate. Stable Audio's intermediates can pin a few
        # hundred MB if not released.
        del audio, waveform
        torch.cuda.empty_cache()

    log.info("done; %d tracks generated", len(to_run))
    return 0


if __name__ == "__main__":
    sys.exit(main())

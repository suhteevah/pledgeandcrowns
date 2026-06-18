# SPDX-License-Identifier: MIT
"""Pledge & Crown diegetic SFX generator.

Reads tools/audioldm2-gen/manifest.json and produces one WAV per sound
effect under game/assets/audio/sfx/<name>.wav. Uses AudioLDM2 via
diffusers' AudioLDM2Pipeline.

This is the sibling tool to synthwave-gen. Where Stable Audio Open is
music-biased and good at melodic/loopable tracks, AudioLDM2 is the
better engine for short, descriptive *diegetic* sound effects - door
creaks, parchment unfurl, footsteps on stone, coin clinks, sword
unsheathe, anvil clang. Same conventions, different model.

Usage:
    python tools/audioldm2-gen/generate.py            # generate missing only
    python tools/audioldm2-gen/generate.py --force    # regenerate everything
    python tools/audioldm2-gen/generate.py --only door_creak,coin_clink
    python tools/audioldm2-gen/generate.py --dry-run  # show plan, do nothing

Hardware: the base model `cvssp/audioldm2` fits the RTX 3070 Ti 8 GB in
fp16 (~6-7 GB peak). The higher-quality `cvssp/audioldm2-large` wants
~12 GB+ and is meant for the P100 16 GB rig - swap MODEL_ID or pass
--model to target it once that hardware lands.

Sample rate note: AudioLDM2 outputs **16 kHz** mono. This differs from
Stable Audio Open (44.1 kHz stereo) - SFX at 16 kHz is fine for short
diegetic cues and keeps the WAVs small, but do not assume parity with
the synthwave-gen output format.

License note: AudioLDM2's released *weights* are CC-BY-NC (non-
commercial). Pledge & Crown is a commercial game, so SFX baked with
these weights are for **pipeline prototyping** only - confirm SFX
licensing (regenerate with a commercially-licensed model, license the
output, or source SFX elsewhere) before shipping. See README.md.
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
# Quiet the noisy third-party loggers that flood DEBUG output during
# HF model downloads. We still want our own DEBUG-level tracing on
# the audioldm2-gen logger.
for noisy in ("urllib3", "filelock", "huggingface_hub.file_download"):
    logging.getLogger(noisy).setLevel(logging.WARNING)
log = logging.getLogger("audioldm2-gen")

REPO_ROOT = Path(__file__).resolve().parents[2]
MANIFEST = REPO_ROOT / "tools" / "audioldm2-gen" / "manifest.json"
# Bevy's AssetServer resolves "audio/sfx/x.wav" relative to <pkg>/assets,
# which on this project is game/assets. Match that. SFX live in their
# own subdir to keep them apart from the synthwave-gen music tracks.
ASSETS_DIR = REPO_ROOT / "game" / "assets" / "audio" / "sfx"
# Base model fits the 3070 Ti 8 GB in fp16. `cvssp/audioldm2-large`
# is the higher-quality option for the P100 16 GB rig (~12 GB+).
MODEL_ID = "cvssp/audioldm2"
# AudioLDM2 emits 16 kHz mono regardless of prompt. The pipeline does
# not expose a sampling_rate attribute the way StableAudioPipeline's
# vae does, so we pin the documented value here.
SAMPLE_RATE = 16000


def load_manifest() -> dict:
    log.info("loading manifest from %s", MANIFEST)
    with MANIFEST.open("r", encoding="utf-8") as f:
        data = json.load(f)
    log.info(
        "manifest: %d sfx, default_steps=%s, default_negative=%r",
        len(data["sfx"]),
        data.get("_default_steps"),
        data.get("_default_negative_prompt"),
    )
    return data


def select_sfx(manifest: dict, only: list[str] | None) -> list[dict]:
    sfx = manifest["sfx"]
    if not only:
        return sfx
    by_name = {s["name"]: s for s in sfx}
    selected = []
    for name in only:
        if name not in by_name:
            log.error("unknown sfx name `%s`; available: %s", name, list(by_name))
            sys.exit(2)
        selected.append(by_name[name])
    return selected


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--force", action="store_true",
                        help="overwrite existing WAVs (default: skip if present)")
    parser.add_argument("--only", default="",
                        help="comma-separated sfx names to generate (default: all)")
    parser.add_argument("--dry-run", action="store_true",
                        help="show plan, load no model, write no files")
    parser.add_argument("--steps", type=int, default=None,
                        help="override inference steps (manifest / per-sfx default applies otherwise)")
    parser.add_argument("--model", default=MODEL_ID,
                        help="HF model id (default: %(default)s; pass cvssp/audioldm2-large "
                             "on the P100 rig for higher quality)")
    parser.add_argument("--device", type=int, default=0,
                        help="CUDA device index to pin this run to. Used by a "
                             "multi-GPU dispatch wrapper to fan one process per "
                             "card. CUDA_VISIBLE_DEVICES at the env level still "
                             "wins; this just selects within what torch can see.")
    args = parser.parse_args()

    manifest = load_manifest()
    only_list = [s.strip() for s in args.only.split(",") if s.strip()]
    sfx_list = select_sfx(manifest, only_list)
    default_steps = args.steps or int(manifest.get("_default_steps", 200))
    default_cfg = float(manifest.get("_default_cfg", 3.5))
    default_negative = manifest.get("_default_negative_prompt", "")

    plan = []
    for s in sfx_list:
        out = ASSETS_DIR / f"{s['name']}.wav"
        action = "skip (exists)" if (out.exists() and not args.force) else "generate"
        steps = int(s.get("steps", default_steps))
        # We deliberately print the *relative* output path so the plan is
        # readable on any machine without leaking the absolute repo root.
        out_rel = out.relative_to(REPO_ROOT)
        plan.append((s, out, action))
        log.info(
            "plan: %s -> %s (%.1fs, seed=%s, steps=%s) [%s]",
            s["name"], out_rel, float(s["duration_s"]), s["seed"], steps, action,
        )

    if args.dry_run:
        log.info("dry-run complete; loaded no model, wrote nothing")
        log.info("model would be `%s` -> %s @ %d Hz mono",
                 args.model, ASSETS_DIR.relative_to(REPO_ROOT), SAMPLE_RATE)
        return 0

    # Only create the output dir on a real run, so --dry-run truly writes
    # nothing (and imports no torch).
    ASSETS_DIR.mkdir(parents=True, exist_ok=True)

    to_run = [(s, out) for s, out, action in plan if action == "generate"]
    if not to_run:
        log.info("nothing to do; pass --force to regenerate")
        return 0

    # Heavy imports only when we know we'll generate. Lets dry-run / help
    # work without the diffusers/torch wall-clock penalty (and without
    # torch installed at all).
    log.info("loading torch + diffusers (this is slow first time)...")
    import torch  # noqa: E402
    from diffusers import AudioLDM2Pipeline  # noqa: E402
    import soundfile as sf  # noqa: E402

    if not torch.cuda.is_available():
        log.error("no CUDA device visible to torch; bailing (CPU inference would take ages)")
        return 3
    n_devices = torch.cuda.device_count()
    if args.device >= n_devices:
        log.error("--device %d invalid; torch sees %d CUDA device(s)", args.device, n_devices)
        return 4
    device = f"cuda:{args.device}"
    torch.cuda.set_device(args.device)
    log.info("using device=%s (of %d visible), torch=%s, cuda=%s, name=%s",
             device, n_devices, torch.__version__, torch.version.cuda,
             torch.cuda.get_device_name(args.device))
    log.info("VRAM total=%.1f GiB, free=%.1f GiB",
             torch.cuda.get_device_properties(args.device).total_memory / 2**30,
             (torch.cuda.mem_get_info(args.device)[0]) / 2**30)

    log.info("loading model `%s` (fp16)", args.model)
    t0 = time.time()
    try:
        pipe = AudioLDM2Pipeline.from_pretrained(args.model, torch_dtype=torch.float16)
    except Exception as e:
        # Common first-run failure: an HF auth / gated-repo problem.
        # AudioLDM2's cvssp repos are not gated today, but the weights
        # are CC-BY-NC and HF occasionally requires a login to pull
        # them, and a custom --model may point at a gated mirror.
        # Translate the cryptic stack trace into a one-paragraph "do
        # these three things" note so Matt isn't fishing through the
        # diffusers internals to figure out what went wrong.
        msg = str(e)
        if "GatedRepoError" in repr(type(e)) or "gated" in msg.lower() or "401" in msg or "403" in msg:
            log.error(
                "HuggingFace refused access to `%s`. To fix:\n"
                "  1. If the repo shows a license gate, visit\n"
                "     https://huggingface.co/%s and click `Agree and access repository`.\n"
                "  2. Create a token at https://huggingface.co/settings/tokens (read scope is fine).\n"
                "  3. Either run `huggingface-cli login` once and paste the token, OR set\n"
                "     the env var HF_TOKEN=<your-token> before invoking this script.\n"
                "Then re-run audioldm2-gen.ps1.",
                args.model, args.model,
            )
            return 5
        raise
    pipe = pipe.to(device)
    log.info("model loaded in %.1fs", time.time() - t0)
    log.info("output sample_rate=%d Hz (AudioLDM2 fixed, mono)", SAMPLE_RATE)

    for s, out in to_run:
        steps = int(s.get("steps", default_steps))
        cfg = float(s.get("guidance_scale", default_cfg))
        negative = s.get("negative_prompt", default_negative)
        log.info("=== generating `%s` on %s ===", s["name"], device)
        log.debug("prompt: %s", s["prompt"])
        log.debug("negative_prompt: %s", negative)
        gen = torch.Generator(device).manual_seed(int(s["seed"]))
        t0 = time.time()
        # AudioLDM2Pipeline call signature:
        #   pipe(prompt, negative_prompt=..., num_inference_steps=...,
        #        audio_length_in_s=..., num_waveforms_per_prompt=...,
        #        generator=...).audios  ->  np.ndarray (batch, samples)
        audio = pipe(
            prompt=s["prompt"],
            negative_prompt=negative,
            num_inference_steps=steps,
            audio_length_in_s=float(s["duration_s"]),
            num_waveforms_per_prompt=1,
            guidance_scale=cfg,
            generator=gen,
        ).audios
        elapsed = time.time() - t0

        # AudioLDM2Pipeline returns a numpy array shaped (batch, samples)
        # of mono float waveforms. Take the first (only) waveform and
        # write it as 16 kHz mono.
        waveform = audio[0]
        sf.write(str(out), waveform, SAMPLE_RATE)
        size_kb = out.stat().st_size / 1024
        log.info("wrote %s (%.1f KiB) in %.1fs",
                 out.relative_to(REPO_ROOT), size_kb, elapsed)

        # Free the per-generation working set so the next sfx has a clean
        # VRAM slate. AudioLDM2's intermediates can pin a few hundred MB
        # if not released.
        del audio, waveform
        torch.cuda.empty_cache()

    log.info("done; %d sfx generated", len(to_run))
    return 0


if __name__ == "__main__":
    sys.exit(main())

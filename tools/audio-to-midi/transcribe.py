# SPDX-License-Identifier: MIT
"""Pledge & Crown audio-to-MIDI transcriber.

Reads tools/audio-to-midi/manifest.json and, for each selected baked
track, runs:

    Demucs (htdemucs 4-stem split: bass/drums/other/vocals)
      -> Spotify Basic Pitch (per-stem polyphonic transcription)
      -> one .mid per stem + a combined .mid

Output lands under tools/audio-to-midi/out/<trackname>/. For every
emitted .mid the script logs a NOTE-RANGE REPORT (note count + lowest
and highest pitch as note names, e.g. "bass stem -> 142 notes, range
C2..B3"). That report is the entire point of this tool: it lets us see
exactly what notes Stable Audio Open chose for each voice and close the
"is the bass actually anchored on B" loop from the synthwave score.

Usage:
    python tools/audio-to-midi/transcribe.py                 # default tracks (village, epilogue)
    python tools/audio-to-midi/transcribe.py --all           # every track in the manifest
    python tools/audio-to-midi/transcribe.py --only village  # one or more, comma-separated
    python tools/audio-to-midi/transcribe.py --force         # re-transcribe even if out/ exists
    python tools/audio-to-midi/transcribe.py --dry-run       # show plan, load no model, write nothing
    python tools/audio-to-midi/transcribe.py --no-stems      # skip Demucs; Basic Pitch on the full mix

Design notes:
- Heavy imports (torch, demucs, basic_pitch, pretty_midi) are deferred
  until we know we will actually transcribe, so --dry-run / --help stay
  instant and don't pay the multi-second import wall.
- Fail-open per track: one bad track logs an error and the run keeps
  going with the next. The process exit code reflects whether ANY track
  failed, so CI can still notice.
- --no-stems is the resilient fallback: if Demucs can't install or run,
  Basic Pitch transcribes the full mix directly and still produces a
  real MIDI.
"""
import argparse
import json
import logging
import shutil
import sys
import time
from pathlib import Path

# Force UTF-8 stdio per the global Windows rule (emoji/non-ASCII in
# third-party logs would otherwise blow up the default cp1252 console).
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
sys.stderr.reconfigure(encoding="utf-8", errors="replace")

logging.basicConfig(
    level=logging.DEBUG,
    format="%(asctime)s [%(levelname)s] %(name)s: %(message)s",
    datefmt="%H:%M:%S",
)
# Quiet the noisy third-party loggers that flood DEBUG output during
# model loading / tensor ops. We still want our own DEBUG tracing on the
# audio-to-midi logger.
for noisy in ("urllib3", "filelock", "matplotlib", "numba",
              "h5py", "PIL", "huggingface_hub.file_download"):
    logging.getLogger(noisy).setLevel(logging.WARNING)
log = logging.getLogger("audio-to-midi")

REPO_ROOT = Path(__file__).resolve().parents[2]
MANIFEST = REPO_ROOT / "tools" / "audio-to-midi" / "manifest.json"
OUT_DIR = REPO_ROOT / "tools" / "audio-to-midi" / "out"

# htdemucs's four stems, in the order the model emits them.
VALID_STEMS = ("bass", "drums", "other", "vocals")


def load_manifest() -> dict:
    log.info("loading manifest from %s", MANIFEST)
    with MANIFEST.open("r", encoding="utf-8") as f:
        data = json.load(f)
    log.info(
        "manifest: %d tracks, default_tracks=%s",
        len(data["tracks"]),
        data.get("_default_tracks"),
    )
    return data


def select_tracks(manifest: dict, only: list[str], do_all: bool) -> list[dict]:
    tracks = manifest["tracks"]
    by_name = {t["name"]: t for t in tracks}

    if do_all:
        return tracks

    wanted = only if only else list(manifest.get("_default_tracks", []))
    if not wanted:
        # No defaults configured and no --only/--all -> nothing to do.
        log.warning("no default_tracks in manifest and no --only/--all given")
        return []

    selected = []
    for name in wanted:
        if name not in by_name:
            log.error("unknown track name `%s`; available: %s", name, list(by_name))
            sys.exit(2)
        selected.append(by_name[name])
    return selected


def note_number_to_name(n: int) -> str:
    """MIDI note number -> scientific pitch name (e.g. 47 -> 'B2').

    Standard convention: MIDI 60 = C4 (middle C). Self-contained so we
    don't depend on a particular pretty_midi helper being present.
    """
    names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"]
    octave = (n // 12) - 1
    return f"{names[n % 12]}{octave}"


def report_midi(label: str, midi_path: Path) -> int:
    """Load a .mid, log its NOTE-RANGE REPORT, return the note count.

    This is the load-bearing output of the whole tool. Keep it loud and
    unmissable in the logs.
    """
    import pretty_midi  # deferred

    try:
        pm = pretty_midi.PrettyMIDI(str(midi_path))
    except Exception as e:  # noqa: BLE001 - report-and-continue
        log.error("could not parse MIDI %s for reporting: %s", midi_path, e)
        return -1

    pitches = []
    for inst in pm.instruments:
        for note in inst.notes:
            pitches.append(note.pitch)

    n = len(pitches)
    size_kb = midi_path.stat().st_size / 1024 if midi_path.exists() else 0.0
    if n == 0:
        log.warning(
            "NOTE-RANGE REPORT | %-22s -> 0 notes (EMPTY) | %s (%.1f KiB)",
            label, midi_path.relative_to(REPO_ROOT), size_kb,
        )
        return 0

    lo, hi = min(pitches), max(pitches)
    lo_name, hi_name = note_number_to_name(lo), note_number_to_name(hi)
    # Most common pitch is handy for spotting a single anchored bass note.
    counts: dict[int, int] = {}
    for p in pitches:
        counts[p] = counts.get(p, 0) + 1
    top_pitch = max(counts, key=counts.get)
    top_share = counts[top_pitch] / n

    log.info(
        "NOTE-RANGE REPORT | %-22s -> %d notes, range %s..%s "
        "(MIDI %d..%d) | most-common %s x%d (%.0f%%) | %s (%.1f KiB)",
        label, n, lo_name, hi_name, lo, hi,
        note_number_to_name(top_pitch), counts[top_pitch], 100.0 * top_share,
        midi_path.relative_to(REPO_ROOT), size_kb,
    )
    return n


def run_demucs(wav_path: Path, work_dir: Path, device: str) -> dict[str, Path]:
    """Split `wav_path` into stems with htdemucs. Returns {stem: wavpath}.

    Uses the demucs Python API (demucs.separate.main) so we stay in one
    process and can pin the output layout. With our `--filename
    {stem}.{ext}` template htdemucs writes <work_dir>/htdemucs/<stem>.wav
    (flat); without the template it would nest under a <trackstem>/ dir.
    We handle both layouts below.
    """
    import demucs.separate  # deferred

    track_stem = wav_path.stem
    log.info("=== demucs htdemucs split: %s (device=%s) ===", wav_path.name, device)
    t0 = time.time()
    # demucs.separate.main parses an argv-style list, same as the CLI.
    argv = [
        "-n", "htdemucs",
        "-d", device,
        "-o", str(work_dir),
        "--filename", "{stem}.{ext}",
        str(wav_path),
    ]
    log.debug("demucs argv: %s", argv)
    demucs.separate.main(argv)
    log.info("demucs split done in %.1fs", time.time() - t0)

    # With `--filename {stem}.{ext}` demucs writes the stems FLAT at
    # <work_dir>/htdemucs/<stem>.wav (the per-track subdirectory only
    # appears when the filename template contains {track}). We check the
    # flat layout first, then fall back to demucs's default nested
    # layout <work_dir>/htdemucs/<trackstem>/<stem>.wav so the function
    # works regardless of how the template is set.
    flat_dir = work_dir / "htdemucs"
    nested_dir = work_dir / "htdemucs" / track_stem
    found: dict[str, Path] = {}
    for stem in VALID_STEMS:
        flat_p = flat_dir / f"{stem}.wav"
        nested_p = nested_dir / f"{stem}.wav"
        if flat_p.exists():
            found[stem] = flat_p
            log.debug("stem ready (flat): %s -> %s", stem, flat_p)
        elif nested_p.exists():
            found[stem] = nested_p
            log.debug("stem ready (nested): %s -> %s", stem, nested_p)
        else:
            log.warning("expected stem %s not found at %s or %s",
                        stem, flat_p, nested_p)
    if not found:
        raise RuntimeError(
            f"demucs produced no stems under {flat_dir} or {nested_dir}"
        )
    return found


def transcribe_wav(src_wav: Path, out_mid: Path) -> None:
    """Run Basic Pitch on a single WAV, writing a .mid to out_mid."""
    from basic_pitch.inference import predict  # deferred
    from basic_pitch import ICASSP_2022_MODEL_PATH  # deferred

    log.debug("basic-pitch predict: %s", src_wav.name)
    t0 = time.time()
    _model_output, midi_data, _note_events = predict(
        str(src_wav), ICASSP_2022_MODEL_PATH
    )
    out_mid.parent.mkdir(parents=True, exist_ok=True)
    midi_data.write(str(out_mid))
    log.debug("basic-pitch wrote %s in %.1fs", out_mid.name, time.time() - t0)


def log_basic_pitch_backend() -> None:
    """Best-effort: log which inference backend Basic Pitch loaded.

    Recent basic-pitch can use onnxruntime, coreml, tflite, or tensorflow
    depending on what's installed. We prefer onnxruntime (light). This
    just surfaces the truth so the requirements note stays honest.
    """
    try:
        import onnxruntime  # noqa: F401
        log.info("basic-pitch backend: onnxruntime available (%s)",
                 getattr(__import__("onnxruntime"), "__version__", "?"))
    except Exception:  # noqa: BLE001
        log.info("basic-pitch backend: onnxruntime NOT importable; "
                 "basic-pitch will use its bundled fallback runtime")


def process_track(track: dict, no_stems: bool, force: bool, device: str) -> bool:
    """Transcribe one track. Returns True on success, False on failure.

    Fail-open: never raises; logs and returns False so the caller can
    keep going with the next track.
    """
    name = track["name"]
    wav_rel = track["wav"]
    wav_path = (REPO_ROOT / wav_rel).resolve()
    track_out = OUT_DIR / name

    log.info("######## track `%s` (%s) ########", name, wav_rel)
    if not wav_path.exists():
        log.error("source WAV missing: %s (skipping `%s`)", wav_path, name)
        return False

    if track_out.exists() and not force:
        log.warning("out dir exists and --force not set; skipping `%s` (%s)",
                    name, track_out.relative_to(REPO_ROOT))
        # Still report whatever MIDIs are already there so the run is useful.
        for mid in sorted(track_out.glob("*.mid")):
            report_midi(f"{name}/{mid.stem}", mid)
        return True

    if track_out.exists() and force:
        log.info("--force: clearing existing %s", track_out.relative_to(REPO_ROOT))
        shutil.rmtree(track_out, ignore_errors=True)
    track_out.mkdir(parents=True, exist_ok=True)

    try:
        if no_stems:
            # Resilient path: transcribe the full mix directly.
            out_mid = track_out / f"{name}_fullmix.mid"
            log.info("--no-stems: transcribing full mix directly")
            transcribe_wav(wav_path, out_mid)
            report_midi(f"{name}/fullmix", out_mid)
            return True

        # Full pipeline: demucs split, then per-requested-stem transcription.
        work_dir = track_out / "_stems"
        work_dir.mkdir(parents=True, exist_ok=True)
        stems = run_demucs(wav_path, work_dir, device)

        wanted_stems = track.get("stems") or list(stems.keys())
        produced: list[Path] = []
        for stem in wanted_stems:
            if stem not in VALID_STEMS:
                log.error("track `%s`: invalid stem `%s` (valid: %s); skipping stem",
                          name, stem, VALID_STEMS)
                continue
            if stem not in stems:
                log.warning("track `%s`: stem `%s` not produced by demucs; skipping",
                            name, stem)
                continue
            out_mid = track_out / f"{name}_{stem}.mid"
            log.info("--- transcribing %s stem of `%s` ---", stem, name)
            transcribe_wav(stems[stem], out_mid)
            report_midi(f"{name}/{stem}", out_mid)
            produced.append(out_mid)

        if not produced:
            log.error("track `%s`: no stems transcribed", name)
            return False

        # Combined MIDI: merge each stem's MIDI onto its own instrument so
        # the whole arrangement is inspectable in one file.
        combined = track_out / f"{name}_combined.mid"
        build_combined_midi(produced, combined)
        report_midi(f"{name}/combined", combined)

        # Tidy: drop the bulky intermediate stem WAVs to save disk; the
        # MIDIs are what we care about. Comment out to keep stems around.
        shutil.rmtree(work_dir, ignore_errors=True)
        log.debug("removed intermediate stem WAVs under %s", work_dir)
        return True

    except Exception as e:  # noqa: BLE001 - fail-open per track
        log.exception("track `%s` failed: %s", name, e)
        return False


def build_combined_midi(stem_mids: list[Path], out_path: Path) -> None:
    """Merge per-stem MIDIs into one multi-instrument .mid."""
    import pretty_midi  # deferred

    merged = pretty_midi.PrettyMIDI()
    for mid in stem_mids:
        try:
            pm = pretty_midi.PrettyMIDI(str(mid))
        except Exception as e:  # noqa: BLE001
            log.warning("combined: skipping unparseable %s (%s)", mid.name, e)
            continue
        for inst in pm.instruments:
            # Name each instrument after its source stem for clarity.
            inst.name = mid.stem
            merged.instruments.append(inst)
    merged.write(str(out_path))
    log.debug("combined MIDI written: %s", out_path.name)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--only", default="",
                        help="comma-separated track names to transcribe "
                             "(default: manifest _default_tracks)")
    parser.add_argument("--all", action="store_true",
                        help="transcribe every track in the manifest")
    parser.add_argument("--force", action="store_true",
                        help="re-transcribe even if out/<track>/ already exists")
    parser.add_argument("--dry-run", action="store_true",
                        help="show plan, load no model, write no files")
    parser.add_argument("--no-stems", action="store_true",
                        help="skip Demucs; run Basic Pitch on the full mix directly "
                             "(resilient fallback if demucs is unavailable)")
    parser.add_argument("--device", default="auto",
                        help="torch device for Demucs: auto|cpu|cuda (default: auto)")
    args = parser.parse_args()

    manifest = load_manifest()
    only_list = [s.strip() for s in args.only.split(",") if s.strip()]
    tracks = select_tracks(manifest, only_list, args.all)

    if not tracks:
        log.info("nothing selected; nothing to do")
        return 0

    OUT_DIR.mkdir(parents=True, exist_ok=True)

    # Plan.
    mode = "full-mix (no stems)" if args.no_stems else "demucs stems -> basic-pitch"
    for t in tracks:
        out = OUT_DIR / t["name"]
        exists = out.exists()
        action = "skip (exists)" if (exists and not args.force) else "transcribe"
        stems = "fullmix" if args.no_stems else ",".join(t.get("stems", VALID_STEMS))
        log.info("plan: %-14s wav=%s stems=[%s] [%s]",
                 t["name"], t["wav"], stems, action)
    log.info("pipeline mode: %s", mode)

    if args.dry_run:
        log.info("dry-run complete; nothing written")
        return 0

    # Resolve the device (only matters for demucs; basic-pitch picks its
    # own backend). Deferred torch import.
    device = "cpu"
    if not args.no_stems:
        log.info("loading torch to resolve device (slow first import)...")
        try:
            import torch  # noqa: E402
            if args.device == "auto":
                device = "cuda" if torch.cuda.is_available() else "cpu"
            else:
                device = args.device
            log.info("torch=%s, cuda_available=%s -> demucs device=%s",
                     torch.__version__, torch.cuda.is_available(), device)
            if device.startswith("cuda"):
                log.info("GPU: %s, VRAM total=%.1f GiB",
                         torch.cuda.get_device_name(0),
                         torch.cuda.get_device_properties(0).total_memory / 2**30)
        except Exception as e:  # noqa: BLE001
            log.error("torch import/device resolve failed (%s); "
                      "falling back to --no-stems full-mix transcription", e)
            args.no_stems = True

    log_basic_pitch_backend()

    ok_count = 0
    fail_count = 0
    for t in tracks:
        if process_track(t, args.no_stems, args.force, device):
            ok_count += 1
        else:
            fail_count += 1

    log.info("done; %d ok, %d failed (of %d tracks)",
             ok_count, fail_count, len(tracks))
    # Non-zero exit if anything failed, so the launcher/CI can notice,
    # but we still processed everything we could (fail-open).
    return 0 if fail_count == 0 else 1


if __name__ == "__main__":
    sys.exit(main())

# audio-to-midi

Offline **audio-to-MIDI transcription** of the Pledge & Crown baked synthwave
score. Given a generated track WAV (from `tools/synthwave-gen`), it tells you
exactly what notes Stable Audio Open chose for each voice -- closing the "is
the bass actually anchored on B?" loop.

## What it does

Pipeline, per track:

```
WAV  ->  Demucs (htdemucs 4-stem split: bass / drums / other / vocals)
     ->  Spotify Basic Pitch (per-stem polyphonic transcription)
     ->  one .mid per requested stem  +  a combined .mid
```

Output lands under `tools/audio-to-midi/out/<trackname>/`, e.g.:

```
out/village/village_bass.mid
out/village/village_other.mid
out/village/village_combined.mid
```

### The point: the NOTE-RANGE REPORT

For every `.mid` it writes, the script logs a loud line like:

```
NOTE-RANGE REPORT | village/bass -> 142 notes, range B1..B3 (MIDI 35..59) | most-common B1 x88 (62%) | tools/audio-to-midi/out/village/village_bass.mid (3.1 KiB)
```

That report -- note count, pitch range as note names, and the single most
common pitch -- is the whole reason this tool exists. It's how you verify the
score's bass is sitting on the note the prompt asked for instead of drifting.

## How to run

From the repo root, on Windows:

```
powershell -ExecutionPolicy Bypass -File scripts/audio-to-midi.ps1 --only village
```

The launcher locates Python 3.10/3.11 (`py -3.11` then `py -3.10`), creates a
venv at `tools/audio-to-midi/.venv`, installs torch+torchaudio from the
PyTorch cu121 index, then the rest from `requirements.txt`, and forwards all
args to `transcribe.py`. A marker file skips the (slow, ~2-3 GB) reinstall on
later runs.

More examples:

```
# default tracks (village + epilogue, the most musical):
powershell -ExecutionPolicy Bypass -File scripts/audio-to-midi.ps1

# every baked track:
powershell -ExecutionPolicy Bypass -File scripts/audio-to-midi.ps1 --all

# re-transcribe (clobber existing out/):
powershell -ExecutionPolicy Bypass -File scripts/audio-to-midi.ps1 --only village --force

# show the plan only, install/transcribe nothing:
powershell -ExecutionPolicy Bypass -File scripts/audio-to-midi.ps1 --dry-run

# skip Demucs entirely -- Basic Pitch straight on the full mix
# (resilient fallback if stem separation is unavailable):
powershell -ExecutionPolicy Bypass -File scripts/audio-to-midi.ps1 --only village --no-stems
```

You can also run the Python directly once the venv exists:

```
tools/audio-to-midi/.venv/Scripts/python.exe tools/audio-to-midi/transcribe.py --only village
```

## Flags

| Flag | Effect |
|------|--------|
| `--only a,b` | Transcribe named tracks (comma-separated). |
| `--all` | Transcribe every track in `manifest.json`. |
| `--force` | Re-transcribe even if `out/<track>/` already exists. |
| `--dry-run` | Print the plan; load no model, write no files. |
| `--no-stems` | Skip Demucs; Basic Pitch on the full mix directly. |
| `--device auto\|cpu\|cuda` | Torch device for Demucs (default `auto`). |

## Runtime backend

Basic Pitch can run on several backends. We install `basic-pitch` with
`onnxruntime` (a far lighter dependency than the full TensorFlow stack -- no
~500 MB tf wheel, no cuDNN version dance). `transcribe.py` logs which backend
actually loaded at startup so the requirements stay honest. If `onnxruntime`
isn't importable for the interpreter, Basic Pitch falls back to its bundled
runtime; transcription still works, just slower.

Demucs runs on GPU when CUDA is available (auto-detected), otherwise CPU.
CPU stem separation of a 30 s clip is on the order of a minute or two -- fine
for this offline, inspection-only use.

## Manifest

`manifest.json` lists each baked track, its source WAV, and which stems to
transcribe. `bass` and `other` are the musically interesting stems for this
score; `drums`/`vocals` are usually empty for instrumental synthwave but are
valid stem names if you want them. Defaults (`_default_tracks`) are `village`
and `epilogue`.

## Optional follow-up: sheet music (NOT implemented here)

Turning these MIDIs into engraved sheet music / PDF is a **documented manual
step**, intentionally left out of this tool to keep its dependency surface
small and offline. To render notation with MuseScore 4's CLI:

```
# one-off, once MuseScore 4 is installed (adjust the exe path):
"C:\Program Files\MuseScore 4\bin\MuseScore4.exe" ^
    tools/audio-to-midi/out/village/village_combined.mid ^
    -o tools/audio-to-midi/out/village/village_combined.pdf
```

MuseScore imports MIDI and exports `.pdf`, `.musicxml`, `.png`, etc. This is a
hand-run convenience step for when someone wants to read the transcription as
notation -- it is not part of the automated pipeline and is not installed by
the launcher.

## Notes

- Output is for **inspection**, not playback in-game. The baked WAVs remain
  the shipped assets; this tool just lets us read their pitch content.
- Basic Pitch is a statistical transcriber: pitch estimates on dense pads or
  reverb tails can be approximate. Trust the bass/lead range read; treat exact
  note timings as indicative.
- `out/`, `.venv/`, and `__pycache__/` are gitignored.

<!-- SPDX-License-Identifier: MIT -->
# audioldm2-gen — diegetic SFX generator

Generates short, **diegetic** sound effects for Pledge & Crown — door
creaks, parchment unfurl, footsteps on stone, coin clinks, sword
unsheathe, anvil clang, quill scratch, latch click — the kind of cues
that the music-biased Stable Audio Open (used by `synthwave-gen`) does
poorly. This is the sibling tool to `tools/synthwave-gen/`; same
conventions, different model.

- **Model:** [`cvssp/audioldm2`](https://huggingface.co/cvssp/audioldm2)
  (base) via diffusers' `AudioLDM2Pipeline`.
- **Output:** one mono WAV per manifest entry under
  `game/assets/audio/sfx/<name>.wav`.
- **Driver:** `tools/audioldm2-gen/generate.py` (manifest-driven,
  fail-open per sfx, `--dry-run` works without torch).
- **Launcher:** `scripts/audioldm2-gen.ps1` (ASCII-only Windows venv
  bootstrap: torch cu121 + deps, then runs the driver).

## Deferred bake — read this first

**No bake has been run, by design.** Per the project handoff, the actual
GPU bake of the SFX is intentionally **deferred until a 3x P100 rig
lands**, so the music (`synthwave-gen`) and SFX (`audioldm2-gen`) sets
can bake in parallel on dedicated hardware. This directory is the
*scaffold*: correct, runnable-once-deps-exist code plus a `--dry-run`
that prints the plan and writes nothing.

Until the rig is ready, the only thing that has been validated is:

- `py -3.10 -m py_compile tools/audioldm2-gen/generate.py` — parses.
- `python tools/audioldm2-gen/generate.py --dry-run` — prints the plan
  without importing torch.

The ~8 GB model has **not** been downloaded, no venv has been created,
and no WAVs have been written.

## Run command (when the rig is ready)

```
powershell -ExecutionPolicy Bypass -File scripts/audioldm2-gen.ps1 --only door_creak
```

Other forms:

```
# all sfx, skip any that already exist
powershell -ExecutionPolicy Bypass -File scripts/audioldm2-gen.ps1

# regenerate everything
powershell -ExecutionPolicy Bypass -File scripts/audioldm2-gen.ps1 --force

# dry run (no model, no files, no torch) - safe on any machine
powershell -ExecutionPolicy Bypass -File scripts/audioldm2-gen.ps1 --dry-run
```

The launcher creates `tools/audioldm2-gen/.venv`, installs the
CUDA-12.1 torch wheel + `requirements.txt`, then forwards all args to
`generate.py`. Delete the `.audioldm2-deps-installed` marker (or set
`AUDIOLDM2_FORCE_DEPS=1`) to reinstall deps.

## VRAM / dtype notes

- Base `cvssp/audioldm2` runs in **fp16** and fits the RTX 3070 Ti
  **8 GB** (~6-7 GB peak). This is the default `MODEL_ID`.
- `cvssp/audioldm2-large` is the higher-quality option but wants
  **~12 GB+** — target it on the **P100 16 GB** rig via
  `--model cvssp/audioldm2-large`.
- The driver pins fp16 (`torch_dtype=torch.float16`). Pascal (P100) has
  no fp16 acceleration, so fp16 is for VRAM headroom, not speed there;
  it still fits and runs.
- `--device N` selects the CUDA index for multi-GPU fan-out (one process
  per card). `CUDA_VISIBLE_DEVICES` at the env level still wins.

## Sample rate

AudioLDM2 outputs **16 kHz mono**, fixed by the model. This **differs**
from `synthwave-gen` (Stable Audio Open → 44.1 kHz stereo). 16 kHz is
fine for short diegetic cues and keeps WAVs small, but do not assume
format parity between the two tools.

## Prompt style

AudioLDM2 prefers **descriptive natural-language** prompts, e.g.
`"a heavy wooden door creaking open slowly on rusty iron hinges,
echoing stone castle interior, mono"` — not the comma-tag style of
music models. Edit `manifest.json` prompts freely and re-run; seeds are
pinned per entry so re-runs reproduce (bump a seed for a fresh take).
Per-sfx `steps`, `guidance_scale`, and `negative_prompt` override the
`_default_*` manifest keys.

## License caveat — confirm before shipping

> **AudioLDM2's released weights are CC-BY-NC (non-commercial).**

Pledge & Crown is a **commercial** game. SFX baked with these weights
are for **pipeline prototyping only**. Before any of this audio ships in
a sold/distributed build, confirm SFX licensing by one of:

1. Regenerating with a commercially-licensed SFX model,
2. Licensing the specific output (if/where permitted), or
3. Sourcing the final SFX from a commercially-clear library and using
   this tool only to prototype timing/feel.

Treat every WAV this tool produces as a **placeholder** until that
clearance is done. The driver and README both flag this so it is not
forgotten when the rig comes online.

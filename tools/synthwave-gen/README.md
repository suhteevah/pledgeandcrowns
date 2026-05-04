# tools/synthwave-gen

Generates the Pledge & Crown synthwave score using
[Stable Audio Open 1.0](https://huggingface.co/stabilityai/stable-audio-open-1.0)
on the local GPU. Outputs WAVs under `game/assets/audio/`.

## Why Python

This is one of the explicit exceptions to the Rust-first rule. There is
no native Rust port of `diffusers` / Stable Audio Open. The HuggingFace
toolchain is Python-canonical and trying to bridge through a Rust shim
would cost more than the bench-time it would save. The driver is small
(~150 LOC), pinned, and only runs at asset-bake time.

## One-time setup

```powershell
powershell -ExecutionPolicy Bypass -File scripts/synthwave-gen.ps1 --dry-run
```

Triggers the venv creation + dep install (torch with CUDA 12.1 + diffusers
+ transformers + soundfile, ~3 GB total). The `--dry-run` flag makes it
do that work without trying to load the model or write any files. After
this, normal runs are quick.

## Generate the score

```powershell
# Generate any tracks not already present in game/assets/audio/
powershell -ExecutionPolicy Bypass -File scripts/synthwave-gen.ps1

# Force-regenerate everything (overwrites existing WAVs)
powershell -ExecutionPolicy Bypass -File scripts/synthwave-gen.ps1 --force

# Regenerate just two tracks
powershell -ExecutionPolicy Bypass -File scripts/synthwave-gen.ps1 --only title,epilogue

# Dry run: show plan, write nothing
powershell -ExecutionPolicy Bypass -File scripts/synthwave-gen.ps1 --dry-run

# Override the inference step count (manifest default is 200)
powershell -ExecutionPolicy Bypass -File scripts/synthwave-gen.ps1 --steps 100
```

Each track takes roughly:

| GPU         | dtype | steps | seconds per track |
|-------------|-------|-------|-------------------|
| RTX 3070 Ti | fp16  | 200   | ~30-60 s          |
| Tesla P100  | fp16  | 200   | ~60-90 s (no fp16 accel on Pascal) |

Six tracks in the default manifest, so a full re-bake is roughly 5-10 min
on the 3070 Ti.

## VRAM

Stable Audio Open 1.0 at fp16 peaks around 6-7 GB during generation. Fits
in the 3070 Ti's 8 GB with room. P100 16 GB owners can drop the fp16
override (edit `generate.py`, change `torch.float16` -> `torch.float32`)
for slightly better numerical fidelity.

## License

Stable Audio Open 1.0 ships under the Stability AI Community License.
Commercial use is allowed under their revenue/use-case threshold; this
project is well below it. Per-asset commercial use is fine.

## Manifest

Edit `manifest.json` to add tracks, change prompts, or pin different
seeds. Each track entry:

```jsonc
{
  "name": "epilogue",         // becomes game/assets/audio/epilogue.wav
  "prompt": "...",            // free-text prompt
  "duration_s": 20.0,         // 1.0-47.0; longer takes more VRAM and time
  "seed": 1060                // pin for reproducibility; bump for a fresh take
}
```

The `_negative_prompt` field at the top of the manifest applies to every
track; it is the place to add things you never want (vocals, low quality,
distortion, etc.). The `_default_steps` and `_default_cfg` fields apply
unless overridden via CLI.

## Once a P100 is in the rig

The manifest stays the same. Optional upgrades:
- Drop `torch_dtype=torch.float16` for fp32 inference.
- Crank `_default_steps` higher (300-500) for higher fidelity at the cost
  of a few extra seconds per track.
- Try other genres / variants by adding manifest entries — the model
  handles a much wider style range than synthwave alone.

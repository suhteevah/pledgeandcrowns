# Design-mode prompts — NPC sprite batch

> Generated 2026-06-19 by art-lead. One copy-paste-ready `claude.ai/design` prompt
> per NPC spec in `design/art/specs/`. Each prompt is **fully standalone**: the
> locked Style Suffix from `design/04-art-handoff-prompts.md` is pasted verbatim
> (palette list included) into every block, then `[RESOLUTION]` is filled to
> `32×32 pixels, transparent background.` and `[SUBJECT-SPECIFIC INSTRUCTIONS]` is
> filled from the spec's Visual brief + Lighting, written as flowing prose with
> palette **hex** inline (translated from the spec's single-char codes / role
> names via `design/art/palette.js` + `design/03-art-style-bible.md`).

## Workflow

1. Open `claude.ai/design` in Matt's logged-in Chrome (via `claude-in-chrome` MCP).
2. Paste **one** prompt block below **verbatim** — do not summarize or "you know
   what I mean" the Style Suffix (per the workflow note in `04-art-handoff-prompts.md`).
3. **Generate 4 variants** per prompt. Pick the best; regenerate the rest.
4. Save the best draft screenshot to `design/art/drafts/<slug>-<n>.png`
   (e.g. `design/art/drafts/quartermaster-1.png`).
5. **Stop and get Matt's approval** on the draft before any JSX/ASCII-grid
   conversion. Matt is the single approver. Rejects → `design/art/drafts/rejected/<slug>-<n>.png`
   with a one-line note.
6. Only after approval: convert the approved screenshot to `refs/ref-<NN>-<slug>.jsx`,
   render via `cargo run -p render-refs --release`, wire into the game.

## Priority order

The **9 flagged-rough** NPCs (first-pass sprites already in the repo, pending
redesign through the canonical claude.ai/design path) come **first**, in this
order:

1. Quartermaster (REF-26 · `slice_basic`)
2. Auditor (REF-27 · `result_question_mark`)
3. Chronicler (REF-28 · `derive_debug`)
4. Alchemist (REF-29 · `iter_map_collect`)
5. Locksmith (REF-32 · `if_let`)
6. Porter (REF-33 · `while_let`)
7. Armorer (REF-35 · `enum_data_match`)
8. Lanternkeeper (REF-40 · `lifetimes`)
9. Loremaster (REF-41 · `assoc_type`)

After those nine, the remaining NPCs follow in curriculum **REF order**
(REF-11 → REF-65, skipping the nine already listed above).

---

# Priority NPCs (flagged rough)

### 26 — The Quartermaster (slice_basic — `fn sum_slice(xs: &[i32]) -> i32`)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = A stout, square-shouldered quartermaster, the village supply officer, standing three-quarter-front and filling the centre of the frame. He wears a knee-length burgundy gambeson in Crypt (#3E1220) with Inkblood (#1B0810) shadow and Wineflesh (#982D52) mid-tones, a plain Bronze (#5E4116) buckle on a wide leather belt — practical, no sash, no sigil, because he is staff, not gentry. Behind and beside him, spanning the lower-right third, a Bog umber (#2D1F0A) supply rack holds a row of five identical grain sacks in Old gold (#D2A53F) with Brass leaf (#F0D27D) highlights and Bronze (#5E4116) ties — a contiguous run. A slim Antique brass (#9C7026) bracket/caliper frames exactly three of the five sacks (a sub-run) — its two arms a pointer and an end, the gap between them the length. His right hand holds a Coalblack (#161313) notched tally rod raised toward the framed sacks but not touching them — reading, not taking; his left hand rests open and empty at his side. A small Aged paper (#BFB2A0) chit (the returned sum) is tucked under the belt. Head: broad, clean-shaven, Stone grey (#7A7064) cropped hair with an Aged paper (#BFB2A0) highlight, weathered pale skin from Pink quartz (#EBC2CC) highlight through Dusty rose (#C56883) mid to Wineflesh (#982D52) shadow, a level appraising gaze toward the bracketed sacks, and a single Antique brass (#9C7026) collar stud. No magic violet, no alarm scarlet, no specular white, no cobalt, no teal, no forest. Mood: methodical, dutiful, careful. Class signature color: Old gold (#D2A53F). Lighting from top-left at 45°: the three bracketed sacks catch Brass leaf (#F0D27D) highlight (the borrowed window is what's lit and counted) while the two unframed sacks fall a step toward Bronze (#5E4116) shadow; the brightest pixel is the caliper arm tip on the lit sack — the slice's start pointer, where the count begins. Single still idle frame.
```

### 27 — The Auditor (result_question_mark — parse + propagate with `?`)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = A severe, upright auditor, narrow-shouldered, standing front-on and still as a plumb line. She wears a floor-length burgundy robe in Crypt (#3E1220) with Inkblood (#1B0810) shadow and Oxblood (#6B1F35) mid, buttoned to a high Aged paper (#BFB2A0) collar — austere, joyless, exact. In her left hand, held chest-high, an open audit ledger: two Parchment cream (#FCEFC8) pages with a column of short Coalblack (#161313) tally rows. Her right hand holds a brass-rimmed loupe (Old gold #D2A53F ring, Brass leaf #F0D27D specular on the glass) to one eye, the eye and loupe aimed at one specific row partway down — the first failure. Pinned on that exact row is a single small Alarm scarlet (#E63946) wax seal / flag — the only warm-danger pixel in the sprite (roughly 0.6% of canvas, under the 1% alarm cap, a deliberate documented exception because this NPC's whole lesson is the error) — and everything below it on the page is left blank and unread, because the function already returned. Head: hair scraped back into a tight Stone grey (#7A7064) bun, sharp pale skin from Pink quartz (#EBC2CC) through Dusty rose (#C56883) to Wineflesh (#982D52), a thin pressed mouth, and an Antique brass (#9C7026) chain holding the loupe. No magic violet, no specular white, no cobalt, no teal, no forest. Mood: severe, exacting, joyless. Class signature color: Old gold (#D2A53F) — with the single Alarm scarlet (#E63946) seal as the lesson accent. Lighting from top-left at 45°: the lit half of the open ledger is the rows above the flag (the Ok values already read, in Parchment cream #FCEFC8) while the rows below the Alarm scarlet (#E63946) seal sit in Basalt (#3E3833) shadow, unreached because `?` bailed; the brightest non-alarm pixel is the Brass leaf (#F0D27D) loupe specular — the instant of inspection where the Err is caught. Single still idle frame.
```

### 28 — The Chronicler (derive_debug — `#[derive(Debug)]`, print with `{:?}`)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = A tall, still robed historian standing left-of-centre with hands clasped behind the back — emphatically NOT writing. He wears a floor-length burgundy scholar's robe in Crypt (#3E1220) with Inkblood (#1B0810) shadow and Wineflesh (#982D52) mid, a plain Bronze (#5E4116) clasp at the throat over an Aged paper (#BFB2A0) undercollar. To his right stands a lectern (Bog umber #2D1F0A post, Bronze #5E4116 trim) bearing a large open tome: two Parchment cream (#FCEFC8) pages already filling with neat Coalblack (#161313) rows. A small Antique brass (#9C7026) clockwork armature rises from the lectern and holds the quill, its gear teeth in Old gold (#D2A53F) with Brass leaf (#F0D27D) specular meshed at the elbow; the quill is upright and writing on its own with no hand touching it, a faint Brass leaf (#F0D27D) line of fresh ink trailing behind the nib — the auto-generated `{:?}` output appearing one field at a time. On the open left page, three short Coalblack (#161313) field-labels are ringed by a tiny Bronze (#5E4116) bracket — the struct's fields the derive reads from. Head: long Stone grey (#7A7064) hair to the collar with an Aged paper (#BFB2A0) highlight, lined pale skin from Pink quartz (#EBC2CC) through Dusty rose (#C56883) to Wineflesh (#982D52), eyes on the self-writing quill, and a single folded Antique brass (#9C7026) clerk's eyeglass arm at the temple. The self-writing is clockwork, NOT magic — keep it entirely off the arcane violet; no magic violet, no alarm scarlet, no specular white, no cobalt, no teal, no forest. Mood: patient, scholarly, observing. Class signature color: Old gold (#D2A53F). Lighting from top-left at 45° onto the open tome: the right page the armature is actively writing catches Parchment cream (#FCEFC8) highlight while the already-recorded left page sits a step toward Aged paper (#BFB2A0); the brightest pixel is the Brass leaf (#F0D27D) specular on the clockwork gear at the quill's elbow — the derive mechanism, the thing doing the work the player never wrote. Single still idle frame.
```

### 29 — The Alchemist (iter_map_collect — map a `Vec` through `|x| x * 2`, collect)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = An alchemist standing behind a workbench that fills the lower third of the frame, leaning slightly forward over the work. He wears a burgundy robe with rolled sleeves in Crypt (#3E1220) with Inkblood (#1B0810) shadow and Oxblood (#6B1F35) mid, and a scorched Bog umber (#2D1F0A) leather apron with a Bronze (#5E4116) buckle. The bench reads the whole algorithm left to right: on the LEFT, the input Vec — a row of three identical small vials each holding a shallow measure of Deep teal (#154548) reagent (the unmapped elements). In the CENTRE, the map — a small Antique brass (#9C7026) still/burner where one vial is mid-transmutation, its contents lifted to a brighter, fuller Bright teal (#5BB8AF) at twice the level (`|x| x * 2`), a thin Mist teal (#A4DED4) vapour curl rising from it. On the RIGHT, the collect — one large round-bottomed flask rimmed in Brass leaf (#F0D27D), already holding a deeper pool of the doubled Main teal (#2A8482) liquid (the new Vec), fed by a glass funnel with an Aged paper (#BFB2A0) glint. His right hand tips the mid-transform vial toward the funnel; his left steadies the collecting flask — one element completing the map→collect hop. Head: short Stone grey (#7A7064) hair, pale skin from Pink quartz (#EBC2CC) through Dusty rose (#C56883) to Wineflesh (#982D52), a leather strap holding a single Antique brass (#9C7026) loupe pushed up onto the brow, no beard, and a faint Bright teal (#5BB8AF) under-light on the jaw from the glowing flask. Keep teal as the alchemical fluid only and well under a quarter of the canvas. No magic violet, no alarm scarlet, no specular white, no cobalt, no forest. Mood: focused, curious, precise. Class signature color: Main teal (#2A8482). Lighting from top-left at 45° with a secondary Bright teal (#5BB8AF) glow rising from the collecting flask (the only non-top light) catching the jaw and the funnel underside; the brightest pixel is the Mist teal (#A4DED4) vapour curl at the burner — the instant of transformation, the `* 2` happening to one element. Single still idle frame.
```

### 32 — The Locksmith (if_let — match one variant, ignore the rest)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = A standing tradesman locksmith. He wears a Crypt (#3E1220) robe with Oxblood (#6B1F35) primary and Wineflesh (#982D52) lit folds, with Antique brass (#9C7026) collar studs — a workmanlike burgundy uniform. Raised high in his right hand, held clear above the shoulder, a single key singled out: an Old gold (#D2A53F) bow (ring-head) with a Brass leaf (#F0D27D) specular pixel, a Coalblack (#161313)-cored shaft, and an Old gold (#D2A53F) bit at the bottom — the one key that fits, the brightest gold mass and the highest point in the sprite. Below the raised key, the hand grips a Bronze (#5E4116)-banded key-ring from which two more Old gold (#D2A53F) keys hang, their shafts Coalblack (#161313)-cored with an Antique brass (#9C7026) shadow between them — the variants `if let` ignores. The hand reads in Pink quartz (#EBC2CC) / Dusty rose (#C56883). Head: short Stone grey (#7A7064) hair with Aged paper (#BFB2A0) highlight, clean-shaven, Basalt (#3E3833) brow and eyes, skin in Pink quartz (#EBC2CC) / Dusty rose (#C56883). Standard Old gold (#D2A53F) belt buckle, Pine (#27502E) hem, Bronze (#5E4116) / Bog umber (#2D1F0A) boots. No magic violet, no alarm scarlet, no specular white, no cobalt, no teal, no forest beyond the pine hem. Mood: deft, decisive, workmanlike. Class signature color: Old gold (#D2A53F). Lighting from top-left at 45°: the raised key takes the Brass leaf (#F0D27D) specular on its bow — the matching variant, lit — while the hanging ring stays in flatter Old gold (#D2A53F) with Antique brass (#9C7026) shadow, deliberately less bright: the ignored variants, present but unhandled. Single still idle frame.
```

### 33 — The Porter (while_let — loop popping while a variant remains)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = A labourer porter in a working tunic rather than a formal robe. He wears a Crypt (#3E1220) tunic with Oxblood (#6B1F35) primary and Wineflesh (#982D52) lit folds and a Bronze (#5E4116) belt band — plainer and shorter than the officer cast, a working garment. Both arms reach up and to his right, lifting one crate clear off the top of the stack: a Bog umber (#2D1F0A) crate with Antique brass (#9C7026) corner brackets and an Old gold (#D2A53F) carry-strap across its face — the current `Some(crate)`. His hands (Pink quartz #EBC2CC / Dusty rose #C56883) grip its underside. Beside him a stack of three more Bog umber (#2D1F0A) crates rises from the floor, each banded with Antique brass (#9C7026) corner brackets, Coalblack (#161313)-outlined, sitting on a Coalblack (#161313) floor line — the remaining stack the loop will keep draining. Head: short Stone grey (#7A7064) hair with Aged paper (#BFB2A0) highlight, Basalt (#3E3833) brow and eyes, skin in Pink quartz (#EBC2CC) / Dusty rose (#C56883). Bronze (#5E4116) / Bog umber (#2D1F0A) boots, Pine (#27502E) hem at the tunic. No magic violet, no alarm scarlet, no specular white, no cobalt, no teal, no forest beyond the pine hem. Mood: steady, labouring, methodical. Class signature color: Old gold (#D2A53F). Lighting from top-left at 45°: the lifted crate catches the most light — Old gold (#D2A53F) strap and Antique brass (#9C7026) brackets reading bright — because it is the element currently in hand; the stacked crates shade uniformly in Bog umber (#2D1F0A): the queued elements waiting their turn to be popped. Single still idle frame.
```

### 35 — The Armorer (enum_data_match — one enum, two payloads, exhaustive match)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = A lean tradesman armorer in a working tunic, standing by a weapon rack (not at an anvil — leaner than the Smith, no apron). He wears a Crypt (#3E1220) tunic with Oxblood (#6B1F35) primary and Wineflesh (#982D52) lit folds and a Bronze (#5E4116) belt band. To his frame-left, a weapon rack: a Bronze (#5E4116) top rail with Antique brass (#9C7026) pegs and spare Stone grey (#7A7064) blades (one tipped Aged paper #BFB2A0) hanging in its slots. In his left hand, raised, the Weapon variant: a Stone grey (#7A7064) blade with an Aged paper (#BFB2A0) highlight at the tip, a Coalblack (#161313) core, an Antique brass (#9C7026) hilt guard, a Bronze (#5E4116) grip, and an Old gold (#D2A53F) pommel — the variant carrying its Blade payload. In his right hand, held out, the Potion variant: a small vial of Main teal (#2A8482) fluid with a Bright teal (#5BB8AF) highlight, Coalblack (#161313)-outlined — the variant carrying its Vial payload; teal appears only on this vial. Head: short Stone grey (#7A7064) hair with Aged paper (#BFB2A0) highlight, Basalt (#3E3833) brow and eyes, skin in Pink quartz (#EBC2CC) / Dusty rose (#C56883). Old gold (#D2A53F) belt buckle area, Pine (#27502E) hem, Bronze (#5E4116) / Bog umber (#2D1F0A) boots. No magic violet, no alarm scarlet, no specular white, no cobalt, no forest beyond the pine hem; teal confined to the single potion vial. Mood: even-handed, practical, dutiful. Class signature color: Old gold (#D2A53F). Lighting from top-left at 45°: the raised blade catches Aged paper (#BFB2A0) at the tip and the Old gold (#D2A53F) pommel anchors the bottom of the weapon; the teal vial glows with a Bright teal (#5BB8AF) highlight on its right — the two variants lit equally, because a `match` must honor both. Single still idle frame.
```

### 40 — The Lanternkeeper (lifetimes — a borrow kept alive for `'a`)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = A figure offset toward frame-left, the right arm raised to hold a lantern aloft in the upper-right of the frame. The lantern hangs from an Antique brass (#9C7026) ring on a short Coalblack (#161313) chain; its body is an Old gold (#D2A53F) glass case, Coalblack (#161313)-framed, with an Antique brass (#9C7026) cap and base. Inside, a small steady Mage glow (#9D6FE0) flame with a Royal arcane (#3A1559) base shadow — contained, never escaping its glass, the one magic accent and well under the 5% cap. The raised hand (Pink quartz #EBC2CC / Dusty rose #C56883) cups the handle. He wears a Crypt (#3E1220) robe with Oxblood (#6B1F35) primary and Wineflesh (#982D52) lit folds. Head: short Stone grey (#7A7064) hair with Aged paper (#BFB2A0) highlight, Basalt (#3E3833) brow and eyes, skin in Pink quartz (#EBC2CC) / Dusty rose (#C56883). An Antique brass (#9C7026) belt band with Old gold (#D2A53F), Pine (#27502E) hem, Bronze (#5E4116) / Bog umber (#2D1F0A) boots. No alarm scarlet, no specular white, no cobalt, no teal, no forest beyond the pine hem. Mood: steadfast, watchful, careful. Class signature color: Mage glow (#9D6FE0) (the contained flame), framed in Old gold (#D2A53F). Lighting from top-left at 45°: the robe's left folds catch Wineflesh (#982D52); the lantern is the bright focal point in the upper right, and the Mage glow (#9D6FE0) flame is the brightest mass but is fully enclosed in Old gold (#D2A53F) glass — the borrow kept alive, contained to its lifetime, never spilling onto the rest of the figure. Single still idle frame.
```

### 41 — The Loremaster (assoc_type — an associated type `type Output` on a trait)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = A scholar, centered and standing, holding an open tome forward at waist height with both hands bracing the outer edges. The tome is Old gold (#D2A53F)-trimmed with a Bronze (#5E4116) spine down the center and an Antique brass (#9C7026) clasp; its two Parchment cream (#FCEFC8) pages each carry a small Coalblack (#161313) inscription — the left page an input glyph, the right page its bound output glyph, one mapping, the `type Output` written as lore. He wears a Crypt (#3E1220) scholar's robe with Oxblood (#6B1F35) primary and Wineflesh (#982D52) lit folds; the hands (Pink quartz #EBC2CC / Dusty rose #C56883) brace the book. Head: short Stone grey (#7A7064) hair with Aged paper (#BFB2A0) highlight, Basalt (#3E3833) brow and eyes, skin in Pink quartz (#EBC2CC) / Dusty rose (#C56883). Pine (#27502E) hem, Bronze (#5E4116) / Bog umber (#2D1F0A) boots. This is lore, not spellcraft — no magic violet, no alarm scarlet, no specular white, no cobalt, no teal, no forest beyond the pine hem. Mood: learned, measured, serene. Class signature color: Old gold (#D2A53F). Lighting from top-left at 45°: the robe's left folds catch Wineflesh (#982D52); the open Parchment cream (#FCEFC8) pages are the brightest mass, framed in Old gold (#D2A53F) — the lore lit and held forward, the eye landing on the two-glyph mapping, the associated type made readable. Single still idle frame.
```

---

# Remaining NPCs (curriculum / REF order)

### 11 — The Smith (mut_binding — `let mut` + compound-assignment)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = A stocky human male village blacksmith, broad shoulders, leather apron, thick forearms, standing square-on (front-facing) gripping a hammer at his right side. The apron is Oxblood (#6B1F35) with Wineflesh (#982D52) mid-tone and Crypt (#3E1220) shadow — burgundy is the brand, worn as the village craft-uniform. Beneath the apron a Pine (#27502E)-shadowed undershirt, a brown leather belt with an Old gold (#D2A53F) buckle, and Bronze (#5E4116) / Bog umber (#2D1F0A) boots. Skin is pale: Pink quartz (#EBC2CC) highlight through Dusty rose (#C56883) mid to Wineflesh (#982D52) shadow. Beard is Bronze (#5E4116) / Antique brass (#9C7026), the gold ramp doing double-duty as warm hair; eyes a single dark pixel each. The hammer head is Basalt (#3E3833) iron core with Stone grey (#7A7064) mid-tone and a single Brass leaf (#F0D27D) highlight pixel on the strike face — the only specular bright on the sprite, where mutation happens; the haft is Bronze (#5E4116) with Bog umber (#2D1F0A) shadow, gripped diagonally so the head sits at his right hip, signalling "tool ready, not in use." No magic violet, no alarm scarlet, no specular white. Mood: solid, honest, gruff-but-kind. Class signature color: Oxblood (#6B1F35). Lighting from top-left at 45°: highlights on the left edges of apron, beard, and hammer head; shadows down-right. Single still idle frame.
```

### 12 — The Cartographer (if_else_sign — `if` / `else if` / `else` as branching expressions)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = A slim human female cartographer of traveller's build, standing three-quarter front. She wears a Cobalt deep (#0E2E54) travelling cloak with Cobalt (#377AB8) mid-tone — keyed off the cool counterweight because mapping the world is the perspective-from-outside it. Hood pulled back, long hair in Crypt (#3E1220) / Basalt (#3E3833) with a single Stone grey (#7A7064) highlight strand on the lit side. Across her chest a Bronze (#5E4116) strap with an Old gold (#D2A53F) buckle holds a rolled Aged paper (#BFB2A0) map at her left hip — a compact roll with Antique brass (#9C7026) end-caps and the corner of an unrolled Parchment cream (#FCEFC8) flap showing one Forest (#487E40) road glyph and one Old gold (#D2A53F) crossroads marker where the road forks into two: the only literal branch visible on the sprite. Skin is the pale ramp: Pink quartz (#EBC2CC) / Dusty rose (#C56883) / Wineflesh (#982D52). Belt is Bronze (#5E4116), boots Bog umber (#2D1F0A). In her right hand at waist height a small brass compass: Old gold (#D2A53F) ring, Coalblack (#161313) needle, Pink quartz (#EBC2CC) north tick — what tells you which branch you're on. No magic violet, no specular white, no alarm scarlet. Mood: thoughtful, observant, wry. Class signature color: Cobalt deep (#0E2E54). Lighting from top-left at 45°: the cloak's left edge gets the Cobalt (#377AB8) mid-tone highlight; the right side stays in Cobalt deep (#0E2E54). Single still idle frame.
```

### 13 — The Trait Mage (double_function — `fn` definitions + explicit return types)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = A tall slender human, robed and hooded but with the hood thrown back — she is not concealing herself in the village. The robe is Oxblood (#6B1F35) primary with Crypt (#3E1220) shadow and Wineflesh (#982D52) mid-tone, with Old gold (#D2A53F) trim at hem and cuffs (a scholar of the Crown's lore as much as of magic); the inside-hood lining shows Royal arcane (#3A1559) in just two pixels along the collar — a peek at her Tower allegiance. She carries a vertical staff in her right hand, hilt at her hip, top above her head: Basalt (#3E3833) shaft with Stone grey (#7A7064) mid-tone, its head a small Mage glow (#9D6FE0) crystal of 3-4 pixels framed by an Old gold (#D2A53F) ring — the entire magic-violet allowance (under 0.8% of canvas). Skin is the pale ramp: Pink quartz (#EBC2CC) / Dusty rose (#C56883) / Wineflesh (#982D52). Hair is Crypt (#3E1220) / Basalt (#3E3833), tied back; eyes a single Bright teal (#5BB8AF) pixel each — a faint scholar's glow hinting she sees more than she says. No alarm scarlet, no specular white, no cobalt. Mood: serene, knowing, scholarly. Class signature color: Mage glow (#9D6FE0). Lighting from top-left at 45°: the robe's left side gets Wineflesh (#982D52), the right Crypt (#3E1220); the staff crystal glows uniformly — magic light is its own source and does not take the global gradient. Single still idle frame.
```

### 14 — The Bellringer (loop_break — `loop { ... break value; }`)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = A wiry human male bellringer in simple village dress: a Forest (#487E40) green tunic over Pine (#27502E) trousers (Hearthstone Village's forest/spring-meadow palette), a brown leather belt with an Old gold (#D2A53F) buckle, Bronze (#5E4116) / Bog umber (#2D1F0A) boots, sleeves rolled. Both arms reach up to grip a single thick rope descending from above the canvas: Antique brass (#9C7026) core with Bronze (#5E4116) shadow, two visible knots at handhold height. Above the rope, in the top 8 pixels, hangs the lower curve of an Old gold (#D2A53F) bell with Bog umber (#2D1F0A) shadow inside its mouth — only the bottom visible, the rest extending beyond frame — with a faint Brass leaf (#F0D27D) highlight pixel on the bell's rim. Skin is the pale ramp: Pink quartz (#EBC2CC) / Dusty rose (#C56883) / Wineflesh (#982D52). Hair short, Bronze (#5E4116) / Antique brass (#9C7026). A single Mist teal (#A4DED4) dot on each side of his head (no more than two pixels each side) suggests the ring — decorative sound-wave idiom, not magic. No magic violet, no alarm scarlet, no specular white, no cobalt. Mood: cheerful, energetic, plain. Class signature color: Old gold (#D2A53F). Lighting from top-left at 45°: the bell catches the highlight on its left rim, the tunic shadow falls to the right, the rope is uniform tone. Single still idle frame.
```

### 15 — The Cooper (vec_iter — `Vec<T>` + `.iter().sum()`)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = A burly human male barrel-maker, mid-height, broad-chested. He wears a Bronze (#5E4116) / Bog umber (#2D1F0A) leather work-vest over a Hayfield (#C9DC97)-cream undershirt (a touch of light against the dark vest), brown trousers in Bronze (#5E4116) / Bog umber (#2D1F0A), and Bog umber (#2D1F0A) leather work-gloves. Skin is the pale ramp: Pink quartz (#EBC2CC) / Dusty rose (#C56883) / Wineflesh (#982D52). Hair short, Crypt (#3E1220) / Basalt (#3E3833) — the dark-haired counterpoint to the gold-haired cast. Occupying the right third of the canvas, a small barrel: a vertical stack of three wooden staves (Bronze #5E4116 core with Bog umber #2D1F0A shadows and Antique brass #9C7026 highlight on the lit side) bound by two Stone grey (#7A7064) iron hoops with a single Brass leaf (#F0D27D) highlight pixel each, and an Old gold (#D2A53F) lid hinting at valuable contents. His right hand rests on top of the barrel, asserting ownership. In his left hand at hip height a small Basalt (#3E3833)-handled adze with a Stone grey (#7A7064) blade. Belt is Bronze (#5E4116) with an Old gold (#D2A53F) buckle. No magic violet, no alarm scarlet, no specular white, no cobalt, no teal. Mood: sturdy, good-natured, proud of the craft. Class signature color: Old gold (#D2A53F). Lighting from top-left at 45°: the barrel staves' left edge gets the Antique brass (#9C7026) highlight, the right edge stays in Bog umber (#2D1F0A); the vest shoulder gets a tiny Antique brass (#9C7026) highlight on its left side. Single still idle frame.
```

### 16 — The Oracle (match_option — `match` on `Option<T>`)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = A tall, slender robed oracle, veiled from forehead to nose-bridge — a single horizontal Crypt (#3E1220)-shadow band stands in for the blindfold (canonically Oracle-blind). Below the veil her mouth is set in a calm half-smile (one Wineflesh #982D52 pixel under the chin line). She holds a small spherical orb at chest height in cupped hands; the orb is two halves, pixel-for-pixel: the upper half Brass leaf (#F0D27D) (the Some answer, illuminated) and the lower half Inkblood (#1B0810) (the None answer, present-but-empty), ringed once in Old gold (#D2A53F). Robe is Crypt (#3E1220) primary with Inkblood (#1B0810) deep shadow and Oxblood (#6B1F35) mid — darker than the Trait Mage, befitting a chthonic seer — with Stone grey (#7A7064) trim at hem and sleeves (no gold trim; she is older than Crown lore). One-pixel hint of bare toes peeking from the hem; hair Crypt (#3E1220) loose past the shoulders. Skin is the pale ramp leaning on Wineflesh (#982D52): Pink quartz (#EBC2CC) / Dusty rose (#C56883) / Wineflesh (#982D52). No magic violet, no alarm scarlet, no specular white, no cobalt, no teal. Mood: solemn, ancient, calm. Class signature color: Brass leaf (#F0D27D) (the lit Some half). Lighting from top-left at 45°: the robe is lit Wineflesh (#982D52) on the left edge and Inkblood (#1B0810) on the right; the orb is its own light source, its top half always Brass leaf (#F0D27D) regardless of the global gradient — the Some half is the highlight. Single still idle frame.
```

### 17 — The Herald (struct_basic — struct definition with named fields)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = A trim, formal herald, feet together, shoulders square. He wears a tabard over a Pine (#27502E) undertunic. The tabard is the focal point: three horizontally-banded blocks down the front, each a distinct heraldic field — top band Oxblood (#6B1F35), middle band Old gold (#D2A53F), bottom band Forest (#487E40) — with one thin Coalblack (#161313) rule between each band like a struct's separators, and a Brass leaf (#F0D27D)-trimmed hem. In his right hand at his side a small rolled scroll: pure Parchment cream (#FCEFC8) core with an Antique brass (#9C7026) seal on the visible end. Belt is Bronze (#5E4116) with a small Old gold (#D2A53F) buckle. Head: short Bronze (#5E4116) hair, square-cut; skin the pale ramp Pink quartz (#EBC2CC) / Dusty rose (#C56883) / Wineflesh (#982D52); one Coalblack (#161313) pixel each for eyes, mouth a single Wineflesh (#982D52) pixel in a neutral line. No magic violet, no alarm scarlet, no specular white, no teal. Mood: ceremonial, formal, dignified. Class signature color: Oxblood (#6B1F35). Lighting from top-left at 45°: each tabard band gets a one-pixel brighter row at its top edge — Wineflesh (#982D52) on Oxblood, Brass leaf (#F0D27D) on Old gold, Spring meadow (#82B450) on Forest — selling the layered-fields read. Single still idle frame.
```

### 18 — The Twin (tuple_destructure — `let (a, b) = pair`)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = A single front-facing 32×32 humanoid in a standard idle pose, whose tunic is bisected vertically down the centre column. The figure's left half (viewer's right) is Oxblood (#6B1F35) with Wineflesh (#982D52) mid and Crypt (#3E1220) shadow; the figure's right half (viewer's left) is Forest (#487E40) with Spring meadow (#82B450) mid and Pine (#27502E) shadow. Skin and outline are unified across the centreline — the same Pink quartz (#EBC2CC) / Dusty rose (#C56883) ramp on both sides — so the read is "one body, two field-names." Hair is split too: viewer's right is Crypt (#3E1220), viewer's left is Bronze (#5E4116). One Old gold (#D2A53F) circlet across the brow ties the halves together — the parens of the destructure. Belt is Stone grey (#7A7064) with a single Antique brass (#9C7026) clasp at centre — the comma between the bindings. Both arms hang at the sides; the left hand wears a single Pink quartz (#EBC2CC) ring (`a`), the right hand a single Hayfield (#C9DC97) ring (`b`). No magic violet, no alarm scarlet, no specular white, no cobalt, no teal. Mood: balanced, calm, uncanny-but-friendly. Class signature color: Oxblood (#6B1F35) and Forest (#487E40) split equally. Lighting from top-left at 45° applies per half: each side gets its own internal highlight, and the bisecting centre column reads sharp where the two ramps meet at maximum contrast. Single still idle frame.
```

### 19 — The Tinker (while_loop — `while cond { ... }`)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = A stocky village fix-it artisan (shorter than the Smith, broader than the Bellringer) wearing a leather hood thrown back over short Bronze (#5E4116) hair, with goggles pushed up on his forehead — two adjacent Stone grey (#7A7064) rectangles ringed in Coalblack (#161313). He wears a forest-green vest (Forest #487E40 primary, Pine #27502E shadow, Spring meadow #82B450 highlight) over a Bog umber (#2D1F0A) undershirt, sleeves rolled to the elbow. The belt is the focal point: four alternating pouches reading brass-bronze-brass-bronze — Antique brass (#9C7026) and Bog umber (#2D1F0A) — a row of repeated cells, the iterations of the loop. A small Old gold (#D2A53F) gear hangs at the right hip from a one-or-two-pixel Stone grey (#7A7064) chain. Trousers are Bog umber (#2D1F0A); boots Bronze (#5E4116) with Coalblack (#161313) soles. In his right hand against the thigh a small mallet — head Basalt (#3E3833), haft Bronze (#5E4116) — at rest like the Smith's hammer but smaller. Skin is the pale ramp Pink quartz (#EBC2CC) / Dusty rose (#C56883) / Wineflesh (#982D52); eyes one Coalblack (#161313) pixel each; mouth a thin Wineflesh (#982D52) line in tinker-concentration. No magic violet, no alarm scarlet, no specular white, no cobalt, no teal. Mood: busy, precise, tinkering. Class signature color: Forest (#487E40). Lighting from top-left at 45°: highlights on the left edges of vest, hood, and mallet head; goggle lenses pick up no specular. Single still idle frame.
```

### 20 — The Heraldic Sage (enum_match — `enum` definition + `match` on its variants)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = An older human sage, slim and tall, in a long deep-burgundy formal robe — Crypt (#3E1220) primary, Inkblood (#1B0810) shadow, Oxblood (#6B1F35) mid — with a wide Old gold (#D2A53F) sash across the chest from left shoulder to right hip. Stitched onto the sash, left-to-right, are four small 3×3 sigil panels, each bordered in Coalblack (#161313): a Forest (#487E40) square (oak field), a Bright teal (#5BB8AF) triangle (river), an Old gold (#D2A53F) lozenge (sun), and a Cobalt (#377AB8) diamond (shield-azure). The four-sigil read is the load-bearing detail — keep the rest of the sprite quiet so the panels carry the eye. The small Cobalt (#377AB8) azure sigil is a deliberate ~0.9%-of-canvas foreshadowing signpost (one 3×3 patch, inside the 1% cap). Hair is white-streaked Stone grey (#7A7064) past the ears, with a neat Bronze (#5E4116) beard; skin the pale ramp Pink quartz (#EBC2CC) / Dusty rose (#C56883) / Wineflesh (#982D52); hands folded in front at waist height. The robe hem trails one row past the boots with Brass leaf (#F0D27D) stitching. No magic violet (the Sage is not a Tower scholar), no alarm scarlet, no specular white. Mood: wise, ceremonial, lexicographic. Class signature color: Old gold (#D2A53F). Lighting from top-left at 45° on the body: sash and sigil panels are flat-rendered (heraldry is read graphically, not modelled); the robe gets the standard Wineflesh (#982D52)-on-left / Inkblood (#1B0810)-on-right gradient. Single still idle frame.
```

### 21 — The Forgewright (borrow_mut — exclusive borrow `&mut T`)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = A wiry, focused human, smaller-framed than the Smith but in the same village burgundy uniform — apron Oxblood (#6B1F35) with Wineflesh (#982D52) mid and Crypt (#3E1220) shadow, cut shorter than the Smith's so they read apart at a glance. No hammer. Instead the figure grips a long pair of Basalt (#3E3833) / Stone grey (#7A7064) iron tongs in front, both hands on the haft, jaw set. The tong-jaws clamp a small workpiece glowing with an Old gold (#D2A53F) core and a single Brass leaf (#F0D27D) specular pixel — that pixel is the value being mutated through the deref `*x`. The forge fire is off-canvas (no scarlet); the glow on the workpiece comes only from the gold ramp. Hair is short Bronze (#5E4116) pulled back; an Antique brass (#9C7026) beard kept tight (one row deeper than the Smith's full beard). Skin is the pale ramp Pink quartz (#EBC2CC) / Dusty rose (#C56883) / Wineflesh (#982D52). Belt with Old gold (#D2A53F) buckle; boots Bronze (#5E4116) / Bog umber (#2D1F0A). Pose: square front, weight balanced, both arms forward in a tight controlled hold — heraldic for "I have it, no one else does." No magic violet, no alarm scarlet, no specular white, no cobalt. Mood: intense, controlled, exacting. Class signature color: Old gold (#D2A53F). Lighting from top-left at 45°: the gold workpiece is the only bright on the sprite — a single Brass leaf (#F0D27D) pixel at its top-left edge — pulling the eye to the mutation point. Single still idle frame.
```

### 22 — The Linguist (string_vs_str — `&str` parameter accepting `String` and a literal)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = A slim scholar in a long Crypt (#3E1220) robe with a Parchment cream (#FCEFC8) undertunic showing at collar and hem — a neutral, bookish silhouette. An Aged paper (#BFB2A0) sash at the waist tied off-centre. Two-toned hair: Stone grey (#7A7064) under, a single row of Aged paper (#BFB2A0) on top — old, but not Sage-old (reads as forty). In the LEFT hand, held open at chest height, a small Parchment cream (#FCEFC8) book with Bog umber (#2D1F0A) binding and a single Old gold (#D2A53F) pixel for the clasp — the owned `String`, weighty, contained. In the RIGHT hand, held lower at the hip, a tightly rolled Parchment cream (#FCEFC8) scroll with Bronze (#5E4116) end-caps — the borrowed `&str` literal, light enough to dangle. The two props match in colour family (both parchment) so the read is "same content, two containers." A small Brass leaf (#F0D27D) speech curl rises from the figure's lips (1-2 pixels) — the "tongue." Skin is the pale ramp Pink quartz (#EBC2CC) / Dusty rose (#C56883) / Wineflesh (#982D52). No staff, no hat, no insignia. No magic violet, no alarm scarlet, no specular white, no cobalt, no teal, no forest. Mood: precise, articulate, unfussy. Class signature color: Old gold (#D2A53F). Lighting from top-left at 45°: the book (left, lit side) gets a Parchment cream (#FCEFC8) highlight row; the scroll (right, shadow side) gets a Bronze (#5E4116) shadow row — reinforcing that the owned and borrowed forms are the same content read under different light. Single still idle frame.
```

### 23 — The Pilgrim (option_unwrap_or — `.unwrap_or(default)`)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = A lean traveller, hooded, with a slight forward lean implying a moderate stride. A long road-cloak in Pine (#27502E) (forest mid) with Mossbed (#142A19) shadows and a single row of Spring meadow (#82B450) highlight along the left edge — green, not burgundy: the Pilgrim is not of the village uniform. Hood up, casting his upper face in Inkblood (#1B0810) shadow — only the lower jaw and a glint of one eye visible (a single Stone grey #7A7064 pixel). An Aged paper (#BFB2A0) wrap around the chin like a pilgrim's scarf. In the right hand, held away from the body at hip height, an iron-frame lantern: Coalblack (#161313) outline cage, Basalt (#3E3833) body, with the glass a 2×2 patch of Old gold (#D2A53F) and one Brass leaf (#F0D27D) specular pixel as the flame core — the only bright on the sprite. A short Bronze (#5E4116) chain links lantern to hand. A walking staff in the left hand, Bronze (#5E4116) with a Stone grey (#7A7064) ferrule. Boots Bog umber (#2D1F0A), with one row of Stone grey (#7A7064) dust flecks at the boot tops. Skin: the visible jaw in Pink quartz (#EBC2CC) / Dusty rose (#C56883). Two Antique brass (#9C7026) pixels on the hood's lower-right edge imply the lantern glow tinting the surroundings, as fallbacks do. No magic violet, no alarm scarlet, no specular white, no cobalt, no teal, no burgundy. Mood: weary, resolute, far-travelled. Class signature color: Old gold (#D2A53F) (the lantern flame). Lighting from top-left at 45° but sourced from the lantern itself: the left edge of the figure (closer to the lantern) gets the implied glow; the right edge stays in Mossbed (#142A19). Single still idle frame.
```

### 24 — The Drillmaster (for_in_range — `for i in 0..10`)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = A tall, square-shouldered human in a parade-ground uniform tunic — Forest (#487E40) mid, Pine (#27502E) shadow, Spring meadow (#82B450) highlight on the left shoulder — over Bronze (#5E4116) trousers tucked into Bog umber (#2D1F0A) boots. Antique brass (#9C7026) epaulettes (one pixel each shoulder) and an Old gold (#D2A53F) chest-stripe running from left shoulder to right hip — military, not heraldic. Stone grey (#7A7064) hair shorn close, square jaw, a single Bronze (#5E4116) moustache row. The right arm is held tight against the side with a short Crypt (#3E1220) baton tucked under the upper arm, a Brass leaf (#F0D27D) end-cap pip just visible at the right hip. The left hand is held forward at chest height, palm out, with three fingers raised distinctly — the visible cadence count, the iteration in progress. A small slate at the right hip strapped with Bronze (#5E4116): a 4×3 patch of Coalblack (#161313) with three Aged paper (#BFB2A0) chalked tally pixels, framed in Stone grey (#7A7064). Skin the pale ramp Pink quartz (#EBC2CC) / Dusty rose (#C56883) / Wineflesh (#982D52). Stance: feet shoulder-width, perfectly square and symmetrical. No magic violet, no alarm scarlet, no specular white, no cobalt, no teal. Mood: rigid, clipped, parade-ground. Class signature color: Old gold (#D2A53F). Lighting from top-left at 45°: the chalked tallies on the slate are the second-brightest element after the chest-stripe, readable as the iteration counter; body shadow on the right keeps the silhouette crisp. Single still idle frame.
```

### 25 — The Reckoner (closure_basic — `let add = |a, b| a + b`)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = A desk-clerk accountant, readable as standing but bowed forward at the waist over a desk that fills the lower third of the frame. A long Crypt (#3E1220) robe with Inkblood (#1B0810) shadow and Wineflesh (#982D52) mid — clerical burgundy, plainer than the Sage (no sash, no sigils) — with an Aged paper (#BFB2A0) undercollar at the throat. The desk is a Bog umber (#2D1F0A) bench-top across the lower frame with Bronze (#5E4116) trim along the leading edge. On it an open ledger: two Parchment cream (#FCEFC8) pages, three short Coalblack (#161313) tally rows on the left page and a single Coalblack (#161313) sum-line at the bottom of the right page — `a + b` and the result — with an Old gold (#D2A53F) ribbon bookmark trailing over the edge. The right hand grips a Coalblack (#161313) quill with a Brass leaf (#F0D27D) specular pixel on the nib, mid-stroke just above the right page — the closure body. The left hand rests flat on the left page, fingers spread, anchoring the binding. Head bowed: short Stone grey (#7A7064) hair with Aged paper (#BFB2A0) highlight, the face foreshortened (only forehead, brow, nose-bridge visible), no beard, and a single Antique brass (#9C7026) earring on the visible side. Skin the pale ramp Pink quartz (#EBC2CC) / Dusty rose (#C56883) / Wineflesh (#982D52). No magic violet, no alarm scarlet, no specular white, no cobalt, no teal, no forest. Mood: hunched, diligent, terse. Class signature color: Old gold (#D2A53F). Lighting from top-left at 45° onto the ledger: the right page (sum side) is in Parchment cream (#FCEFC8) highlight, the left page (input side) shades a row toward Aged paper (#BFB2A0); the single brightest pixel is the Brass leaf (#F0D27D) quill-nib — the closure being written now. Single still idle frame.
```

### 30 — The Guildmaster (impl_method — give a struct behavior with an `impl` block)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = A tall, dignified authority figure, centered and standing. A long Crypt (#3E1220) robe of office with Oxblood (#6B1F35) primary and Wineflesh (#982D52) lit folds — the most formal cut in the cast, full-length, deep, unbroken by tradesman's apron or tunic — with Inkblood (#1B0810) for the deepest shadow seams. On the chest, a guild medallion: an Old gold (#D2A53F) disc on an Antique brass (#9C7026) chain looping over the shoulders, with a single Brass leaf (#F0D27D) specular highlight on the disc's upper-left curve — the seal of the type, the one bright gold mass on the body. In his right hand, planted on the ground, a staff of office: an Antique brass (#9C7026) collar, a Bronze (#5E4116) shaft (Coalblack #161313 outline), and an Old gold (#D2A53F) finial cap with a Brass leaf (#F0D27D) specular pixel — the method made visible. Head: short Stone grey (#7A7064) hair and a trimmed Stone grey (#7A7064) beard with Aged paper (#BFB2A0) highlights (older, senior), Basalt (#3E3833) brow and eye line, skin in Pink quartz (#EBC2CC) / Dusty rose (#C56883). A standard Old gold (#D2A53F) belt buckle, Pine (#27502E) hem under-robe, Bronze (#5E4116) / Bog umber (#2D1F0A) boots. No magic violet, no alarm scarlet, no specular white, no cobalt, no teal, no forest beyond the pine hem. Mood: dignified, authoritative, senior. Class signature color: Old gold (#D2A53F). Lighting from top-left at 45°: the robe's left-facing folds catch Wineflesh (#982D52), the right side falls to Crypt (#3E1220); the two brightest accents are the Brass leaf (#F0D27D) specs on the medallion disc and the staff finial. Single still idle frame.
```

### 31 — The Recruiter (assoc_new — `Self::new(name)` returns a fresh instance)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = A standing officer, slightly turned, working. A Crypt (#3E1220) robe with Oxblood (#6B1F35) primary and Wineflesh (#982D52) lit folds, with Antique brass (#9C7026) collar studs at the throat — clerical-formal, plainer than the Guildmaster. In the left hand, held open across the body, the recruit-roll: a Parchment cream (#FCEFC8) page on a Bronze (#5E4116) spindle rod, with three short Coalblack (#161313) text lines (the existing entries) and a fresh Old gold (#D2A53F) wax seal pressed near the bottom (the new member, just forged), an Aged paper (#BFB2A0) shadow row giving the page its curl. In the right hand, raised, a Coalblack (#161313) quill with a Brass leaf (#F0D27D) specular pixel on the nib — caught mid-stamp, the single brightest point — the hand in Pink quartz (#EBC2CC) / Dusty rose (#C56883). Head: short Stone grey (#7A7064) hair with Aged paper (#BFB2A0) highlight, clean-shaven, Basalt (#3E3833) brow and eyes, skin in Pink quartz (#EBC2CC) / Dusty rose (#C56883). Standard Old gold (#D2A53F) belt buckle, Pine (#27502E) hem, Bronze (#5E4116) / Bog umber (#2D1F0A) boots. No magic violet, no alarm scarlet, no specular white, no cobalt, no teal, no forest beyond the pine hem. Mood: brisk, welcoming, official. Class signature color: Old gold (#D2A53F). Lighting from top-left at 45° onto the open roll: the roll's upper edge catches Parchment cream (#FCEFC8), the lower curl shades toward Aged paper (#BFB2A0); the single brightest pixel is the Brass leaf (#F0D27D) quill nib — the new member written now. Single still idle frame.
```

### 34 — The Surveyor (tuple_struct — `struct Meters(f64)`, a named scalar)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = A standing surveyor in a Crypt (#3E1220) robe with Oxblood (#6B1F35) primary and Wineflesh (#982D52) lit folds and Antique brass (#9C7026) collar studs — field-officer burgundy, neat. In the right hand, gripped at chest height and planted to the floor, a tall graduated measuring rod that runs nearly the full height of the frame: a Bronze (#5E4116) shaft (Coalblack #161313 outline) ticked at regular intervals with Old gold (#D2A53F) graduation bands, the topmost tick taking a single Brass leaf (#F0D27D) specular pixel, and an Antique brass (#9C7026) foot capping the rod at the ground. The hand wraps the shaft in Pink quartz (#EBC2CC) / Dusty rose (#C56883), with a Bronze (#5E4116) pixel where fingers cross the rod; the left hand rests on the hip. Head: short Stone grey (#7A7064) hair with Aged paper (#BFB2A0) highlight, clean-shaven, Basalt (#3E3833) brow and eyes, skin in Pink quartz (#EBC2CC) / Dusty rose (#C56883). Standard Old gold (#D2A53F) belt buckle, Pine (#27502E) hem, Bronze (#5E4116) / Bog umber (#2D1F0A) boots. No magic violet, no alarm scarlet, no specular white, no cobalt, no teal, no forest beyond the pine hem. Mood: precise, measured, level-headed. Class signature color: Old gold (#D2A53F). Lighting from top-left at 45° along the rod: the graduation ticks read brightest in Old gold (#D2A53F) against the darker Bronze (#5E4116) shaft — the named units standing out from the raw length — with the single Brass leaf (#F0D27D) specular on the topmost tick. Single still idle frame.
```

### 36 — Vexis the Archmage (trait_def — define a `trait` that grants a capability)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = The tallest, most imposing figure in the cast, centered and standing. A full Crypt (#3E1220) mantle of office with Oxblood (#6B1F35) primary and Wineflesh (#982D52) lit folds — the deepest, most formal robe yet, doubled at the shoulders with Inkblood (#1B0810) seam shadow so it reads heavier than the Guildmaster's. An Old gold (#D2A53F) collar yoke loops the shoulders with an Antique brass (#9C7026) chain base and a Brass leaf (#F0D27D) specular run. In the right hand, raised, a tall staff: a Bronze (#5E4116) shaft (Coalblack #161313 outline), an Antique brass (#9C7026) collar, an Old gold (#D2A53F) finial cup, and at the very top a small glowing orb — a Mage glow (#9D6FE0) core ringed by a Royal arcane (#3A1559) halo — the one arcane accent, at the top of the frame so the eye reads it as the source of power. Head: short Stone grey (#7A7064) hair and a trimmed Stone grey (#7A7064) beard with Aged paper (#BFB2A0) highlights, Basalt (#3E3833) brow and eye line, skin in Pink quartz (#EBC2CC) / Dusty rose (#C56883). An Old gold (#D2A53F) mantle belt at the waist, Pine (#27502E) hem under-mantle, Bronze (#5E4116) / Bog umber (#2D1F0A) boots. No alarm scarlet, no specular white, no cobalt, no teal, no forest beyond the pine hem. Mood: imposing, commanding, arcane. Class signature color: Mage glow (#9D6FE0). Lighting from top-left at 45°: the mantle's left-facing folds catch Wineflesh (#982D52), the right side falls to Crypt (#3E1220) and Inkblood (#1B0810); the brightest mass is the Mage glow (#9D6FE0) orb at the staff top, with the Brass leaf (#F0D27D) specular on the collar yoke as the secondary read. Single still idle frame.
```

### 37 — The Wandwright (generic_fn — a function generic over `T` with trait bounds)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = A lean craftsman in a working Crypt (#3E1220) apron-tunic with Oxblood (#6B1F35) primary and Wineflesh (#982D52) lit folds, offset toward frame-right so the bench reads in the corner. In the raised left hand, a wand-blank: a Stone grey (#7A7064) shaft with an Aged paper (#BFB2A0) highlight at the tip and a Coalblack (#161313) outline — deliberately plain, no element bound yet. At the grip, the gold fittings: an Antique brass (#9C7026) collar with an Old gold (#D2A53F) specular ferrule — the bounds the blank is machined to. The right hand is held open and empty. In the lower frame-left corner, a small work-bench: a Bronze (#5E4116) top over a Bog umber (#2D1F0A) apron, Coalblack (#161313) legs, with a single tool on it — an Antique brass (#9C7026) head with an Old gold (#D2A53F) glint and a Coalblack (#161313) haft. Head: short Stone grey (#7A7064) hair with Aged paper (#BFB2A0) highlight, Basalt (#3E3833) brow and eyes, skin in Pink quartz (#EBC2CC) / Dusty rose (#C56883). Antique brass (#9C7026) belt band, Pine (#27502E) hem, Bronze (#5E4116) / Bog umber (#2D1F0A) boots. An unbound wand carries no element — no magic violet, no alarm scarlet, no specular white, no cobalt, no teal, no forest beyond the pine hem. Mood: workmanlike, exacting, quietly clever. Class signature color: Old gold (#D2A53F). Lighting from top-left at 45°: the raised blank catches Aged paper (#BFB2A0) at the tip; the Old gold (#D2A53F) ferrule at the grip is the one bright fitting — the bounds, lit; the bench falls into Bog umber (#2D1F0A) shadow so it stays subordinate to the figure. Single still idle frame.
```

### 38 — The Conjurer (generic_struct — a struct generic over `T`, `Pair<T>`)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = A centered, standing mage in a Crypt (#3E1220) robe with Oxblood (#6B1F35) primary and Wineflesh (#982D52) lit folds, with Inkblood (#1B0810) seam shadow at the widening hem. The hands come forward at waist height, each cupping a small conjured orb. The two orbs are deliberately, pixel-for-pixel identical: a Mage glow (#9D6FE0) core with a thin Royal arcane (#3A1559) halo, one at frame-left and one at frame-right at the same height — the two fields of `Pair<T>`, the one type appearing twice. Head: short Stone grey (#7A7064) hair with Aged paper (#BFB2A0) highlight, Basalt (#3E3833) brow and eyes, skin in Pink quartz (#EBC2CC) / Dusty rose (#C56883), hands in Pink quartz (#EBC2CC) / Dusty rose (#C56883). An Antique brass (#9C7026) belt band with an Old gold (#D2A53F) buckle, Pine (#27502E) hem, Bronze (#5E4116) / Bog umber (#2D1F0A) boots. No alarm scarlet, no specular white, no cobalt, no teal, no forest beyond the pine hem. Mood: poised, symmetrical, arcane. Class signature color: Mage glow (#9D6FE0). Lighting from top-left at 45°: the robe's left folds catch Wineflesh (#982D52), the right falls to Crypt (#3E1220) / Inkblood (#1B0810); the two Mage glow (#9D6FE0) orbs are lit equally and identically — the point of the piece is that they match, because `Pair<T>` binds both fields to the one `T`. Single still idle frame.
```

### 39 — The Familiar Keeper (dyn_trait — `Box<dyn Trait>`, dynamic dispatch)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = A figure offset toward frame-left, the right arm raised to hold a small cage aloft in the upper-right of the frame. The cage is a Bronze (#5E4116) frame with Coalblack (#161313) bars and Antique brass (#9C7026) rings top and bottom. Inside, the familiar reads as many types in one container: a Main teal (#2A8482) wing, an Old gold (#D2A53F) body, a Forest (#487E40) tail, and a single Basalt (#3E3833) eye — the deliberate clash of teal + gold + forest in one tiny creature is the whole point, concrete types hidden behind one dyn container. The figure wears a Crypt (#3E1220) robe with Oxblood (#6B1F35) primary and Wineflesh (#982D52) lit folds; the raised hand (Pink quartz #EBC2CC / Dusty rose #C56883) grips the cage hook. Head: short Stone grey (#7A7064) hair with Aged paper (#BFB2A0) highlight, Basalt (#3E3833) brow and eyes, skin in Pink quartz (#EBC2CC) / Dusty rose (#C56883). An Antique brass (#9C7026) belt band with Old gold (#D2A53F), Pine (#27502E) hem, Bronze (#5E4116) / Bog umber (#2D1F0A) boots. This is dynamic dispatch, not spellcraft — no magic violet, no alarm scarlet, no specular white, no cobalt; teal and forest confined to the single caged creature. Mood: eccentric, curious, showman-like. Class signature color: Old gold (#D2A53F) (with the creature's teal+gold+forest clash as the lesson accent). Lighting from top-left at 45°: the robe's left folds catch Wineflesh (#982D52); the cage is the bright focal cluster in the upper right — teal, gold, and forest jammed together — so the eye reads "many types, one box" before it reads the keeper. Single still idle frame.
```

### 42 — The Barkeep (result_match — `match` on a `Result`, handle `Ok` and `Err`)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = A genial tradesman barkeep standing behind a Bronze (#5E4116) bar/counter that fills the lower third of the frame — a Bog umber (#2D1F0A) front face with Bronze (#5E4116) plank seams and a Bronze (#5E4116) base trim. He wears a Crypt (#3E1220) apron-tunic with Oxblood (#6B1F35) primary and Wineflesh (#982D52) lit folds — the house burgundy in a working apron cut — with an Antique brass (#9C7026) waistband and Inkblood (#1B0810) seaming the apron's deepest shadow at the sides. In the frame-left hand, a mug standing upright and full: a Bronze (#5E4116) body with Antique brass (#9C7026) bands, brimming with Old gold (#D2A53F) ale topped by a Brass leaf (#F0D27D) foam head — the `Ok(value)`, the good pour. In the frame-right hand, a mug tipped and spilling: rim-down, ale gone, Antique brass (#9C7026) bands and a Bronze (#5E4116) body now empty, with a single Alarm scarlet (#E63946) drip falling from it — the `Err`, the lone scarlet pixel and the only alarm on the sprite (well under 1%). Head: short Stone grey (#7A7064) hair with Aged paper (#BFB2A0) highlights, Basalt (#3E3833) brow and eye line, skin in Pink quartz (#EBC2CC) / Dusty rose (#C56883). Pine (#27502E) hem below the bar, Bronze (#5E4116) / Bog umber (#2D1F0A) boots. No magic violet, no specular white, no cobalt, no teal, no forest beyond the pine hem; the single scarlet drip is the only alarm pixel. Mood: genial, easy, attentive. Class signature color: Old gold (#D2A53F) (with the single Alarm scarlet #E63946 drip as the Err accent). Lighting from top-left at 45°: the full mug's Old gold (#D2A53F) ale and Brass leaf (#F0D27D) foam are the brightest mass on the frame-left; the spilled mug on the right reads darker and broken, ending in the lone scarlet drip — the two outcomes lit unequally on purpose. Single still idle frame.
```

### 43 — The Bouncer (custom_error — define your own error `enum` of named variants)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = The widest, heaviest figure in the tavern cast — broad-shouldered, thick-necked, arms crossed over the chest so the crossed forearms are the stance and there is no prop in the hands. A Crypt (#3E1220) robe with Oxblood (#6B1F35) primary and Wineflesh (#982D52) lit folds, the most massive cut in the cast (no apron), with Inkblood (#1B0810) seaming the deep shadow down both sides and across the heavy hem. Head: short, flat Stone grey (#7A7064) hair with Aged paper (#BFB2A0) highlight, a heavy Basalt (#3E3833) brow line and Basalt (#3E3833) eyes, a broad face and jaw in Pink quartz (#EBC2CC) / Dusty rose (#C56883); the crossed forearms cross the chest in Pink quartz (#EBC2CC) / Dusty rose (#C56883) skin tones over the burgundy. To his frame-right, a small Old gold (#D2A53F) tally board on an Antique brass (#9C7026) frame, with Coalblack (#161313) rule marks down its face — each mark one named variant of the error enum — and a single Brass leaf (#F0D27D) specular pixel at the board's top corner: the one bright gold mass, the enumerated list of named failures. An Antique brass (#9C7026) belt band with an Old gold (#D2A53F) buckle, Pine (#27502E) hem, Bronze (#5E4116) / Bog umber (#2D1F0A) boots on thick legs. No magic violet, no alarm scarlet, no specular white, no cobalt, no teal, no forest beyond the pine hem. Mood: immovable, watchful, deadpan. Class signature color: Old gold (#D2A53F). Lighting from top-left at 45°: the crossed forearms catch the light across the chest with the Wineflesh (#982D52) lit fold running under the arms; the brightest accent is the Brass leaf (#F0D27D) spec on the tally board off to the side where the roster of named troubles hangs. Single still idle frame.
```

### 44 — The Interpreter (from_error — implement `From` to convert one error into another)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = A poised figure, both hands raised, holding a small sign/token in each. A Crypt (#3E1220) robe with Oxblood (#6B1F35) primary and Wineflesh (#982D52) lit folds — the house burgundy, a clerk-translator's cut — with Inkblood (#1B0810) seaming the deep shadow down the sides and the widening hem. In the frame-left raised hand, the source token: a small Forest (#487E40) card with a Spring meadow (#82B450) highlight, Coalblack (#161313)-edged — the incoming complaint in its original type. In the frame-right raised hand, the target token: a small Main teal (#2A8482) card with a Bright teal (#5BB8AF) highlight, Coalblack (#161313)-edged — the same complaint translated into the destination type. The two token colors (forest → teal) are the only accents on the figure, and the gesture of the two raised hands reads as the conversion between them. Head: short Stone grey (#7A7064) hair with Aged paper (#BFB2A0) highlights, Basalt (#3E3833) brow and eye line, skin in Pink quartz (#EBC2CC) / Dusty rose (#C56883). An Antique brass (#9C7026) belt band with an Old gold (#D2A53F) buckle, Pine (#27502E) hem, Bronze (#5E4116) / Bog umber (#2D1F0A) boots. Translation is craft, not spellcraft — no magic violet, no alarm scarlet, no specular white, no cobalt; forest confined to the source token, teal to the target token. Mood: poised, fluent, even. Class signature color: Main teal (#2A8482) (the target type). Lighting from top-left at 45°: both tokens are raised to the same height and lit equally — the forest source on the left, the teal target on the right — because `From` is a faithful conversion; the robe's Wineflesh (#982D52) fold catches the light at the chest between the two raised arms. Single still idle frame.
```

### 45 — The Mixologist (option_map — `.map()` a closure over the inside of an `Option`)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = A deft figure mid-pour, a tool in each hand. A Crypt (#3E1220) tunic with Oxblood (#6B1F35) primary and Wineflesh (#982D52) lit folds — the house burgundy, a working bartender's cut — with Inkblood (#1B0810) seaming the deep shadow down the sides and the widening hem. In the frame-left hand, a Bronze (#5E4116) cocktail shaker: a Bronze (#5E4116) body with Antique brass (#9C7026) bands, capped by an Old gold (#D2A53F) top with a Brass leaf (#F0D27D) specular pixel — the transform she is about to apply. In the frame-right hand, a glass holding a Main teal (#2A8482) drink with a Bright teal (#5BB8AF) highlight, Coalblack (#161313)-outlined; a short Mist teal (#A4DED4) pour stream connects the shaker's reach to the glass — the transform landing on the contents. The teal drink is the single accent that marks the `Some` value; an empty glass would simply show no teal at all (the `None` case the map leaves alone). Head: short Stone grey (#7A7064) hair with Aged paper (#BFB2A0) highlights, Basalt (#3E3833) brow and eye line, skin in Pink quartz (#EBC2CC) / Dusty rose (#C56883). An Antique brass (#9C7026) belt band with an Old gold (#D2A53F) buckle, Pine (#27502E) hem, Bronze (#5E4116) / Bog umber (#2D1F0A) boots. No magic violet, no alarm scarlet, no specular white, no cobalt, no forest beyond the pine hem; teal confined to the drink and the pour stream. Mood: deft, quick, breezy. Class signature color: Main teal (#2A8482). Lighting from top-left at 45°: the Brass leaf (#F0D27D) spec on the shaker cap is the brightest point top-left; the teal drink glows with its Bright teal (#5BB8AF) highlight on the right, the pour stream reading as motion between them — the transform applied to contents that are there. Single still idle frame.
```

### 46 — The Tabkeeper (and_then — chain fallible steps with `.and_then`, short-circuit)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = A clerk-figure holding a long tab. A Crypt (#3E1220) robe with Oxblood (#6B1F35) primary and Wineflesh (#982D52) lit folds — the house burgundy, a tab-keeper's cut, drawn slightly off-center so the tab hangs down the frame-right side — with Inkblood (#1B0810) seaming the deep shadow down the sides and the widening hem. From an Antique brass (#9C7026) / Old gold (#D2A53F) clip at the top-right (with a single Brass leaf #F0D27D specular pixel), a long Parchment cream (#FCEFC8) tab strip hangs down the frame-right edge. Down its length, a chain of order-entries: small Coalblack (#161313) rule marks, one entry under the next — each link in the `.and_then` chain. Near the bottom the strip ends in a solid Coalblack (#161313) bar: the chain stops there, the first failure that short-circuits the rest. His frame-right hand reaches up to grip the clip; the Pink quartz (#EBC2CC) / Dusty rose (#C56883) hands brace the tab. Head: short Stone grey (#7A7064) hair with Aged paper (#BFB2A0) highlights, Basalt (#3E3833) brow and eye line, skin in Pink quartz (#EBC2CC) / Dusty rose (#C56883). An Antique brass (#9C7026) belt band with an Old gold (#D2A53F) buckle, Pine (#27502E) hem, Bronze (#5E4116) / Bog umber (#2D1F0A) boots. No magic violet, no alarm scarlet, no specular white, no cobalt, no teal, no forest beyond the pine hem; parchment cream confined to the tab strip. Mood: harried, orderly, tracking. Class signature color: Old gold (#D2A53F). Lighting from top-left at 45°: the Brass leaf (#F0D27D) spec on the tab clip is the brightest point top-right; the cream tab strip runs bright down the right edge with the Coalblack (#161313) rule marks reading as the chain's links, and the solid Coalblack (#161313) bar at the strip's foot is where the eye stops — the short-circuit. Single still idle frame.
```

### 47 — The Cellarer (unwrap_or_else — lazy default via `.unwrap_or_else(|| ...)`)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = A figure standing beside a cask, ladle in hand. A Crypt (#3E1220) cellar tunic with Oxblood (#6B1F35) primary and Wineflesh (#982D52) lit folds — the house burgundy, drawn slightly off-center so the cask sits to his frame-left — with Inkblood (#1B0810) seaming the deep shadow down the sides and the widening hem. To the frame-left, a wooden cask/barrel: Bronze (#5E4116) staves bound by Antique brass (#9C7026) hoops top and bottom, with a band of Old gold (#D2A53F) brew level visible near the top (a single Brass leaf #F0D27D specular pixel on it) and a Bog umber (#2D1F0A) shadow at its base — the source you draw the default from. In the frame-right hand, a Bronze (#5E4116) ladle: a Bronze (#5E4116) bowl on a Bronze (#5E4116) handle, Coalblack (#161313)-outlined, raised as if mid-scoop — the tool that either draws from the cask or serves the fresh-brewed default — gripped by a Pink quartz (#EBC2CC) / Dusty rose (#C56883) hand. Head: short Stone grey (#7A7064) hair with Aged paper (#BFB2A0) highlights, Basalt (#3E3833) brow and eye line, skin in Pink quartz (#EBC2CC) / Dusty rose (#C56883). An Antique brass (#9C7026) belt band with an Old gold (#D2A53F) buckle, Pine (#27502E) hem, Bronze (#5E4116) / Bog umber (#2D1F0A) boots. No magic violet, no alarm scarlet, no specular white, no cobalt, no teal, no forest beyond the pine hem; the Old gold (#D2A53F) brew level is the one bright accent. Mood: unhurried, capable, easy. Class signature color: Old gold (#D2A53F). Lighting from top-left at 45°: the Brass leaf (#F0D27D) spec on the cask's brew level is the brightest point on the frame-left; the Bronze (#5E4116) ladle catches the light on the right, and the cask's Bog umber (#2D1F0A) base falls to shadow — the empty bottom that, when reached, sends the Cellarer to brew the lazy default rather than to error. Single still idle frame.
```

### 48 — The Keymaster (hashmap_basic — insert and get values by key in a `HashMap`)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = A figure standing before a rack of slots, key ring in his raised frame-right hand. A Crypt (#3E1220) keeper's tunic with Oxblood (#6B1F35) primary and Wineflesh (#982D52) lit folds — the house burgundy, drawn slightly off-center so the rack sits to his frame-left — with Inkblood (#1B0810) seaming the deep shadow down the sides and the widening hem. To the frame-left/back, a rack of pigeonhole slots: Antique brass (#9C7026) frame bands stacked in rows, each slot a Bronze (#5E4116) interior dropping to a Bog umber (#2D1F0A) shadow at its mouth — the named cubbies, the keyed storage the map represents. In the frame-right hand, a ring of keys: an Old gold (#D2A53F) ring with hanging key bows, Coalblack (#161313)-outlined teeth, and a single Brass leaf (#F0D27D) specular pixel on the bow that catches the light — the key the player holds, the one bright gold mass on the figure's side — gripped by a Pink quartz (#EBC2CC) / Dusty rose (#C56883) hand. Head: short Stone grey (#7A7064) hair with Aged paper (#BFB2A0) highlights, Basalt (#3E3833) brow and eye line, skin in Pink quartz (#EBC2CC) / Dusty rose (#C56883). An Antique brass (#9C7026) belt band, Pine (#27502E) hem, Bronze (#5E4116) / Bog umber (#2D1F0A) boots. No magic violet, no alarm scarlet, no specular white, no cobalt, no teal, no forest beyond the pine hem. Mood: orderly, efficient, sure. Class signature color: Old gold (#D2A53F). Lighting from top-left at 45°: the Brass leaf (#F0D27D) spec on the key bow is the brightest point on the frame-right; the Antique brass (#9C7026) rack frames catch the light on the left while the slot mouths fall to Bog umber (#2D1F0A) shadow — the cubbies waiting for their keyed value. Single still idle frame.
```

### 49 — The Sifter (iter_filter — keep only items that pass a predicate via `.filter`)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = A figure standing, both hands raising a round sieve at chest height. A Crypt (#3E1220) miller's tunic with Oxblood (#6B1F35) primary and Wineflesh (#982D52) lit folds — the house burgundy, centered — with Inkblood (#1B0810) seaming the deep shadow. In both Pink quartz (#EBC2CC) / Dusty rose (#C56883) hands, a round sieve: a Bronze (#5E4116) hoop, an Antique brass (#9C7026) mesh frame top and bottom, and a row of mesh wires across the middle tinted with a touch of Forest (#487E40) (the screen the grain is tested against) — the sieve is the predicate. Below the sieve, a scatter of Old gold (#D2A53F) grains falling through the mesh — the kept items — with a single Brass leaf (#F0D27D) specular grain catching the light. These are the elements that passed the test. Head: short Stone grey (#7A7064) hair with Aged paper (#BFB2A0) highlights, Basalt (#3E3833) brow and eye line, skin in Pink quartz (#EBC2CC) / Dusty rose (#C56883). An Inkblood (#1B0810) belt seam under the sieve, Pine (#27502E) hem, Bronze (#5E4116) / Bog umber (#2D1F0A) boots. No magic violet, no alarm scarlet, no specular white, no cobalt, no teal; forest appears only as the mesh tint. Mood: steady, discerning, rhythmic. Class signature color: Old gold (#D2A53F) (the kept grain). Lighting from top-left at 45°: the Brass leaf (#F0D27D) spec sits on one falling grain; the Bronze (#5E4116) sieve hoop catches the light along its upper-left arc, and the grain below the mesh is what passed the predicate — the Forest (#487E40)-tinted wires the test it had to clear. Single still idle frame.
```

### 50 — The Smelter (iter_fold — reduce many items to one value via `.fold`)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = A figure standing at a furnace, frame-right hand tipping a crucible. A Crypt (#3E1220) forge tunic with Oxblood (#6B1F35) primary and Wineflesh (#982D52) lit folds — the house burgundy, drawn off-center so the crucible and mold sit to his frame-right — with Inkblood (#1B0810) seaming the deep shadow and the widening hem. To the frame-right, a Bronze (#5E4116) crucible bound with an Antique brass (#9C7026) band, brimming with molten metal in Old gold (#D2A53F) with a Brass leaf (#F0D27D) lit core. From its lip a stream of molten Old gold (#D2A53F) pours down into a single ingot mold below — a Bronze (#5E4116) mold with an Antique brass (#9C7026) base band and a Bog umber (#2D1F0A) shadow, filling with Old gold (#D2A53F) that brightens to Brass leaf (#F0D27D) as the ingot gathers. Many melts, one ingot — the fold's single result. A single Main teal (#2A8482) pixel of heat-shimmer rises above the crucible — the only cool accent, reading as forge heat, not magic. Head: short Stone grey (#7A7064) hair with Aged paper (#BFB2A0) highlights, Basalt (#3E3833) brow and eye line, skin in Pink quartz (#EBC2CC) / Dusty rose (#C56883). An Antique brass (#9C7026) belt band, Pine (#27502E) hem, Bronze (#5E4116) / Bog umber (#2D1F0A) boots; the Pink quartz (#EBC2CC) / Dusty rose (#C56883) hand grips the crucible. No magic violet, no alarm scarlet, no specular white, no cobalt, no forest; the lone Main teal (#2A8482) pixel is heat-shimmer, capped at one. Mood: laborious, patient, decisive. Class signature color: Old gold (#D2A53F). Lighting from top-left at 45°: the brightest points are the Brass leaf (#F0D27D) core of the molten metal in the crucible and the gathering ingot; the molten Old gold (#D2A53F) glow lights the whole frame-right, and the mold's Bog umber (#2D1F0A) base falls to shadow beneath the finished ingot — the one value left when every item has been folded in. Single still idle frame.
```

### 51 — The Tallywright (iter_enumerate — pair each item with its index via `.enumerate`)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = A figure standing, frame-right hand wielding a counter-stamp raised mid-stroke. A Crypt (#3E1220) clerk tunic with Oxblood (#6B1F35) primary and Wineflesh (#982D52) lit folds — the house burgundy, drawn slightly off-center so the rail of stamped items runs along the frame's lower edge — with Inkblood (#1B0810) seaming the deep shadow. In the raised frame-right hand, a counter-stamp: an Old gold (#D2A53F) stamp head with a Brass leaf (#F0D27D) specular pixel, on a Bronze (#5E4116) shaft, Coalblack (#161313)-outlined — the numbering tool — gripped by a Pink quartz (#EBC2CC) / Dusty rose (#C56883) hand. Along the bottom of the frame, a low Bronze (#5E4116) rail with Antique brass (#9C7026) caps carries a row of Old gold (#D2A53F) tags, each printed with a small Coalblack (#161313)-digit mark (0, 1, 2) — the items, each now stamped with its index — with a Bog umber (#2D1F0A) shadow under the rail. Head: short Stone grey (#7A7064) hair with Aged paper (#BFB2A0) highlights, Basalt (#3E3833) brow and eye line, skin in Pink quartz (#EBC2CC) / Dusty rose (#C56883). An Antique brass (#9C7026) belt band, Pine (#27502E) hem, Bronze (#5E4116) / Bog umber (#2D1F0A) boots. No magic violet, no alarm scarlet, no specular white, no cobalt, no teal, no forest beyond the pine hem. Mood: precise, brisk, methodical. Class signature color: Old gold (#D2A53F). Lighting from top-left at 45°: the Brass leaf (#F0D27D) spec on the stamp head is the brightest point, top-center where the tool is raised; the Old gold (#D2A53F) tags catch the light along the rail below, their Coalblack (#161313) digits reading as the stamped indices. Single still idle frame.
```

### 52 — The Riveter (iter_zip — pair two iterators elementwise via `.zip`)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = A figure standing, frame-right hand raising a rivet hammer mid-stroke. A Crypt (#3E1220) fitter tunic with Oxblood (#6B1F35) primary and Wineflesh (#982D52) lit folds — the house burgundy, drawn slightly off-center so the two riveted strips run along the frame's lower edge — with Inkblood (#1B0810) seaming the deep shadow. In the raised frame-right hand, a rivet hammer: an Old gold (#D2A53F)-faced head with a Brass leaf (#F0D27D) specular pixel, on a Bronze (#5E4116) haft, Coalblack (#161313)-outlined — the tool driving each pair together — gripped by a Pink quartz (#EBC2CC) / Dusty rose (#C56883) hand. Along the bottom of the frame, two Bronze (#5E4116) metal strips run side by side, one over the other, edged in Antique brass (#9C7026), pinned along their seam by a line of Old gold (#D2A53F) rivets — the zipped pairs, one rivet binding each matched hole; the strips meet and stop together. Head: short Stone grey (#7A7064) hair with Aged paper (#BFB2A0) highlights, Basalt (#3E3833) brow and eye line, skin in Pink quartz (#EBC2CC) / Dusty rose (#C56883). An Antique brass (#9C7026) belt band, Pine (#27502E) hem, Bronze (#5E4116) / Bog umber (#2D1F0A) boots. No magic violet, no alarm scarlet, no specular white, no cobalt, no teal, no forest beyond the pine hem. Mood: rhythmic, sturdy, focused. Class signature color: Old gold (#D2A53F). Lighting from top-left at 45°: the Brass leaf (#F0D27D) spec on the hammer face is the brightest point, top-center where the tool is raised; the Old gold (#D2A53F) rivets catch the light in a row along the seam below — the pairs the zip has bound, two iterators walked in step. Single still idle frame.
```

### 53 — The Bondsmith (closure_move — take ownership of captured values via `move`)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = A figure standing, both hands cupped close against the chest around a vessel. A Crypt (#3E1220) keeper tunic with Oxblood (#6B1F35) primary and Wineflesh (#982D52) lit folds — the house burgundy, centered — with Inkblood (#1B0810) seaming the deep shadow and the widening hem. At chest center, a Bronze (#5E4116) locket-vessel with an Antique brass (#9C7026) band, held close. Inside it, a sealed glow in Old gold (#D2A53F) brightening to a Brass leaf (#F0D27D) lit core — the captured value, now owned. Both Pink quartz (#EBC2CC) / Dusty rose (#C56883) hands cup over the vessel, closing the seal — ownership moved in, not lent out — and below the glow the vessel band closes over with Antique brass (#9C7026), the item sealed inside. Head: short Stone grey (#7A7064) hair with Aged paper (#BFB2A0) highlights, Basalt (#3E3833) brow and eye line, skin in Pink quartz (#EBC2CC) / Dusty rose (#C56883). An Antique brass (#9C7026) belt band below the vessel, Pine (#27502E) hem, Bronze (#5E4116) / Bog umber (#2D1F0A) boots. The glow is deliberately warm Old gold (#D2A53F), not arcane violet — this is ownership taken in, not a spell. No magic violet, no alarm scarlet, no specular white, no cobalt, no teal, no forest beyond the pine hem. Mood: guarded, possessive, quiet. Class signature color: Old gold (#D2A53F). Lighting from top-left at 45°: the Brass leaf (#F0D27D) core of the sealed glow is the brightest point, at chest center inside the cupped hands; the Bronze (#5E4116) vessel and Antique brass (#9C7026) band catch the light around it — the value moved into the closure rather than borrowed from the scope outside. Single still idle frame.
```

### 54 — The Dockmaster (thread_spawn — launch a concurrent worker via `thread::spawn`)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = A coastal dispatcher, centered and standing, mid-gesture. The house Crypt (#3E1220) / Oxblood (#6B1F35) coat with Wineflesh (#982D52) lit folds — a working harbor coat rather than the inland guild's formal robe — with Inkblood (#1B0810) for the deepest seams. Across the shoulders, a Main teal (#2A8482) harbor sash with Deep teal (#154548) shadow and Bright teal (#5BB8AF) highlight — the coastal house mark, reading as a sash band, not a robe. In his right hand, raised to his lips, a dispatcher's whistle: a Bronze (#5E4116) body with an Antique brass (#9C7026) band and an Old gold (#D2A53F) bell, a single Brass leaf (#F0D27D) specular pixel on the bell. His left arm is flung out to frame-left, the Pink quartz (#EBC2CC) / Dusty rose (#C56883) hand pointing down the pier — the spawn gesture, a worker launched. At his right hip a clipboard manifest: Parchment cream (#FCEFC8) pages with Aged paper (#BFB2A0) ruled lines under an Old gold (#D2A53F) clip. Head: short Stone grey (#7A7064) hair with Aged paper (#BFB2A0) highlights, Basalt (#3E3833) brow/eye line, skin in Pink quartz (#EBC2CC) / Dusty rose (#C56883). An Antique brass (#9C7026) belt with an Old gold (#D2A53F) buckle, Pine (#27502E) hem, Bronze (#5E4116) / Bog umber (#2D1F0A) boots. No magic violet, no alarm scarlet, no specular white beyond the one whistle-bell pixel, no cobalt, no forest beyond the pine hem. Mood: brisk, commanding, salt-weathered. Class signature color: Main teal (#2A8482). Lighting from top-left at 45°: the coat's left-facing folds catch Wineflesh (#982D52), the right side falls to Crypt (#3E1220); the two brightest accents are the Brass leaf (#F0D27D) spec on the whistle bell and the Bright teal (#5BB8AF) highlight on the sash. Single still idle frame.
```

### 55 — The Lighthouse Keeper (arc_mutex — share state across threads with `Arc<Mutex<T>>`)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = A keeper standing at frame-left beside a small lighthouse that fills the right column of the frame. The house Crypt (#3E1220) / Oxblood (#6B1F35) coat with Wineflesh (#982D52) lit folds and Inkblood (#1B0810) for the deepest seams. In his raised hand, the single key to the lantern room: an Old gold (#D2A53F) bow and shaft with Coalblack (#161313) teeth and a Brass leaf (#F0D27D) specular pixel — one key, held up, the lock made visible, the one bright mass on the figure's body. To frame-right, the lighthouse: a Bronze (#5E4116) tower body banded with Aged paper (#BFB2A0) courses and Antique brass (#9C7026) seams, outlined in Coalblack (#161313), flaring to a wider Bronze (#5E4116) base at the waterline. At the top, the lantern room — an Antique brass (#9C7026) frame around an Old gold (#D2A53F) lamp core (Main teal #2A8482 glow inside) — casts a single Bright teal (#5BB8AF) beam fanning across the upper-right, fading through Main teal (#2A8482) to Deep teal (#154548) at its tail — the shared light every ship depends on. Head: short Stone grey (#7A7064) hair with Aged paper (#BFB2A0) highlights, Basalt (#3E3833) brow/eye line, skin in Pink quartz (#EBC2CC) / Dusty rose (#C56883). An Antique brass (#9C7026) belt with an Old gold (#D2A53F) buckle, Pine (#27502E) hem, Bronze (#5E4116) / Bog umber (#2D1F0A) boots. No magic violet, no alarm scarlet, no specular white beyond the one key pixel, no cobalt, no forest beyond the pine hem. Mood: solitary, vigilant, steady. Class signature color: Main teal (#2A8482). Lighting from top-left at 45° on the figure, with the lamp its own emissive source at the tower top: the two brightest accents are the Brass leaf (#F0D27D) spec on the held key and the Bright teal (#5BB8AF) lamp beam. Single still idle frame.
```

### 56 — The Signaler (mpsc_channel — pass values between threads over a channel)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = A signalman, centered and standing, both arms out — one raised to frame-left, one lowered to frame-right — gripping two flag poles. The house Crypt (#3E1220) / Oxblood (#6B1F35) coat with Wineflesh (#982D52) lit folds and Inkblood (#1B0810) for the deepest seams. Two semaphore flags, one per hand: Coalblack (#161313) poles topped with an Antique brass (#9C7026) finial (a Brass leaf #F0D27D specular pixel at the very tip), each flag a triangular field of Main teal (#2A8482) with a Deep teal (#154548) shadow edge and Bright teal (#5BB8AF) lit edge — sea-signal teal, the channel ends. The raised flag (frame-left) reads as the message being sent down the line. The hands gripping the poles are Pink quartz (#EBC2CC) / Dusty rose (#C56883). Head: short Stone grey (#7A7064) hair with Aged paper (#BFB2A0) highlights, Basalt (#3E3833) brow/eye line, skin in Pink quartz (#EBC2CC) / Dusty rose (#C56883). An Antique brass (#9C7026) belt with an Old gold (#D2A53F) buckle, Pine (#27502E) hem, Bronze (#5E4116) / Bog umber (#2D1F0A) boots. No magic violet, no alarm scarlet, no specular white beyond the two finial pixels, no cobalt, no forest beyond the pine hem. Mood: alert, signalling, taut. Class signature color: Main teal (#2A8482). Lighting from top-left at 45°: the coat's left-facing folds catch Wineflesh (#982D52), the right side falls to Crypt (#3E1220); the brightest accents are the Bright teal (#5BB8AF) lit edges of the two flags — one channel, two ends, the message streaming frame-left toward the receiver down the line. Single still idle frame.
```

### 57 — The Tidewatch (atomic — increment a shared counter with `AtomicUsize`)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = A tidewatcher standing at frame-left, one arm reaching to a gauge post at frame-right. The house Crypt (#3E1220) / Oxblood (#6B1F35) coat with Wineflesh (#982D52) lit folds and Inkblood (#1B0810) for the deepest seams. The tide-gauge post fills the right column: a Bronze (#5E4116) timber outlined in Coalblack (#161313), capped with an Antique brass (#9C7026) head, cut with regular Aged paper (#BFB2A0) notch ticks up its length. Partway up the post stands the current count — a single Old gold (#D2A53F) tick mark with a Brass leaf (#F0D27D) specular pixel, the atomic counter's value, climbing — and her Pink quartz (#EBC2CC) / Dusty rose (#C56883) hand reaches across to read it. At the post's foot, the tide: a band of Main teal (#2A8482) water with Deep teal (#154548) shadow and Bright teal (#5BB8AF) sparkle — the rising water whose every push fetch-adds the mark higher, the only teal mass, kept low at the waterline. Head: short Stone grey (#7A7064) hair with Aged paper (#BFB2A0) highlights, Basalt (#3E3833) brow/eye line, skin in Pink quartz (#EBC2CC) / Dusty rose (#C56883). An Antique brass (#9C7026) belt with an Old gold (#D2A53F) buckle, Pine (#27502E) hem, Bronze (#5E4116) / Bog umber (#2D1F0A) boots. No magic violet, no alarm scarlet, no specular white beyond the one tick pixel, no cobalt, no forest beyond the pine hem. Mood: watchful, exact, calm. Class signature color: Old gold (#D2A53F) (with the Main teal #2A8482 tide as the coastal accent). Lighting from top-left at 45° on figure and post: the brightest accents are the Brass leaf (#F0D27D) spec on the climbing Old gold (#D2A53F) tick and the Bright teal (#5BB8AF) sparkle on the tide — the one shared count, the rising water that drives it. Single still idle frame.
```

### 58 — The Harbormaster (thread_scope — spawn scoped threads that all join before scope ends)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = A tall, senior authority figure, centered and standing — the most formal of the coastal cast. The house Crypt (#3E1220) / Oxblood (#6B1F35) coat of office with Wineflesh (#982D52) lit folds and Inkblood (#1B0810) for the deepest seams. Across the chest, a Main teal (#2A8482) harbor sash with Deep teal (#154548) shadow and Bright teal (#5BB8AF) highlight — the mark of harbor authority. At frame-right, hung at his side, a great harbor bell: an Old gold (#D2A53F) body with an Antique brass (#9C7026) rim and mouth, a Bronze (#5E4116) yoke above, a Coalblack (#161313) clapper, and a single Brass leaf (#F0D27D) specular highlight on the bell's upper curve — the scope's bound, rung to open, rung to close, never closed early. In his left hand, angled across the body, a Bronze (#5E4116) spyglass (Coalblack #161313 outline, Antique brass #9C7026 trim) — counting the boats home. Head: short Stone grey (#7A7064) hair and a trimmed Stone grey (#7A7064) beard with Aged paper (#BFB2A0) highlights (older, senior), Basalt (#3E3833) brow/eye line, skin in Pink quartz (#EBC2CC) / Dusty rose (#C56883). An Antique brass (#9C7026) belt with an Old gold (#D2A53F) buckle, Pine (#27502E) hem, Bronze (#5E4116) / Bog umber (#2D1F0A) boots. No magic violet, no alarm scarlet, no specular white beyond the one bell pixel, no cobalt, no forest beyond the pine hem. Mood: authoritative, senior, sea-hardened. Class signature color: Main teal (#2A8482). Lighting from top-left at 45°: the coat's left-facing folds catch Wineflesh (#982D52), the right side falls to Crypt (#3E1220); the two brightest accents are the Brass leaf (#F0D27D) spec on the harbor bell and the Bright teal (#5BB8AF) highlight on the sash. Single still idle frame.
```

### 59 — The Tideforecaster (async_fn — write an `async fn` returning a future you `await`)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = A forecaster standing at the shoreline, centered, spyglass raised to one eye. The house Crypt (#3E1220) / Oxblood (#6B1F35) coat with Wineflesh (#982D52) lit folds and Inkblood (#1B0810) for the deepest seams. In her raised hand, a spyglass to the eye: a Bronze (#5E4116) body (Coalblack #161313 outline, Antique brass #9C7026 trim) with an Old gold (#D2A53F) end cap and a Brass leaf (#F0D27D) specular pixel at the lens — the `await`, poised on the promised tide. Under her other arm, a tide-almanac: Parchment cream (#FCEFC8) pages with Aged paper (#BFB2A0) ruled lines and Coalblack (#161313) binding — the `fn` signature that declares the future will come. A thin Cobalt deep (#0E2E54) band runs across the very top — the night sky, a small cool accent. Across the bottom rows, the sea: Main teal (#2A8482) water with Deep teal (#154548) shadow and a Bright teal (#5BB8AF) horizon line and sparkle — the future not yet arrived, the tide she awaits. Head: short Stone grey (#7A7064) hair with Aged paper (#BFB2A0) highlights, Basalt (#3E3833) brow/eye line, skin in Pink quartz (#EBC2CC) / Dusty rose (#C56883). An Antique brass (#9C7026) belt with an Old gold (#D2A53F) buckle, Pine (#27502E) hem, Bronze (#5E4116) / Bog umber (#2D1F0A) boots. No magic violet, no alarm scarlet, no specular white beyond the one lens pixel; cobalt limited to the single night-sky band at top. Mood: patient, far-seeing, expectant. Class signature color: Main teal (#2A8482). Lighting from top-left at 45° on the figure: the two brightest accents are the Brass leaf (#F0D27D) spec on the raised spyglass lens and the Bright teal (#5BB8AF) horizon line of the sea — the await, and the tide that has not yet come in. Single still idle frame.
```

### 60 — The Vaultwright (box_basic — put a value on the heap with `Box::new`)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = A figure standing, centered, both arms coming round to cradle a strongbox held against the body. A Crypt (#3E1220) keeper tunic with Oxblood (#6B1F35) primary and Wineflesh (#982D52) lit folds — the house burgundy — with Inkblood (#1B0810) seaming the deep shadow and the widening hem. The strongbox dominates the lower chest: a Bronze (#5E4116) body with Antique brass (#9C7026) bands top and bottom, an Old gold (#D2A53F) latched lid, and a central lock plate in Old gold (#D2A53F) carrying a single Brass leaf (#F0D27D) specular highlight. The keyhole and chest seam are picked out in Coalblack (#161313); the box base sits in Bog umber (#2D1F0A). Both Pink quartz (#EBC2CC) / Dusty rose (#C56883) arms wrap around the box from the sides, cradling it. Head: short Stone grey (#7A7064) hair with Aged paper (#BFB2A0) highlights, Basalt (#3E3833) brow and eye line, skin in Pink quartz (#EBC2CC) / Dusty rose (#C56883). Pine (#27502E) hem, Bronze (#5E4116) / Bog umber (#2D1F0A) boots. No magic violet, no alarm scarlet, no specular white beyond the single lock spec, no cobalt, no teal, no forest beyond the pine hem. Mood: careful, custodial, steady. Class signature color: Old gold (#D2A53F). Lighting from top-left at 45°: the brightest accent is the Brass leaf (#F0D27D) specular on the lock plate at chest center; the Old gold (#D2A53F) lid catches the light above the Bronze (#5E4116) body — the whole gold mass of the strongbox is the heap allocation, one bright solid object cradled close. Single still idle frame.
```

### 61 — The Sharekeeper (rc_basic — share ownership of a value with `Rc`)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = A figure standing, centered, holding a single shared hoard-chest at chest height. A Crypt (#3E1220) keeper tunic with Oxblood (#6B1F35) primary and Wineflesh (#982D52) lit folds — the house burgundy — with Inkblood (#1B0810) seaming the deep shadow and hem. At chest center, one Bronze (#5E4116)-bodied hoard with Antique brass (#9C7026) bands, brimming with Old gold (#D2A53F) coins, two of which carry Brass leaf (#F0D27D) specular highlights. Reaching in from both the left and right edges of the frame are several Pink quartz (#EBC2CC) / Dusty rose (#C56883) hands — multiple owners, each claiming the same hoard. To the upper-right, a small Old gold (#D2A53F) count marker (a two-tile token with a Coalblack #161313 digit) records the reference count. Head: short Stone grey (#7A7064) hair with Aged paper (#BFB2A0) highlights, Basalt (#3E3833) brow and eye line, skin in Pink quartz (#EBC2CC) / Dusty rose (#C56883). An Antique brass (#9C7026) belt, Pine (#27502E) hem, Bronze (#5E4116) / Bog umber (#2D1F0A) boots. No magic violet, no alarm scarlet, no specular white beyond the coin specs, no cobalt, no teal, no forest beyond the pine hem. Mood: open-handed, communal, watchful. Class signature color: Old gold (#D2A53F). Lighting from top-left at 45°: the brightest accents are the Brass leaf (#F0D27D) specs on the hoard coins; the single gold hoard with many hands on it is the shared value, and the small Old gold (#D2A53F) count marker is the reference count — when it reaches zero the hoard is freed. Single still idle frame.
```

### 62 — The Warden (refcell — mutate through a shared handle with `RefCell`)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = A figure standing, one arm latched around a lockbox, the other reaching across to turn a key in its lock. A Crypt (#3E1220) keeper tunic with Oxblood (#6B1F35) primary and Wineflesh (#982D52) lit folds — the house burgundy — with Inkblood (#1B0810) seaming the deep shadow and hem. The lockbox sits at chest/waist: a Bronze (#5E4116) body with Antique brass (#9C7026) bands and an Old gold (#D2A53F) lock plate, the lid cracked open along an Antique brass (#9C7026) seam. An Old gold (#D2A53F) key, with a Brass leaf (#F0D27D) specular highlight, is turned in the lock by a Pink quartz (#EBC2CC) / Dusty rose (#C56883) hand. Keyhole and seams in Coalblack (#161313); box base in Bog umber (#2D1F0A). At the figure's hip, a small Parchment cream (#FCEFC8) borrow-ledger with Coalblack (#161313) tick marks and an Aged paper (#BFB2A0) ruled line — the record of the current borrower. Head: short Stone grey (#7A7064) hair with Aged paper (#BFB2A0) highlights, Basalt (#3E3833) brow and eye line, skin in Pink quartz (#EBC2CC) / Dusty rose (#C56883). Pine (#27502E) hem, Bronze (#5E4116) / Bog umber (#2D1F0A) boots. No magic violet, no alarm scarlet, no specular white beyond the key spec, no cobalt, no teal, no forest beyond the pine hem. Mood: vigilant, careful, by-the-book. Class signature color: Old gold (#D2A53F). Lighting from top-left at 45°: the brightest accent is the Brass leaf (#F0D27D) spec on the turning key; the Old gold (#D2A53F) lock plate and key are the runtime borrow check, the lifting lid is the interior mutation, and the Parchment cream (#FCEFC8) ledger is the borrow flag — one borrower at a time, checked when you reach in, not when you compile. Single still idle frame.
```

### 63 — The Swapwarden (cell — set or replace a single value through `Cell`)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = A figure standing over a pedestal-slot, pinching a coin between two fingers above it. A Crypt (#3E1220) keeper tunic with Oxblood (#6B1F35) primary and Wineflesh (#982D52) lit folds — the house burgundy — with Inkblood (#1B0810) seaming the deep shadow and hem. The pedestal fills the lower-center of the frame: a Bronze (#5E4116) column with a Bog umber (#2D1F0A) base, topped by a single socket framed in Antique brass (#9C7026), one Old gold (#D2A53F) coin seated in the slot. Above the slot, a second Old gold (#D2A53F) coin — each coin carrying a Brass leaf (#F0D27D) specular highlight — is lifted out by a Pink quartz (#EBC2CC) / Dusty rose (#C56883) pinch of two fingers, mid-swap. Slot seam and outlines in Coalblack (#161313). Head: short Stone grey (#7A7064) hair with Aged paper (#BFB2A0) highlights, Basalt (#3E3833) brow and eye line, skin in Pink quartz (#EBC2CC) / Dusty rose (#C56883). Pine (#27502E) hem, Bronze (#5E4116) / Bog umber (#2D1F0A) boots. No magic violet, no alarm scarlet, no specular white beyond the coin specs, no cobalt, no teal, no forest beyond the pine hem. Mood: deft, precise, unhurried. Class signature color: Old gold (#D2A53F). Lighting from top-left at 45°: the two brightest accents are the Brass leaf (#F0D27D) specs on the two coins — one lifting out above the slot, one seated in it — reading as one-replacing-the-other in the same single Bronze (#5E4116) socket: the swap, set and replace, the old value moving out whole. Single still idle frame.
```

### 64 — The Strongbox (rc_refcell — share *and* mutate with `Rc<RefCell<T>>`)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = A figure standing behind a heavy strongbox that fills the chest and waist. A Crypt (#3E1220) keeper tunic with Oxblood (#6B1F35) primary and Wineflesh (#982D52) lit folds — the house burgundy — with Inkblood (#1B0810) seaming the deep shadow and hem. The strongbox is a broad Bronze (#5E4116) body with several Antique brass (#9C7026) bands across it, a Bog umber (#2D1F0A) base, and a row of Old gold (#D2A53F) lock plates — multiple of them — each with a Coalblack (#161313) keyhole, several plates carrying a Brass leaf (#F0D27D) specular highlight. Above the box, a Pink quartz (#EBC2CC) / Dusty rose (#C56883) hand lifts a shared ring of several Old gold (#D2A53F) keys; the top band reads as a lid cracked open mid-turn. Seams and keyholes in Coalblack (#161313). Head: short Stone grey (#7A7064) hair with Aged paper (#BFB2A0) highlights, Basalt (#3E3833) brow and eye line, skin in Pink quartz (#EBC2CC) / Dusty rose (#C56883). Pine (#27502E) hem, Bronze (#5E4116) / Bog umber (#2D1F0A) boots. No magic violet, no alarm scarlet, no specular white beyond the key/plate specs, no cobalt, no teal, no forest beyond the pine hem. Mood: heavy-handed, guarded, communal. Class signature color: Old gold (#D2A53F). Lighting from top-left at 45°: the brightest accents are the Brass leaf (#F0D27D) specs on the shared keys and lock plates; the heavy strongbox with multiple keyholes and several shared keys is the `Rc` (shared ownership), and the lid cracked open mid-turn is the `RefCell` (interior mutation, borrow-checked at runtime) — many owners, each able to open and change. Single still idle frame.
```

### 65 — The Ghostkeeper (weak_ref — hold a non-owning `Weak` reference)

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel outline on
all character and object sprites using Coalblack (#161313), never pure black.
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects.

Color palette is the custom 32-color "Heraldic Code" palette (split-complementary
@ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker):
Burgundy ramp - Inkblood #1B0810, Crypt #3E1220, Oxblood #6B1F35 (PRIMARY BRAND),
Wineflesh #982D52, Dusty rose #C56883, Pink quartz #EBC2CC.
Gold ramp - Bog umber #2D1F0A, Bronze #5E4116, Antique brass #9C7026,
Old gold #D2A53F (THE CROWN), Brass leaf #F0D27D, Parchment cream #FCEFC8.
Teal ramp (the Borrow Checker's domain) - Abyssal #062123, Deep teal #154548,
Main teal #2A8482, Bright teal #5BB8AF, Mist teal #A4DED4.
Forest ramp - Mossbed #142A19, Pine #27502E, Forest #487E40, Spring meadow #82B450,
Hayfield #C9DC97.
Neutrals - Coalblack #161313, Basalt #3E3833, Stone grey #7A7064, Aged paper #BFB2A0.
Magic violet (sparing, <5% of frame) - Royal arcane #3A1559, Mage glow #9D6FE0.
Cool counterweight - Cobalt deep #0E2E54, Cobalt #377AB8.
Alarms / specular - Alarm scarlet #E63946 (errors only), Specular white #FFFFFF (<=1%).

No off-palette colors. No text, no signage with text, no logos. No extra fingers,
no melted features, no over-detailed hair. Cohesive heraldic-manuscript-meets-
Stardew-Valley mood (warm parchment + heraldic burgundy + cool teal magic),
deliberately NOT a generic warm-orange pixel-art palette. Resolution 32×32 pixels, transparent background.
SUBJECT = A deliberately faded, translucent figure — the only ghostly member of the cast — rendered almost entirely in low-contrast Aged paper (#BFB2A0) and Stone grey (#7A7064) with only sparing Crypt (#3E1220) to hint at the house tunic, and NO hard Coalblack outline so the silhouette reads as half-dissolved. Skin is faint Pink quartz (#EBC2CC) / Dusty rose (#C56883) with a barely-there Basalt (#3E3833) brow. One hand opens to frame-left. From that hand a thin Cobalt (#377AB8) dashed tether runs out to frame-right — broken into dashes (Cobalt #377AB8 highlight, Cobalt deep #0E2E54 shadow) so it reads as ghostly and non-binding. At the end of the tether, a hoard that has half-vanished: a broken Cobalt deep (#0E2E54) outline with gaps where walls should be, and inside it a single faded Old gold (#D2A53F) coin — the ghost of the value the tether points at. The cobalt pair is used precisely because it is not the warm burgundy/gold of the owning cast: it marks this figure and its link as cold, faint, non-owning. Boots and form fade out in Bronze (#5E4116) and Aged paper (#BFB2A0) with no solid soles — the figure does not fully touch the ground. Only a sparse Coalblack (#161313) outline; no magic violet, no alarm scarlet, no specular white. Mood: faded, mournful, insubstantial. Class signature color: Cobalt (#377AB8). Lighting flat and faint by design — the Ghostkeeper has no strong top-left key light because he is barely there; the one note of saturation is the Cobalt (#377AB8) dashed tether, and the one warm pixel is the faded Old gold (#D2A53F) coin in the half-vanished hoard. The broken hoard outline and gapped tether together say it plainly: this reference points, but does not own, and `upgrade()` may already return `None`. Single still idle frame.
```

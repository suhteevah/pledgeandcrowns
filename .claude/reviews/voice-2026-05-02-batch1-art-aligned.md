# Voice Audit — batch 1, art-aligned (HEAD = 5d928f8)

Scope: 5 missions whose NPCs got visual specs + sprites in commit `03e565c`:
`mut_binding` (Smith), `if_else_sign` (Cartographer), `double_function`
(Trait Mage), `loop_break` (Bellringer), `vec_iter` (Cooper).

Read each spec at `design/art/specs/<slug>.md`, then walked the existing
tutorial (`mission.rs`) and pass/fail flavor (`grader.rs` mirrored to
`stub_grader.rs`) against it. Goal: tutorial Hint and pass flavor should
*reinforce the same prop the player will see on screen* rather than
generic exposition.

## Findings, per mission

### Smith — `mut_binding`

- Spec emphasises the hammer's strike face as the only specular bright on the sprite — "where mutation *happens*." Spec also names the Smith's tutorial mantra: "name what changes."
- Current Hint just lists grader needles (`let mut` + `+= 1`) — fine for the harness but doesn't reinforce the Smith's voice.
- Current pass `"taps the anvil. \"good — that one bends.\""` — anvil is fine (workshop has both forge + anvil) but the *strike face* is the visual.
- Edit: lean Hint into Smith's mantra; pass flavor moves to the hammer strike on the work.

### Cartographer — `if_else_sign`

- Spec foregrounds two props: the rolled map showing one literal fork with a road glyph, and the brass compass that "tells you which branch you're on."
- Current Concept is pure type-system exposition. No hook to the compass or map. Tutorial reads as a stack-overflow answer with the Cartographer's name on it.
- Current pass `"three roads diverge — well chosen."` — already on register.
- Edit: lighten Hint with a compass/fork nudge; keep pass as-is.

### Trait Mage — `double_function`

- Spec frames her lesson as "the simplest possible *named transformation*: a tiny spell — input to output, no surprises." She carries a staff with a violet crystal in a gold ring (the only magic-violet allowance in the prelude).
- Current pass: `"smiles: \`double(21)\` would yield 42."` — exposition wrapped in her name, no staff/spell metaphor. Last audit (P2) flagged this as exposition-not-character.
- Edit: shift pass to incantation register tied to the staff; tutorial Concept can pick up "named spell" framing.

### Bellringer — `loop_break`

- Spec's metaphor is unusually explicit: "the rope is the loop body, the bell is the threshold, the sound is the break."
- Current Concept doesn't reference rope/bell at all. The metaphor is wasted on the player who walked up to a man holding a bell-rope.
- Current pass `"pulls the rope. \"the loop yielded its prize — 128.\""` — last audit flagged "prize" as faintly flowery; replacing with the spec's own metaphor (the bell sounding) is cleaner and ties tighter to the visual.
- Edit: Concept gets a one-line rope/bell/sound mapping; Hint stays needle-listy; pass flavor swaps "prize" for the bell sounding.

### Cooper — `vec_iter`

- Spec proposes the lesson framing verbatim: "open the lid, walk every item, do something with each, close the lid." Sprite shows a small barrel with a *gold lid* — contents are valuable, the lid opens.
- Current Concept is a `Vec<T>` exposition with no barrel reference. The Cooper's whole reason for being the NPC of this mission is the barrel-as-Vec metaphor.
- Current pass `"hoops the barrel. \"every stave counted, sum sealed.\""` — already on register.
- Edit: Concept opens with the barrel/iterator-triple framing; Hint stays as-is; pass unchanged.

## Cross-cutting

- All four "needs work" tutorials had the same shape: technically fine, character-mute. The fix is one to two sentences in Concept (or Hint) that reach for the prop the player just walked up to. Length stays inside 100-200 words.
- No grader needle was touched. No section header (`## Concept`/`## Syntax`/`## Task`/`## Hint`) was removed. No NPC name was changed. No tutorial code-fence example was edited.
- Server flavor edits mirrored byte-identically into `stub_grader.rs` per the contract test `server_and_stub_flavor_agree_byte_for_byte`.

## Edits applied

`mission.rs` (5 missions, tutorials only):

- `mut_binding` Hint: append "Smiths name what changes — the `mut` keyword is the name."
- `if_else_sign` Hint: lead with the Cartographer's compass before listing needles.
- `double_function` Concept: add one line framing `fn` as a "named spell — same inputs, same output."
- `loop_break` Concept: add one line mapping rope/bell/sound to body/threshold/break.
- `vec_iter` Concept: add the open-the-lid / walk-every-stave framing the spec proposes.

`grader.rs` + `stub_grader.rs` (mirrored):

- `mut_binding` pass: `"taps the anvil. \"good — that one bends.\""` → `"the Smith strikes the work. \"good — that one bends.\""` (hammer strike, the spec's specular bright).
- `double_function` pass: `"the Trait Mage smiles: \`double(21)\` would yield 42."` → `"the Trait Mage's staff hums. \"named once, doubled twice over.\""` (incantation tied to staff; keeps her name for the http test assertion).
- `loop_break` pass: `"pulls the rope. \"the loop yielded its prize — 128.\""` → `"pulls the rope — the bell sounds. \"the loop returned its number.\""` (the spec's own metaphor; "prize" gone).

Cartographer + Cooper pass flavor unchanged — already on register.

No fail-flavor strings edited — they all already name the NPC and stay terse.

# Voice Audit — batch 2, art-aligned (cursor was 5d928f8)

Scope: 5 missions whose NPCs got visual specs in batch 2 (`6d99ef5`).
Per-NPC mission mapping VERIFIED against `mission.rs` + `grader.rs`:

| NPC              | spec mission   | grader needles                                      |
|------------------|----------------|-----------------------------------------------------|
| The Oracle       | match_option   | `match`, `Some(`, `None`                            |
| The Herald       | struct_basic   | `struct Knight`, `name:`, `hp:`, `.name`            |
| The Twin         | tuple_destructure | `let (`, `,`, `) =`                              |
| The Tinker       | while_loop     | `while `, `> 0`, `-= 1`                             |
| The Heraldic Sage| enum_match     | `enum Direction`, `match `, `Direction::`           |

NB: the assignment prose said "Heraldic Sage (`iter_map_collect`)" and asked
the pass flavor to recap "iter→map→collect". That contradicts both the spec
file (`heraldic-sage.md` says `enum_match` — sum types, cloak of variants)
and the live code mapping. Per assignment rule "GRADER NEEDLES win", the
Sage's voice was aligned to the enum/match lesson the spec actually
describes. The Alchemist remains the iter→map→collect NPC.

## Findings, per mission

### Oracle — `match_option`

- Spec emphasises *exhaustive sight* — the orb's two halves are pixel-for-pixel
  the `Some` (Brass leaf, illuminated) and the `None` (Inkblood, present-but-empty).
  Veil + half-smile = "I see what is there *and* what is not."
- Current pass `"both paths walked. nothing slips through."` — already strong.
  The "nothing slips through" line maps cleanly to the exhaustiveness check.
  Keep.
- Current Concept opens with `Option<T>` exposition, no orb/veil/sight reference.
  The Oracle's whole sprite is the lesson; the Concept should say so in one line.
- Edit: prepend Concept with one line tying the orb's two halves to `Some` /
  `None`. Pass and fail flavor unchanged.

### Herald — `struct_basic`

- Spec: tabard is "literally a struct laid flat" — three banded color blocks =
  three named fields; he reads the scroll aloud at announcements.
- Current pass `"so named, so numbered."` — terse and on-register, but doesn't
  reach for the tabard or the scroll. The Herald's whole job is *reading
  named fields aloud*; pass should reference the tabard or the scroll.
- Current Concept is plain struct exposition — no Herald reference.
- Edit: Concept gains one line framing a struct as the Herald's "row of named
  fields"; pass moves to the Herald reading the field aloud.

### Twin — `tuple_destructure`

- Spec: one body, two halves — Oxblood and Forest split down the centre, two
  rings (Pink quartz `a` on left hand, Hayfield `b` on right). The destructure's
  parens are the gold circlet across the brow; the comma is the brass clasp.
- Current pass `"two names, one breath. cleanly split."` — already on-register.
- Current Concept is generic tuple exposition. The Twin sprite gives us
  unusually specific visual hooks (rings = `a`/`b`, clasp = `,`); spending
  one line on that earns the sprite.
- Edit: Concept opens with the Twin's "one body, two names" framing.
  Pass unchanged.

### Tinker — `while_loop`

- Spec: belt of alternating pouches = repeated cells = loop iterations.
  Mallet small (finer work than Smith). Goggles up = looking at the predicate.
- Current pass `"counted down, gear by gear."` — clean, on-register, even
  picks up the gear from the spec.
- Current Concept is generic while/for/loop disambiguation. No Tinker
  reference. Could use one line of "the Tinker's craft is repetition until
  the work is right".
- Edit: Concept gains one line on the Tinker's tighten-check-tighten cycle.
  Pass unchanged.

### Heraldic Sage — `enum_match`

- Spec: the four sigil panels stitched on the sash ARE the variants of the
  enum — the `match` arm walks left-to-right across the chest. "Heraldry is,
  in its own ancient language, the sum-type idea."
- Current pass `"every variant named, every blazon read."` — already very
  good; the blazon metaphor is exactly the spec's framing. Keep, but
  the assignment specifically asks the Sage's pass flavor to clearly tell
  the player what they just learned. Tighten so it names the lesson:
  enum + match.
- Current Concept opens "A Rust `enum` is a *sum type*..." — strong, but
  doesn't claim the heraldic frame the sprite gives us. The whole point of
  the Sage is that heraldry IS the enum lesson; one line up top earns it.
- Edit: Concept opens with the heraldry-as-sum-type line; pass tightened
  to name "every variant" + "every arm" so the curriculum lesson lands.

## Cross-cutting

- Same shape as batch-1 audit: tutorials are technically fine but
  character-mute. Fix is one to two lead lines in Concept that reach for
  the prop the player just walked up to. Pass-flavor mostly already on
  register; only the Herald and the Sage needed a pass tightening.
- No grader needle touched. No section header removed. No NPC name
  changed. No tutorial code-fence example edited. All pass-flavor edits
  mirrored byte-identically into `stub_grader.rs`.

## Edits applied

`mission.rs` (5 missions, tutorials only — Concept opens):

- `match_option` Concept: prepend "The Oracle holds an orb whose two halves
  are the answer present and the answer absent..." tying sprite to `Some`/`None`.
- `struct_basic` Concept: prepend one line framing a struct as the Herald's
  row of named fields read aloud from the tabard.
- `tuple_destructure` Concept: prepend "The Twin is one body bound to two
  names..." mirroring the spec's destructure-as-figure framing.
- `while_loop` Concept: prepend "The Tinker tightens, checks, tightens..."
  tying the rep-cells belt to the predicate-checked loop body.
- `enum_match` Concept: prepend "The Heraldic Sage's sash carries one
  panel per variant..." tying the four sigils to the `match` arms.

`grader.rs` + `stub_grader.rs` (mirrored, byte-identical):

- `struct_basic` pass: `"the Herald unfurls the scroll. \"so named, so numbered.\""`
  → `"the Herald reads the tabard. \"by name and by number — both fields announced.\""`
  (tabard + named-fields-read-aloud, matching the spec's load-bearing prop).
- `enum_match` pass: `"the Heraldic Sage opens the book of arms. \"every variant named, every blazon read.\""`
  → `"the Heraldic Sage walks the sash. \"every variant a panel, every arm a blazon read.\""`
  (sash + four sigils + match arms = the lesson named).

Oracle, Twin, Tinker pass flavor unchanged — already on register.
No fail-flavor strings edited. NPC names ("Oracle", "Herald", "Twin",
"Tinker", "Heraldic Sage") preserved verbatim in their pass strings —
the HTTP integration tests assert these.

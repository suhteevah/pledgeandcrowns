# Bosses — Art Prompts

> Composes with the **Style Suffix** from `design/04-art-handoff-prompts.md`. Always paste the Style Suffix verbatim before each prompt below.
>
> Bosses are the visual highlights of each act. Each gets its own multi-phase animation set (idle, attack wind-up, attack strike, hit, defeated, plus phase-transition art for multi-phase fights). Naming: `boss_<n>_<state>_<frame>.png`.
>
> **MVP scope:** only the Borrow Checker (Act 2) is required for the 30-day vertical slice. Every other boss in this doc is post-MVP.

## Production rules for bosses

1. **Silhouette readability.** A boss must read as a clear silhouette at half-size. If you squint and it looks like a generic blob, regenerate.
2. **Phase visual evolution.** Multi-phase bosses must look visibly different in each phase. Lighting changes, additional limbs/heads appear, color shifts — something obvious.
3. **Boss resolution standards:** mini-bosses 64×64, full bosses 96×96 to 128×128, final bosses (Act 10, Act 11) up to 160×160.
4. **Animation set per phase:** idle (4 frames), attack wind-up (3 frames), attack strike (3 frames), hit flash (1 frame), defeated/transitioning (4 frames). Optionally: special-attack frames for signature moves.
5. **Centered on canvas with breathing room.** Leave at least 8px of transparent space on every side for animation overshoot.

---

## Act 2 — The Borrow Checker (MVP-CRITICAL)

The only boss required for MVP. Treat with extra care.

### Borrow Checker — Idle

```
[STYLE SUFFIX] RESOLUTION = 64×64 transparent. SUBJECT = Tall austere robed
figure standing on a stone bridge surface. Cobalt deep robe (#0E2E54) with
embroidered glowing Main teal (#2A8482) ampersand "&" and "&mut" rune symbols
along hem and sleeves. Hood pulled low, two faint Bright teal-glowing (#5BB8AF)
eye-points visible in shadow. Iron staff in right hand, topped with a balance scale. Posture:
upright, formal, judicial. Robe has subtle motion as if magical breeze.
Idle frame 0 of 4 — base position.
```

```
[STYLE SUFFIX] RESOLUTION = 64×64 transparent. SUBJECT = Same Borrow Checker,
idle frame 1 of 4 — robe hem slightly raised by breeze, eye glow slightly
brighter. Subtle.
```

```
[STYLE SUFFIX] RESOLUTION = 64×64 transparent. SUBJECT = Same Borrow Checker,
idle frame 2 of 4 — staff tilted very slightly forward, eye glow at peak.
```

```
[STYLE SUFFIX] RESOLUTION = 64×64 transparent. SUBJECT = Same Borrow Checker,
idle frame 3 of 4 — return position, eye glow dimming back to baseline.
```

### Borrow Checker — Pointing (rejection animation, plays when player code fails)

```
[STYLE SUFFIX] RESOLUTION = 64×64 transparent. SUBJECT = Same Borrow Checker,
right hand extended forward, index finger pointing directly at viewer, head
tilted slightly down in disapproval. Robe swept back by gesture. Eyes glowing
intense Alarm scarlet (#E63946) instead of teal. Single frame.
```

### Borrow Checker — Approving (acceptance animation, plays when player code passes)

```
[STYLE SUFFIX] RESOLUTION = 64×64 transparent. SUBJECT = Same Borrow Checker,
both arms raised slightly outward in formal acknowledgment, robe glowing at
hems with bright Bright teal (#5BB8AF) light, head bowed slightly in respect. Single frame.
```

### Borrow Checker — Phase transition (between boss stages)

```
[STYLE SUFFIX] RESOLUTION = 64×64 transparent. SUBJECT = Same Borrow Checker,
all rune symbols glowing intensely, eye glow at maximum, faint Mist teal (#A4DED4)
aura surrounding entire body, robe hem floating up as if suspended. Used for the
between-stage impact moment. Single frame.
```

### Borrow Checker — Final bow (defeated, post-victory)

```
[STYLE SUFFIX] RESOLUTION = 64×64 transparent. SUBJECT = Same Borrow Checker,
deep formal bow, hood revealing very briefly a hint of an actual face (kind,
weathered, ancient), staff lowered to the ground beside him in respectful
stance. Single frame.
```

### Borrow Checker — Stage backgrounds (parchment overlays)

The Borrow Checker fight has 4 stages, each with a different code-puzzle theme.
Each stage gets a small overlay element that frames the encounter:

```
[STYLE SUFFIX] RESOLUTION = 96×96 transparent. SUBJECT = Floating glowing
Main teal (#2A8482) ampersand "&" rune, decorative element to overlay during
Stage 1 (simple borrow). Pixel-art glow effect, animated 4-frame loop, rotating
slowly clockwise. Single frame from loop.
```

```
[STYLE SUFFIX] RESOLUTION = 96×96 transparent. SUBJECT = Floating glowing
Bright teal (#5BB8AF) "&mut" rune cluster, overlay for Stage 2 (mutable borrow).
Slightly more intense glow than Stage 1 rune.
```

```
[STYLE SUFFIX] RESOLUTION = 96×96 transparent. SUBJECT = Floating glowing
"'a" lifetime tick rune, Old gold (#D2A53F) instead of teal. Overlay for
Stage 3 (lifetime annotation).
```

```
[STYLE SUFFIX] RESOLUTION = 96×96 transparent. SUBJECT = Three intertwined
runes (ampersand, &mut, 'a) forming a triangular composition, all glowing
their respective colors (Main teal, Bright teal, Old gold). Overlay for Stage
4 (final integration puzzle).
```

---

## Act 3 — The Variant Wraith (mini-boss)

```
[STYLE SUFFIX] RESOLUTION = 64×64 transparent. SUBJECT = Shapeshifter wraith
in form 1: "Solid" — armored knightly silhouette, Cobalt deep plate (#0E2E54),
glowing Alarm scarlet visor slit (#E63946), heavy two-handed sword. Idle pose.
```

```
[STYLE SUFFIX] RESOLUTION = 64×64 transparent. SUBJECT = Same wraith
transitioning to form 2: "Mist" — body partially dissolved into pale vapor
(Aged paper #BFB2A0), armor pieces floating loose, sword becoming translucent.
```

```
[STYLE SUFFIX] RESOLUTION = 64×64 transparent. SUBJECT = Wraith in form 2:
"Mist" — fully vaporous, no solid body, just a swirling pillar of pale
mist with two glowing Alarm scarlet eye-points and a hint of sword-shape inside.
```

```
[STYLE SUFFIX] RESOLUTION = 64×64 transparent. SUBJECT = Wraith transitioning
to form 3: "Shadow" — vapor condensing into solid darkness, Cobalt deep and
Coalblack silhouette emerging.
```

```
[STYLE SUFFIX] RESOLUTION = 64×64 transparent. SUBJECT = Wraith in form 3:
"Shadow" — pure dark silhouette (Coalblack #161313), Alarm scarlet eye-points,
holds a sword made of pure shadow with Alarm scarlet highlights.
```

```
[STYLE SUFFIX] RESOLUTION = 64×64 transparent. SUBJECT = Wraith in surprise
fourth form 4: "Possessed" — chaotic mix of all three previous forms,
flickering between solid/mist/shadow, multiple eye-points, sword splitting
into three. Used in the boss reveal moment that tests non-exhaustive match.
```

---

## Act 4 — The Chimera of Vexis

```
[STYLE SUFFIX] RESOLUTION = 96×96 transparent. SUBJECT = Three-headed
chimera. Body of a lion (Antique brass #9C7026 with Bronze shadows), three
necks extending: left head = goat (Aged paper / Stone grey, fire-element flames
coming from mouth), center head = lion (Old gold #D2A53F, water-element), right
head = eagle (Old gold #D2A53F, earth-element, Bog umber brown feathers). Wings
spread half-outward. Standing on stone
floor of mage tower. Idle pose.
```

```
[STYLE SUFFIX] RESOLUTION = 96×96 transparent. SUBJECT = Same chimera mid-
attack: all three heads roaring forward simultaneously, each spitting their
respective element (fire from goat, water spout from lion, earth shards from
eagle). Wings raised high.
```

```
[STYLE SUFFIX] RESOLUTION = 96×96 transparent. SUBJECT = Chimera reveal of
fourth head: a fourth neck has emerged from the back, ending in a serpent
head crackling with violet plasma (Mage glow #9D6FE0 / Bright teal #5BB8AF mix).
Other three heads
look surprised. Used for the mid-fight twist.
```

```
[STYLE SUFFIX] RESOLUTION = 96×96 transparent. SUBJECT = Chimera defeated:
all four heads bowed, wings drooping, body collapsing to one knee. Aura of
fading magic.
```

---

## Act 5 — The Tavernkeeper's Wager

```
[STYLE SUFFIX] RESOLUTION = 96×96 transparent. SUBJECT = The Tavernkeeper
revealed as a powerful figure: an enormous bearded innkeeper (warm peasant
build but huge, 2x normal NPC scale), in lamplight Old gold and Antique brass
tones, holding a giant ledger book in one hand and a tankard in the other,
standing behind a tavern counter that stretches across the lower edge of the
canvas. Eyes glow Old gold (#D2A53F) with bookkeeper's intensity. Idle pose.
```

```
[STYLE SUFFIX] RESOLUTION = 96×96 transparent. SUBJECT = Same Tavernkeeper
casting his quest-spell: ledger book floating open in front of him, pages
flying out as glowing parchment quest-receipts swirling around him. Each
page glowing with a different element of cascading Result types.
```

```
[STYLE SUFFIX] RESOLUTION = 96×96 transparent. SUBJECT = Tavernkeeper
defeated/satisfied: a wide grin, ledger closed, raising his tankard in a
toast to the player. The implication is he was always testing, and the
player passed.
```

---

## Act 6 — The Forge-Golem

```
[STYLE SUFFIX] RESOLUTION = 128×128 transparent. SUBJECT = Massive
mechanical golem made of forge-iron (Basalt / Stone grey) and Bronze conveyor-
belt segments. Body has visible iterator-pipeline mechanisms: belts running
across the chest, gear-pumps for shoulders, glowing Alarm scarlet (#E63946)
furnace heart visible behind iron grating. Two huge fists, articulated arms,
columnar legs.
Steam venting from joints. Idle pose, slight steam animation.
```

```
[STYLE SUFFIX] RESOLUTION = 128×128 transparent. SUBJECT = Same Forge-Golem
attacking: one fist raised high, other fist extending forward as if
hammering the ground, conveyor belts on chest spinning faster (motion lines),
forge heart (Alarm scarlet) glowing brighter.
```

```
[STYLE SUFFIX] RESOLUTION = 128×128 transparent. SUBJECT = Forge-Golem
phase 2 transformation: chest plates open to reveal the central pipeline
mechanism, exposing a glowing core that the player must target. Limbs
spread defensively, structure becomes more clearly mechanical.
```

```
[STYLE SUFFIX] RESOLUTION = 128×128 transparent. SUBJECT = Forge-Golem
defeated: collapsing in on itself, conveyor belts stopped, forge heart
dimming, steam venting wildly, kneeling pose with one fist on the ground.
```

---

## Act 7 — The Tide Hydra

```
[STYLE SUFFIX] RESOLUTION = 128×128 transparent. SUBJECT = Sea-serpent
hydra with six heads emerging from churning water at the canvas bottom.
Each head a slightly different shade of sea-blue (Cobalt deep #0E2E54 /
Cobalt #377AB8 / Bright teal #5BB8AF variations), scales with Mist teal
highlights, glowing Specular white eyes. Heads positioned at varied heights
and angles. Body submerged. Idle pose.
```

```
[STYLE SUFFIX] RESOLUTION = 128×128 transparent. SUBJECT = Same Hydra
mid-attack: three of six heads lunging forward simultaneously, jaws open
with water-magic energy crackling, while the other three rear back to
prepare their own attacks. Demonstrates the concurrency the player must
match.
```

```
[STYLE SUFFIX] RESOLUTION = 128×128 transparent. SUBJECT = Hydra phase
2: two of the heads have been struck down (severed, regrowth-particles
visible at neck stumps). Remaining four heads roar in unison. Body
emerging slightly more from water, revealing the central core target.
```

```
[STYLE SUFFIX] RESOLUTION = 128×128 transparent. SUBJECT = Hydra defeated:
all heads slumped, body sinking back into the sea, water settling, last
glowing eye-points fading.
```

---

## Act 8 — The Memory Warden

```
[STYLE SUFFIX] RESOLUTION = 96×96 transparent. SUBJECT = The Memory Warden:
an austere armored guardian in Cobalt deep and Old gold plate armor (#0E2E54 /
#D2A53F), face hidden behind a visored helm with Bright teal eye-slits. Holds
two ornate keys, one in each hand: left key glowing Oxblood (#6B1F35) ("Box"),
right key glowing Main teal (#2A8482) ("Rc"). Standing in the Vault interior.
Idle pose.
```

```
[STYLE SUFFIX] RESOLUTION = 96×96 transparent. SUBJECT = Same Warden in
phase 2: now holding four keys (two more have appeared, one in each
elbow-joint), each glowing a different color representing Box, Rc, Arc,
RefCell. Defensive stance.
```

```
[STYLE SUFFIX] RESOLUTION = 96×96 transparent. SUBJECT = Warden in cycle-
test phase: the keys he holds are now connected by glowing chains that form
visible loops between them — representing Rc cycles. He gestures for the
player to break the cycle (the answer being Weak references).
```

```
[STYLE SUFFIX] RESOLUTION = 96×96 transparent. SUBJECT = Warden defeated:
all keys laid out in a fan in front of him as he kneels, vault doors
opening behind him. Ceremonial pose of yielding.
```

---

## Act 9 — The Librarian

```
[STYLE SUFFIX] RESOLUTION = 96×96 transparent. SUBJECT = The Librarian: an
ancient, gaunt sage in long Royal arcane (#3A1559) robes covered in scroll
fragments and hanging ink-pots. White-grey beard down to chest. Holding
an enormous book that hovers open in front of him, pages flipping by
themselves. Eyes are pure white pinpoints. Surrounded by floating books
and scroll fragments. Idle pose.
```

```
[STYLE SUFFIX] RESOLUTION = 96×96 transparent. SUBJECT = Same Librarian
casting an unsafe spell: the floating book in front of him is now glowing
Alarm scarlet (#E63946), tendrils of corruption-magic (Alarm scarlet glitch-
pixel artifacts, palette only) extending from the book toward the viewer, his
eyes glowing Alarm scarlet instead of white.
```

```
[STYLE SUFFIX] RESOLUTION = 96×96 transparent. SUBJECT = Librarian's macro-
challenge phase: he is gesturing toward an enormous ornate scroll that
unfurls beside him, the scroll covered in elaborate placeholder rune-
symbols (no text — engine renders the actual macro). The scroll is the
boss puzzle the player must solve.
```

```
[STYLE SUFFIX] RESOLUTION = 96×96 transparent. SUBJECT = Librarian
defeated: his book closes gently, floating books settle to the floor,
he bows and offers an open palm presenting a small glowing artifact
(the unsafe codex reward).
```

---

## Act 10 — The Compiler (final boss)

The largest, most elaborate boss in the campaign.

```
[STYLE SUFFIX] RESOLUTION = 160×160 transparent. SUBJECT = The Compiler:
a colossal silver-and-chrome figure on a celestial throne. Humanoid silhouette
but entirely constructed of polished steel, brass gears, and silver plate.
Crown of forged iron with embedded crystal facets representing each Rust
ecosystem tool (cargo, clippy, rustfmt, etc.). Eyes are pure white light.
Holds a massive double-edged sword inscribed with placeholder runes (no
text). Throne behind him is ornate, made of stacked book-shaped silver
plates. Idle pose.
```

```
[STYLE SUFFIX] RESOLUTION = 160×160 transparent. SUBJECT = Same Compiler
mid-attack: sword raised in a two-handed overhead strike position, throne
behind him cracking slightly with white energy emerging through the
fractures. Both feet planted, body coiled.
```

```
[STYLE SUFFIX] RESOLUTION = 160×160 transparent. SUBJECT = The Compiler
phase 2: he has stood up from the throne and is now floating slightly
above it. His armor has opened panels to reveal inner mechanisms (gears,
flowing data-streams shown as palette-only pixel patterns), and additional
weapons orbit him (a hammer, a chisel, a quill, a scale — symbols of
his work).
```

```
[STYLE SUFFIX] RESOLUTION = 160×160 transparent. SUBJECT = The Compiler
phase 3: full power. Body now surrounded by an aura of pure white-gold
light, sword glowing with multi-colored energy representing every previous
boss's element combined, throne shattered behind him as he hovers in
mid-air.
```

```
[STYLE SUFFIX] RESOLUTION = 160×160 transparent. SUBJECT = The Compiler
defeated: kneeling, sword embedded in the ground in front of him hilt-up
in a gesture of yielding. Crown removed, held out toward the viewer.
Behind him, the throne has been replaced by a soft golden light suggesting
ascension.
```

---

## Act 11 (Temple) — The Final Cantor

The post-game challenge boss for the HolyC roguelike.

```
[STYLE SUFFIX] RESOLUTION = 160×160 transparent. SUBJECT = The Final Cantor:
a towering figure of stained-glass and chrome standing in a cathedral nave.
Body composed of fitted stained-glass panels (red, blue, gold primaries)
held together by chrome lead-came. No visible face — the head is a single
large pane of stained-glass showing an abstract eye-pattern. Holding a
giant scroll of placeholder code (no text). Hovering slightly above the
floor. Cathedral architecture frames him. Idle pose.
```

```
[STYLE SUFFIX] RESOLUTION = 160×160 transparent. SUBJECT = Final Cantor
mid-recitation phase: the scroll has unfurled fully to fill the air around
him, glowing with light passing through stained-glass colors. Cantor's
posture upright and proclamatory. Beams of colored light radiate from
his body in cathedral-style rays.
```

```
[STYLE SUFFIX] RESOLUTION = 160×160 transparent. SUBJECT = Cantor in
heresy phase: stained-glass panels of his body have shifted color
(originally Oxblood panels are now Mage glow violet, Cobalt panels are now
Main teal, etc.), suggesting inversion. Scroll text appears mirrored. Slight
flickering effect (palette-only stripe artifacts).
```

```
[STYLE SUFFIX] RESOLUTION = 160×160 transparent. SUBJECT = Cantor in
final-verdict phase: descended to floor level, scroll consumed, both
hands extended forward holding only a single quill made of pure light.
Awaiting the player's final program.
```

```
[STYLE SUFFIX] RESOLUTION = 160×160 transparent. SUBJECT = Cantor accepting
the player's offering: bowing deeply, all stained-glass panels glowing
Old gold (#D2A53F) in benediction, beams of colored light radiating out
toward the viewer.
```

```
[STYLE SUFFIX] RESOLUTION = 160×160 transparent. SUBJECT = Cantor defeated
by self-segfault (alternate ending): he is reading the player's program
out loud (placeholder rune-symbols floating from his mouth), and his
stained-glass body is fracturing — cracks spreading across panels, several
panels falling away revealing empty space inside the figure. Mood: tragic-
comic, knowing.
```

---

## Production checklist per boss

- [ ] Idle animation set complete (4 frames minimum)
- [ ] Attack wind-up + attack strike frames
- [ ] Hit-flash frame
- [ ] Defeated/transitioning frames
- [ ] Phase-transition art for multi-phase bosses
- [ ] Special-move art for signature attacks (where applicable)
- [ ] All frames silhouette-test passed (recognizable at 50% scale)
- [ ] Filed under `assets/sprites/bosses/<n>/<state>_<frame>.png`
- [ ] Logged in `assets/sprites/bosses/MANIFEST.md`
- [ ] Reviewed against style bible v1.0 (or current version)

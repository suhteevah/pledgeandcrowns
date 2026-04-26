# Characters — Art Prompts

> Composes with the **Style Suffix** from `design/04-art-handoff-prompts.md`. Always paste the Style Suffix verbatim before each prompt below.
>
> Naming convention: `npc_<name>_<state>_<frame>.png`
> Example: `npc_borrow_checker_idle_0.png`, `npc_borrow_checker_dialogue_0.png`

## Player character

### Default — male preset

```
[STYLE SUFFIX] RESOLUTION = 32×32 transparent. SUBJECT = Front-facing human
adventurer, idle, Cobalt deep tunic (#0E2E54), tan leather belt, brown boots,
hip satchel, short brown hair, neutral expression, wooden staff with Old gold
(#D2A53F) crystal tip. Light skin (designed for palette swap).
```

### Default — female preset

```
[STYLE SUFFIX] RESOLUTION = 32×32 transparent. SUBJECT = Front-facing human
adventurer, idle, Forest hooded cloak (#487E40), tan trousers, brown boots,
hip pouch, brown braid over one shoulder, neutral expression, wooden staff
with Old gold (#D2A53F) crystal tip. Light skin (designed for palette swap).
```

### Player walk-cycle frames (×4 directions, ×4 frames each)

```
[STYLE SUFFIX] RESOLUTION = 32×32 transparent. SUBJECT = Same player character
(<preset name>), <direction = down/up/left/right>, walking animation frame
<0/1/2/3> of 4. Frame 0 = both feet planted, frame 1 = right foot forward,
frame 2 = both feet planted slightly different, frame 3 = left foot forward.
```

### Player attack — staff swing

```
[STYLE SUFFIX] RESOLUTION = 32×32 transparent. SUBJECT = Same player character,
attack animation frame <0..5> of 6. Frame 0 = staff raised wind-up, frame 1 =
staff at peak, frame 2 = mid-strike with motion blur lines (palette only),
frame 3 = strike connect, frame 4 = recovery, frame 5 = return to idle.
```

### Player cast — magic invocation

```
[STYLE SUFFIX] RESOLUTION = 32×32 transparent. SUBJECT = Same player character,
casting animation frame <0..5> of 6. Both arms raised, crystal at staff tip
glowing brighter each frame, magic particles (Old gold and Mage glow violet) gathering.
```

### Player hit + death

```
[STYLE SUFFIX] RESOLUTION = 32×32 transparent. SUBJECT = Same player character,
hit-flash frame, full character whitened to Parchment cream (#FCEFC8) silhouette,
single frame.
```

```
[STYLE SUFFIX] RESOLUTION = 32×32 transparent. SUBJECT = Same player character,
death animation frame <0..3>. Frame 0 = staggered backward, frame 1 = falling,
frame 2 = collapsed, frame 3 = silhouette only fading.
```

## Ferris guide

### Ferris idle

```
[STYLE SUFFIX] RESOLUTION = 32×32 transparent. SUBJECT = Small friendly crab
mascot character, NOT the official Rust Foundation Ferris (distinct silhouette,
burgundy-and-gold instead of orange, more cartoonish). Round body, expressive
eyes on stalks, two front claws raised in welcome. Body Wineflesh (#982D52) midtone
with Old gold (#D2A53F) highlights on claw-tips and shell ridges, Crypt (#3E1220)
shadows underneath, Dusty rose (#C56883) belly. Idle pose, looking up-right as if
narrating. Single still frame.
```

### Ferris walk (×4 directions, ×4 frames)

```
[STYLE SUFFIX] RESOLUTION = 32×32 transparent. SUBJECT = Same Ferris,
<direction>, walk frame <0..3>. Sideways scuttle motion (crab-style — even
when "walking up" or "down," he moves at a slight diagonal). Claws sway
opposite to feet for animation life.
```

### Ferris excited (used during quest reveals)

```
[STYLE SUFFIX] RESOLUTION = 32×32 transparent. SUBJECT = Same Ferris, both
claws raised high above head, eyes wide, body slightly bouncing off ground
(small dust puff under feet, palette-only). Excited pose, single frame.
```

### Ferris worried (used when player is failing repeatedly)

```
[STYLE SUFFIX] RESOLUTION = 32×32 transparent. SUBJECT = Same Ferris, claws
held in front like wringing hands, eyes drooping, mouth in small frown.
Single frame.
```

## Hearthstone Village NPCs

### Village Elder

```
[STYLE SUFFIX] RESOLUTION = 32×32 transparent. SUBJECT = Elderly human, kind
expression, long grey beard, simple Parchment cream robe (#FCEFC8) with Oxblood
(#6B1F35) sash, wooden staff with no crystal, slightly hunched. Idle frame.
```

### Hearthstone Innkeeper

```
[STYLE SUFFIX] RESOLUTION = 32×32 transparent. SUBJECT = Stout middle-aged
woman, apron over Oxblood (#6B1F35) dress, hair tied up, holding a wooden ladle.
Welcoming expression. Idle frame.
```

### Bell-Ringer Apprentice (NPC for Bell Tower quest)

```
[STYLE SUFFIX] RESOLUTION = 32×32 transparent. SUBJECT = Young teen, eager
expression, simple beige tunic, wooden mallet in hand, looking up at bell.
Idle frame.
```

### Generic villager (×3 variants for crowd)

```
[STYLE SUFFIX] RESOLUTION = 32×32 transparent. SUBJECT = Adult villager,
variant <A/B/C>, simple peasant clothes in brown/cream/grey, neutral pose.
A = farmer with hoe, B = merchant with small box, C = traveler with backpack.
```

## Borrow Bridge zone

### The Borrow Checker (boss)

#### Idle

```
[STYLE SUFFIX] RESOLUTION = 64×64 transparent. SUBJECT = Tall austere robed
figure on stone bridge. Cobalt deep robe (#0E2E54) with embroidered glowing
Main teal (#2A8482) ampersand and "&mut" rune symbols along hem and sleeves.
Hood pulled low, two Bright teal-glowing (#5BB8AF) eyes faintly visible in
the shadow. Iron staff topped with balance scale. Upright formal posture.
Subtle robe motion as if magical breeze. Idle frame.
```

#### Pointing (rejection animation)

```
[STYLE SUFFIX] RESOLUTION = 64×64 transparent. SUBJECT = Same Borrow Checker,
right hand extended forward, index finger pointing, head tilted slightly down
in disapproval, robe swept by gesture. Single frame.
```

#### Approving (acceptance animation)

```
[STYLE SUFFIX] RESOLUTION = 64×64 transparent. SUBJECT = Same Borrow Checker,
both arms raised slightly, robe glowing at the hems with Bright teal (#5BB8AF)
light, head slightly bowed in respect. Single frame.
```

#### Glow phase (used during boss fight stage transitions)

```
[STYLE SUFFIX] RESOLUTION = 64×64 transparent. SUBJECT = Same Borrow Checker,
all rune symbols glowing brighter, eye glow brighter, faint Mist teal (#A4DED4)
aura around entire body. Frame for stage-transition impact moment.
```

### Two Travellers NPC pair (Act 2 encounter)

```
[STYLE SUFFIX] RESOLUTION = 32×32 transparent each. SUBJECT = Two travellers
on a road, variant A and B. A = scholar with parchment scroll, simple Bog umber
brown robe. B = wanderer with walking stick, Cobalt (#377AB8) cloak. Both reaching
toward a shared scroll between them.
```

### The Mutable Quill (NPC who needs `&mut`)

```
[STYLE SUFFIX] RESOLUTION = 32×32 transparent. SUBJECT = Hunched scribe-NPC,
ink-stained fingers, wire spectacles, bald head with ink-spot, holding a
goose-quill pen poised over the air as if waiting to write. Single frame.
```

## Generic enemies (Acts 1–2)

### Bandit (sparring dummy graduates to real enemy)

```
[STYLE SUFFIX] RESOLUTION = 32×32 transparent. SUBJECT = Adult human bandit,
brown leather vest, Alarm scarlet (#E63946) bandana over lower face, short sword
in right hand, crouched aggressive stance. Skin tone: light. Single frame.
```

### Goblin (Act 2 encounter)

```
[STYLE SUFFIX] RESOLUTION = 32×32 transparent. SUBJECT = Small green goblin
(Forest #487E40 mid with Pine #27502E shadows), pointed ears, opportunistic
smirk, brown leather scraps, rusty short sword, hunched. Single frame.
```

### Bridge Wraith (mid-bridge enemy in Act 2)

```
[STYLE SUFFIX] RESOLUTION = 48×48 transparent. SUBJECT = Translucent ghostly
figure made of pale mist (Aged paper #BFB2A0 base with Mist teal #A4DED4
highlights), no clearly defined limbs, two glowing Specular white eye-points.
Floating. Single frame.
```

## Future-act NPCs (defer until earlier batches approved)

Stub list, full prompts to be added in subsequent design rounds:

- Vexis the Trait Mage (Act 4, tower exterior + tower interior outfits)
- Old Maybe (Act 5 tavernkeeper)
- The Forge-Smiths (Act 6, ×3 variants)
- The Lighthouse Keepers, North + South (Act 7)
- The Vault Warden (Act 8 boss, large)
- The Librarian (Act 9 boss)
- The Compiler (Act 10 final boss, very large, multiple phases)
- The Final Cantor (Act 11 / Temple boss)

These are not for MVP. Generation can begin when their respective acts come into scope.

## Production checklist per character

When finalizing a character:
- [ ] Idle frame approved
- [ ] Walk cycle complete (4 directions × 4 frames if applicable)
- [ ] Attack animation if combat-relevant
- [ ] Hit flash + death frames if combat-relevant
- [ ] Cast/dialogue special poses if narrative-relevant
- [ ] Filed in `assets/sprites/<category>/<name>_<state>_<frame>.png`
- [ ] Logged in `assets/sprites/MANIFEST.md` with date and approver

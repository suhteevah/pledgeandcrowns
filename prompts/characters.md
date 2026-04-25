# Characters — Art Prompts

> Composes with the **Style Suffix** from `design/04-art-handoff-prompts.md`. Always paste the Style Suffix verbatim before each prompt below.
>
> Naming convention: `npc_<name>_<state>_<frame>.png`
> Example: `npc_borrow_checker_idle_0.png`, `npc_borrow_checker_dialogue_0.png`

## Player character

### Default — male preset

```
[STYLE SUFFIX] RESOLUTION = 32×32 transparent. SUBJECT = Front-facing human
adventurer, idle, deep-blue tunic (#124e89), tan leather belt, brown boots,
hip satchel, short brown hair, neutral expression, wooden staff with rust-orange
crystal tip. Light skin (designed for palette swap).
```

### Default — female preset

```
[STYLE SUFFIX] RESOLUTION = 32×32 transparent. SUBJECT = Front-facing human
adventurer, idle, forest-green hooded cloak (#3e8948), tan trousers, brown boots,
hip pouch, brown braid over one shoulder, neutral expression, wooden staff
with rust-orange crystal tip. Light skin (designed for palette swap).
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
glowing brighter each frame, magic particles (rust-orange and cyan) gathering.
```

### Player hit + death

```
[STYLE SUFFIX] RESOLUTION = 32×32 transparent. SUBJECT = Same player character,
hit-flash frame, full character whitened to palette cream (#ead4aa) silhouette,
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
lighter shell, more cartoonish). Round body, expressive eyes on stalks, two
front claws raised in welcome. Body warm rust orange (#be4a2f / #d77643), lighter
belly. Idle pose, looking up-right as if narrating. Single still frame.
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
expression, long grey beard, simple cream robe (#ead4aa) with rust-orange
sash, wooden staff with no crystal, slightly hunched. Idle frame.
```

### Hearthstone Innkeeper

```
[STYLE SUFFIX] RESOLUTION = 32×32 transparent. SUBJECT = Stout middle-aged
woman, apron over rust-orange dress, hair tied up, holding a wooden ladle.
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
figure on stone bridge. Deep navy robe (#262b44) with embroidered glowing
cyan ampersand and "&mut" rune symbols along hem and sleeves. Hood pulled
low, two cyan-glowing eyes faintly visible in the shadow. Iron staff topped
with balance scale. Upright formal posture. Subtle robe motion as if magical
breeze. Idle frame.
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
both arms raised slightly, robe glowing at the hems with cyan light, head
slightly bowed in respect. Single frame.
```

#### Glow phase (used during boss fight stage transitions)

```
[STYLE SUFFIX] RESOLUTION = 64×64 transparent. SUBJECT = Same Borrow Checker,
all rune symbols glowing brighter, eye glow brighter, faint cyan aura around
entire body. Frame for stage-transition impact moment.
```

### Two Travellers NPC pair (Act 2 encounter)

```
[STYLE SUFFIX] RESOLUTION = 32×32 transparent each. SUBJECT = Two travellers
on a road, variant A and B. A = scholar with parchment scroll, simple brown
robe. B = wanderer with walking stick, blue cloak. Both reaching toward a
shared scroll between them.
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
brown leather vest, red bandana over lower face, short sword in right hand,
crouched aggressive stance. Skin tone: light. Single frame.
```

### Goblin (Act 2 encounter)

```
[STYLE SUFFIX] RESOLUTION = 32×32 transparent. SUBJECT = Small green goblin
(#3e8948 with #265c42 shadows), pointed ears, opportunistic smirk, brown
leather scraps, rusty short sword, hunched. Single frame.
```

### Bridge Wraith (mid-bridge enemy in Act 2)

```
[STYLE SUFFIX] RESOLUTION = 48×48 transparent. SUBJECT = Translucent ghostly
figure made of pale blue mist (#c0cbdc with cyan #2ce8f5 highlights), no
clearly defined limbs, two glowing white eye-points. Floating. Single frame.
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

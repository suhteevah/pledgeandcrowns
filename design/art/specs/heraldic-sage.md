# The Heraldic Sage — visual spec

**REF-20**, 32×32, idle, transparent bg.
**Mission:** `enum_match` — teaches `enum` definition + `match` on its variants.

## Curriculum cue

An `enum` is a *sum type*: a value is exactly one of a fixed roster of named variants. Heraldry is, in its own ancient language, exactly the same idea — a coat of arms is a finite vocabulary of charges (lion, cross, sun, oak) and a viewer reads the device by matching against that vocabulary. The Heraldic Sage is the village's living lexicon of those variants. His robe is openly a *cloak of variants*: four small distinct sigil panels stitched onto the front, each a single-charge field, each a different ramp. The mission's `match` arm walks left-to-right across his chest.

This is the Sage's first appearance in Act 2 (he is the one Act-2 NPC shipped this batch). His Act-2 zone allegiance is the **Codex Court**, mapped to the gold-on-burgundy heraldic tradition; this is signalled by his grounded heraldic-gold ramp without any Tower-violet allowance.

## Visual brief

Older human, slim and tall. Long deep-burgundy formal robe — **Crypt** primary, **Inkblood** shadow, **Oxblood** mid — with a wide **Old gold** sash across the chest from left shoulder to right hip. Onto the sash are stitched **four** small 3×3 sigil panels, left-to-right: a **Forest** square (charge: oak field), a **Bright teal** triangle (charge: river), an **Old gold** lozenge (charge: sun), a **Cobalt** diamond (charge: shield-azure). Each is bordered in **Coalblack**. The `match` arms made literal. The four-sigil read is the load-bearing detail — keep the rest of the sprite quiet so the panels carry the eye.

White-streaked **Stone grey** hair past the ears, neat **Bronze** beard. Skin is the standard human ramp. Hands are folded in front at waist height — heralds and lexicographers, not warriors. The robe hem trails one row past the boots; **Brass leaf** stitching at the hem.

Foreshadowing license used: the small **Cobalt** sigil panel (one of the four variants) is a deliberate ≤1% canvas signpost toward Act 6's cool-counterweight zone, where the Sage's variant taxonomy expands beyond the warm palette. Per the bible's color rule, cobalt is a counterweight reserved for that zone; on this sprite it is one 3×3 patch (~0.9% of the 1024-pixel canvas), inside the Trait-Mage allowance precedent confirmed for batch 2.

## Palette compliance

Burgundy (robe, mouth): `K Y O W R P`. Gold (sash, hem, sun sigil, beard, sigil border): `U Z N G L`. Forest (oak sigil): `F E`. Teal (river sigil): `T I`. Cool counterweight (azure sigil — foreshadowing allowance, ~9 pixels of 1024 ≤ 0.9% < 1% bible cap): `<` `>`. Neutrals (hair, outline): `X B S V`.

No magic violet (Sage is not a Tower scholar), no alarm scarlet, no specular white.

## Lighting

Top-left 45° on the body. Sash and sigil panels are flat-rendered — heraldry is read graphically, not modelled. Robe gets standard Wineflesh-on-left / Inkblood-on-right gradient.

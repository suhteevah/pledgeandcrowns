# The Oracle — visual spec

**REF-16**, 32×32, idle, transparent bg.
**Mission:** `match_option` — teaches `match` on `Option<T>`.

## Curriculum cue

The Oracle's lesson is *exhaustive sight*. `match` on `Option<T>` forces both arms — `Some(n)` and `None` — and the compiler refuses to let you walk away from either. The Oracle is the village seer who answers every question by acknowledging the absent answer as cleanly as the present one. Her sprite reads "I see what is there *and* what is not."

## Visual brief

Tall, slender, robed and **veiled** from forehead to nose-bridge — a single horizontal Crypt-shadow band stands in for the blindfold (canonically Oracle-blind, as the Pythia is). Below the veil her mouth is set in a calm half-smile (one Wineflesh pixel under the chin line). She holds a small spherical orb at chest height in cupped hands; the orb has two halves pixel-for-pixel: the upper half is **Brass leaf** (the `Some` answer, illuminated) and the lower half is **Inkblood** (the `None` answer, present-but-empty). The orb's outline is one ring of **Old gold**.

Robe is **Crypt** primary with **Inkblood** deep shadow and **Oxblood** mid — darker than the Trait Mage, befitting a chthonic seer. **Stone grey** trim at hem and sleeves (no gold trim — the Oracle is not a scholar of Crown lore, she is older). Bare feet (or one-pixel hint of toes peeking from hem). Hair is **Crypt** loose past the shoulders.

Skin is the standard human ramp — Pink quartz / Dusty rose / Wineflesh — though slightly desaturated by leaning more on Wineflesh than the others.

## Palette compliance

Burgundy ramp (robe, skin): `K Y O W R P`. Gold ramp (orb half, orb ring): `N G L`. Neutrals (veil shadow, trim, outline, hair shadow): `X B S Y`. The orb's two halves embody the Some/None duality but the colors are both already in-palette — no special allowance.

No magic violet, no alarm scarlet, no specular white, no cobalt blue, no teal.

## Lighting

Top-left 45°. Robe lit Wineflesh on the left edge, Inkblood on the right. The orb is its own light source — top half always Brass leaf regardless of global gradient (the `Some` half *is* the highlight).

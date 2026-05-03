# The Linguist — visual spec

**REF-22**, 32×32, idle, transparent bg.
**Mission:** `string_vs_str` — `&str` parameter accepting both `String` and string literal.

## Curriculum cue

The Linguist is the village's authority on *one tongue, two voices*: a single function signature (`&str`) that accepts both `String` (owned, heap) and `&'static str` (literal, in the binary). The visual contract makes "one form, two channels" literal: an open book in the left hand (the owned `String` — bound, on-heap, long-lived), a small parchment scroll in the right hand (the borrowed literal — short, fixed, weightless). The figure stands centred between them. Voice spec from the editor pass: "one tongue, two voices".

## Visual brief

Slim scholar in a long **Crypt** robe with a **Parchment cream** undertunic showing at collar and hem — neutral, bookish silhouette. **Aged paper** sash at the waist tied off-centre. Two-toned hair: **Stone grey** under, single row of **Aged paper** on top — old, but not Sage-old. Reads as forty, not seventy.

In the **left** hand, held open at chest height, a small **Parchment cream** book with **Bog umber** binding and a single **Old gold** pixel for the clasp — the *owned* `String`, weighty, contained. In the **right** hand, held lower at hip, a tightly rolled **Parchment cream** scroll with **Bronze** end-caps — the *borrowed* `&str` literal, light enough to dangle. The two props match in colour family (both parchment) so the read is "same content, two containers" — exactly the deref-coercion lesson.

A small **Brass leaf** speech curl rises from the figure's lips (1–2 pixels) — the "tongue" of the voice spec. Skin is the standard human palette. No staff, no hat, no insignia: the Linguist is functional, not ceremonial.

## Palette compliance

Burgundy (robe): `K Y O W`. Gold (clasp, scroll caps, speech curl, sash hi): `Z N G L`. Neutrals (hair, outline, sash, book/scroll): `X B S V C` (parchment cream `C` is in the gold ramp). Skin: `P R W`. Bog umber `U` for the book binding.

No magic violet, no alarm scarlet, no specular white, no cobalt, no teal, no forest.

## Lighting

Top-left 45°. The book (left, lit side) gets a `C` highlight row; the scroll (right, shadow side) gets a `Z` shadow row. This reinforces that the "owned" and "borrowed" forms are the same content read under different light — the exact deref-coercion intuition.

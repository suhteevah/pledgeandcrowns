# The Trait Mage — visual spec

**REF-13**, 32×32, idle, transparent bg.
**Mission:** `double_function` — teaches `fn` definitions and explicit return types.

## Curriculum cue

The Trait Mage's domain in the bible is the Trait Mage's Tower (Act 4, magic violet zone). In the Act-1 prelude she appears in Hearthstone Village as the village's resident scholar, where her lesson is the simplest possible *named transformation*: `fn double(n) -> n * 2`. A function is a tiny spell — input to output, no surprises. Her sprite is allowed the **only** dose of magic violet in the prelude cast (per bible scarcity rules: ≤5% of any frame; she gets a single rune glow on her staff, well under the cap), as a signpost that she belongs to the Tower elsewhere in the game.

## Visual brief

Tall slender human, robed and hooded but with the hood thrown back — she's not concealing herself in the village like the Borrow Checker does on his bridge. The robe is **Oxblood** primary (heraldic family) with **Crypt** shadow and **Wineflesh** mid-tone, **Old gold** trim at hem and cuffs (she is a scholar of the Crown's lore as much as of magic). Inside-hood lining shows **Royal arcane** in two pixels along the collar — a peek at her Tower allegiance.

She carries a vertical staff in her right hand, hilt at her hip, top above her head. The staff is **Basalt** with **Stone grey** mid-tone; its head is a small **Mage glow** crystal (3-4 pixels of `%` = #9D6FE0) framed by an **Old gold** ring. This is the entire magic-violet allowance for the sprite. Skin uses the standard Pink quartz / Dusty rose / Wineflesh ramp. Hair is **Crypt**/**Basalt**, tied back. Eyes are a single **Bright teal** pixel each — a faint scholar's glow, hinting she sees more than she says.

## Palette compliance

Burgundy (robe, skin): `K Y O W R P`. Gold (trim, staff ring): `N G L`. Neutrals (staff, hair, outline): `X B S`. Magic (crystal, hood lining): `* %` — capped at <8 pixels total of the 1024-pixel canvas (≤0.8%, well under the 5% rule). Teal (eye glow): `I`.

No alarm scarlet, no specular white, no Cobalt blue.

## Lighting

Top-left 45°. Robe left side gets Wineflesh; right side Crypt. Staff crystal glows uniformly — magic light is its own source, doesn't take the global gradient.

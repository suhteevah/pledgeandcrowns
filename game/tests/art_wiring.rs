// SPDX-License-Identifier: MIT
//! Audit: approved art is wired where the game loads it.
//!
//! - every roster NPC's sprite exists, is its own file, and no Act 3-8 NPC
//!   still uses a shared placeholder (the player sprite or a byte copy of
//!   another NPC);
//! - the 30 tranche-2 villagers (approved 2026-11-15) ship exactly their
//!   `REFNNv2.png` render, and their ref headers say so;
//! - the six kit holders (Guildmaster, Conjurer, Barkeep, Smelter,
//!   Dockmaster, Warden) ship their act's approved costume-kit exemplar
//!   (REF-69..74), and their first-pass refs are marked superseded;
//! - the Act 9 cast (approved 2026-11-15) ships its REF-80..85 and REF-93
//!   renders; the Library kit exemplar REF-79 is not any NPC's sprite;
//! - the walk sheets (REF-77, REF-78) have the size and tile order
//!   `plugins::animation` slices them by.

use pledge_and_crown::assets::{
    SPRITE_BORROW_CHECKER, SPRITE_FERRIS, SPRITE_FERRIS_WALK, SPRITE_PLAYER, SPRITE_PLAYER_WALK,
};
use pledge_and_crown::plugins::animation::{
    FERRIS_SHEET_COLS, FERRIS_SHEET_ROWS, FERRIS_SHEET_TILES, FRAME_PX, PLAYER_SHEET_COLS,
    PLAYER_SHEET_ROWS, PLAYER_SHEET_TILES,
};
use pledge_and_crown::plugins::npc::NPC_ROSTER;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

fn game_assets() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets")
}

fn refs_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../design/art/refs")
}

fn read(path: &Path) -> Vec<u8> {
    std::fs::read(path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()))
}

/// Width and height from a PNG's IHDR chunk.
fn png_size(path: &Path) -> (u32, u32) {
    let b = read(path);
    assert!(
        b.len() > 24 && b.starts_with(b"\x89PNG\r\n\x1a\n") && &b[12..16] == b"IHDR",
        "{} is not a PNG",
        path.display()
    );
    let be = |i: usize| u32::from_be_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]]);
    (be(16), be(20))
}

/// Roster name of the first Act 3 NPC; everyone from here on is Acts 3-8.
const FIRST_ACT3_NPC: &str = "The Guildmaster";

/// Tranche 2 (approved by Matt 2026-11-15): roster name -> ref number and
/// slug of its `ref-NN-<slug>-v2.jsx` redraw.
const TRANCHE2: &[(&str, &str, &str)] = &[
    ("The Recruiter", "31", "recruiter"),
    ("The Locksmith", "32", "locksmith"),
    ("The Porter", "33", "porter"),
    ("The Surveyor", "34", "surveyor"),
    ("The Armorer", "35", "armorer"),
    ("Vexis the Archmage", "36", "vexis"),
    ("The Wandwright", "37", "wandwright"),
    ("The Familiar", "39", "familiar"),
    ("The Lanternkeeper", "40", "lanternkeeper"),
    ("The Loremaster", "41", "loremaster"),
    ("The Bouncer", "43", "bouncer"),
    ("The Interpreter", "44", "interpreter"),
    ("The Mixologist", "45", "mixologist"),
    ("The Tabkeeper", "46", "tabkeeper"),
    ("The Cellarer", "47", "cellarer"),
    ("The Keymaster", "48", "keymaster"),
    ("The Sifter", "49", "sifter"),
    ("The Tallywright", "51", "tallywright"),
    ("The Riveter", "52", "riveter"),
    ("The Bondsmith", "53", "bondsmith"),
    ("The Lighthouse Keeper", "55", "lighthousekeeper"),
    ("The Signaler", "56", "signaler"),
    ("The Tidewatch", "57", "tidewatch"),
    ("The Harbormaster", "58", "harbormaster"),
    ("The Tideforecaster", "59", "tideforecaster"),
    ("The Vaultwright", "60", "vaultwright"),
    ("The Sharekeeper", "61", "sharekeeper"),
    ("The Swapwarden", "63", "swapwarden"),
    ("The Strongbox", "64", "strongbox"),
    ("The Ghostkeeper", "65", "ghostkeeper"),
];

/// The six kit holders: roster name -> kit exemplar ref (`ref-NN-<kit>.jsx`,
/// approved 2026-11-13 as their act's costume kit and confirmed by Matt on
/// 2026-11-15 as these NPCs' sprites), sprite slug, and the first-pass ref
/// it replaces. Mapping from each kit ref's header ("Act 3 Guildhall
/// Quarter (Guildmaster)", ...) and the act casts.
const KIT_HOLDERS: &[(&str, &str, &str, &str, &str)] = &[
    ("The Guildmaster", "69", "guildkit", "guildmaster", "30"),
    ("The Conjurer", "70", "towerkit", "conjurer", "38"),
    ("The Barkeep", "71", "tavernkit", "barkeep", "42"),
    ("The Smelter", "72", "forgekit", "smelter", "50"),
    ("The Dockmaster", "73", "coastkit", "dockmaster", "54"),
    ("The Warden", "74", "vaultkit", "warden", "62"),
];

/// The Act 9 cast (approved by Matt 2026-11-15): roster name -> ref
/// number, ref slug (= sprite slug) and native size.
const ACT9_CAST: &[(&str, &str, &str, u32)] = &[
    ("The Shelfmark", "80", "shelfmark", 32),
    ("The Envoy", "81", "envoy", 32),
    ("The Macro Monk", "82", "macromonk", 32),
    ("The Sexton", "83", "sexton", 32),
    ("The Courier", "84", "courier", 32),
    ("The Bookbinder", "85", "bookbinder", 32),
    ("The Librarian", "93", "librarian", 64),
];

#[test]
fn kit_holders_ship_their_approved_kit_exemplar() {
    let root = game_assets();
    let refs = refs_dir();
    for (name, num, kit, slug, old) in KIT_HOLDERS {
        let spec = NPC_ROSTER
            .iter()
            .find(|s| s.name == *name)
            .unwrap_or_else(|| panic!("{name} not in the roster"));
        let expected_path = format!("sprites/npc/{slug}_idle_0.png");
        assert_eq!(spec.sprite_path, expected_path, "{name}");
        let game = read(&root.join(spec.sprite_path));
        assert!(
            game == read(&refs.join(format!("png/REF{num}.png"))),
            "{name}: {} is not the REF{num}.png render",
            spec.sprite_path
        );
        assert!(
            game != read(&refs.join(format!("png/REF{old}.png"))),
            "{name} still ships its first-pass sprite (REF{old})"
        );
        assert_eq!(png_size(&root.join(spec.sprite_path)), (32, 32), "{name}");

        let src = std::fs::read_to_string(refs.join(format!("ref-{num}-{kit}.jsx")))
            .unwrap_or_else(|e| panic!("ref-{num}-{kit}.jsx: {e}"));
        let wired = format!("Wired as game/assets/{expected_path}");
        assert!(
            src.lines()
                .any(|l| l.starts_with("// APPROVED by Matt 2026-11-13") && l.contains(&wired)),
            "ref-{num}-{kit}.jsx header does not say it is wired as {expected_path}"
        );
        assert!(
            !src.contains("NOT wired"),
            "ref-{num}-{kit}.jsx still says NOT wired"
        );
        let first = std::fs::read_dir(&refs)
            .expect("refs dir")
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .find(|f| f.starts_with(&format!("ref-{old}-")) && f.ends_with(".jsx"))
            .unwrap_or_else(|| panic!("no first-pass ref-{old}"));
        let first_src = std::fs::read_to_string(refs.join(&first)).expect("first-pass ref");
        assert!(
            first_src.contains(&format!("SUPERSEDED by ref-{num}-{kit}.jsx")),
            "{first} is not marked superseded"
        );
    }
}

#[test]
fn act9_cast_ships_its_approved_render() {
    let root = game_assets();
    let refs = refs_dir();
    for (name, num, slug, px) in ACT9_CAST {
        let spec = NPC_ROSTER
            .iter()
            .find(|s| s.name == *name)
            .unwrap_or_else(|| panic!("{name} not in the roster"));
        let expected_path = format!("sprites/npc/{slug}_idle_0.png");
        assert_eq!(spec.sprite_path, expected_path, "{name}");
        assert_eq!(spec.native_px as u32, *px, "{name}");
        assert!(
            read(&root.join(spec.sprite_path)) == read(&refs.join(format!("png/REF{num}.png"))),
            "{name}: {} is not the REF{num}.png render",
            spec.sprite_path
        );
        assert_eq!(png_size(&root.join(spec.sprite_path)), (*px, *px), "{name}");
        let src = std::fs::read_to_string(refs.join(format!("ref-{num}-{slug}.jsx")))
            .unwrap_or_else(|e| panic!("ref-{num}-{slug}.jsx: {e}"));
        let header =
            format!("// APPROVED by Matt 2026-11-15. Wired as game/assets/{expected_path}.");
        assert!(
            src.lines().any(|l| l.trim_end() == header),
            "ref-{num} header"
        );
        assert!(
            !src.contains("DRAFT") && !src.contains("Not wired yet"),
            "ref-{num}"
        );
    }
    // The Library kit exemplar is approved and rendered, but no NPC's sprite.
    let kit = read(&refs.join("png/REF79.png"));
    for s in NPC_ROSTER {
        assert!(
            read(&root.join(s.sprite_path)) != kit,
            "{} ships REF-79",
            s.name
        );
    }
    let kit_src = std::fs::read_to_string(refs.join("ref-79-librarykit.jsx")).expect("REF-79");
    assert!(kit_src.contains("as a kit exemplar. NOT wired to any NPC"));
}

#[test]
fn every_roster_sprite_resolves_to_an_existing_file() {
    let root = game_assets();
    let missing: Vec<String> = NPC_ROSTER
        .iter()
        .filter(|s| !root.join(s.sprite_path).is_file())
        .map(|s| format!("{} -> {}", s.name, s.sprite_path))
        .collect();
    assert!(missing.is_empty(), "missing NPC sprites:\n{missing:#?}");
}

#[test]
fn no_act3_to_8_npc_uses_a_shared_placeholder() {
    let start = NPC_ROSTER
        .iter()
        .position(|s| s.name == FIRST_ACT3_NPC)
        .unwrap_or_else(|| panic!("{FIRST_ACT3_NPC} not in the roster"));
    let root = game_assets();
    let player_png = read(&root.join(SPRITE_PLAYER));
    // Path-level sharing anywhere in the roster.
    let mut by_path: HashMap<&str, &str> = HashMap::new();
    for s in NPC_ROSTER {
        if let Some(other) = by_path.insert(s.sprite_path, s.name) {
            panic!("{} and {other} share sprite path {}", s.name, s.sprite_path);
        }
    }
    // Byte-level sharing: a placeholder copied under several names.
    let mut by_bytes: HashMap<Vec<u8>, &str> = HashMap::new();
    for s in NPC_ROSTER {
        let bytes = read(&root.join(s.sprite_path));
        if let Some(other) = by_bytes.insert(bytes, s.name) {
            panic!("{} and {other} have byte-identical sprites", s.name);
        }
    }
    for s in &NPC_ROSTER[start..] {
        assert_ne!(
            s.sprite_path, SPRITE_PLAYER,
            "{} uses the player sprite",
            s.name
        );
        assert_ne!(
            read(&root.join(s.sprite_path)),
            player_png,
            "{} is a copy of the player sprite",
            s.name
        );
    }
}

#[test]
fn tranche2_villagers_ship_their_approved_v2_render() {
    let root = game_assets();
    let refs = refs_dir();
    assert_eq!(TRANCHE2.len(), 30);
    for (name, num, slug) in TRANCHE2 {
        let spec = NPC_ROSTER
            .iter()
            .find(|s| s.name == *name)
            .unwrap_or_else(|| panic!("{name} not in the roster"));
        let expected_path = format!("sprites/npc/{slug}_idle_0.png");
        assert_eq!(spec.sprite_path, expected_path, "{name}");
        let game = read(&root.join(spec.sprite_path));
        let render = read(&refs.join(format!("png/REF{num}v2.png")));
        assert!(
            game == render,
            "{name}: {} is not the REF{num}v2.png render",
            spec.sprite_path
        );
        let first_pass = read(&refs.join(format!("png/REF{num}.png")));
        assert!(
            game != first_pass,
            "{name} still ships its first-pass sprite"
        );
        assert_eq!(png_size(&root.join(spec.sprite_path)), (32, 32), "{name}");

        let src = std::fs::read_to_string(refs.join(format!("ref-{num}-{slug}-v2.jsx")))
            .unwrap_or_else(|e| panic!("ref-{num}-{slug}-v2.jsx: {e}"));
        let header =
            format!("// APPROVED by Matt 2026-11-15. Wired as game/assets/{expected_path}.");
        assert!(
            src.lines().any(|l| l.trim_end() == header),
            "ref-{num} header"
        );
        assert!(
            !src.contains("DRAFT"),
            "ref-{num}-{slug}-v2.jsx still says DRAFT"
        );
        let old = std::fs::read_to_string(refs.join(format!("ref-{num}-{slug}.jsx")))
            .unwrap_or_else(|e| panic!("ref-{num}-{slug}.jsx: {e}"));
        assert!(
            old.contains(&format!("SUPERSEDED by ref-{num}-{slug}-v2.jsx")),
            "ref-{num}-{slug}.jsx is not marked superseded"
        );
    }
}

/// Tile names of a ref tile sheet, in declaration order (= atlas index).
fn tile_names(src: &str, num: &str) -> Vec<String> {
    let open = format!("const REF{num}_TILES = {{");
    let body = src
        .split_once(open.as_str())
        .map(|(_, rest)| rest)
        .unwrap_or_else(|| panic!("REF{num}_TILES not found"));
    body.lines()
        .map(str::trim)
        .take_while(|l| !l.starts_with("};"))
        .filter_map(|l| l.strip_suffix(": ["))
        .map(str::to_owned)
        .collect()
}

fn cols(src: &str, num: &str) -> u32 {
    let key = format!("const REF{num}_COLS = ");
    src.lines()
        .find_map(|l| l.strip_prefix(key.as_str()))
        .and_then(|v| v.trim_end_matches(';').trim().parse().ok())
        .unwrap_or_else(|| panic!("REF{num}_COLS not found"))
}

#[test]
fn walk_sheets_match_their_refs_and_the_atlas_layout() {
    let root = game_assets();
    let refs = refs_dir();
    let cases = [
        (
            "77",
            "ref-77-playerwalk.jsx",
            SPRITE_PLAYER_WALK,
            PLAYER_SHEET_TILES,
            PLAYER_SHEET_COLS,
            PLAYER_SHEET_ROWS,
        ),
        (
            "78",
            "ref-78-ferriswalk.jsx",
            SPRITE_FERRIS_WALK,
            FERRIS_SHEET_TILES,
            FERRIS_SHEET_COLS,
            FERRIS_SHEET_ROWS,
        ),
    ];
    for (num, file, asset, tiles, c, r) in cases {
        let src =
            std::fs::read_to_string(refs.join(file)).unwrap_or_else(|e| panic!("{file}: {e}"));
        assert!(!src.contains("DRAFT"), "{file} still says DRAFT");
        assert_eq!(tile_names(&src, num), tiles, "{file} tile order");
        assert_eq!(cols(&src, num), c, "{file} columns");
        assert_eq!((tiles.len() as u32).div_ceil(c), r, "{file} rows");
        assert_eq!(
            png_size(&root.join(asset)),
            (c * FRAME_PX, r * FRAME_PX),
            "{asset} size"
        );
        assert_eq!(
            read(&root.join(asset)),
            read(&refs.join(format!("png/REF{num}.png"))),
            "{asset} is not the REF{num}.png render"
        );
    }
    // The idle sprites that complete the cycles are single 32 px frames.
    for idle in [SPRITE_PLAYER, SPRITE_FERRIS] {
        assert_eq!(png_size(&root.join(idle)), (FRAME_PX, FRAME_PX), "{idle}");
    }
}

#[test]
fn npc_sprite_sizes_match_their_native_px() {
    let root = game_assets();
    for s in NPC_ROSTER {
        let px = s.native_px as u32;
        assert_eq!(png_size(&root.join(s.sprite_path)), (px, px), "{}", s.name);
    }
    assert_eq!(png_size(&root.join(SPRITE_BORROW_CHECKER)), (64, 64));
}

/// The Act 10 cast (approved by Matt 2026-11-15): roster name -> ref
/// number, ref slug (= sprite slug) and native size.
const ACT10_CAST: &[(&str, &str, &str, u32)] = &[
    ("The Assayer", "87", "assayer", 32),
    ("The Copyist", "88", "copyist", 32),
    ("The Astronomer", "89", "astronomer", 32),
    ("The Choirmaster", "90", "choirmaster", 32),
    ("The Hermit", "91", "hermit", 32),
    ("The Registrar", "92", "registrar", 32),
    ("The Compiler", "94", "compiler", 64),
];

/// The Act 10 cast ships its approved REF-87..92 and REF-94 renders, and
/// each ref header says where it is wired (the approval line and the
/// designer's notes kept as written). The Throne kit exemplar REF-86 is
/// no NPC's sprite.
#[test]
fn act10_cast_ships_its_approved_render() {
    let root = game_assets();
    let refs = refs_dir();
    for (name, num, slug, px) in ACT10_CAST {
        let spec = NPC_ROSTER
            .iter()
            .find(|s| s.name == *name)
            .unwrap_or_else(|| panic!("{name} not in the roster"));
        let expected_path = format!("sprites/npc/{slug}_idle_0.png");
        assert_eq!(spec.sprite_path, expected_path, "{name}");
        assert_eq!(spec.native_px as u32, *px, "{name}");
        assert!(
            read(&root.join(spec.sprite_path)) == read(&refs.join(format!("png/REF{num}.png"))),
            "{name}: {} is not the REF{num}.png render",
            spec.sprite_path
        );
        assert_eq!(png_size(&root.join(spec.sprite_path)), (*px, *px), "{name}");
        let src = std::fs::read_to_string(refs.join(format!("ref-{num}-{slug}.jsx")))
            .unwrap_or_else(|e| panic!("ref-{num}-{slug}.jsx: {e}"));
        assert!(
            src.lines()
                .any(|l| l.starts_with("// APPROVED by Matt 2026-11-15")),
            "ref-{num}: approval line"
        );
        let wired = format!("// Wired as game/assets/{expected_path}.");
        assert!(
            src.lines().any(|l| l.trim_end() == wired),
            "ref-{num} header does not say {wired}"
        );
        assert!(
            !src.contains("DRAFT") && !src.contains("Not wired yet"),
            "ref-{num}"
        );
    }
    let kit = read(&refs.join("png/REF86.png"));
    for s in NPC_ROSTER {
        assert!(
            read(&root.join(s.sprite_path)) != kit,
            "{} ships REF-86",
            s.name
        );
    }
    let kit_src = std::fs::read_to_string(refs.join("ref-86-thronekit.jsx")).expect("REF-86");
    assert!(kit_src.contains("not wired to any NPC"));
}

/// The two district landmarks ship their approved renders as props: the
/// Throne Dais (REF-96, approved 2026-11-15) and the Sealed Stacks (REF-95,
/// approved 2026-11-16). Each PNG is byte-identical to its render, 64x64,
/// and its ref header says where it is wired.
#[test]
fn the_landmarks_ship_their_approved_renders() {
    use pledge_and_crown::assets::{
        ALL_SPRITE_PATHS, SPRITE_PROP_SEALED_STACKS, SPRITE_PROP_THRONE_DAIS,
    };
    let root = game_assets();
    let refs = refs_dir();
    for (path, want, render, jsx, approved) in [
        (
            SPRITE_PROP_THRONE_DAIS,
            "sprites/props/throne_dais.png",
            "png/REF96.png",
            "ref-96-thronedais.jsx",
            "2026-11-15",
        ),
        (
            SPRITE_PROP_SEALED_STACKS,
            "sprites/props/sealed_stacks.png",
            "png/REF95.png",
            "ref-95-sealedstacks.jsx",
            "2026-11-16",
        ),
    ] {
        assert_eq!(path, want);
        assert!(ALL_SPRITE_PATHS.contains(&path), "{path} is not preloaded");
        assert!(
            read(&root.join(path)) == read(&refs.join(render)),
            "{path} is not the {render} render"
        );
        assert_eq!(png_size(&root.join(path)), (64, 64), "{path}");
        let src = std::fs::read_to_string(refs.join(jsx)).expect(jsx);
        let header = format!("// APPROVED by Matt {approved}. Wired as game/assets/{want}.");
        assert!(
            src.lines().any(|l| l.trim_end() == header),
            "{jsx}: no approval header"
        );
        assert!(!src.contains("DRAFT"), "{jsx} still says DRAFT");
    }
}

/// The Forge district set (approved by Matt 2026-11-18): the Great Furnace
/// (REF-98, 64x64) and the four yard props (REF-99..102, 32x32) ship
/// byte-identical to their renders, preloaded, with approval headers; the
/// REF-97 slab ships inside the terrain atlas, which is the REF-76 render
/// (`world.rs` pins that and the tile indices).
#[test]
fn the_forge_set_ships_its_approved_renders() {
    use pledge_and_crown::assets::{
        ALL_SPRITE_PATHS, SPRITE_PROP_ANVIL, SPRITE_PROP_COAL_HEAP, SPRITE_PROP_GREAT_FURNACE,
        SPRITE_PROP_QUENCH_TROUGH, SPRITE_PROP_TOOL_RACK, SPRITE_TILES_VILLAGE_V3,
    };
    let root = game_assets();
    let refs = refs_dir();
    for (path, want, render, jsx, px) in [
        (
            SPRITE_PROP_GREAT_FURNACE,
            "sprites/props/great_furnace.png",
            "png/REF98.png",
            "ref-98-greatfurnace.jsx",
            64,
        ),
        (
            SPRITE_PROP_ANVIL,
            "sprites/props/anvil.png",
            "png/REF99.png",
            "ref-99-anvil.jsx",
            32,
        ),
        (
            SPRITE_PROP_QUENCH_TROUGH,
            "sprites/props/quench_trough.png",
            "png/REF100.png",
            "ref-100-quenchtrough.jsx",
            32,
        ),
        (
            SPRITE_PROP_COAL_HEAP,
            "sprites/props/coal_heap.png",
            "png/REF101.png",
            "ref-101-coalheap.jsx",
            32,
        ),
        (
            SPRITE_PROP_TOOL_RACK,
            "sprites/props/tool_rack.png",
            "png/REF102.png",
            "ref-102-toolrack.jsx",
            32,
        ),
    ] {
        assert_eq!(path, want);
        assert!(ALL_SPRITE_PATHS.contains(&path), "{path} is not preloaded");
        assert!(
            read(&root.join(path)) == read(&refs.join(render)),
            "{path} is not the {render} render"
        );
        assert_eq!(png_size(&root.join(path)), (px, px), "{path}");
        let src = std::fs::read_to_string(refs.join(jsx)).expect(jsx);
        let header = format!("// APPROVED by Matt 2026-11-18. Wired as game/assets/{want}.");
        assert!(
            src.lines().any(|l| l.trim_end() == header),
            "{jsx}: no approval header"
        );
        assert!(!src.contains("DRAFT"), "{jsx} still says DRAFT");
    }
    // REF-97 reaches the game through the combined atlas (5 rows of 8).
    let atlas = root.join(SPRITE_TILES_VILLAGE_V3);
    assert!(read(&atlas) == read(&refs.join("png/REF76.png")));
    assert_eq!(png_size(&atlas), (128, 80));
    let src = std::fs::read_to_string(refs.join("ref-97-forgeslab.jsx")).expect("REF-97");
    assert!(
        src.lines().any(|l| l.trim_end()
            == "// APPROVED by Matt 2026-11-18. Wired as game/assets/sprites/tiles_village_v3.png (via the REF-76 combined atlas, indices 24-25)."),
        "ref-97-forgeslab.jsx: no approval header"
    );
    assert!(!src.contains("DRAFT"));
}

/// The free demo's plaza set (approved by Matt 2026-11-18): three 64x64
/// landmarks (REF-103 Bell Tower, REF-107 market stall, REF-111 Checker's
/// Gate) and ten 32x32 props ship byte-identical to their renders,
/// preloaded, with approval headers and no DRAFT left.
#[test]
fn the_demo_plaza_set_ships_its_approved_renders() {
    use pledge_and_crown::assets::{
        ALL_SPRITE_PATHS, SPRITE_PROP_BARRELS, SPRITE_PROP_BELL_TOWER, SPRITE_PROP_BENCH,
        SPRITE_PROP_CHECKERS_GATE, SPRITE_PROP_CHICKEN_COOP, SPRITE_PROP_CRATES, SPRITE_PROP_FENCE,
        SPRITE_PROP_HANDCART, SPRITE_PROP_LANTERN_POST, SPRITE_PROP_MARKET_STALL,
        SPRITE_PROP_SIGNPOST, SPRITE_PROP_WELL, SPRITE_PROP_WOODPILE,
    };
    let root = game_assets();
    let refs = refs_dir();
    for (path, file, n, jsx, px) in [
        (
            SPRITE_PROP_BELL_TOWER,
            "bell_tower",
            103,
            "ref-103-belltower.jsx",
            64,
        ),
        (SPRITE_PROP_WELL, "well", 104, "ref-104-well.jsx", 32),
        (
            SPRITE_PROP_WOODPILE,
            "woodpile",
            105,
            "ref-105-woodpile.jsx",
            32,
        ),
        (
            SPRITE_PROP_SIGNPOST,
            "signpost",
            106,
            "ref-106-signpost.jsx",
            32,
        ),
        (
            SPRITE_PROP_MARKET_STALL,
            "market_stall",
            107,
            "ref-107-marketstall.jsx",
            64,
        ),
        (SPRITE_PROP_CRATES, "crates", 108, "ref-108-crates.jsx", 32),
        (
            SPRITE_PROP_BARRELS,
            "barrels",
            109,
            "ref-109-barrels.jsx",
            32,
        ),
        (
            SPRITE_PROP_HANDCART,
            "handcart",
            110,
            "ref-110-handcart.jsx",
            32,
        ),
        (
            SPRITE_PROP_CHECKERS_GATE,
            "checkers_gate",
            111,
            "ref-111-checkersgate.jsx",
            64,
        ),
        (
            SPRITE_PROP_LANTERN_POST,
            "lantern_post",
            112,
            "ref-112-lanternpost.jsx",
            32,
        ),
        (SPRITE_PROP_BENCH, "bench", 113, "ref-113-bench.jsx", 32),
        (SPRITE_PROP_FENCE, "fence", 114, "ref-114-fence.jsx", 32),
        (
            SPRITE_PROP_CHICKEN_COOP,
            "chicken_coop",
            115,
            "ref-115-chickencoop.jsx",
            32,
        ),
    ] {
        let want = format!("sprites/props/{file}.png");
        let render = format!("png/REF{n}.png");
        assert_eq!(path, want);
        assert!(ALL_SPRITE_PATHS.contains(&path), "{path} is not preloaded");
        assert!(
            read(&root.join(path)) == read(&refs.join(&render)),
            "{path} is not the {render} render"
        );
        assert_eq!(png_size(&root.join(path)), (px, px), "{path}");
        let src = std::fs::read_to_string(refs.join(jsx)).expect(jsx);
        let header = format!("// APPROVED by Matt 2026-11-18. Wired as game/assets/{want}.");
        assert!(
            src.lines().any(|l| l.trim_end() == header),
            "{jsx}: no approval header"
        );
        assert!(!src.contains("DRAFT"), "{jsx} still says DRAFT");
        assert!(!src.contains("Not wired yet"), "{jsx} still says not wired");
    }
}

/// The density pass set (approved by Matt 2026-11-18): the REF-116..121
/// Forge yard props, the REF-128 strongbox and the REF-129 planter ship
/// byte-identical to their renders, preloaded, 32x32, with approval
/// headers naming their game file and no DRAFT left.
#[test]
fn the_density_pass_set_ships_its_approved_renders() {
    use pledge_and_crown::assets::{
        ALL_SPRITE_PATHS, SPRITE_PROP_BELLOWS, SPRITE_PROP_BUCKET_SHOVEL, SPRITE_PROP_GRINDSTONE,
        SPRITE_PROP_INGOTS, SPRITE_PROP_PLANTER, SPRITE_PROP_STRONGBOX, SPRITE_PROP_WEAPON_RACK,
        SPRITE_PROP_WORKBENCH,
    };
    let root = game_assets();
    let refs = refs_dir();
    for (path, file, n, jsx) in [
        (
            SPRITE_PROP_GRINDSTONE,
            "grindstone",
            116,
            "ref-116-grindstone.jsx",
        ),
        (SPRITE_PROP_INGOTS, "ingots", 117, "ref-117-ingots.jsx"),
        (SPRITE_PROP_BELLOWS, "bellows", 118, "ref-118-bellows.jsx"),
        (
            SPRITE_PROP_WEAPON_RACK,
            "weapon_rack",
            119,
            "ref-119-weaponrack.jsx",
        ),
        (
            SPRITE_PROP_WORKBENCH,
            "workbench",
            120,
            "ref-120-workbench.jsx",
        ),
        (
            SPRITE_PROP_BUCKET_SHOVEL,
            "bucket_shovel",
            121,
            "ref-121-bucketshovel.jsx",
        ),
        (
            SPRITE_PROP_STRONGBOX,
            "strongbox",
            128,
            "ref-128-strongbox.jsx",
        ),
        (SPRITE_PROP_PLANTER, "planter", 129, "ref-129-planter.jsx"),
    ] {
        let want = format!("sprites/props/{file}.png");
        let render = format!("png/REF{n}.png");
        assert_eq!(path, want);
        assert!(ALL_SPRITE_PATHS.contains(&path), "{path} is not preloaded");
        assert!(
            read(&root.join(path)) == read(&refs.join(&render)),
            "{path} is not the {render} render"
        );
        assert_eq!(png_size(&root.join(path)), (32, 32), "{path}");
        let src = std::fs::read_to_string(refs.join(jsx)).expect(jsx);
        let header = format!("// APPROVED by Matt 2026-11-18. Wired as game/assets/{want}.");
        assert!(
            src.lines().any(|l| l.trim_end() == header),
            "{jsx}: no approval header"
        );
        assert!(!src.contains("DRAFT"), "{jsx} still says DRAFT");
        assert!(!src.contains("Not wired yet"), "{jsx} still says not wired");
    }
}

/// The Acts 3-5 district sets (approved by Matt 2026-11-18): three 64x64
/// landmarks (REF-123 Guildhall, REF-131 Mage's Tower, REF-139 tavern) and
/// sixteen 32x32 props ship byte-identical to their renders, preloaded,
/// with approval headers naming their game file and no DRAFT left. The
/// three grounds (REF-122, REF-130, REF-138) ship inside the terrain
/// atlas, which is the REF-76 render (`world.rs` pins the indices).
#[test]
fn the_act3_to_5_sets_ship_their_approved_renders() {
    use pledge_and_crown::assets::{
        ALL_SPRITE_PATHS, SPRITE_PROP_CAULDRON, SPRITE_PROP_CRYSTAL, SPRITE_PROP_FIREPIT,
        SPRITE_PROP_GUILD_BANNER, SPRITE_PROP_GUILDHALL, SPRITE_PROP_KEG_RACK, SPRITE_PROP_LECTERN,
        SPRITE_PROP_LEDGER_DESK, SPRITE_PROP_MAGES_TOWER, SPRITE_PROP_NOTICE_BOARD,
        SPRITE_PROP_ORRERY, SPRITE_PROP_ROUND_TABLE, SPRITE_PROP_RUNE_STONE,
        SPRITE_PROP_SCROLL_SHELF, SPRITE_PROP_TAB_SLATE, SPRITE_PROP_TAVERN,
        SPRITE_PROP_TAVERN_SIGN, SPRITE_PROP_TELESCOPE, SPRITE_PROP_TRESTLE_TABLE,
        SPRITE_TILES_VILLAGE_V3,
    };
    let root = game_assets();
    let refs = refs_dir();
    for (path, file, n, jsx, px) in [
        (
            SPRITE_PROP_GUILDHALL,
            "guildhall",
            123,
            "ref-123-guildhall.jsx",
            64,
        ),
        (
            SPRITE_PROP_LEDGER_DESK,
            "ledger_desk",
            124,
            "ref-124-ledgerdesk.jsx",
            32,
        ),
        (
            SPRITE_PROP_NOTICE_BOARD,
            "notice_board",
            125,
            "ref-125-noticeboard.jsx",
            32,
        ),
        (
            SPRITE_PROP_GUILD_BANNER,
            "guild_banner",
            126,
            "ref-126-guildbanner.jsx",
            32,
        ),
        (
            SPRITE_PROP_SCROLL_SHELF,
            "scroll_shelf",
            127,
            "ref-127-scrollshelf.jsx",
            32,
        ),
        (
            SPRITE_PROP_MAGES_TOWER,
            "mages_tower",
            131,
            "ref-131-magestower.jsx",
            64,
        ),
        (SPRITE_PROP_ORRERY, "orrery", 132, "ref-132-orrery.jsx", 32),
        (
            SPRITE_PROP_RUNE_STONE,
            "rune_stone",
            133,
            "ref-133-runestone.jsx",
            32,
        ),
        (
            SPRITE_PROP_LECTERN,
            "lectern",
            134,
            "ref-134-lectern.jsx",
            32,
        ),
        (
            SPRITE_PROP_CRYSTAL,
            "crystal",
            135,
            "ref-135-crystal.jsx",
            32,
        ),
        (
            SPRITE_PROP_CAULDRON,
            "cauldron",
            136,
            "ref-136-cauldron.jsx",
            32,
        ),
        (
            SPRITE_PROP_TELESCOPE,
            "telescope",
            137,
            "ref-137-telescope.jsx",
            32,
        ),
        (SPRITE_PROP_TAVERN, "tavern", 139, "ref-139-tavern.jsx", 64),
        (
            SPRITE_PROP_TRESTLE_TABLE,
            "trestle_table",
            140,
            "ref-140-trestletable.jsx",
            32,
        ),
        (
            SPRITE_PROP_KEG_RACK,
            "keg_rack",
            141,
            "ref-141-kegrack.jsx",
            32,
        ),
        (
            SPRITE_PROP_FIREPIT,
            "firepit",
            142,
            "ref-142-firepit.jsx",
            32,
        ),
        (
            SPRITE_PROP_TAVERN_SIGN,
            "tavern_sign",
            143,
            "ref-143-tavernsign.jsx",
            32,
        ),
        (
            SPRITE_PROP_ROUND_TABLE,
            "round_table",
            144,
            "ref-144-roundtable.jsx",
            32,
        ),
        (
            SPRITE_PROP_TAB_SLATE,
            "tab_slate",
            145,
            "ref-145-tabslate.jsx",
            32,
        ),
    ] {
        let want = format!("sprites/props/{file}.png");
        let render = format!("png/REF{n}.png");
        assert_eq!(path, want);
        assert!(ALL_SPRITE_PATHS.contains(&path), "{path} is not preloaded");
        assert!(
            read(&root.join(path)) == read(&refs.join(&render)),
            "{path} is not the {render} render"
        );
        assert_eq!(png_size(&root.join(path)), (px, px), "{path}");
        let src = std::fs::read_to_string(refs.join(jsx)).expect(jsx);
        let header = format!("// APPROVED by Matt 2026-11-18. Wired as game/assets/{want}.");
        assert!(
            src.lines().any(|l| l.trim_end() == header),
            "{jsx}: no approval header"
        );
        assert!(!src.contains("DRAFT"), "{jsx} still says DRAFT");
        assert!(!src.contains("Not wired yet"), "{jsx} still says not wired");
    }
    // The grounds reach the game through the combined atlas (5 rows).
    let atlas = root.join(SPRITE_TILES_VILLAGE_V3);
    assert!(read(&atlas) == read(&refs.join("png/REF76.png")));
    assert_eq!(png_size(&atlas), (128, 80));
    for (jsx, indices) in [
        ("ref-122-hallfloor.jsx", "26-27"),
        ("ref-130-courtfloor.jsx", "28-29"),
        ("ref-138-tavernplanks.jsx", "30-31"),
    ] {
        let src = std::fs::read_to_string(refs.join(jsx)).expect(jsx);
        let header = format!(
            "// APPROVED by Matt 2026-11-18. Wired as game/assets/sprites/tiles_village_v3.png (via the REF-76 combined atlas, indices {indices})."
        );
        assert!(
            src.lines().any(|l| l.trim_end() == header),
            "{jsx}: no approval header"
        );
        assert!(!src.contains("Not wired yet"), "{jsx} still says not wired");
    }
}

/// Checks one wired 2D asset: the path is `sprites/<dir>/<file>.png`,
/// preloaded, byte-identical to its `REF<n>.png` render and `px` square,
/// and its ref header says it is approved on `date` and wired there, with
/// no DRAFT or "Not wired yet" left.
fn assert_wired(path: &str, dir: &str, file: &str, n: u32, jsx: &str, px: u32, date: &str) {
    use pledge_and_crown::assets::ALL_SPRITE_PATHS;
    let root = game_assets();
    let refs = refs_dir();
    let want = format!("sprites/{dir}/{file}.png");
    let render = format!("png/REF{n}.png");
    assert_eq!(path, want);
    assert!(ALL_SPRITE_PATHS.contains(&path), "{path} is not preloaded");
    assert!(
        read(&root.join(path)) == read(&refs.join(&render)),
        "{path} is not the {render} render"
    );
    assert_eq!(png_size(&root.join(path)), (px, px), "{path}");
    let src = std::fs::read_to_string(refs.join(jsx)).expect(jsx);
    let header = format!("// APPROVED by Matt {date}. Wired as game/assets/{want}.");
    assert!(
        src.lines().any(|l| l.trim_end() == header),
        "{jsx}: no approval header"
    );
    assert!(!src.contains("DRAFT"), "{jsx} still says DRAFT");
    assert!(!src.contains("Not wired yet"), "{jsx} still says not wired");
}

/// The Acts 8-10 district sets (approved by Matt 2026-11-18): the vault
/// door (REF-155, 64x64) and sixteen 32x32 props (REF-156..161,
/// REF-163..172) ship byte-identical to their renders, preloaded, with
/// approval headers naming their game file. The vault floor (REF-154) and
/// the carpet runner (REF-162) ship inside the terrain atlas, the REF-76
/// render (`world.rs` pins the indices).
#[test]
fn the_act8_to_10_sets_ship_their_approved_renders() {
    use pledge_and_crown::assets::{
        SPRITE_PROP_BOOK_PILE, SPRITE_PROP_BRAZIER, SPRITE_PROP_CANDELABRA,
        SPRITE_PROP_CHAIN_BARRIER, SPRITE_PROP_CHAINED_CASKET, SPRITE_PROP_COIN_PILE,
        SPRITE_PROP_COLUMN, SPRITE_PROP_CROWN_PLINTH, SPRITE_PROP_GLOBE, SPRITE_PROP_GUARD_STATUE,
        SPRITE_PROP_IRON_SAFE, SPRITE_PROP_KEY_RACK, SPRITE_PROP_READING_DESK,
        SPRITE_PROP_ROYAL_BANNER, SPRITE_PROP_SCALES, SPRITE_PROP_TALL_BOOKSHELF,
        SPRITE_PROP_VAULT_DOOR, SPRITE_TILES_VILLAGE_V3,
    };
    for (path, file, n, jsx, px) in [
        (
            SPRITE_PROP_VAULT_DOOR,
            "vault_door",
            155,
            "ref-155-vaultdoor.jsx",
            64,
        ),
        (
            SPRITE_PROP_COIN_PILE,
            "coin_pile",
            156,
            "ref-156-coinpile.jsx",
            32,
        ),
        (
            SPRITE_PROP_CHAINED_CASKET,
            "chained_casket",
            157,
            "ref-157-chainedcasket.jsx",
            32,
        ),
        (
            SPRITE_PROP_CHAIN_BARRIER,
            "chain_barrier",
            158,
            "ref-158-chainbarrier.jsx",
            32,
        ),
        (SPRITE_PROP_SCALES, "scales", 159, "ref-159-scales.jsx", 32),
        (
            SPRITE_PROP_KEY_RACK,
            "key_rack",
            160,
            "ref-160-keyrack.jsx",
            32,
        ),
        (
            SPRITE_PROP_IRON_SAFE,
            "iron_safe",
            161,
            "ref-161-ironsafe.jsx",
            32,
        ),
        (
            SPRITE_PROP_TALL_BOOKSHELF,
            "tall_bookshelf",
            163,
            "ref-163-tallbookshelf.jsx",
            32,
        ),
        (
            SPRITE_PROP_READING_DESK,
            "reading_desk",
            164,
            "ref-164-readingdesk.jsx",
            32,
        ),
        (
            SPRITE_PROP_BOOK_PILE,
            "book_pile",
            165,
            "ref-165-bookpile.jsx",
            32,
        ),
        (SPRITE_PROP_GLOBE, "globe", 166, "ref-166-globe.jsx", 32),
        (
            SPRITE_PROP_CANDELABRA,
            "candelabra",
            167,
            "ref-167-candelabra.jsx",
            32,
        ),
        (SPRITE_PROP_COLUMN, "column", 168, "ref-168-column.jsx", 32),
        (
            SPRITE_PROP_BRAZIER,
            "brazier",
            169,
            "ref-169-brazier.jsx",
            32,
        ),
        (
            SPRITE_PROP_ROYAL_BANNER,
            "royal_banner",
            170,
            "ref-170-royalbanner.jsx",
            32,
        ),
        (
            SPRITE_PROP_GUARD_STATUE,
            "guard_statue",
            171,
            "ref-171-guardstatue.jsx",
            32,
        ),
        (
            SPRITE_PROP_CROWN_PLINTH,
            "crown_plinth",
            172,
            "ref-172-crownplinth.jsx",
            32,
        ),
    ] {
        assert_wired(path, "props", file, n, jsx, px, "2026-11-18");
    }
    let root = game_assets();
    let refs = refs_dir();
    let atlas = root.join(SPRITE_TILES_VILLAGE_V3);
    assert!(read(&atlas) == read(&refs.join("png/REF76.png")));
    assert_eq!(png_size(&atlas), (128, 80));
    for (jsx, indices) in [
        ("ref-154-vaultfloor.jsx", "32-33"),
        ("ref-162-runner.jsx", "34-35"),
    ] {
        let src = std::fs::read_to_string(refs.join(jsx)).expect(jsx);
        let header = format!(
            "// APPROVED by Matt 2026-11-18. Wired as game/assets/sprites/tiles_village_v3.png (via the REF-76 combined atlas, indices {indices})."
        );
        assert!(
            src.lines().any(|l| l.trim_end() == header),
            "{jsx}: no approval header"
        );
        assert!(!src.contains("Not wired yet"), "{jsx} still says not wired");
    }
}

/// The Wharf set (approved by Matt 2026-11-18): the harbour crane
/// (REF-147, 64x64) and six 32x32 props (REF-148..153) ship
/// byte-identical to their renders, preloaded, with approval headers
/// naming their game file. The dock planks and the two water frames
/// (REF-146) ship inside the terrain atlas, the REF-76 render, at indices
/// 36-39 (`world.rs` pins the indices).
#[test]
fn the_wharf_set_ships_its_approved_renders() {
    use pledge_and_crown::assets::{
        SPRITE_PROP_ANCHOR, SPRITE_PROP_BOLLARD, SPRITE_PROP_FISH_CRATES,
        SPRITE_PROP_HARBOUR_CRANE, SPRITE_PROP_NET_RACK, SPRITE_PROP_ROWBOAT,
        SPRITE_PROP_SIGNAL_MAST, SPRITE_TILES_VILLAGE_V3,
    };
    for (path, file, n, jsx, px) in [
        (
            SPRITE_PROP_HARBOUR_CRANE,
            "harbour_crane",
            147,
            "ref-147-harbourcrane.jsx",
            64,
        ),
        (
            SPRITE_PROP_BOLLARD,
            "bollard",
            148,
            "ref-148-bollard.jsx",
            32,
        ),
        (
            SPRITE_PROP_NET_RACK,
            "net_rack",
            149,
            "ref-149-netrack.jsx",
            32,
        ),
        (
            SPRITE_PROP_ROWBOAT,
            "rowboat",
            150,
            "ref-150-rowboat.jsx",
            32,
        ),
        (
            SPRITE_PROP_FISH_CRATES,
            "fish_crates",
            151,
            "ref-151-fishcrates.jsx",
            32,
        ),
        (SPRITE_PROP_ANCHOR, "anchor", 152, "ref-152-anchor.jsx", 32),
        (
            SPRITE_PROP_SIGNAL_MAST,
            "signal_mast",
            153,
            "ref-153-signalmast.jsx",
            32,
        ),
    ] {
        assert_wired(path, "props", file, n, jsx, px, "2026-11-18");
    }
    let root = game_assets();
    let refs = refs_dir();
    let atlas = root.join(SPRITE_TILES_VILLAGE_V3);
    assert!(read(&atlas) == read(&refs.join("png/REF76.png")));
    assert_eq!(png_size(&atlas), (128, 80));
    // The REF-146 sheet itself is rendered and committed (4 tiles across).
    assert_eq!(png_size(&refs.join("png/REF146.png")), (64, 16));
    let src = std::fs::read_to_string(refs.join("ref-146-dock.jsx")).expect("REF-146");
    let header = "// APPROVED by Matt 2026-11-18. Wired as game/assets/sprites/tiles_village_v3.png (via the REF-76 combined atlas, indices 36-39).";
    assert!(
        src.lines().any(|l| l.trim_end() == header),
        "ref-146-dock.jsx: no approval header"
    );
    assert!(
        !src.contains("Not wired yet"),
        "REF-146 still says not wired"
    );
    // No ref anywhere is left approved but unwired: every district is
    // dressed.
    for entry in std::fs::read_dir(&refs).expect("refs dir") {
        let path = entry.expect("entry").path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if !name.starts_with("ref-") {
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("ref");
        assert!(!text.contains("Not wired yet"), "{name} is still not wired");
    }
}

/// The ambient townsfolk (REF-173..179, approved by Matt 2026-11-19) ship
/// byte-identical to their 32x32 renders, preloaded, with approval headers
/// naming their game file; each `townsfolk::Folk` kind loads its own.
#[test]
fn the_townsfolk_ship_their_approved_renders() {
    use pledge_and_crown::plugins::townsfolk::Folk;
    for (folk, file, n, jsx) in [
        (Folk::Farmer, "farmer_idle_0", 173, "ref-173-farmer.jsx"),
        (
            Folk::Goodwife,
            "goodwife_idle_0",
            174,
            "ref-174-goodwife.jsx",
        ),
        (Folk::Elder, "elder_idle_0", 175, "ref-175-elder.jsx"),
        (Folk::Boy, "boy_idle_0", 176, "ref-176-child.jsx"),
        (
            Folk::Merchant,
            "merchant_idle_0",
            177,
            "ref-177-merchant.jsx",
        ),
        (Folk::Guard, "guard_idle_0", 178, "ref-178-guard.jsx"),
        (Folk::Girl, "girl_idle_0", 179, "ref-179-girl.jsx"),
    ] {
        assert_wired(
            folk.sprite_path(),
            "townsfolk",
            file,
            n,
            jsx,
            32,
            "2026-11-19",
        );
    }
}

/// The boss intro and defeat frames (REF-180..185, approved by Matt
/// 2026-11-19) ship byte-identical to their 64x64 renders, preloaded, with
/// approval headers naming their game file, and are exactly the frames
/// `boss_frames::BOSS_FRAMES` swaps in for the three bosses, whose idle
/// sprites are their own renders (REF-03, REF-93, REF-94).
#[test]
fn the_boss_frames_ship_their_approved_renders() {
    use pledge_and_crown::assets::{
        SPRITE_BORROW_CHECKER_DEFEAT, SPRITE_BORROW_CHECKER_INTRO, SPRITE_COMPILER_DEFEAT,
        SPRITE_COMPILER_INTRO, SPRITE_LIBRARIAN_DEFEAT, SPRITE_LIBRARIAN_INTRO,
    };
    use pledge_and_crown::plugins::boss_frames::BOSS_FRAMES;
    for (path, file, n, jsx) in [
        (
            SPRITE_BORROW_CHECKER_INTRO,
            "borrow_checker_intro_0",
            180,
            "ref-180-checker-intro.jsx",
        ),
        (
            SPRITE_BORROW_CHECKER_DEFEAT,
            "borrow_checker_defeat_0",
            181,
            "ref-181-checker-defeat.jsx",
        ),
        (
            SPRITE_LIBRARIAN_INTRO,
            "librarian_intro_0",
            182,
            "ref-182-librarian-intro.jsx",
        ),
        (
            SPRITE_LIBRARIAN_DEFEAT,
            "librarian_defeat_0",
            183,
            "ref-183-librarian-defeat.jsx",
        ),
        (
            SPRITE_COMPILER_INTRO,
            "compiler_intro_0",
            184,
            "ref-184-compiler-intro.jsx",
        ),
        (
            SPRITE_COMPILER_DEFEAT,
            "compiler_defeat_0",
            185,
            "ref-185-compiler-defeat.jsx",
        ),
    ] {
        assert_wired(path, "npc", file, n, jsx, 64, "2026-11-19");
    }
    let root = game_assets();
    let refs = refs_dir();
    let renders = [
        ("03", "180", "181"),
        ("93", "182", "183"),
        ("94", "184", "185"),
    ];
    assert_eq!(BOSS_FRAMES.len(), renders.len());
    for (f, (idle, intro, defeat)) in BOSS_FRAMES.iter().zip(renders) {
        for (path, r) in [(f.idle, idle), (f.intro, intro), (f.defeat, defeat)] {
            assert!(
                read(&root.join(path)) == read(&refs.join(format!("png/REF{r}.png"))),
                "{}: {path} is not the REF{r}.png render",
                f.npc
            );
        }
    }
}

// SPDX-License-Identifier: MIT
//! Audit: approved art is wired where the game loads it.
//!
//! - every roster NPC's sprite exists, is its own file, and no Act 3-8 NPC
//!   still uses a shared placeholder (the player sprite or a byte copy of
//!   another NPC);
//! - the 30 tranche-2 villagers (approved 2026-11-15) ship exactly their
//!   `REFNNv2.png` render, and their ref headers say so;
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

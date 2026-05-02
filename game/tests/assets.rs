// SPDX-License-Identifier: MIT
//! Audit: every asset path the game references must exist on disk.
//! Catches typos, deleted files, case-mismatch on case-sensitive FS.

use pledge_and_crown::assets::ALL_SPRITE_PATHS;
use std::path::PathBuf;

fn assets_root() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest.join("assets")
}

#[test]
fn every_sprite_path_resolves_to_an_existing_file() {
    let root = assets_root();
    let mut missing = Vec::new();
    for path in ALL_SPRITE_PATHS {
        let abs = root.join(path);
        if !abs.exists() {
            missing.push(format!("  - {path} (looked for {})", abs.display()));
        }
    }
    assert!(
        missing.is_empty(),
        "missing sprite assets:\n{}",
        missing.join("\n")
    );
}

#[test]
fn sprite_path_list_has_no_duplicates() {
    let mut seen = std::collections::HashSet::new();
    for path in ALL_SPRITE_PATHS {
        assert!(
            seen.insert(*path),
            "duplicate sprite path in registry: {path}"
        );
    }
}

#[test]
fn every_referenced_sprite_is_a_png() {
    for path in ALL_SPRITE_PATHS {
        assert!(
            path.ends_with(".png"),
            "non-png asset: {path} — pipeline expects PNG-32 per design/04b-art-deliverables.md"
        );
    }
}

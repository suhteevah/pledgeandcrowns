// SPDX-License-Identifier: MIT
//! Build-time gate for the Tauri shell (used by `build.rs`).
//!
//! The shell bundles whatever is in `../../web` (`frontendDist`). The
//! free mobile build is the demo, so the build fails unless
//! `web/pkg/build-stamp.json` (written by `scripts/web-build.ps1`) says
//! the web build there is the demo, or `PNC_ALLOW_FULL_BUILD=1` is set
//! for a deliberate full-game bundle. No stamp means nobody knows which
//! game it is, so that fails too. (The PowerShell packaging scripts also
//! check the `.wasm` hash against the stamp; this gate has no hashing
//! crate and checks the flavour only.)
//!
//! Kept free of dependencies so it can be unit-tested on its own:
//! `rustc --edition 2021 --test web_gate.rs -o web_gate_test && ./web_gate_test`.

/// Set to `1` to allow bundling the full game.
pub const ALLOW_FULL_VAR: &str = "PNC_ALLOW_FULL_BUILD";

/// The string value of `"key": "value"` in the stamp's fixed layout.
pub fn stamp_field<'a>(stamp: &'a str, key: &str) -> Option<&'a str> {
    let needle = format!("\"{key}\"");
    let at = stamp.find(&needle)? + needle.len();
    let rest = stamp[at..].trim_start().strip_prefix(':')?.trim_start();
    let rest = rest.strip_prefix('"')?;
    let end = rest.find('"')?;
    Some(&rest[..end])
}

/// `Ok(flavour)` when this web build may be bundled, else why not.
pub fn check(stamp: Option<&str>, allow_full: bool) -> Result<&'static str, String> {
    let Some(stamp) = stamp else {
        return Err(
            "no web build stamp at web/pkg/build-stamp.json: cannot tell whether web/ holds \
             the demo or the full game. Run scripts/web-build.ps1 (demo) first."
                .to_string(),
        );
    };
    match stamp_field(stamp, "flavour") {
        Some("demo") => Ok("demo"),
        Some("full") if allow_full => Ok("full"),
        Some("full") => Err(format!(
            "web/ holds the FULL game, but the free mobile build is the demo. Rebuild with \
             scripts/web-build.ps1 (demo), or set {ALLOW_FULL_VAR}=1 to bundle the full game \
             on purpose."
        )),
        other => Err(format!(
            "web/pkg/build-stamp.json has no valid flavour ({other:?}); rebuild with \
             scripts/web-build.ps1."
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stamp(flavour: &str) -> String {
        format!(
            "{{\n  \"flavour\": \"{flavour}\",\n  \"features\": \"demo\",\n  \"commit\": \"abc\",\n  \"dirty\": false\n}}\n"
        )
    }

    #[test]
    fn reads_fields_from_the_fixed_layout() {
        let s = stamp("demo");
        assert_eq!(stamp_field(&s, "flavour"), Some("demo"));
        assert_eq!(stamp_field(&s, "commit"), Some("abc"));
        assert_eq!(stamp_field(&s, "dirty"), None, "not a string");
        assert_eq!(stamp_field(&s, "missing"), None);
        assert_eq!(stamp_field("{\"flavour\":\"full\"}", "flavour"), Some("full"));
    }

    #[test]
    fn demo_always_passes() {
        assert_eq!(check(Some(&stamp("demo")), false), Ok("demo"));
        assert_eq!(check(Some(&stamp("demo")), true), Ok("demo"));
    }

    #[test]
    fn full_needs_the_opt_in() {
        assert!(check(Some(&stamp("full")), false).is_err());
        assert_eq!(check(Some(&stamp("full")), true), Ok("full"));
    }

    #[test]
    fn missing_or_garbled_stamp_fails() {
        assert!(check(None, true).is_err());
        assert!(check(Some("not json"), true).is_err());
        assert!(check(Some(&stamp("beta")), true).is_err());
    }
}

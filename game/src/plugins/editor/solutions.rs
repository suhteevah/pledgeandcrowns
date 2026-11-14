// SPDX-License-Identifier: MIT
//! Reference solutions shown behind the "Show solution" gate.
//!
//! The solution text lives in [`data`] as a plain `(id, source)` table so
//! the content can be reviewed (and voice-edited) without touching logic.
//! Every entry is a complete, warning-free program written as a teaching
//! artifact; `game/tests/solutions.rs` proves each one passes the graders
//! and (with `--ignored`) really compiles natively and for `wasm32-wasip1`.

mod data;

/// Reference solution for `mission_id`, if one has been authored.
pub fn canonical_solution(mission_id: &str) -> Option<&'static str> {
    let found = data::SOLUTIONS
        .iter()
        .find(|(id, _)| *id == mission_id)
        .map(|(_, src)| *src);
    tracing::trace!(
        "canonical_solution lookup for {mission_id}: {}",
        if found.is_some() { "found" } else { "none" }
    );
    found
}

/// Every authored `(mission_id, solution)` pair, in registry order. Used
/// by the audit tests to catch solutions for ids the registry no longer has.
pub fn all_solutions() -> &'static [(&'static str, &'static str)] {
    data::SOLUTIONS
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn known_mission_has_a_solution() {
        let src = canonical_solution("intro_let_binding").expect("authored");
        assert!(src.contains("fn main"));
    }

    #[test]
    fn unknown_mission_has_none() {
        assert_eq!(canonical_solution("no_such_mission"), None);
    }

    #[test]
    fn ids_are_unique() {
        let mut seen = HashSet::new();
        for (id, _) in all_solutions() {
            assert!(seen.insert(*id), "duplicate solution id `{id}`");
        }
    }
}

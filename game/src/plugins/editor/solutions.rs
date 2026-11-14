// SPDX-License-Identifier: MIT
//! Reference solutions shown behind the "Show solution" gate.
//!
//! `Mission` (in `mission.rs`) carries no canonical solution, so this
//! accessor returns `None` for every mission until solutions are
//! authored as player-facing content. Grader-passing fixtures exist in
//! `game/tests/contract.rs::canonical_solution` (all 57 missions) and
//! the Act 3-8 specs under `docs/superpowers/specs/`, but those are test
//! fixtures sized to the pattern grader (several lack `fn main` and
//! would not build under `/compile-real`), so they are deliberately not
//! surfaced to players as-is.

/// Reference solution for `mission_id`, if one has been authored.
pub fn canonical_solution(mission_id: &str) -> Option<&'static str> {
    tracing::trace!("canonical_solution lookup for {mission_id}: none authored");
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_solutions_authored_yet() {
        assert_eq!(canonical_solution("intro_let_binding"), None);
    }
}

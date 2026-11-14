// SPDX-License-Identifier: MIT
//! Pure editor-session rules: the per-mission failed-compile counter,
//! the show-solution gate, and the reset-confirm rule. No Bevy or egui
//! types here so every rule is unit-testable.

use super::diagnostics::ResultKind;
use std::collections::HashMap;

/// Failed compiles on one mission (this session) before the
/// "Show solution" button unlocks.
pub const SOLUTION_UNLOCK_FAILURES: u32 = 3;

/// Session-scoped (not persisted) failed-compile counts per mission id.
#[derive(Debug, Default, Clone)]
pub struct FailureTracker {
    failures: HashMap<String, u32>,
}

impl FailureTracker {
    /// Record one compile verdict for `mission_id`. Only `Fail` counts:
    /// a pass leaves the count alone (a solution once earned stays
    /// earned), and `Error` / `Pending` are not the player's mistake.
    /// Returns the mission's count after recording.
    pub fn record(&mut self, mission_id: &str, kind: ResultKind) -> u32 {
        if kind == ResultKind::Fail {
            let count = self.failures.entry(mission_id.to_string()).or_insert(0);
            *count = count.saturating_add(1);
            *count
        } else {
            self.failures(mission_id)
        }
    }

    pub fn failures(&self, mission_id: &str) -> u32 {
        self.failures.get(mission_id).copied().unwrap_or(0)
    }
}

/// State of the "Show solution" button for the current mission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SolutionGate<'a> {
    /// Not enough failed compiles yet; `remaining` more are needed.
    Locked { remaining: u32 },
    /// Threshold reached but no reference solution has been authored.
    Unauthored,
    /// Threshold reached and a solution exists.
    Available(&'a str),
}

pub fn solution_gate(failures: u32, solution: Option<&str>) -> SolutionGate<'_> {
    if failures < SOLUTION_UNLOCK_FAILURES {
        return SolutionGate::Locked {
            remaining: SOLUTION_UNLOCK_FAILURES - failures,
        };
    }
    match solution {
        Some(s) => SolutionGate::Available(s),
        None => SolutionGate::Unauthored,
    }
}

/// Resetting only needs a confirmation when it would destroy work, i.e.
/// the buffer differs from the mission's starter code.
pub fn reset_needs_confirm(current: &str, starter: &str) -> bool {
    current != starter
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_fail_increments() {
        let mut t = FailureTracker::default();
        assert_eq!(t.record("m", ResultKind::Fail), 1);
        assert_eq!(t.record("m", ResultKind::Error), 1);
        assert_eq!(t.record("m", ResultKind::Pending), 1);
        assert_eq!(t.record("m", ResultKind::Pass), 1);
        assert_eq!(t.record("m", ResultKind::Fail), 2);
    }

    #[test]
    fn counts_are_per_mission_and_survive_switching() {
        let mut t = FailureTracker::default();
        t.record("a", ResultKind::Fail);
        t.record("a", ResultKind::Fail);
        t.record("b", ResultKind::Fail);
        assert_eq!(t.failures("a"), 2);
        assert_eq!(t.failures("b"), 1);
        assert_eq!(t.failures("never"), 0);
    }

    #[test]
    fn gate_unlocks_at_exactly_three_failures() {
        assert_eq!(
            solution_gate(0, Some("s")),
            SolutionGate::Locked { remaining: 3 }
        );
        assert_eq!(
            solution_gate(2, Some("s")),
            SolutionGate::Locked { remaining: 1 }
        );
        assert_eq!(solution_gate(3, Some("s")), SolutionGate::Available("s"));
        assert_eq!(solution_gate(9, Some("s")), SolutionGate::Available("s"));
    }

    #[test]
    fn gate_reports_unauthored_solutions() {
        assert_eq!(
            solution_gate(1, None),
            SolutionGate::Locked { remaining: 2 }
        );
        assert_eq!(solution_gate(3, None), SolutionGate::Unauthored);
    }

    #[test]
    fn pass_after_unlock_keeps_solution_available() {
        let mut t = FailureTracker::default();
        for _ in 0..3 {
            t.record("m", ResultKind::Fail);
        }
        t.record("m", ResultKind::Pass);
        assert_eq!(
            solution_gate(t.failures("m"), Some("s")),
            SolutionGate::Available("s")
        );
    }

    #[test]
    fn reset_confirm_only_when_buffer_differs() {
        let starter = "fn main() {}\n";
        assert!(!reset_needs_confirm(starter, starter));
        assert!(reset_needs_confirm("fn main() { let a = 1; }\n", starter));
        assert!(reset_needs_confirm("", starter));
        // Whitespace edits are still edits.
        assert!(reset_needs_confirm("fn main() {}", starter));
    }
}

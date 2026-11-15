// SPDX-License-Identifier: MIT
//! Pure layout maths for the Cast Editor. The editor is a fixed fraction
//! of the screen, centred, at any window size; every pane gets a fixed
//! rectangle and scrolls internally, so nothing can grow past the window
//! edge or hide behind another pane. No egui state is touched here, only
//! `Rect` arithmetic, so the maths is unit-tested for many window sizes.
//!
//! ```text
//! +------------------------------------------------------------+
//! | title bar                                                  |
//! | toolbar                                                    |
//! | +-------------------------------+ +----------------------+ |
//! | | code page                     | | goal (or solution)   | |
//! | |                               | |                      | |
//! | |                               | +----------------------+ |
//! | |                               | | result               | |
//! | +-------------------------------+ +----------------------+ |
//! +------------------------------------------------------------+
//! ```

use bevy_egui::egui::{Pos2, Rect, Vec2};

/// Share of the screen width / height the editor covers.
pub const WIDTH_FRACTION: f32 = 0.94;
pub const HEIGHT_FRACTION: f32 = 0.92;
/// Double-line frame (2 px line, 2 px gap, 2 px line) plus padding.
pub const FRAME_INSET: f32 = 10.0;
pub const TITLE_HEIGHT: f32 = 26.0;
pub const TOOLBAR_HEIGHT: f32 = 32.0;
/// Gap between panes.
pub const GAP: f32 = 8.0;
/// Code page share of the body width.
pub const CODE_FRACTION: f32 = 0.58;
/// Result pane share of the right column while a result is showing.
pub const RESULT_FRACTION: f32 = 0.5;
/// Below this right-column height (points) the window counts as small
/// (960x540 is ~406; 1280x720 is ~572): a showing result then takes
/// [`RESULT_FRACTION_COMPACT`] of the column, because rustc output needs
/// lines more than the goal pane (which scrolls) does.
pub const COMPACT_COLUMN: f32 = 520.0;
/// Result pane share of a small right column while a result is showing.
pub const RESULT_FRACTION_COMPACT: f32 = 0.68;
/// The goal pane's floor in a small column.
pub const GOAL_MIN_COMPACT: f32 = 96.0;
/// Result pane height bounds (points) while a result is showing.
pub const RESULT_MIN: f32 = 120.0;
/// The goal pane never shrinks below this while a result shows.
pub const GOAL_MIN: f32 = 110.0;
/// Result pane height with nothing to show yet (a one-line prompt).
pub const RESULT_IDLE: f32 = 64.0;

/// Every rectangle the editor draws, in screen points.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EditorLayout {
    pub window: Rect,
    pub title: Rect,
    pub toolbar: Rect,
    pub code: Rect,
    /// Goal pane, or the solution pane when the solution is revealed.
    pub goal: Rect,
    pub result: Rect,
}

impl EditorLayout {
    /// All panes, for checks.
    pub fn panes(&self) -> [Rect; 5] {
        [self.title, self.toolbar, self.code, self.goal, self.result]
    }
}

/// `(result share, goal floor)` of a right column `column` points tall
/// with a result showing. Small columns favour the result, unless the
/// revealed solution is in the goal slot (then it gets the even split).
pub fn result_split(column: f32, solution_shown: bool) -> (f32, f32) {
    if column < COMPACT_COLUMN && !solution_shown {
        (RESULT_FRACTION_COMPACT, GOAL_MIN_COMPACT)
    } else {
        (RESULT_FRACTION, GOAL_MIN)
    }
}

/// Layout for a screen of `screen` points. `has_result` gives the result
/// pane its full share; otherwise it is a one-line strip and the goal
/// pane gets the room. `solution_shown` (the solution is in the goal
/// slot) keeps the even split on small screens.
pub fn editor_layout(screen: Vec2, has_result: bool, solution_shown: bool) -> EditorLayout {
    let size = Vec2::new(
        (screen.x * WIDTH_FRACTION).floor(),
        (screen.y * HEIGHT_FRACTION).floor(),
    );
    let min = Pos2::new(
        ((screen.x - size.x) / 2.0).floor(),
        ((screen.y - size.y) / 2.0).floor(),
    );
    let window = Rect::from_min_size(min, size);
    let inner = window.shrink(FRAME_INSET);

    let title = Rect::from_min_size(inner.min, Vec2::new(inner.width(), TITLE_HEIGHT));
    let toolbar = Rect::from_min_size(
        Pos2::new(inner.min.x, title.max.y + 2.0),
        Vec2::new(inner.width(), TOOLBAR_HEIGHT),
    );
    let body = Rect::from_min_max(Pos2::new(inner.min.x, toolbar.max.y + 2.0), inner.max);

    let code_w = ((body.width() - GAP) * CODE_FRACTION).floor();
    let code = Rect::from_min_size(body.min, Vec2::new(code_w, body.height()));
    let side = Rect::from_min_max(Pos2::new(code.max.x + GAP, body.min.y), body.max);

    let column = side.height() - GAP;
    let result_h = if has_result {
        let (share, goal_min) = result_split(column, solution_shown);
        (column * share)
            .max(RESULT_MIN)
            .min(column - goal_min)
            .max(0.0)
    } else {
        RESULT_IDLE.min(column / 2.0)
    }
    .floor();
    let result = Rect::from_min_max(Pos2::new(side.min.x, side.max.y - result_h), side.max);
    let goal = Rect::from_min_max(side.min, Pos2::new(side.max.x, result.min.y - GAP));

    EditorLayout {
        window,
        title,
        toolbar,
        code,
        goal,
        result,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIZES: &[(f32, f32)] = &[
        (960.0, 540.0),
        (1024.0, 768.0),
        (1280.0, 720.0),
        (1366.0, 768.0),
        (1600.0, 900.0),
        (1920.0, 1080.0),
        (2560.0, 1440.0),
        (3840.0, 2160.0),
        (1280.0, 1024.0),
    ];

    fn inside(outer: Rect, inner: Rect) -> bool {
        outer.contains_rect(inner)
    }

    #[test]
    fn editor_fits_the_screen_at_every_size() {
        for &(w, h) in SIZES {
            let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(w, h));
            for (has_result, solution) in [(false, false), (true, false), (true, true)] {
                let l = editor_layout(Vec2::new(w, h), has_result, solution);
                assert!(inside(screen, l.window), "{w}x{h}: window off screen");
                for p in l.panes() {
                    assert!(inside(l.window, p), "{w}x{h}: pane {p:?} outside window");
                    assert!(p.width() > 0.0 && p.height() > 0.0, "{w}x{h}: empty pane");
                }
                // Fixed fraction, centred.
                assert!((l.window.width() - (w * WIDTH_FRACTION).floor()).abs() < 1.0);
                assert!((l.window.height() - (h * HEIGHT_FRACTION).floor()).abs() < 1.0);
                assert!((l.window.center().x - w / 2.0).abs() <= 1.0);
            }
        }
    }

    #[test]
    fn panes_never_overlap() {
        for &(w, h) in SIZES {
            for (has_result, solution) in [(false, false), (true, false), (true, true)] {
                let l = editor_layout(Vec2::new(w, h), has_result, solution);
                let panes = l.panes();
                for (i, a) in panes.iter().enumerate() {
                    for b in &panes[i + 1..] {
                        let overlap = a.intersect(*b);
                        assert!(
                            overlap.width() <= 0.0 || overlap.height() <= 0.0,
                            "{w}x{h}: {a:?} overlaps {b:?}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn result_pane_is_bounded_and_goal_keeps_room() {
        for &(w, h) in SIZES {
            let l = editor_layout(Vec2::new(w, h), true, false);
            let column = l.goal.height() + GAP + l.result.height();
            let (share, goal_min) = result_split(column, false);
            assert!(
                l.result.height() >= RESULT_MIN.min(column - goal_min),
                "{w}x{h}"
            );
            assert!(l.result.height() <= column * share + 1.0, "{w}x{h}");
            assert!(l.goal.height() >= goal_min - 1.0, "{w}x{h}");
            // Goal sits directly above the result, same column.
            assert_eq!(l.goal.min.x, l.result.min.x);
            assert!(l.goal.max.y <= l.result.min.y);

            let idle = editor_layout(Vec2::new(w, h), false, false);
            assert!(idle.result.height() <= RESULT_IDLE);
            assert!(idle.goal.height() > l.goal.height());
        }
    }

    #[test]
    fn smallest_supported_window_still_has_usable_panes() {
        let l = editor_layout(Vec2::new(960.0, 540.0), true, false);
        assert!(l.code.width() >= 480.0, "{:?}", l.code);
        assert!(l.code.height() >= 400.0, "{:?}", l.code);
        assert!(l.goal.width() >= 340.0, "{:?}", l.goal);
        assert!(l.result.height() >= RESULT_MIN, "{:?}", l.result);
    }

    /// The rebalance, at the three reference sizes: at 960x540 a result
    /// takes about two thirds of the right column (it used to get half,
    /// ~200 pt, a handful of rustc lines); larger windows keep the even
    /// split; a revealed solution keeps the even split everywhere.
    #[test]
    fn small_windows_give_the_result_pane_more_room() {
        let at = |w: f32, h: f32, sol: bool| editor_layout(Vec2::new(w, h), true, sol);
        let small = at(960.0, 540.0, false);
        let column = small.goal.height() + small.result.height();
        assert!(column < COMPACT_COLUMN, "{column}");
        assert!(small.result.height() >= 270.0, "{:?}", small.result);
        assert!(small.goal.height() >= GOAL_MIN_COMPACT, "{:?}", small.goal);
        let sol = at(960.0, 540.0, true);
        assert!((sol.result.height() - (column * RESULT_FRACTION).floor()).abs() <= 1.0);
        for (w, h) in [(1280.0, 720.0), (1920.0, 1080.0)] {
            let l = at(w, h, false);
            let col = l.goal.height() + l.result.height();
            assert!(col >= COMPACT_COLUMN, "{w}x{h}: {col}");
            assert!(
                (l.result.height() - (col * RESULT_FRACTION).floor()).abs() <= 1.0,
                "{w}x{h}: {:?}",
                l.result
            );
            assert_eq!(l, at(w, h, true), "{w}x{h}: solution changes nothing");
        }
    }
}

// SPDX-License-Identifier: MIT
//! Dialogue text, keyed by NPC id (the roster `mission_id`, which is unique
//! per NPC; display names are not guaranteed unique), plus one row per
//! boss-trial stage in [`TRIAL_STAGES`], keyed by the stage's mission id.
//!
//! Every status falls back to [`GENERIC`] when an NPC has no authored lines
//! for it, so all 57 roster NPCs can talk today. Authored lines exist only
//! for the nine Hearthstone Square NPCs (roster order, Ferris through the
//! Cooper). Every other NPC is awaiting the voice-editor pass: add an entry
//! to [`AUTHORED`] in that NPC's craft metaphor (see
//! `.claude/agents/voice-editor.md`) rather than editing the fallbacks.
//!
//! Lines are templates: `{next}` is replaced with the name of the NPC the
//! player should see next (see [`super::next_hint`]). Keep each line at or
//! under [`super::MAX_LINE_CHARS`] after substitution (a test enforces it).
//! Anything said about Rust must be technically correct (hard rule 3).
//!
//! `not_yet` lines are only used when `{next}` is the speaker's DIRECT
//! prerequisite (the mission right before theirs), so "see {next} first,
//! then come back" is literally true. When the next playable mission is
//! further back, every NPC says [`NOT_YET_ELSEWHERE`]; when nothing is
//! playable (a mission outside this build), [`NOT_YET_NOTHING`].
//!
//! Inline code goes in backticks; the box renders it as a code chip and
//! never shows the backticks themselves. Backticks must pair up.

/// Lines for one NPC (or the generic fallback), one slice per status. An
/// empty slice means "use the generic lines for this status".
#[derive(Debug, Clone, Copy)]
pub struct StatusLines {
    /// Their mission is locked and `{next}` is their direct prerequisite.
    pub not_yet: &'static [&'static str],
    /// Unlocked and never opened (saved per mission). The mission panel
    /// opens after the last line.
    pub offer: &'static [&'static str],
    /// Opened before but not cleared. The mission panel opens after the
    /// last line.
    pub in_progress: &'static [&'static str],
    /// Cleared. Flavour only; no mission panel.
    pub cleared: &'static [&'static str],
}

/// One authored table row.
#[derive(Debug, Clone, Copy)]
pub struct NpcLines {
    /// Roster `mission_id` of the speaking NPC.
    pub npc_id: &'static str,
    pub lines: StatusLines,
}

/// Fallback for any NPC/status without authored text.
pub const GENERIC: StatusLines = StatusLines {
    not_yet: &["Not yet. {next} has the lesson just before mine. See them first, then come back."],
    offer: &["Well met. I have a task for you, if you have the time."],
    in_progress: &["Back again? The task is where you left it."],
    cleared: &["Thank you. That was good work."],
};

/// Locked, and the next playable mission is not the speaker's direct
/// prerequisite: point at it without promising the speaker comes next.
pub const NOT_YET_ELSEWHERE: &str = "I'm not ready for you yet. See {next} first.";

/// Locked, and no mission is playable at all (this build ends earlier).
pub const NOT_YET_NOTHING: &str = "I'm not ready for you yet. Come back another day.";

/// Authored lines for boss-trial stages, keyed by stage mission id (see
/// `plugins::trial`). The owner speaks them when that stage is their
/// current mission. Which statuses can occur for a stage:
///
/// - `offer` / `in_progress`: every stage.
/// - `cleared`: only the final stage (a trial NPC is Cleared only once
///   every stage is); its lines close the trial.
/// - `not_yet`: only stage 1 of a trial whose owner has no prologue
///   mission. The Borrow Checker has one (`borrow_preview`) and talks as a
///   cleared villager until the Bridge opens, so his stages leave
///   `not_yet` empty.
///
/// Every compiler error named here is compiled in
/// `tests/mission_text.rs` (`BORROW_CHECKER_CLAIMS`), hard rule 3.
pub const TRIAL_STAGES: &[NpcLines] = &[
    NpcLines {
        npc_id: "bridge_toll",
        lines: StatusLines {
            not_yet: &[],
            offer: &[
                "So. You would cross. The Bridge has three tolls, and I collect every one.",
                "First toll: `weigh` takes your `String` and keeps it. Then you reach for it again.",
                "The compiler calls that E0382, borrow of moved value. Lend the cargo. Do not hand it over.",
            ],
            in_progress: &[
                "Here. Here you have moved what you intended to lend. Let `weigh` take `&str`, and pass `&cargo`.",
            ],
            cleared: &[],
        },
    },
    NpcLines {
        npc_id: "bridge_quill",
        lines: StatusLines {
            not_yet: &[],
            offer: &[
                "Second toll. Two travellers read the scroll at once. Many `&` borrows may share it.",
                "Then the quill writes while their borrows are still in use. That is E0502.",
                "The compiler: cannot borrow `scroll` as mutable because it is also borrowed as immutable.",
            ],
            in_progress: &[
                "One `&mut` or many `&`, never both. Let the readers finish, then pass `&mut scroll` to `inscribe`.",
            ],
            cleared: &[],
        },
    },
    NpcLines {
        npc_id: "bridge_lantern",
        lines: StatusLines {
            not_yet: &[],
            offer: &[
                "Last toll: the lantern. `before` hands back part of `line`, but never says whose part.",
                "E0106, missing lifetime specifier. Name it `'a`, and tie the answer to `line` alone.",
            ],
            in_progress: &[
                "The compiler's help ties `sep` to `'a` as well. Follow it, and `sep` does not live long enough: E0597.",
            ],
            cleared: &[
                "You may cross. The Old Tongue speaks through you.",
                "Lend, do not give. One writer or many readers. Say what your answer borrows. Remember the Bridge.",
            ],
        },
    },
];

/// Authored lines for the Hearthstone Square cast.
pub const AUTHORED: &[NpcLines] = &[
    NpcLines {
        npc_id: "intro_let_binding",
        lines: StatusLines {
            not_yet: &["Not yet, friend. Go and see {next} first. I'll be right here."],
            offer: &[
                "Welcome to Hearthstone! Everything here starts with a name.",
                "Give the number forty-two a name with `let`, and we're off.",
            ],
            in_progress: &["Still on that binding? `let`, then the name, then the value."],
            cleared: &[
                "Forty-two, bound and named. A `let` binding holds still unless you mark it `mut`.",
            ],
        },
    },
    NpcLines {
        npc_id: "double_function",
        lines: StatusLines {
            not_yet: &["My spells are not for you yet. {next} has the lesson you need first."],
            offer: &[
                "A function is a named spell: same inputs, same output.",
                "Name one that doubles an `i32`. Declare what it returns.",
            ],
            in_progress: &[
                "The spell is half-spoken. Leave the semicolon off the last expression and it returns.",
            ],
            cleared: &["Named once, cast as often as you like. That is the whole art."],
        },
    },
    NpcLines {
        npc_id: "borrow_preview",
        lines: StatusLines {
            not_yet: &["Not yet. Your case is not on today's docket. See {next}."],
            offer: &[
                "Halt. Every value here has exactly one owner.",
                "You may look without taking. Borrow it with `&`. The owner keeps it.",
            ],
            in_progress: &[
                "Still deliberating? A shared reference may read the value. It may not move it out.",
            ],
            cleared: &["The borrow ended before the owner did. \"...acceptable. for now.\""],
        },
    },
    NpcLines {
        npc_id: "mut_binding",
        lines: StatusLines {
            not_yet: &["The forge is cold for you yet. Go see {next}, then come back."],
            offer: &[
                "Iron holds its shape until you heat it. A binding is the same.",
                "Mark it `mut`, then strike it once: add one.",
            ],
            in_progress: &["Still cold? Without `mut`, the compiler won't let you change it."],
            cleared: &["Good. That one bends, and anyone reading can see it does."],
        },
    },
    NpcLines {
        npc_id: "if_else_sign",
        lines: StatusLines {
            not_yet: &["My maps can wait. {next} is on the road ahead of me."],
            offer: &[
                "Every fork wants a choice: negative, zero, or positive.",
                "Chart all three roads with `if` and `else`.",
            ],
            in_progress: &["Lost at the fork? Every branch must hand back the same type of value."],
            cleared: &["Three roads, one choice each time. A sound map."],
        },
    },
    NpcLines {
        npc_id: "loop_break",
        lines: StatusLines {
            not_yet: &["The bell's not ready for you. Find {next} first."],
            offer: &[
                "I pull, and pull, until the bell rings. That's a `loop`.",
                "Double a number until it reaches a hundred, then `break` with it.",
            ],
            in_progress: &[
                "Still pulling? A `break` can carry a value out of a `loop`. Let it ring.",
            ],
            cleared: &["There's the bell. The loop gave back its number."],
        },
    },
    NpcLines {
        npc_id: "match_option",
        lines: StatusLines {
            not_yet: &["The orb shows you elsewhere. Seek {next} first."],
            offer: &[
                "The orb has two halves: the value present, and the value absent.",
                "Answer both with `match`. The compiler will not let you skip one.",
            ],
            in_progress: &[
                "Still gazing? `Some` holds a value. `None` holds nothing. Answer both.",
            ],
            cleared: &["Both paths walked. Nothing slips through."],
        },
    },
    NpcLines {
        npc_id: "struct_basic",
        lines: StatusLines {
            not_yet: &["No proclamation for you yet. Report to {next}."],
            offer: &[
                "A tabard carries many fields under one banner. So does a `struct`.",
                "Raise a Knight with a name and hit points, then read the name aloud.",
            ],
            in_progress: &[
                "Fields are named, not numbered. Build the Knight, then call out `.name`.",
            ],
            cleared: &["By name and by number. Both fields announced."],
        },
    },
    NpcLines {
        npc_id: "vec_iter",
        lines: StatusLines {
            not_yet: &["Barrel's not ready for you. See {next} first."],
            offer: &[
                "A barrel is staves in one hoop. A `Vec` is values in one list.",
                "Walk every stave with `.iter()`, then `.sum()` them up.",
            ],
            in_progress: &[
                "Still counting? Tell `sum` which number type to add into, or the compiler can't pick.",
            ],
            cleared: &["Every stave counted, sum sealed."],
        },
    },
];

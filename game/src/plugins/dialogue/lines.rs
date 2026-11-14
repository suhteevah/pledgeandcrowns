// SPDX-License-Identifier: MIT
//! Dialogue text, keyed by NPC id (the roster `mission_id`, which is unique
//! per NPC; display names are not guaranteed unique).
//!
//! Every status falls back to [`GENERIC`] when an NPC has no authored lines
//! for it, so all 57 roster NPCs can talk today. Authored lines exist only
//! for the nine Hearthstone Square NPCs (roster order, Ferris through the
//! Cooper). Every other NPC is awaiting the voice-editor pass: add an entry
//! to [`AUTHORED`] in that NPC's craft metaphor (see
//! `.claude/agents/voice-editor.md`) rather than editing the fallbacks.
//!
//! Lines are templates: `{next}` is replaced with the name of the NPC whose
//! mission is next in registry order. Keep each line at or under
//! [`super::MAX_LINE_CHARS`] after substitution (a test enforces it).
//! Anything said about Rust must be technically correct (hard rule 3).

/// Lines for one NPC (or the generic fallback), one slice per status. An
/// empty slice means "use the generic lines for this status".
#[derive(Debug, Clone, Copy)]
pub struct StatusLines {
    /// Their mission is still locked behind an earlier one.
    pub not_yet: &'static [&'static str],
    /// Unlocked and never opened this session. The mission panel opens
    /// after the last line.
    pub offer: &'static [&'static str],
    /// Opened this session but not cleared. The mission panel opens after
    /// the last line.
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
    not_yet: &["Not yet. Find {next} first, then come back to me."],
    offer: &["Well met. I have a task for you, if you have the time."],
    in_progress: &["Back again? The task is where you left it."],
    cleared: &["Thank you. That was good work."],
};

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

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
///   mission. Every trial owner so far has one (the Borrow Checker's
///   `borrow_preview`, the Guildmaster's `impl_method`, Vexis's
///   `trait_def`, the Barkeep's `result_match`, the Keymaster's
///   `hashmap_basic`, the Dockmaster's `thread_spawn`, the Vaultwright's
///   `box_basic`) and talks as a cleared villager until the trial opens,
///   so their stages leave `not_yet` empty.
///
/// Every compiler error named here is compiled in
/// `tests/mission_text.rs` (`trial_claims`), hard rule 3.
pub const TRIAL_STAGES: &[NpcLines] = &[
    NpcLines {
        npc_id: "bridge_toll",
        lines: StatusLines {
            not_yet: &[],
            offer: &[
                "So. You would cross. The Bridge has three tolls, and I collect every one.",
                "First toll: `weigh` takes your `String`, and it does not come back. Then you reach for it again.",
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
                "The ruling reads: cannot borrow `scroll` as mutable because it is also borrowed as immutable.",
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
                "Last toll: the lantern. `before` hands back a borrow, but its signature never says whose.",
                "E0106, missing lifetime specifier: the compiler will not guess. Name it `'a`; tie the answer to `line` alone.",
            ],
            in_progress: &[
                "Mind the compiler's help. It ties `sep` to `'a` as well, and then `sep` does not live long enough: E0597.",
            ],
            cleared: &[
                "Three tolls, paid in full. \"...acceptable.\"",
                "Lend, do not give. One writer or many readers. Say what your answer borrows. Remember the Bridge.",
            ],
        },
    },
    // ── The Guildmaster: The Variant Wraith (Act 3) ──────────────────
    NpcLines {
        npc_id: "wraith_forms",
        lines: StatusLines {
            not_yet: &[],
            offer: &[
                "One last assessment for this quarter: the Variant Wraith. Three forms, three stages.",
                "First, its forms. Your `counter` lets `_` answer the Shadow with steel. Steel passes through it.",
                "Give every form its own arm. A member is defined by their duties, and so is a `match`.",
            ],
            in_progress: &[
                "Strike the `_` arm and write `Wraith::Shadow => \"light\"`. Then every form is answered by name.",
            ],
            cleared: &[],
        },
    },
    NpcLines {
        npc_id: "wraith_essence",
        lines: StatusLines {
            not_yet: &[],
            offer: &[
                "Second stage. The forms carry something now: armor on the Solid, density in the Mist.",
                "Your arms name the forms but never open them. The compiler calls that E0533.",
                "Open each one in its pattern, `Wraith::Solid { armor }`, and put what you find to work.",
            ],
            in_progress: &[
                "Bind the field in the pattern itself. Then `armor` is a local, ready for `blow - armor`.",
            ],
            cleared: &[],
        },
    },
    NpcLines {
        npc_id: "wraith_possessed",
        lines: StatusLines {
            not_yet: &[],
            offer: &[
                "Final stage. Mid-fight the Wraith takes a fourth form, `Possessed`. Your `match` has no arm for it.",
                "E0004, non-exhaustive patterns. That error is the Guild's ledger doing its duty.",
                "Answer the new form by name. A blanket `_` would let the next form in unseen.",
            ],
            in_progress: &[
                "A possessed host fights back: `blow + will`. Give `Wraith::Possessed { will }` its own arm.",
            ],
            cleared: &[
                "Four forms, four arms, none left to chance. The Wraith is unmade.",
                "Name every variant, and the compiler keeps the roll for you. The Guild is proud to count you a member.",
            ],
        },
    },
    // ── Vexis the Archmage: The Chimera of Vexis (Act 4) ─────────────
    NpcLines {
        npc_id: "chimera_heads",
        lines: StatusLines {
            not_yet: &[],
            offer: &[
                "You have climbed my tower. Now meet what guards its top: the Chimera. Three heads, three stages.",
                "First, the heads themselves. Only Fire has been granted the `Enemy` trait. Frost and Storm have not.",
                "Call a trait method on them and the compiler answers E0599: no such method. Grant the trait.",
            ],
            in_progress: &[
                "A capability named once, granted to a type. Write `impl Enemy for Frost`, then the same for Storm.",
            ],
            cleared: &[],
        },
    },
    NpcLines {
        npc_id: "chimera_bound",
        lines: StatusLines {
            not_yet: &[],
            offer: &[
                "Second stage. One spell, `volley`, cast at any head: generic over `T`, as a good spell should be.",
                "But nothing promises that `T` can breathe, so the compiler refuses `head.name()` with E0599.",
                "Bind the parameter: `T: Enemy`. Then every head that holds the trait may answer.",
            ],
            in_progress: &[
                "Write `: Enemy` after the `T` in `volley<T>`. The bound is a promise, and the compiler holds you to it.",
            ],
            cleared: &[],
        },
    },
    NpcLines {
        npc_id: "chimera_plasma",
        lines: StatusLines {
            not_yet: &[],
            offer: &[
                "Final stage. The Chimera shows a fourth head, `Plasma`. Your `battle` knows only three.",
                "It matches on a list of heads fixed in advance, and today the heads arrive as trait objects.",
                "Take `&[Box<dyn Enemy>]` and call the trait. Then any head, known or not, can be fought.",
            ],
            in_progress: &[
                "`battle` still expects `&[Head]`: E0308, mismatched types. Let it take `&[Box<dyn Enemy>]`.",
            ],
            cleared: &[
                "Four heads, one `battle`, and not one arm per head. The Chimera falls silent.",
                "Generics when you know the type, trait objects when you do not. The top of the tower is yours.",
            ],
        },
    },
    // ── The Barkeep: The Tavernkeeper's Wager (Act 5) ────────────────
    NpcLines {
        npc_id: "wager_stake",
        lines: StatusLines {
            not_yet: &[],
            offer: &[
                "Fancy a wager? The Tavernkeeper's game: three rounds, and the house collects on every slip.",
                "First round, the stake. `?` can hand a bad stake back, but only from a function that returns `Result`.",
                "Yours returns a plain `u32`, so the compiler says E0277. Let the signature admit it might fail.",
            ],
            in_progress: &[
                "Return `Result<u32, ParseIntError>` and wrap the doubled stake in `Ok`. Poured or spilled, both go back.",
            ],
            cleared: &[],
        },
    },
    NpcLines {
        npc_id: "wager_from",
        lines: StatusLines {
            not_yet: &[],
            offer: &[
                "Second round. Two things go wrong at this table: a stake that is no number, and a purse too light.",
                "Your `?` must turn both into a `WagerError`, and it does not know how. E0277 again.",
                "Teach it with `From`, one impl per error. Then `?` converts as it pours.",
            ],
            in_progress: &[
                "`impl From<ParseIntError> for WagerError`, and another for `Short`. Leave `place_bet` alone.",
            ],
            cleared: &[],
        },
    },
    NpcLines {
        npc_id: "wager_quest",
        lines: StatusLines {
            not_yet: &[],
            offer: &[
                "Last round, the wager proper. A quest of three steps, and any one of them can go wrong.",
                "Your `quest` unwraps every step. One bad map and the whole tavern goes down with it.",
                "Chain them with `?` instead. No `match`, no `unwrap`, no `expect`, no `panic!`. The house is watching.",
            ],
            in_progress: &[
                "`read_map(map)?`, then `open_vault(key)?`, then `pay_guide(gold, leagues)?`. The first failure pays out.",
            ],
            cleared: &[
                "Every step chained, every failure handed back, and not one broken glass. The wager is yours.",
                "A `Result` tells the truth about failure. Keep it, and a bad night ends in an `Err`, not a crash.",
            ],
        },
    },
    // ── The Keymaster: The Forge-Golem (Act 6) ───────────────────────
    NpcLines {
        npc_id: "golem_parts",
        lines: StatusLines {
            not_yet: &[],
            offer: &[
                "The Forge-Golem is awake, and every forge key is in my keeping. Three stages to bring it down.",
                "First, its weak points. Your chain turns each part into a label, then asks the label for armour.",
                "A label has no armour field, so the compiler says E0609. Filter while you still hold the whole part.",
            ],
            in_progress: &[
                "Put `.filter(|p| !p.armoured)` before the `.map(...)`. Sort the parts first, label them after.",
            ],
            cleared: &[],
        },
    },
    NpcLines {
        npc_id: "golem_sum",
        lines: StatusLines {
            not_yet: &[],
            offer: &[
                "Second stage. The core takes every weak point's weakness, added up and doubled.",
                "`sum` can make many kinds of number, and nothing tells it which. The compiler says E0283.",
                "Name the type you want. Every store in my keeping carries a label.",
            ],
            in_progress: &[
                "Write `let total: u32 = ...`, or `.sum::<u32>()`. Then `total * 2` is plain arithmetic.",
            ],
            cleared: &[],
        },
    },
    NpcLines {
        npc_id: "golem_core",
        lines: StatusLines {
            not_yet: &[],
            offer: &[
                "Final stage. One blow at the whole Golem: every weak point takes it, times its weakness.",
                "Your loop stops at the first armoured plate and never reaches the knee or the eye.",
                "Write it as one chain: filter, map, fold. No `for`, no `while`, no `loop`, no `mut`. I will check.",
            ],
            in_progress: &[
                "`.filter(..)` skips a plate and keeps going. Then `.map(..)` to the hit, and `.fold(0, ..)` the total.",
            ],
            cleared: &[
                "One chain, top to bottom, and the Golem comes apart at every seam.",
                "Filter skips, map changes, fold gathers. Every key back on its hook.",
            ],
        },
    },
    // ── The Dockmaster: The Tide Hydra (Act 7) ───────────────────────
    NpcLines {
        npc_id: "hydra_senders",
        lines: StatusLines {
            not_yet: &[],
            offer: &[
                "The Tide Hydra is rising in the harbour. Three heads, and we fight it in three stages.",
                "First, the signals. Every head reports on one channel, but your loop hands the only sender to the first head.",
                "The next head finds it gone: E0382, use of moved value. Give each head its own `tx.clone()`.",
            ],
            in_progress: &["`fight(head, hp, tx.clone())`. Every clone feeds the same receiver."],
            cleared: &[],
        },
    },
    NpcLines {
        npc_id: "hydra_threads",
        lines: StatusLines {
            not_yet: &[],
            offer: &[
                "Second stage. One worker per head now, each on a thread of its own.",
                "Your closure only borrows `hp`, and a thread can outlive the function that sent it. E0373.",
                "Send each worker off with `move`, so it owns everything it carries.",
            ],
            in_progress: &[
                "`thread::spawn(move || fight(head, hp, tx))`. Nothing fixes the order the reports come back in, so keep the sort.",
            ],
            cleared: &[],
        },
    },
    NpcLines {
        npc_id: "hydra_body",
        lines: StatusLines {
            not_yet: &[],
            offer: &[
                "Final stage. The body opens only when every head is down.",
                "Your watch strikes at the first `Down`, before the other two heads report down.",
                "Count the heads as they report, and strike only when all of them have.",
            ],
            in_progress: &[
                "Count the `Down` reports and stop when the count reaches `heads`. Every hit before that counts.",
            ],
            cleared: &[
                "Three workers out, three reports in, one strike at the body. The Hydra sinks.",
                "Send them off with `move`, hear them on one channel, wait for all. That is how a harbour runs.",
            ],
        },
    },
    // ── The Vaultwright: The Memory Warden (Act 8) ───────────────────
    NpcLines {
        npc_id: "warden_box",
        lines: StatusLines {
            not_yet: &[],
            offer: &[
                "The Memory Warden holds the Vault below us. Three phases, and each wants a different grip.",
                "First, his ledger. Each entry holds the rest of the ledger inline, so the type never ends. E0072.",
                "Put the rest in a `Box`. A box is one pointer wide, however long the ledger runs.",
            ],
            in_progress: &[
                "`Entry(u32, Box<Ledger>)`, and `Box::new(rest)` where you build an entry.",
            ],
            cleared: &[],
        },
    },
    NpcLines {
        npc_id: "warden_shared",
        lines: StatusLines {
            not_yet: &[],
            offer: &[
                "Second phase. Two debtors pay into one vault, and each holds a handle to it.",
                "An `Rc` shares the vault, but only to read. Adding coins through one is E0594.",
                "Put a `RefCell` inside the `Rc`. Then any handle can borrow the vault to write.",
            ],
            in_progress: &[
                "`vault: Rc<RefCell<u32>>`, then `*debtor.vault.borrow_mut() += coins`.",
            ],
            cleared: &[],
        },
    },
    NpcLines {
        npc_id: "warden_weak",
        lines: StatusLines {
            not_yet: &[],
            offer: &[
                "Final phase. The Warden owes you, you owe the teller, and the teller owes the Warden.",
                "With strong links each debtor keeps the next alive. Drop every handle and the ring is still there.",
                "Make each debt a `Weak`, and walk the ring with `.upgrade()` until you are back where you began.",
            ],
            in_progress: &[
                "`owes: RefCell<Weak<Debtor>>`, `Rc::downgrade` to link, `.upgrade()` to follow. Stop at the start.",
            ],
            cleared: &[
                "The ring walked once, and nothing left behind when it closed. The Warden steps aside.",
                "Box to hold, Rc to share, RefCell to change, Weak to let go. The Vault is yours.",
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

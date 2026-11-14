// SPDX-License-Identifier: MIT
//! Offline fallback grader for the in-game compile client.
//!
//! When the live `compile-api` server at 127.0.0.1:7878 is unreachable
//! (Hetzner hiccup, demo machine without server, kiosk mode, etc.) the
//! game must not show a dead-end `[client error]`. Instead, this module
//! re-implements the same pattern checks the server-side grader uses,
//! returning a `[stub]`-prefixed verdict so the player can still feel
//! progression while knowing the grade isn't authoritative. On wasm it
//! is the ONLY grader.
//!
//! IMPORTANT: this MUST NOT add a runtime dependency on
//! `pledge_compile_api` (MIT game vs AGPL server). The rule table is
//! duplicated on purpose and kept TEXTUALLY IDENTICAL to the one in
//! `compile-api/src/grader.rs` between the `BEGIN MISSION TABLE` /
//! `END MISSION TABLE` markers. `game/tests/contract.rs` enforces both
//! the textual parity and byte-for-byte output parity.
//!
//! Hard rule: a stub pass is NEVER persisted as a real mission clear.
//! The caller in `compile_client.rs` is responsible for gating
//! `MissionProgress::mark_cleared` on a "this came from the live API"
//! flag — see `CompileOutcome::from_stub`.
//!
//! # Adding a mission
//!
//! Add ONE [`MissionRule`] entry to [`MISSION_RULES`] here, and the
//! identical entry to `MISSION_RULES` in `compile-api/src/grader.rs`:
//!
//! ```text
//! MissionRule {
//!     id: "mut_binding",                  // MissionRegistry encounter id
//!     npc: "the Smith eyes you",          // fail = "{npc} — {hint}"
//!     checks: &[
//!         // Checked in order; the first check whose `any_of` tokens are
//!         // ALL absent from the source fails with its hint.
//!         check(&["let mut"], "missing required: `let mut`"),
//!         check(&["+= 1", "+=1"], "missing required: `+= 1`"),
//!     ],
//!     pass: "the Smith strikes the work. \"good — that one bends.\"",
//! },
//! ```
//!
//! Every check is a plain substring test: a check passes when the source
//! contains at least one of its `any_of` alternatives (one alternative =
//! a required token; several = whitespace variants like `*= 2` / `*=2`).
//! If a future mission genuinely needs logic the table cannot express,
//! give it a named custom fn rather than contorting the table.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StubVerdict {
    pub ok: bool,
    pub stdout: String,
    pub stderr: String,
}

impl StubVerdict {
    fn pass(msg: impl Into<String>) -> Self {
        Self {
            ok: true,
            stdout: format!("[stub] {}", msg.into()),
            stderr: String::new(),
        }
    }
    fn fail(msg: impl Into<String>) -> Self {
        Self {
            ok: false,
            stdout: String::new(),
            stderr: format!("[stub] {}", msg.into()),
        }
    }
}

/// One ordered grading step: satisfied when the source contains ANY of
/// `any_of`; otherwise the mission fails with `"{npc} — {hint}"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Check {
    pub any_of: &'static [&'static str],
    pub hint: &'static str,
}

/// Shorthand constructor so table entries stay on one line each.
pub const fn check(any_of: &'static [&'static str], hint: &'static str) -> Check {
    Check { any_of, hint }
}

/// The full grading rule for one mission. See the module docs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MissionRule {
    pub id: &'static str,
    pub npc: &'static str,
    pub checks: &'static [Check],
    pub pass: &'static str,
}

impl MissionRule {
    /// Run the checks in order. `Ok(pass message)` or `Err(fail message)`.
    pub fn evaluate(&self, source: &str) -> Result<&'static str, String> {
        match self
            .checks
            .iter()
            .find(|c| !c.any_of.iter().any(|tok| source.contains(tok)))
        {
            Some(failed) => Err(format!("{} — {}", self.npc, failed.hint)),
            None => Ok(self.pass),
        }
    }
}

/// Look up the rule for an encounter id.
pub fn mission_rule(encounter_id: &str) -> Option<&'static MissionRule> {
    MISSION_RULES.iter().find(|r| r.id == encounter_id)
}

/// Re-implementation of the server's per-encounter pattern checks.
///
/// Returns `Some(verdict)` for any mission currently shipped in
/// `MissionRegistry::default()`, or `None` for an unknown encounter id
/// (the caller falls back to the existing `[client error]` path so
/// unexpected ids aren't silently graded as a freeform pass). The
/// `stub_covers_every_mission_in_the_registry` test in `tests/stub_grader.rs`
/// will scream if a new mission is added without a table entry here.
pub fn stub_verdict(encounter_id: &str, source: &str) -> Option<StubVerdict> {
    let Some(rule) = mission_rule(encounter_id) else {
        tracing::debug!(encounter_id, "stub grader: unknown encounter id");
        return None;
    };
    let verdict = match rule.evaluate(source) {
        Ok(msg) => StubVerdict::pass(msg),
        Err(msg) => StubVerdict::fail(msg),
    };
    tracing::debug!(
        encounter_id,
        ok = verdict.ok,
        bytes = source.len(),
        "stub grader verdict"
    );
    Some(verdict)
}

// BEGIN MISSION TABLE — must stay byte-identical to compile-api/src/grader.rs.
#[rustfmt::skip]
pub static MISSION_RULES: &[MissionRule] = &[
    // ── Act 1 (Ferris's Village).
    MissionRule {
        id: "intro_let_binding",
        npc: "not yet",
        checks: &[
            check(&["let answer"], "missing required: `let answer`"),
            check(&["42"], "missing required: `42`"),
        ],
        pass: "answer bound. Ferris nods approvingly.",
    },
    MissionRule {
        id: "double_function",
        npc: "not yet",
        checks: &[
            check(&["fn double"], "missing required: `fn double`"),
            check(&["i32"], "missing required: `i32`"),
            check(&["* 2", "*2"], "missing required: `*2`"),
        ],
        pass: "the Trait Mage's staff hums. \"named once, doubled twice over.\"",
    },
    MissionRule {
        id: "borrow_preview",
        npc: "the Borrow Checker is silent",
        checks: &[
            check(&["&value"], "missing required: `&value`"),
            check(&["println!"], "missing required: `println!`"),
        ],
        pass: "the Borrow Checker stirs. \"...acceptable. for now.\"",
    },
    MissionRule {
        id: "mut_binding",
        npc: "the Smith eyes you",
        checks: &[
            check(&["let mut"], "missing required: `let mut`"),
            check(&["+= 1", "+=1"], "missing required: `+= 1`"),
        ],
        pass: "the Smith strikes the work. \"good — that one bends.\"",
    },
    MissionRule {
        id: "if_else_sign",
        npc: "the Cartographer frowns",
        checks: &[
            check(&["if "], "missing required: `if `"),
            check(&["else"], "missing required: `else`"),
            check(&["< 0", "<0"], "missing required: `< 0`"),
        ],
        pass: "the Cartographer nods. \"three roads diverge — well chosen.\"",
    },
    MissionRule {
        id: "loop_break",
        npc: "the Bellringer waits",
        checks: &[
            check(&["loop"], "missing required: `loop`"),
            check(&["break"], "missing required: `break`"),
            check(&["*= 2", "*=2"], "the value should double each iteration: `*= 2`"),
        ],
        pass: "the Bellringer pulls the rope — the bell sounds. \"the loop returned its number.\"",
    },
    MissionRule {
        id: "match_option",
        npc: "the Oracle's eyes are closed",
        checks: &[
            check(&["match"], "missing required: `match`"),
            check(&["Some("], "missing required: `Some(`"),
            check(&["None"], "missing required: `None`"),
        ],
        pass: "the Oracle exhales. \"both paths walked. nothing slips through.\"",
    },
    MissionRule {
        id: "vec_iter",
        npc: "the Cooper waits",
        checks: &[
            check(&["vec!"], "missing required: `vec!`"),
            check(&[".iter()"], "call `.iter()` on the vector"),
            check(&[".sum"], "finish with `.sum()` to reduce"),
        ],
        pass: "the Cooper hoops the barrel. \"every stave counted, sum sealed.\"",
    },
    MissionRule {
        id: "tuple_destructure",
        npc: "the Twin waits",
        checks: &[
            check(&["let ("], "missing required: `let (`"),
            check(&[","], "a tuple needs a comma between its parts"),
            check(&[") =", ")="], "close the pattern with `) =`"),
        ],
        pass: "the Twin grins. \"two names, one breath. cleanly split.\"",
    },
    MissionRule {
        id: "while_loop",
        npc: "the Tinker frowns",
        checks: &[
            check(&["while "], "missing required: `while `"),
            check(&["> 0", ">0"], "the predicate should test `> 0`"),
            check(&["-= 1", "-=1"], "decrement with `-= 1` each iteration"),
        ],
        pass: "the Tinker pockets the spring. \"counted down, gear by gear.\"",
    },
    MissionRule {
        id: "struct_basic",
        npc: "the Herald squints",
        checks: &[
            check(&["struct Knight"], "missing required: `struct Knight`"),
            check(&["name:"], "Knight needs a `name:` field"),
            check(&["hp:"], "Knight needs an `hp:` field"),
            check(&[".name"], "read the field with `.name`"),
        ],
        pass: "the Herald reads the tabard. \"by name and by number — both fields announced.\"",
    },
    // ── Act 2 (Borrow Checker's Bridge).
    MissionRule {
        id: "borrow_mut",
        npc: "the Forgewright glares",
        checks: &[
            check(&["fn bump"], "missing required: `fn bump`"),
            check(&["&mut i32"], "the parameter must be `&mut i32` (exclusive borrow)"),
            check(&["*x"], "write through the reference with a `*x` deref"),
        ],
        pass: "the Forgewright nods. \"one writer at the forge — the borrow holds.\"",
    },
    MissionRule {
        id: "string_vs_str",
        npc: "the Linguist tilts her head",
        checks: &[
            check(&["fn greet"], "missing required: `fn greet`"),
            check(&["&str"], "the parameter type should be `&str`"),
            check(&["String::from"], "call `greet` once with a `String::from(...)` value"),
        ],
        pass: "the Linguist smiles. \"one tongue, two voices — &str carries both.\"",
    },
    MissionRule {
        id: "option_unwrap_or",
        npc: "the Pilgrim shakes his head",
        checks: &[
            check(&["Option<"], "missing required: `Option<`"),
            check(&[".unwrap_or("], "collapse the absent case with `.unwrap_or(default)`"),
        ],
        pass: "the Pilgrim raises his lantern. \"the absent path lit a default.\"",
    },
    MissionRule {
        id: "for_in_range",
        npc: "the Drillmaster barks",
        checks: &[
            check(&["for "], "missing required: `for `"),
            check(&[" in "], "`for` needs a binding `in` an iterable"),
            check(&["0..10", "0 .. 10"], "iterate the exact range `0..10`"),
        ],
        pass: "the Drillmaster claps once. \"ten paces, exact and ordered.\"",
    },
    MissionRule {
        id: "closure_basic",
        npc: "the Reckoner's quill hovers",
        checks: &[
            check(&["let add"], "missing required: `let add`"),
            check(&["= |"], "bind a closure literal with `= |...|`"),
            check(&["+ b", "+b"], "the body should compute `a + b`"),
        ],
        pass: "the Reckoner inks the ledger. \"summed in a single stroke.\"",
    },
    MissionRule {
        id: "slice_basic",
        npc: "the Quartermaster squints",
        checks: &[
            check(&["fn sum_slice"], "missing required: `fn sum_slice`"),
            check(&["&[i32]"], "the parameter must be a slice `&[i32]`"),
            check(&[".iter()"], "walk the slice with `.iter()`"),
        ],
        pass: "the Quartermaster ticks the manifest. \"a window into the stores — counted, not claimed.\"",
    },
    MissionRule {
        id: "result_question_mark",
        npc: "the Auditor's pen taps",
        checks: &[
            check(&["Result<"], "missing required: `Result<`"),
            check(&[".parse"], "call `.parse` on a string source"),
            check(&["?;", "?\n"], "propagate the error with the `?` operator"),
        ],
        pass: "the Auditor marks the tally. \"errors travel upward — the books balance.\"",
    },
    MissionRule {
        id: "derive_debug",
        npc: "the Chronicler shakes his head",
        checks: &[
            check(&["#[derive(Debug)]"], "missing required: `#[derive(Debug)]`"),
            check(&["struct Item"], "define `struct Item`"),
            check(&[":?"], "print with the debug formatter `{:?}`"),
        ],
        pass: "the Chronicler dips his quill. \"derived, not authored.\"",
    },
    MissionRule {
        id: "iter_map_collect",
        npc: "the Alchemist stirs",
        checks: &[
            check(&[".map("], "missing required: `.map(`"),
            check(&[".collect"], "finish with `.collect` to gather the result"),
            check(&["|x|", "|x |"], "the closure should bind one element as `|x|`"),
        ],
        pass: "the Alchemist decants the flask. \"every drop transmuted.\"",
    },
    MissionRule {
        id: "enum_match",
        npc: "the Heraldic Sage frowns",
        checks: &[
            check(&["enum Direction"], "missing required: `enum Direction`"),
            check(&["match "], "inspect the variant with a `match ` expression"),
            check(&["Direction::"], "name a variant with the `Direction::` path"),
        ],
        pass: "the Heraldic Sage walks the sash. \"every variant a panel, every arm a blazon read.\"",
    },
    // ── Act 3 (Guildhall Quarter).
    MissionRule {
        id: "impl_method",
        npc: "the Guildmaster waits",
        checks: &[
            check(&["impl"], "missing required: `impl`"),
            check(&["&self"], "a method borrows the instance with `&self`"),
            check(&["self."], "read the instance's data with `self.`"),
        ],
        pass: "the Guildmaster nods. \"duties bound to a name — that is a method.\"",
    },
    MissionRule {
        id: "assoc_new",
        npc: "the Recruiter shakes her head",
        checks: &[
            check(&["fn new"], "missing required: `fn new`"),
            check(&["Self"], "a constructor returns `Self`"),
            check(&["::new"], "call it with the `::new` path"),
        ],
        pass: "the Recruiter stamps the roll. \"Self::new — a member forged from a name.\"",
    },
    MissionRule {
        id: "if_let",
        npc: "the Locksmith frowns",
        checks: &[
            check(&["if let"], "missing required: `if let`"),
            check(&["Some("], "match the present case with `Some(`"),
        ],
        pass: "the Locksmith turns the one key that fits. \"Some — and only Some — opens.\"",
    },
    MissionRule {
        id: "while_let",
        npc: "the Porter sets the crate down",
        checks: &[
            check(&["while let"], "missing required: `while let`"),
            check(&[".pop("], "drain the stack with `.pop()`"),
        ],
        pass: "the Porter empties the cart, crate by crate. \"while there is a Some, keep unloading.\"",
    },
    MissionRule {
        id: "tuple_struct",
        npc: "the Surveyor squints down the line",
        checks: &[
            check(&["struct Meters("], "missing required: `struct Meters(`"),
            check(&[".0"], "read the wrapped value with `.0`"),
        ],
        pass: "the Surveyor reads the rod. \"a bare number, now named Meters — a newtype.\"",
    },
    MissionRule {
        id: "enum_data_match",
        npc: "the Armorer taps the anvil",
        checks: &[
            check(&["enum Item"], "missing required: `enum Item`"),
            check(&["Weapon"], "give Item a `Weapon` variant"),
            check(&["match"], "inspect the item with `match`"),
            check(&["=>"], "each variant needs a `=>` arm"),
        ],
        pass: "the Armorer lays out the rack. \"weapon or potion — name every kind, miss none.\"",
    },
    // ── Act 4 (Trait Mage's Tower).
    MissionRule {
        id: "trait_def",
        npc: "Vexis waits",
        checks: &[
            check(&["trait "], "missing required: a `trait ` definition"),
            check(&["impl "], "implement it with an `impl ` block"),
            check(&[" for "], "bind the trait to a type: `impl Trait for Type`"),
        ],
        pass: "Vexis lowers his staff. \"a capability named once, granted to a type — that is a trait.\"",
    },
    MissionRule {
        id: "generic_fn",
        npc: "the Wandwright frowns",
        checks: &[
            check(&["<T"], "missing required: a type parameter `<T`"),
            check(&["PartialOrd"], "bound the parameter so it can be compared: `T: PartialOrd`"),
        ],
        pass: "the Wandwright sights down the blank. \"one wand, any element — bounded by what it can compare.\"",
    },
    MissionRule {
        id: "generic_struct",
        npc: "the Conjurer waits",
        checks: &[
            check(&["struct "], "missing required: a `struct ` definition"),
            check(&["<T>"], "make it generic with a `<T>` parameter"),
            check(&[": T"], "give it a field of the generic type `: T`"),
        ],
        pass: "the Conjurer cups two matching lights. \"a vessel for any type — so long as both halves agree.\"",
    },
    MissionRule {
        id: "dyn_trait",
        npc: "the Familiar flickers",
        checks: &[
            check(&["Box<dyn"], "missing required: `Box<dyn ...>`"),
            check(&["Box::new"], "box each value onto the heap with `Box::new`"),
        ],
        pass: "the Familiar shifts through its forms. \"many shapes, one cage — dispatched at a touch.\"",
    },
    MissionRule {
        id: "lifetimes",
        npc: "the Lanternkeeper waits",
        checks: &[
            check(&["<'a>"], "missing required: a lifetime parameter `<'a>`"),
            check(&["&'a"], "annotate the references with `&'a`"),
        ],
        pass: "the Lanternkeeper keeps the flame. \"the borrow lives exactly as long as 'a — no shorter.\"",
    },
    MissionRule {
        id: "assoc_type",
        npc: "the Loremaster turns the page",
        checks: &[
            check(&["type Output"], "missing required: `type Output`"),
            check(&["Self::Output"], "refer to it as `Self::Output` in the method"),
        ],
        pass: "the Loremaster turns the page. \"each producer names its own yield — the type follows the trait.\"",
    },
    // ── Act 5 (Tavern of Tribulations).
    MissionRule {
        id: "result_match",
        npc: "the Barkeep waits",
        checks: &[
            check(&["match "], "missing required: `match `"),
            check(&["Ok("], "handle the success arm `Ok(`"),
            check(&["Err("], "handle the failure arm `Err(`"),
        ],
        pass: "the Barkeep nods. \"poured or spilled — you accounted for both.\"",
    },
    MissionRule {
        id: "custom_error",
        npc: "the Bouncer crosses his arms",
        checks: &[
            check(&["enum "], "missing required: an error `enum `"),
            check(&["Result<"], "return a `Result<...>` from check"),
            check(&["Err("], "produce a failure with `Err(...)`"),
        ],
        pass: "the Bouncer grunts. \"every kind of trouble, named and on the list.\"",
    },
    MissionRule {
        id: "from_error",
        npc: "the Interpreter pauses",
        checks: &[
            check(&["impl From<"], "missing required: `impl From<...>`"),
            check(&[" for "], "implement it `for` your error type"),
            check(&["fn from"], "the conversion is a `fn from` method"),
        ],
        pass: "the Interpreter inclines her head. \"one tongue's failure, spoken in another.\"",
    },
    MissionRule {
        id: "option_map",
        npc: "the Mixologist waits",
        checks: &[
            check(&["Option<"], "keep it an `Option<...>`"),
            check(&[".map("], "transform the inside with `.map(`"),
        ],
        pass: "the Mixologist swirls the glass. \"changed within — if there was anything to change.\"",
    },
    MissionRule {
        id: "and_then",
        npc: "the Tabkeeper waits",
        checks: &[
            check(&["Option"], "the steps should yield an `Option`"),
            check(&[".and_then("], "chain the fallible steps with `.and_then(`"),
        ],
        pass: "the Tabkeeper closes the ledger. \"one failed round and the tab is cut.\"",
    },
    MissionRule {
        id: "unwrap_or_else",
        npc: "the Cellarer waits",
        checks: &[
            check(&[".unwrap_or_else("], "missing required: `.unwrap_or_else(`"),
            check(&["||"], "the lazy default is a closure `|| ...`"),
        ],
        pass: "the Cellarer taps the cask. \"a fresh draught — drawn only when the old runs dry.\"",
    },
    // ── Act 6 (Iterator Forge).
    MissionRule {
        id: "hashmap_basic",
        npc: "the Keymaster waits",
        checks: &[
            check(&["HashMap"], "missing required: `HashMap`"),
            check(&[".insert("], "store a value with `.insert(`"),
            check(&[".get("], "look it up with `.get(`"),
        ],
        pass: "the Keymaster turns the ring. \"every store under its own key.\"",
    },
    MissionRule {
        id: "iter_filter",
        npc: "the Sifter shakes the screen",
        checks: &[
            check(&[".filter("], "missing required: `.filter(`"),
            check(&[".collect"], "gather the survivors with `.collect`"),
        ],
        pass: "the Sifter shakes the screen. \"only what fits the mesh falls through.\"",
    },
    MissionRule {
        id: "iter_fold",
        npc: "the Smelter waits",
        checks: &[
            check(&[".fold("], "missing required: `.fold(`"),
        ],
        pass: "the Smelter tips the crucible. \"many ores, one ingot.\"",
    },
    MissionRule {
        id: "iter_enumerate",
        npc: "the Tallywright waits",
        checks: &[
            check(&[".enumerate("], "missing required: `.enumerate(`"),
        ],
        pass: "the Tallywright stamps the row. \"each item numbered as it passes.\"",
    },
    MissionRule {
        id: "iter_zip",
        npc: "the Riveter waits",
        checks: &[
            check(&[".zip("], "missing required: `.zip(`"),
        ],
        pass: "the Riveter drives the rivet. \"two plates, aligned pair by pair.\"",
    },
    MissionRule {
        id: "closure_move",
        npc: "the Bondsmith waits",
        checks: &[
            check(&["move |"], "missing required: a `move` closure (`move |...|`)"),
        ],
        pass: "the Bondsmith seals the charge. \"owned, and carried where I go.\"",
    },
    // ── Act 7 (Concurrent Coast).
    MissionRule {
        id: "thread_spawn",
        npc: "the Dockmaster waits",
        checks: &[
            check(&["thread::spawn"], "missing required: `thread::spawn`"),
            check(&[".join("], "wait for the worker with `.join(`"),
        ],
        pass: "the Dockmaster waves them off. \"sent to work — and met again at the gate.\"",
    },
    MissionRule {
        id: "arc_mutex",
        npc: "the Lighthouse Keeper waits",
        checks: &[
            check(&["Arc"], "share ownership with `Arc`"),
            check(&["Mutex"], "guard the value with `Mutex`"),
            check(&[".lock("], "take exclusive access with `.lock(`"),
        ],
        pass: "the Lighthouse Keeper turns the key. \"one hand on the lamp at a time.\"",
    },
    MissionRule {
        id: "mpsc_channel",
        npc: "the Signaler waits",
        checks: &[
            check(&["mpsc::channel"], "missing required: `mpsc::channel`"),
            check(&[".send("], "push a value in with `.send(`"),
            check(&[".recv("], "read it at the far end with `.recv(`"),
        ],
        pass: "the Signaler flashes the lamp. \"sent down the coast, read at the next tower.\"",
    },
    MissionRule {
        id: "atomic",
        npc: "the Tidewatch waits",
        checks: &[
            check(&["Atomic"], "use an `Atomic` type"),
            check(&[".fetch_add("], "bump it atomically with `.fetch_add(`"),
        ],
        pass: "the Tidewatch clicks the gauge. \"one notch up — no gatekeeper.\"",
    },
    MissionRule {
        id: "thread_scope",
        npc: "the Harbormaster waits",
        checks: &[
            check(&["thread::scope"], "missing required: `thread::scope`"),
            check(&[".spawn("], "launch a scoped thread with `.spawn(`"),
        ],
        pass: "the Harbormaster rings the bell. \"every boat in before the harbor closes.\"",
    },
    MissionRule {
        id: "async_fn",
        npc: "the Tideforecaster waits",
        checks: &[
            check(&["async fn"], "missing required: `async fn`"),
            check(&[".await"], "drive the future with `.await`"),
        ],
        pass: "the Tideforecaster lowers the glass. \"the tide will come — awaited, not forced.\"",
    },
    // ── Act 8 (Vault of Pointers).
    MissionRule {
        id: "box_basic",
        npc: "the Vaultwright waits",
        checks: &[
            check(&["Box<"], "missing required: `Box<...>`"),
            check(&["Box::new"], "put the value on the heap with `Box::new`"),
        ],
        pass: "the Vaultwright seals the box. \"on the heap it holds what the stack cannot.\"",
    },
    MissionRule {
        id: "rc_basic",
        npc: "the Sharekeeper waits",
        checks: &[
            check(&["Rc::new"], "missing required: `Rc::new`"),
            check(&["Rc::clone"], "hand out a second handle with `Rc::clone`"),
        ],
        pass: "the Sharekeeper tallies the claims. \"one hoard, many holders.\"",
    },
    MissionRule {
        id: "refcell",
        npc: "the Warden waits",
        checks: &[
            check(&["RefCell"], "missing required: `RefCell`"),
            check(&[".borrow_mut("], "mutate the inner value with `.borrow_mut(`"),
        ],
        pass: "the Warden checks the ledger. \"changed within — by the runtime's leave.\"",
    },
    MissionRule {
        id: "cell",
        npc: "the Swapwarden waits",
        checks: &[
            check(&["Cell::new"], "missing required: `Cell::new`"),
            check(&[".set("], "swap the whole value with `.set(`"),
        ],
        pass: "the Swapwarden trades the coin. \"the whole value, swapped clean.\"",
    },
    MissionRule {
        id: "rc_refcell",
        npc: "the Strongbox waits",
        checks: &[
            check(&["Rc::new"], "share ownership with `Rc::new`"),
            check(&["RefCell"], "make it mutable inside with `RefCell`"),
            check(&[".borrow_mut("], "open and change it via `.borrow_mut(`"),
        ],
        pass: "the Strongbox turns the keys. \"many owners, and the box still opens.\"",
    },
    MissionRule {
        id: "weak_ref",
        npc: "the Ghostkeeper waits",
        checks: &[
            check(&["Weak"], "missing required: a `Weak` reference"),
            check(&["downgrade"], "make the weak handle with `Rc::downgrade`"),
            check(&[".upgrade("], "reclaim it (or not) with `.upgrade(`"),
        ],
        pass: "the Ghostkeeper tugs the tether. \"a hold that holds nothing alive.\"",
    },
];
// END MISSION TABLE

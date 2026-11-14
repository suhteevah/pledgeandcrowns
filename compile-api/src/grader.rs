// SPDX-License-Identifier: AGPL-3.0-or-later
//! Per-encounter source-code grader. Pure function, fully unit-tested.
//!
//! v0 is pattern-matching; v1 will swap the body of `grade` for a
//! real `cargo build --offline` + wasmtime invocation per
//! `design/05-tech-architecture.md` §2 — the test suite below stays
//! valid because it only exercises the public contract.
//!
//! The rule table is duplicated on purpose in the MIT-licensed game crate
//! (`game/src/plugins/stub_grader.rs`, the offline / wasm grader); the two
//! crates must not share code. Keep the region between the
//! `BEGIN MISSION TABLE` / `END MISSION TABLE` markers TEXTUALLY
//! IDENTICAL in both files — `game/tests/contract.rs` enforces textual
//! parity and byte-for-byte output parity.
//!
//! # Adding a mission
//!
//! Add ONE [`MissionRule`] entry to [`MISSION_RULES`] here, and the
//! identical entry to `MISSION_RULES` in `game/src/plugins/stub_grader.rs`:
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
pub struct Verdict {
    pub ok: bool,
    pub stdout: String,
    pub stderr: String,
}

impl Verdict {
    pub fn pass(msg: impl Into<String>) -> Self {
        Self {
            ok: true,
            stdout: msg.into(),
            stderr: String::new(),
        }
    }
    pub fn fail(msg: impl Into<String>) -> Self {
        Self {
            ok: false,
            stdout: String::new(),
            stderr: msg.into(),
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

/// Grade `source` for `encounter_id`. Unknown ids get a freeform pass so
/// unshipped encounters stay playable against the server.
pub fn grade(encounter_id: &str, source: &str) -> Verdict {
    let Some(rule) = mission_rule(encounter_id) else {
        tracing::debug!(encounter_id, "grader: unknown encounter id, freeform pass");
        return Verdict::pass(format!(
            "[freeform] received {} bytes. encounter `{encounter_id}` has no grader yet.",
            source.len()
        ));
    };
    let verdict = match rule.evaluate(source) {
        Ok(msg) => Verdict::pass(msg),
        Err(msg) => Verdict::fail(msg),
    };
    tracing::debug!(
        encounter_id,
        ok = verdict.ok,
        bytes = source.len(),
        "grader verdict"
    );
    verdict
}

// BEGIN MISSION TABLE — must stay byte-identical to game/src/plugins/stub_grader.rs.
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

#[cfg(test)]
mod tests {
    use super::*;

    // ── rule table shape ───────────────────────────────────────────

    #[test]
    fn rule_ids_are_unique_and_every_rule_has_checks() {
        let mut seen = std::collections::BTreeSet::new();
        for rule in MISSION_RULES {
            assert!(seen.insert(rule.id), "duplicate rule id `{}`", rule.id);
            assert!(!rule.checks.is_empty(), "rule `{}` has no checks", rule.id);
            assert!(
                rule.checks.iter().all(|c| !c.any_of.is_empty()),
                "rule `{}` has an empty check",
                rule.id
            );
        }
    }

    // ── intro_let_binding ──────────────────────────────────────────

    #[test]
    fn intro_pass_canonical() {
        let src = r#"fn main() {
    let answer = 42;
    println!("{answer}");
}"#;
        let v = grade("intro_let_binding", src);
        assert!(v.ok, "expected pass, got {v:?}");
        assert!(v.stdout.contains("Ferris"));
    }

    #[test]
    fn intro_pass_with_type_annotation() {
        let src = "fn main() { let answer: i32 = 42; }";
        assert!(grade("intro_let_binding", src).ok);
    }

    #[test]
    fn intro_fail_missing_value() {
        let src = "fn main() { let answer = 7; }";
        let v = grade("intro_let_binding", src);
        assert!(!v.ok);
        assert!(
            v.stderr.contains("42"),
            "stderr should name the missing token, got {}",
            v.stderr
        );
    }

    #[test]
    fn intro_fail_wrong_name() {
        let src = "fn main() { let x = 42; }";
        let v = grade("intro_let_binding", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("let answer"));
    }

    #[test]
    fn intro_fail_empty() {
        let v = grade("intro_let_binding", "");
        assert!(!v.ok);
    }

    // ── double_function ────────────────────────────────────────────

    #[test]
    fn double_pass_canonical() {
        let src = "fn double(n: i32) -> i32 { n * 2 }";
        assert!(grade("double_function", src).ok);
    }

    #[test]
    fn double_pass_no_space_in_multiply() {
        let src = "fn double(n: i32) -> i32 { n*2 }";
        assert!(grade("double_function", src).ok, "should accept `n*2`");
    }

    #[test]
    fn double_fail_addition() {
        let src = "fn double(n: i32) -> i32 { n + n }";
        assert!(!grade("double_function", src).ok);
    }

    #[test]
    fn double_fail_wrong_type() {
        let src = "fn double(n: u32) -> u32 { n * 2 }";
        let v = grade("double_function", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("i32"));
    }

    #[test]
    fn double_fail_wrong_name() {
        let src = "fn doubl(n: i32) -> i32 { n * 2 }";
        assert!(!grade("double_function", src).ok);
    }

    // ── borrow_preview ─────────────────────────────────────────────

    #[test]
    fn borrow_pass_canonical() {
        let src = r#"fn main() {
    let value = String::from("borrow me");
    let r = &value;
    println!("{r}");
}"#;
        assert!(grade("borrow_preview", src).ok);
    }

    #[test]
    fn borrow_fail_no_borrow() {
        let src = r#"fn main() {
    let value = String::from("borrow me");
    let r = value;
    println!("{r}");
}"#;
        let v = grade("borrow_preview", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("&value"));
    }

    #[test]
    fn borrow_fail_no_println() {
        let src = "fn main() { let value = 1; let _r = &value; }";
        let v = grade("borrow_preview", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("println!"));
    }

    // ── unknown encounter falls back to freeform ───────────────────

    #[test]
    fn unknown_encounter_passes_with_freeform_marker() {
        let v = grade("totally_made_up_id", "fn main() {}");
        assert!(v.ok);
        assert!(v.stdout.contains("freeform"));
        assert!(v.stdout.contains("totally_made_up_id"));
    }

    // ── Sanity: solving one mission must not satisfy another ───────

    #[test]
    fn solutions_are_not_cross_compatible() {
        let intro_solution = "fn main() { let answer = 42; }";
        assert!(!grade("double_function", intro_solution).ok);
        assert!(!grade("borrow_preview", intro_solution).ok);

        let double_solution = "fn double(n: i32) -> i32 { n * 2 }";
        assert!(!grade("intro_let_binding", double_solution).ok);
        assert!(!grade("borrow_preview", double_solution).ok);

        let borrow_solution = r#"fn main() { let value = 1; let r = &value; println!("{r}"); }"#;
        assert!(!grade("intro_let_binding", borrow_solution).ok);
        assert!(!grade("double_function", borrow_solution).ok);

        let loop_solution =
            "fn main() { let mut n = 1; loop { if n >= 100 { break n; } n *= 2; }; }";
        assert!(!grade("intro_let_binding", loop_solution).ok);
        assert!(!grade("match_option", loop_solution).ok);

        let match_solution = "fn f(x: Option<i32>) -> i32 { match x { Some(n) => n, None => 0 } }";
        assert!(!grade("loop_break", match_solution).ok);
        assert!(!grade("struct_basic", match_solution).ok);

        let struct_solution = "struct Knight { name: String, hp: i32 } fn main() { let k = Knight { name: String::new(), hp: 0 }; let _ = k.name; }";
        assert!(!grade("loop_break", struct_solution).ok);
        assert!(!grade("match_option", struct_solution).ok);

        let vec_solution = "fn main() { let v = vec![1,2]; let _: i32 = v.iter().sum(); }";
        assert!(!grade("tuple_destructure", vec_solution).ok);
        assert!(!grade("while_loop", vec_solution).ok);

        let tuple_solution = "fn main() { let (a, b) = (1, 2); let _ = (a, b); }";
        assert!(!grade("vec_iter", tuple_solution).ok);
        assert!(!grade("while_loop", tuple_solution).ok);

        let while_solution = "fn main() { let mut n = 3; while n > 0 { n -= 1; } let _ = n; }";
        assert!(!grade("vec_iter", while_solution).ok);
        assert!(!grade("tuple_destructure", while_solution).ok);

        // ── Act 2 additions ──
        let borrow_mut_solution = "fn bump(x: &mut i32) { *x = 99; }";
        assert!(!grade("borrow_preview", borrow_mut_solution).ok);
        assert!(!grade("mut_binding", borrow_mut_solution).ok);
        assert!(!grade("string_vs_str", borrow_mut_solution).ok);

        let str_solution = r#"fn greet(name: &str) { let _ = name; } fn main() { let s = String::from("x"); greet(&s); }"#;
        assert!(!grade("borrow_preview", str_solution).ok);
        assert!(!grade("borrow_mut", str_solution).ok);
        assert!(!grade("option_unwrap_or", str_solution).ok);

        let unwrap_or_solution = "fn safe(x: Option<i32>) -> i32 { x.unwrap_or(0) }";
        assert!(!grade("match_option", unwrap_or_solution).ok);
        assert!(!grade("for_in_range", unwrap_or_solution).ok);

        let for_solution = "fn main() { for i in 0..10 { let _ = i; } }";
        assert!(!grade("while_loop", for_solution).ok);
        assert!(!grade("loop_break", for_solution).ok);
        assert!(!grade("closure_basic", for_solution).ok);

        let closure_solution = "fn main() { let add = |a, b| a + b; let _ = add(1, 2); }";
        assert!(!grade("intro_let_binding", closure_solution).ok);
        assert!(!grade("for_in_range", closure_solution).ok);
        assert!(!grade("borrow_mut", closure_solution).ok);

        // Existing solutions must not pass new graders.
        assert!(!grade("borrow_mut", borrow_solution).ok);
        assert!(!grade("borrow_mut", "fn main() { let mut x = 0; x += 1; }").ok);
        assert!(!grade("string_vs_str", borrow_solution).ok);
        assert!(!grade("option_unwrap_or", match_solution).ok);
        assert!(!grade("for_in_range", while_solution).ok);
        assert!(!grade("closure_basic", "fn main() { let answer = 42; }").ok);

        // ── Act 2 wave 2 additions ──
        let slice_solution = "fn sum_slice(xs: &[i32]) -> i32 { xs.iter().sum() }";
        assert!(!grade("vec_iter", slice_solution).ok);
        assert!(!grade("borrow_mut", slice_solution).ok);
        assert!(!grade("string_vs_str", slice_solution).ok);

        let result_qm_solution = "fn parse_int(s: &str) -> Result<i32, String> { let n = s.parse::<i32>().map_err(|e| e.to_string())?; Ok(n) }";
        assert!(!grade("option_unwrap_or", result_qm_solution).ok);
        assert!(!grade("match_option", result_qm_solution).ok);

        let derive_debug_solution = "#[derive(Debug)] struct Item { name: String } fn main() { let i = Item { name: String::new() }; println!(\"{i:?}\"); }";
        assert!(!grade("struct_basic", derive_debug_solution).ok);
        assert!(!grade("string_vs_str", derive_debug_solution).ok);

        let iter_map_solution = "fn main() { let v = vec![1, 2]; let _: Vec<i32> = v.iter().map(|x| x * 2).collect(); }";
        assert!(!grade("vec_iter", iter_map_solution).ok);
        assert!(!grade("closure_basic", iter_map_solution).ok);

        let enum_match_solution = "enum Direction { N, S } fn f(d: Direction) -> i32 { match d { Direction::N => 1, Direction::S => 0 } }";
        assert!(!grade("match_option", enum_match_solution).ok);
        assert!(!grade("struct_basic", enum_match_solution).ok);

        // Existing solutions must not pass the new graders.
        assert!(!grade("slice_basic", borrow_solution).ok);
        assert!(!grade("slice_basic", vec_solution).ok);
        assert!(!grade("result_question_mark", match_solution).ok);
        assert!(!grade("derive_debug", struct_solution).ok);
        assert!(!grade("iter_map_collect", vec_solution).ok);
        assert!(!grade("iter_map_collect", closure_solution).ok);
        assert!(!grade("enum_match", match_solution).ok);
        assert!(!grade("enum_match", struct_solution).ok);
    }

    // ── mut_binding ────────────────────────────────────────────────

    #[test]
    fn mut_pass_canonical() {
        let src = "fn main() { let mut x = 0; x += 1; }";
        assert!(grade("mut_binding", src).ok);
    }

    #[test]
    fn mut_pass_no_space() {
        let src = "fn main() { let mut x = 0; x +=1; }";
        assert!(grade("mut_binding", src).ok);
    }

    #[test]
    fn mut_fail_no_mut_keyword() {
        let src = "fn main() { let x = 0; let y = x + 1; }";
        assert!(!grade("mut_binding", src).ok);
    }

    #[test]
    fn mut_fail_no_increment() {
        let src = "fn main() { let mut x = 0; }";
        let v = grade("mut_binding", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("+= 1"));
    }

    // ── if_else_sign ──────────────────────────────────────────────

    #[test]
    fn if_else_pass_canonical() {
        let src = r#"fn sign(n: i32) -> &'static str { if n < 0 { "neg" } else if n == 0 { "zero" } else { "pos" } }"#;
        assert!(grade("if_else_sign", src).ok);
    }

    #[test]
    fn if_else_fail_no_branch() {
        let src = "fn sign(_: i32) -> &'static str { \"pos\" }";
        assert!(!grade("if_else_sign", src).ok);
    }

    #[test]
    fn if_else_fail_no_negative_check() {
        let src = "fn main() { if true { } else { } }";
        let v = grade("if_else_sign", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("< 0"));
    }

    // ── loop_break ────────────────────────────────────────────────

    #[test]
    fn loop_break_pass_canonical() {
        let src = r#"fn main() {
    let mut n = 1;
    let result = loop {
        if n >= 100 { break n; }
        n *= 2;
    };
    println!("{result}");
}"#;
        assert!(grade("loop_break", src).ok);
    }

    #[test]
    fn loop_break_pass_no_space_in_multiply() {
        let src = "fn main() { let mut n = 1; loop { if n >= 100 { break; } n *=2; } }";
        assert!(grade("loop_break", src).ok);
    }

    #[test]
    fn loop_break_fail_no_loop() {
        let src = "fn main() { let n = 128; println!(\"{n}\"); }";
        let v = grade("loop_break", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("loop"));
    }

    #[test]
    fn loop_break_fail_no_doubling() {
        let src = "fn main() { let mut n = 0; loop { if n >= 100 { break; } n += 1; } }";
        let v = grade("loop_break", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("*= 2"));
    }

    // ── match_option ──────────────────────────────────────────────

    #[test]
    fn match_option_pass_canonical() {
        let src = r#"fn unwrap_or_zero(x: Option<i32>) -> i32 {
    match x {
        Some(n) => n,
        None => 0,
    }
}"#;
        assert!(grade("match_option", src).ok);
    }

    #[test]
    fn match_option_fail_no_match() {
        let src = "fn f(x: Option<i32>) -> i32 { x.unwrap_or(0) }";
        let v = grade("match_option", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("match"));
    }

    #[test]
    fn match_option_fail_no_some_arm() {
        let src = "fn f(x: Option<i32>) -> i32 { match x { _ => 0 } }";
        let v = grade("match_option", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("Some("));
    }

    #[test]
    fn match_option_fail_no_none_arm() {
        let src = "fn f(x: Option<i32>) -> i32 { match x { Some(n) => n, _ => 0 } }";
        let v = grade("match_option", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("None"));
    }

    // ── struct_basic ──────────────────────────────────────────────

    #[test]
    fn struct_pass_canonical() {
        let src = r#"struct Knight { name: String, hp: i32 }

fn main() {
    let k = Knight { name: String::from("Garin"), hp: 30 };
    println!("{} {}", k.name, k.hp);
}"#;
        assert!(grade("struct_basic", src).ok);
    }

    #[test]
    fn struct_fail_no_struct() {
        let src = "fn main() { let name = \"x\"; println!(\"{name}\"); }";
        let v = grade("struct_basic", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("struct Knight"));
    }

    #[test]
    fn struct_fail_missing_hp_field() {
        let src = "struct Knight { name: String } fn main() { let k = Knight { name: String::new() }; println!(\"{}\", k.name); }";
        let v = grade("struct_basic", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("hp:"));
    }

    #[test]
    fn struct_fail_no_field_access() {
        let src = "struct Knight { name: String, hp: i32 } fn main() {}";
        let v = grade("struct_basic", src);
        assert!(!v.ok);
        assert!(v.stderr.contains(".name"));
    }

    // ── vec_iter ──────────────────────────────────────────────────

    #[test]
    fn vec_iter_pass_canonical() {
        let src =
            "fn main() { let v = vec![1, 2, 3]; let s: i32 = v.iter().sum(); println!(\"{s}\"); }";
        assert!(grade("vec_iter", src).ok);
    }

    #[test]
    fn vec_iter_fail_no_vec_macro() {
        let src = "fn main() { let v = [1, 2, 3]; let _: i32 = v.iter().sum(); }";
        let v = grade("vec_iter", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("vec!"));
    }

    #[test]
    fn vec_iter_fail_no_iter() {
        let src = "fn main() { let v = vec![1, 2, 3]; let _ = v.len(); }";
        let v = grade("vec_iter", src);
        assert!(!v.ok);
        assert!(v.stderr.contains(".iter()"));
    }

    #[test]
    fn vec_iter_fail_no_sum() {
        let src = "fn main() { let v = vec![1, 2, 3]; let _ = v.iter().count(); }";
        let v = grade("vec_iter", src);
        assert!(!v.ok);
        assert!(v.stderr.contains(".sum"));
    }

    // ── tuple_destructure ────────────────────────────────────────

    #[test]
    fn tuple_destructure_pass_canonical() {
        let src = "fn main() { let (a, b) = (3, 4); println!(\"{a} {b}\"); }";
        assert!(grade("tuple_destructure", src).ok);
    }

    #[test]
    fn tuple_destructure_pass_no_space_in_assign() {
        let src = "fn main() { let (a, b)=(3, 4); let _ = (a, b); }";
        assert!(grade("tuple_destructure", src).ok);
    }

    #[test]
    fn tuple_destructure_fail_single_binding() {
        let src = "fn main() { let pair = (3, 4); }";
        let v = grade("tuple_destructure", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("let ("));
    }

    #[test]
    fn tuple_destructure_fail_no_comma() {
        let src = "fn main() { let (a) = (3); let _ = a; }";
        let v = grade("tuple_destructure", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("comma"));
    }

    // ── while_loop ────────────────────────────────────────────────

    #[test]
    fn while_loop_pass_canonical() {
        let src = "fn main() { let mut n = 5; while n > 0 { n -= 1; } println!(\"{n}\"); }";
        assert!(grade("while_loop", src).ok);
    }

    #[test]
    fn while_loop_pass_no_spaces() {
        let src = "fn main() { let mut n = 5; while n>0 { n -=1; } }";
        assert!(grade("while_loop", src).ok);
    }

    #[test]
    fn while_loop_fail_no_while() {
        let src = "fn main() { let mut n = 5; loop { if n > 0 { n -= 1; } else { break; } } }";
        let v = grade("while_loop", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("while "));
    }

    #[test]
    fn while_loop_fail_no_decrement() {
        let src = "fn main() { let mut n = 5; while n > 0 { n += 1; } }";
        let v = grade("while_loop", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("-= 1"));
    }

    // ── borrow_mut ────────────────────────────────────────────────

    #[test]
    fn borrow_mut_pass_canonical() {
        let src = "fn bump(x: &mut i32) { *x = 99; }";
        assert!(grade("borrow_mut", src).ok);
    }

    #[test]
    fn borrow_mut_pass_with_assignment_expr() {
        let src = "fn bump(x: &mut i32) { *x = *x + 1; } fn main() {}";
        assert!(grade("borrow_mut", src).ok);
    }

    #[test]
    fn borrow_mut_fail_no_mut_borrow() {
        let src = "fn bump(x: &i32) -> i32 { *x }";
        let v = grade("borrow_mut", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("&mut i32"));
    }

    #[test]
    fn borrow_mut_fail_no_deref() {
        let src = "fn bump(x: &mut i32) { let _ = x; }";
        let v = grade("borrow_mut", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("*x"));
    }

    // ── string_vs_str ─────────────────────────────────────────────

    #[test]
    fn string_vs_str_pass_canonical() {
        let src = r#"fn greet(name: &str) { println!("hi {name}"); }
fn main() { let s = String::from("you"); greet(&s); greet("anon"); }"#;
        assert!(grade("string_vs_str", src).ok);
    }

    #[test]
    fn string_vs_str_pass_minimal() {
        let src = "fn greet(n: &str) { let _ = n; } fn main() { let s = String::from(\"x\"); greet(&s); }";
        assert!(grade("string_vs_str", src).ok);
    }

    #[test]
    fn string_vs_str_fail_no_str_param() {
        let src = "fn greet(name: String) { let _ = name; } fn main() { let s = String::from(\"x\"); greet(s); }";
        let v = grade("string_vs_str", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("&str"));
    }

    #[test]
    fn string_vs_str_fail_no_string_caller() {
        let src = "fn greet(name: &str) { let _ = name; } fn main() { greet(\"anon\"); }";
        let v = grade("string_vs_str", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("String::from"));
    }

    // ── option_unwrap_or ──────────────────────────────────────────

    #[test]
    fn option_unwrap_or_pass_canonical() {
        let src = "fn safe(x: Option<i32>) -> i32 { x.unwrap_or(0) }";
        assert!(grade("option_unwrap_or", src).ok);
    }

    #[test]
    fn option_unwrap_or_pass_with_nonzero_default() {
        let src = "fn safe(x: Option<i32>) -> i32 { x.unwrap_or(42) }";
        assert!(grade("option_unwrap_or", src).ok);
    }

    #[test]
    fn option_unwrap_or_fail_no_option_type() {
        let src = "fn safe(x: i32) -> i32 { x }";
        let v = grade("option_unwrap_or", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("Option<"));
    }

    #[test]
    fn option_unwrap_or_fail_used_match_instead() {
        let src = "fn safe(x: Option<i32>) -> i32 { match x { Some(n) => n, None => 0 } }";
        let v = grade("option_unwrap_or", src);
        assert!(!v.ok);
        assert!(v.stderr.contains(".unwrap_or("));
    }

    // ── for_in_range ──────────────────────────────────────────────

    #[test]
    fn for_in_range_pass_canonical() {
        let src = "fn main() { for i in 0..10 { println!(\"{i}\"); } }";
        assert!(grade("for_in_range", src).ok);
    }

    #[test]
    fn for_in_range_pass_spaced_range() {
        let src = "fn main() { for i in 0 .. 10 { let _ = i; } }";
        assert!(grade("for_in_range", src).ok);
    }

    #[test]
    fn for_in_range_fail_used_while_instead() {
        let src = "fn main() { let mut i = 0; while i < 10 { i += 1; } }";
        let v = grade("for_in_range", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("for "));
    }

    #[test]
    fn for_in_range_fail_wrong_range() {
        let src = "fn main() { for i in 0..5 { let _ = i; } }";
        let v = grade("for_in_range", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("0..10"));
    }

    // ── closure_basic ─────────────────────────────────────────────

    #[test]
    fn closure_basic_pass_canonical() {
        let src = "fn main() { let add = |a, b| a + b; println!(\"{}\", add(2, 3)); }";
        assert!(grade("closure_basic", src).ok);
    }

    #[test]
    fn closure_basic_pass_no_space_before_b() {
        let src = "fn main() { let add = |a, b| a +b; let _ = add(1, 2); }";
        assert!(grade("closure_basic", src).ok);
    }

    #[test]
    fn closure_basic_fail_used_fn_instead() {
        let src = "fn add(a: i32, b: i32) -> i32 { a + b } fn main() { let _ = add(1, 2); }";
        let v = grade("closure_basic", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("let add"));
    }

    #[test]
    fn closure_basic_fail_wrong_body() {
        let src = "fn main() { let add = |a, b| a * 2 + 0; let _ = add(1, 2); }";
        let v = grade("closure_basic", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("+ b"));
    }

    // ── slice_basic ───────────────────────────────────────────────

    #[test]
    fn slice_basic_pass_canonical() {
        let src = "fn sum_slice(xs: &[i32]) -> i32 { xs.iter().sum() }";
        assert!(grade("slice_basic", src).ok);
    }

    #[test]
    fn slice_basic_pass_with_loop_body() {
        let src =
            "fn sum_slice(xs: &[i32]) -> i32 { let mut s = 0; for x in xs.iter() { s += *x; } s }";
        assert!(grade("slice_basic", src).ok);
    }

    #[test]
    fn slice_basic_fail_no_slice_param() {
        let src = "fn sum_slice(xs: Vec<i32>) -> i32 { xs.iter().sum() }";
        let v = grade("slice_basic", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("&[i32]"));
    }

    #[test]
    fn slice_basic_fail_used_indexing_only() {
        let src = "fn sum_slice(xs: &[i32]) -> i32 { xs[0] }";
        let v = grade("slice_basic", src);
        assert!(!v.ok);
        assert!(v.stderr.contains(".iter()"));
    }

    // ── result_question_mark ──────────────────────────────────────

    #[test]
    fn result_qm_pass_canonical() {
        let src = r#"fn parse_int(s: &str) -> Result<i32, String> {
    let n = s.parse::<i32>().map_err(|e| e.to_string())?;
    Ok(n)
}"#;
        assert!(grade("result_question_mark", src).ok);
    }

    #[test]
    fn result_qm_pass_inline_qmark_newline() {
        let src = "fn f() -> Result<i32, String> { let n: i32 = \"1\".parse().map_err(|e: std::num::ParseIntError| e.to_string())?\n;Ok(n) }";
        assert!(grade("result_question_mark", src).ok);
    }

    #[test]
    fn result_qm_fail_no_result_type() {
        let src = "fn f() -> i32 { \"42\".parse::<i32>().unwrap() }";
        let v = grade("result_question_mark", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("Result<"));
    }

    #[test]
    fn result_qm_fail_no_question_mark() {
        let src = "fn f() -> Result<i32, String> { let n = \"1\".parse::<i32>().unwrap(); Ok(n) }";
        let v = grade("result_question_mark", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("?"));
    }

    // ── derive_debug ──────────────────────────────────────────────

    #[test]
    fn derive_debug_pass_canonical() {
        let src = r#"#[derive(Debug)]
struct Item { name: String }
fn main() { let item = Item { name: String::from("ring") }; println!("{item:?}"); }"#;
        assert!(grade("derive_debug", src).ok);
    }

    #[test]
    fn derive_debug_pass_with_qmark_format() {
        let src = "#[derive(Debug)] struct Item { name: String } fn main() { let i = Item { name: String::new() }; println!(\"{:?}\", i); }";
        assert!(grade("derive_debug", src).ok);
    }

    #[test]
    fn derive_debug_fail_no_derive() {
        let src = "struct Item { name: String } fn main() { let i = Item { name: String::new() }; println!(\"{i:?}\"); }";
        let v = grade("derive_debug", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("#[derive(Debug)]"));
    }

    #[test]
    fn derive_debug_fail_used_display_format() {
        let src = "#[derive(Debug)] struct Item { name: String } fn main() { let i = Item { name: String::new() }; println!(\"{}\", i.name); }";
        let v = grade("derive_debug", src);
        assert!(!v.ok);
        assert!(v.stderr.contains(":?"));
    }

    // ── iter_map_collect ──────────────────────────────────────────

    #[test]
    fn iter_map_collect_pass_canonical() {
        let src = "fn main() { let v = vec![1, 2, 3]; let doubled: Vec<i32> = v.iter().map(|x| x * 2).collect(); let _ = doubled; }";
        assert!(grade("iter_map_collect", src).ok);
    }

    #[test]
    fn iter_map_collect_pass_with_turbofish() {
        let src =
            "fn main() { let v = [1, 2]; let _ = v.iter().map(|x| x + 1).collect::<Vec<_>>(); }";
        assert!(grade("iter_map_collect", src).ok);
    }

    #[test]
    fn iter_map_collect_fail_no_map() {
        let src = "fn main() { let v = vec![1, 2, 3]; let _: Vec<&i32> = v.iter().collect(); }";
        let v = grade("iter_map_collect", src);
        assert!(!v.ok);
        assert!(v.stderr.contains(".map("));
    }

    #[test]
    fn iter_map_collect_fail_no_collect() {
        let src = "fn main() { let v = vec![1, 2, 3]; let s: i32 = v.iter().map(|x| x * 2).sum(); let _ = s; }";
        let v = grade("iter_map_collect", src);
        assert!(!v.ok);
        assert!(v.stderr.contains(".collect"));
    }

    // ── enum_match ────────────────────────────────────────────────

    #[test]
    fn enum_match_pass_canonical() {
        let src = r#"enum Direction { North, South, East, West }
fn name(d: Direction) -> &'static str {
    match d {
        Direction::North => "n",
        Direction::South => "s",
        Direction::East => "e",
        Direction::West => "w",
    }
}"#;
        assert!(grade("enum_match", src).ok);
    }

    #[test]
    fn enum_match_pass_minimal_two_variants() {
        let src = "enum Direction { Up, Down } fn f(d: Direction) -> i32 { match d { Direction::Up => 1, Direction::Down => 0 } }";
        assert!(grade("enum_match", src).ok);
    }

    #[test]
    fn enum_match_fail_no_enum_decl() {
        let src = "fn name(d: i32) -> &'static str { match d { 0 => \"n\", _ => \"x\" } }";
        let v = grade("enum_match", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("enum Direction"));
    }

    #[test]
    fn enum_match_fail_no_variant_path() {
        let src = "enum Direction { North } fn f(_d: Direction) -> i32 { match 0 { _ => 0 } }";
        let v = grade("enum_match", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("Direction::"));
    }

    // ── Act 3: Guildhall Quarter ──────────────────────────────────
    #[test]
    fn impl_method_pass_canonical() {
        let src = "struct Hero { level: i32 } impl Hero { fn power(&self) -> i32 { self.level } } fn main() {}";
        assert!(grade("impl_method", src).ok);
    }
    #[test]
    fn impl_method_fail_no_impl() {
        let v = grade("impl_method", "struct Hero { level: i32 } fn main() {}");
        assert!(!v.ok);
        assert!(v.stderr.contains("impl"));
    }
    #[test]
    fn impl_method_fail_no_self_field_access() {
        // has `impl` and `&self` but never reads `self.field`.
        let v = grade(
            "impl_method",
            "struct Hero { level: i32 } impl Hero { fn power(&self) -> i32 { 0 } } fn main() {}",
        );
        assert!(!v.ok);
        assert!(v.stderr.contains("self."));
    }
    #[test]
    fn assoc_new_pass_canonical() {
        let src = "struct Hero { level: i32 } impl Hero { fn new(level: i32) -> Self { Self { level } } } fn main() { let _ = Hero::new(5); }";
        assert!(grade("assoc_new", src).ok);
    }
    #[test]
    fn assoc_new_fail_no_new_fn() {
        let v = grade("assoc_new", "struct Hero { level: i32 } fn main() {}");
        assert!(!v.ok);
        assert!(v.stderr.contains("fn new"));
    }
    #[test]
    fn if_let_pass_canonical() {
        let src = "fn main() { let m: Option<i32> = Some(7); if let Some(n) = m { let _ = n; } }";
        assert!(grade("if_let", src).ok);
    }
    #[test]
    fn if_let_fail_uses_full_match_instead() {
        // A full match should NOT satisfy if_let (no `if let`).
        let src = "fn main() { let m: Option<i32> = Some(7); match m { Some(n) => { let _ = n; } None => {} } }";
        let v = grade("if_let", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("if let"));
    }
    #[test]
    fn while_let_pass_canonical() {
        let src =
            "fn main() { let mut s = vec![1, 2]; while let Some(t) = s.pop() { let _ = t; } }";
        assert!(grade("while_let", src).ok);
    }
    #[test]
    fn while_let_fail_no_pop() {
        let src = "fn main() { let mut s = vec![1, 2]; while let Some(_t) = s.iter().next() {} }";
        let v = grade("while_let", src);
        assert!(!v.ok);
        assert!(v.stderr.contains(".pop("));
    }
    #[test]
    fn tuple_struct_pass_canonical() {
        let src = "struct Meters(f64); fn main() { let d = Meters(3.5); let _ = d.0; }";
        assert!(grade("tuple_struct", src).ok);
    }
    #[test]
    fn tuple_struct_fail_named_struct() {
        // A named-field struct is not a tuple struct.
        let src = "struct Meters { v: f64 } fn main() { let d = Meters { v: 3.5 }; let _ = d.v; }";
        let v = grade("tuple_struct", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("struct Meters("));
    }
    #[test]
    fn enum_data_match_pass_canonical() {
        let src = "enum Item { Weapon { damage: i32 }, Potion { heal: i32 } } fn value(i: Item) -> i32 { match i { Item::Weapon { damage } => damage, Item::Potion { heal } => heal } } fn main() {}";
        assert!(grade("enum_data_match", src).ok);
    }
    #[test]
    fn enum_data_match_fail_no_enum() {
        let v = grade("enum_data_match", "fn value(i: i32) -> i32 { i }");
        assert!(!v.ok);
        assert!(v.stderr.contains("enum Item"));
    }

    // ── Act 4: Trait Mage's Tower ─────────────────────────────────
    #[test]
    fn trait_def_pass_canonical() {
        let src = "trait E { fn n(&self) -> i32; } struct F; impl E for F { fn n(&self) -> i32 { 1 } } fn main() {}";
        assert!(grade("trait_def", src).ok);
    }
    #[test]
    fn trait_def_fail_no_trait() {
        let v = grade("trait_def", "struct F; fn main() {}");
        assert!(!v.ok);
        assert!(v.stderr.contains("trait "));
    }
    #[test]
    fn trait_def_fail_no_impl_for_type() {
        // A bare trait without `impl ... for` doesn't pass.
        let v = grade("trait_def", "trait E { fn n(&self); } fn main() {}");
        assert!(!v.ok);
    }
    #[test]
    fn generic_fn_pass_canonical() {
        let src =
            "fn larger<T: PartialOrd>(a: T, b: T) -> T { if a > b { a } else { b } } fn main() {}";
        assert!(grade("generic_fn", src).ok);
    }
    #[test]
    fn generic_fn_fail_unbounded() {
        // Generic but no `PartialOrd` bound — we require the bound.
        let v = grade("generic_fn", "fn id<T>(a: T) -> T { a } fn main() {}");
        assert!(!v.ok);
        assert!(v.stderr.contains("PartialOrd"));
    }
    #[test]
    fn generic_struct_pass_canonical() {
        let src = "struct Pair<T> { a: T, b: T } fn main() { let _ = Pair { a: 1, b: 2 }; }";
        assert!(grade("generic_struct", src).ok);
    }
    #[test]
    fn generic_struct_fail_not_generic() {
        let v = grade(
            "generic_struct",
            "struct Pair { a: i32, b: i32 } fn main() {}",
        );
        assert!(!v.ok);
        assert!(v.stderr.contains("<T>"));
    }
    #[test]
    fn dyn_trait_pass_canonical() {
        let src = "fn main() { let _v: Vec<Box<dyn std::fmt::Debug>> = vec![Box::new(1)]; }";
        assert!(grade("dyn_trait", src).ok);
    }
    #[test]
    fn dyn_trait_fail_no_box_dyn() {
        let v = grade("dyn_trait", "fn main() { let _v = vec![1, 2]; }");
        assert!(!v.ok);
        assert!(v.stderr.contains("Box<dyn"));
    }
    #[test]
    fn lifetimes_pass_canonical() {
        let src = "fn longest<'a>(x: &'a str, y: &'a str) -> &'a str { if x.len() > y.len() { x } else { y } } fn main() {}";
        assert!(grade("lifetimes", src).ok);
    }
    #[test]
    fn lifetimes_fail_no_lifetime() {
        let v = grade(
            "lifetimes",
            "fn longest(x: &str, _y: &str) -> &str { x } fn main() {}",
        );
        assert!(!v.ok);
        assert!(v.stderr.contains("<'a>"));
    }
    #[test]
    fn assoc_type_pass_canonical() {
        let src = "trait P { type Output; fn make(&self) -> Self::Output; } struct C; impl P for C { type Output = i32; fn make(&self) -> Self::Output { 1 } } fn main() {}";
        assert!(grade("assoc_type", src).ok);
    }
    #[test]
    fn assoc_type_fail_no_assoc_type() {
        let v = grade(
            "assoc_type",
            "trait P { fn make(&self) -> i32; } fn main() {}",
        );
        assert!(!v.ok);
        assert!(v.stderr.contains("type Output"));
    }

    // ── Act 5: Tavern of Tribulations ─────────────────────────────
    #[test]
    fn result_match_pass_canonical() {
        let src = "fn d(r: Result<i32, String>) -> i32 { match r { Ok(v) => v, Err(_e) => -1 } } fn main() {}";
        assert!(grade("result_match", src).ok);
    }
    #[test]
    fn result_match_fail_no_err_arm() {
        let src =
            "fn d(r: Result<i32, String>) -> i32 { match r { Ok(v) => v, _ => -1 } } fn main() {}";
        let v = grade("result_match", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("Err("));
    }
    #[test]
    fn custom_error_pass_canonical() {
        let src = "enum E { Bad } fn check(t: i32) -> Result<i32, E> { if t < 0 { Err(E::Bad) } else { Ok(t) } } fn main() {}";
        assert!(grade("custom_error", src).ok);
    }
    #[test]
    fn custom_error_fail_no_enum() {
        let v = grade("custom_error", "fn check(t: i32) -> i32 { t }");
        assert!(!v.ok);
        assert!(v.stderr.contains("enum "));
    }
    #[test]
    fn from_error_pass_canonical() {
        let src = "struct A; enum E { X } impl From<A> for E { fn from(_a: A) -> Self { E::X } } fn main() {}";
        assert!(grade("from_error", src).ok);
    }
    #[test]
    fn from_error_fail_no_impl() {
        let v = grade("from_error", "struct A; enum E { X } fn main() {}");
        assert!(!v.ok);
        assert!(v.stderr.contains("impl From<"));
    }
    #[test]
    fn option_map_pass_canonical() {
        let src = "fn f(o: Option<i32>) -> Option<i32> { o.map(|x| x + 1) } fn main() {}";
        assert!(grade("option_map", src).ok);
    }
    #[test]
    fn option_map_fail_no_map() {
        let v = grade(
            "option_map",
            "fn f(o: Option<i32>) -> Option<i32> { o } fn main() {}",
        );
        assert!(!v.ok);
        assert!(v.stderr.contains(".map("));
    }
    #[test]
    fn and_then_pass_canonical() {
        let src =
            "fn h(n: i32) -> Option<i32> { Some(n) } fn main() { let _ = Some(8).and_then(h); }";
        assert!(grade("and_then", src).ok);
    }
    #[test]
    fn and_then_fail_no_and_then() {
        let v = grade("and_then", "fn main() { let _: Option<i32> = Some(8); }");
        assert!(!v.ok);
        assert!(v.stderr.contains(".and_then("));
    }
    #[test]
    fn unwrap_or_else_pass_canonical() {
        let src = "fn f(o: Option<i32>) -> i32 { o.unwrap_or_else(|| 0) } fn main() {}";
        assert!(grade("unwrap_or_else", src).ok);
    }
    #[test]
    fn unwrap_or_else_fail_eager_unwrap_or() {
        // plain `.unwrap_or(` (the earlier mission) must NOT satisfy this one.
        let v = grade(
            "unwrap_or_else",
            "fn f(o: Option<i32>) -> i32 { o.unwrap_or(0) } fn main() {}",
        );
        assert!(!v.ok);
        assert!(v.stderr.contains(".unwrap_or_else("));
    }

    // ── Act 6: Iterator Forge ─────────────────────────────────────
    #[test]
    fn hashmap_basic_pass_canonical() {
        let src = "use std::collections::HashMap; fn main() { let mut m = HashMap::new(); m.insert(1, 2); let _ = m.get(&1); }";
        assert!(grade("hashmap_basic", src).ok);
    }
    #[test]
    fn hashmap_basic_fail_no_hashmap() {
        let v = grade("hashmap_basic", "fn main() { let _ = (); }");
        assert!(!v.ok);
        assert!(v.stderr.contains("HashMap"));
    }
    #[test]
    fn iter_filter_pass_canonical() {
        let src = "fn main() { let v = vec![1, 2]; let _: Vec<&i32> = v.iter().filter(|x| **x > 0).collect(); }";
        assert!(grade("iter_filter", src).ok);
    }
    #[test]
    fn iter_filter_fail_no_filter() {
        let v = grade(
            "iter_filter",
            "fn main() { let v = vec![1]; let _: Vec<&i32> = v.iter().collect(); }",
        );
        assert!(!v.ok);
        assert!(v.stderr.contains(".filter("));
    }
    #[test]
    fn iter_fold_pass_canonical() {
        let src = "fn main() { let v = vec![1, 2]; let _ = v.iter().fold(0, |a, x| a + x); }";
        assert!(grade("iter_fold", src).ok);
    }
    #[test]
    fn iter_fold_fail_used_sum() {
        let v = grade(
            "iter_fold",
            "fn main() { let v = vec![1]; let _: i32 = v.iter().sum(); }",
        );
        assert!(!v.ok);
        assert!(v.stderr.contains(".fold("));
    }
    #[test]
    fn iter_enumerate_pass_canonical() {
        let src =
            "fn main() { let v = vec![1]; for (i, x) in v.iter().enumerate() { let _ = (i, x); } }";
        assert!(grade("iter_enumerate", src).ok);
    }
    #[test]
    fn iter_enumerate_fail_no_enumerate() {
        let v = grade(
            "iter_enumerate",
            "fn main() { let v = vec![1]; for x in v.iter() { let _ = x; } }",
        );
        assert!(!v.ok);
        assert!(v.stderr.contains(".enumerate("));
    }
    #[test]
    fn iter_zip_pass_canonical() {
        let src = "fn main() { let a = vec![1]; let b = vec![2]; for (x, y) in a.iter().zip(b.iter()) { let _ = (x, y); } }";
        assert!(grade("iter_zip", src).ok);
    }
    #[test]
    fn iter_zip_fail_no_zip() {
        let v = grade(
            "iter_zip",
            "fn main() { let a = vec![1]; let _ = a.iter(); }",
        );
        assert!(!v.ok);
        assert!(v.stderr.contains(".zip("));
    }
    #[test]
    fn closure_move_pass_canonical() {
        let src =
            "fn main() { let s = String::from(\"x\"); let f = move || { let _ = &s; }; f(); }";
        assert!(grade("closure_move", src).ok);
    }
    #[test]
    fn closure_move_fail_borrowing_closure() {
        // a plain (non-move) closure must NOT satisfy this.
        let v = grade(
            "closure_move",
            "fn main() { let f = |a: i32| a + 1; let _ = f(1); }",
        );
        assert!(!v.ok);
        assert!(v.stderr.contains("move"));
    }

    // ── Act 7: Concurrent Coast ───────────────────────────────────
    #[test]
    fn thread_spawn_pass_canonical() {
        let src = "use std::thread; fn main() { let h = thread::spawn(|| 1); let _ = h.join(); }";
        assert!(grade("thread_spawn", src).ok);
    }
    #[test]
    fn thread_spawn_fail_no_join() {
        let v = grade(
            "thread_spawn",
            "use std::thread; fn main() { thread::spawn(|| 1); }",
        );
        assert!(!v.ok);
        assert!(v.stderr.contains(".join("));
    }
    #[test]
    fn arc_mutex_pass_canonical() {
        let src = "use std::sync::{Arc, Mutex}; fn main() { let m = Arc::new(Mutex::new(0)); *m.lock().unwrap() += 1; }";
        assert!(grade("arc_mutex", src).ok);
    }
    #[test]
    fn arc_mutex_fail_no_lock() {
        let v = grade(
            "arc_mutex",
            "use std::sync::{Arc, Mutex}; fn main() { let _ = Arc::new(Mutex::new(0)); }",
        );
        assert!(!v.ok);
        assert!(v.stderr.contains(".lock("));
    }
    #[test]
    fn mpsc_channel_pass_canonical() {
        let src = "use std::sync::mpsc; fn main() { let (tx, rx) = mpsc::channel(); tx.send(1).unwrap(); let _ = rx.recv(); }";
        assert!(grade("mpsc_channel", src).ok);
    }
    #[test]
    fn mpsc_channel_fail_no_recv() {
        let v = grade(
            "mpsc_channel",
            "use std::sync::mpsc; fn main() { let (tx, _rx) = mpsc::channel::<i32>(); tx.send(1).unwrap(); }",
        );
        assert!(!v.ok);
        assert!(v.stderr.contains(".recv("));
    }
    #[test]
    fn atomic_pass_canonical() {
        let src = "use std::sync::atomic::{AtomicUsize, Ordering}; fn main() { let c = AtomicUsize::new(0); c.fetch_add(1, Ordering::SeqCst); }";
        assert!(grade("atomic", src).ok);
    }
    #[test]
    fn atomic_fail_no_fetch_add() {
        let v = grade(
            "atomic",
            "use std::sync::atomic::AtomicUsize; fn main() { let _ = AtomicUsize::new(0); }",
        );
        assert!(!v.ok);
        assert!(v.stderr.contains(".fetch_add("));
    }
    #[test]
    fn thread_scope_pass_canonical() {
        let src = "use std::thread; fn main() { let d = vec![1]; thread::scope(|s| { s.spawn(|| { let _ = &d; }); }); }";
        assert!(grade("thread_scope", src).ok);
    }
    #[test]
    fn thread_scope_fail_no_scope() {
        // plain thread::spawn is not thread::scope.
        let v = grade(
            "thread_scope",
            "use std::thread; fn main() { let h = thread::spawn(|| 1); let _ = h.join(); }",
        );
        assert!(!v.ok);
        assert!(v.stderr.contains("thread::scope"));
    }
    #[test]
    fn async_fn_pass_canonical() {
        let src = "async fn d(x: i32) -> i32 { x } async fn run() -> i32 { d(1).await } fn main() { let _ = run(); }";
        assert!(grade("async_fn", src).ok);
    }
    #[test]
    fn async_fn_fail_no_await() {
        let v = grade("async_fn", "async fn d(x: i32) -> i32 { x } fn main() {}");
        assert!(!v.ok);
        assert!(v.stderr.contains(".await"));
    }

    // ── Act 8: Vault of Pointers ──────────────────────────────────
    #[test]
    fn box_basic_pass_canonical() {
        let src = "struct N { next: Option<Box<N>> } fn main() { let _ = N { next: Some(Box::new(N { next: None })) }; }";
        assert!(grade("box_basic", src).ok);
    }
    #[test]
    fn box_basic_fail_no_box() {
        let v = grade("box_basic", "struct N { next: Option<i32> } fn main() {}");
        assert!(!v.ok);
        assert!(v.stderr.contains("Box<"));
    }
    #[test]
    fn rc_basic_pass_canonical() {
        let src = "use std::rc::Rc; fn main() { let a = Rc::new(1); let _b = Rc::clone(&a); }";
        assert!(grade("rc_basic", src).ok);
    }
    #[test]
    fn rc_basic_fail_no_clone() {
        let v = grade(
            "rc_basic",
            "use std::rc::Rc; fn main() { let _a = Rc::new(1); }",
        );
        assert!(!v.ok);
        assert!(v.stderr.contains("Rc::clone"));
    }
    #[test]
    fn refcell_pass_canonical() {
        let src =
            "use std::cell::RefCell; fn main() { let c = RefCell::new(0); *c.borrow_mut() += 1; }";
        assert!(grade("refcell", src).ok);
    }
    #[test]
    fn refcell_fail_no_borrow_mut() {
        let v = grade(
            "refcell",
            "use std::cell::RefCell; fn main() { let _c = RefCell::new(0); }",
        );
        assert!(!v.ok);
        assert!(v.stderr.contains(".borrow_mut("));
    }
    #[test]
    fn cell_pass_canonical() {
        let src = "use std::cell::Cell; fn main() { let c = Cell::new(1); c.set(5); }";
        assert!(grade("cell", src).ok);
    }
    #[test]
    fn cell_fail_no_set() {
        let v = grade(
            "cell",
            "use std::cell::Cell; fn main() { let c = Cell::new(1); let _ = c.get(); }",
        );
        assert!(!v.ok);
        assert!(v.stderr.contains(".set("));
    }
    #[test]
    fn rc_refcell_pass_canonical() {
        let src = "use std::cell::RefCell; use std::rc::Rc; fn main() { let s = Rc::new(RefCell::new(0)); *s.borrow_mut() += 1; }";
        assert!(grade("rc_refcell", src).ok);
    }
    #[test]
    fn rc_refcell_fail_no_refcell() {
        // plain Rc without RefCell is not the shared-mutable pattern.
        let v = grade(
            "rc_refcell",
            "use std::rc::Rc; fn main() { let _ = Rc::new(0); }",
        );
        assert!(!v.ok);
        assert!(v.stderr.contains("RefCell"));
    }
    #[test]
    fn weak_ref_pass_canonical() {
        let src = "use std::rc::{Rc, Weak}; fn main() { let s = Rc::new(1); let w: Weak<i32> = Rc::downgrade(&s); let _ = w.upgrade(); }";
        assert!(grade("weak_ref", src).ok);
    }
    #[test]
    fn weak_ref_fail_no_upgrade() {
        let v = grade(
            "weak_ref",
            "use std::rc::{Rc, Weak}; fn main() { let s = Rc::new(1); let _w: Weak<i32> = Rc::downgrade(&s); }",
        );
        assert!(!v.ok);
        assert!(v.stderr.contains(".upgrade("));
    }
}

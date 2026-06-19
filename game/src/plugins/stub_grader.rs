// SPDX-License-Identifier: MIT
//! Offline fallback grader for the in-game compile client.
//!
//! When the live `compile-api` server at 127.0.0.1:7878 is unreachable
//! (Hetzner hiccup, demo machine without server, kiosk mode, etc.) the
//! game must not show a dead-end `[client error]`. Instead, this module
//! re-implements the same pattern checks the server-side grader uses,
//! returning a `[stub]`-prefixed verdict so the player can still feel
//! progression while knowing the grade isn't authoritative.
//!
//! IMPORTANT: this MUST NOT add a runtime dependency on
//! `pledge_compile_api`. The pattern table is duplicated on purpose.
//! The cross-crate contract test in `game/tests/contract.rs` keeps the
//! server-side grader honest; the audit harness ensures the duplicated
//! table doesn't drift in observable behaviour.
//!
//! Hard rule: a stub pass is NEVER persisted as a real mission clear.
//! The caller in `compile_client.rs` is responsible for gating
//! `MissionProgress::mark_cleared` on a "this came from the live API"
//! flag — see `CompileOutcome::from_stub`.

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

fn requires_all(source: &str, needles: &[&str]) -> Result<(), String> {
    for n in needles {
        if !source.contains(n) {
            return Err(format!("missing required: `{n}`"));
        }
    }
    Ok(())
}

/// Re-implementation of the server's per-encounter pattern checks.
///
/// Returns `Some(verdict)` for any mission currently shipped in
/// `MissionRegistry::default()`, or `None` for an unknown encounter id
/// (the caller falls back to the existing `[client error]` path so
/// unexpected ids aren't silently graded as a freeform pass). The
/// `stub_covers_every_mission_in_the_registry` test in `tests/stub_grader.rs`
/// will scream if a new mission is added without a stub arm here.
pub fn stub_verdict(encounter_id: &str, source: &str) -> Option<StubVerdict> {
    let v = match encounter_id {
        "intro_let_binding" => match requires_all(source, &["let answer", "42"]) {
            Ok(()) => StubVerdict::pass("answer bound. Ferris nods approvingly."),
            Err(e) => StubVerdict::fail(format!("not yet — {e}")),
        },
        "double_function" => match requires_all(source, &["fn double", "i32", "* 2"])
            .or_else(|_| requires_all(source, &["fn double", "i32", "*2"]))
        {
            Ok(()) => StubVerdict::pass(
                "the Trait Mage's staff hums. \"named once, doubled twice over.\"",
            ),
            Err(e) => StubVerdict::fail(format!("not yet — {e}")),
        },
        "borrow_preview" => match requires_all(source, &["&value", "println!"]) {
            Ok(()) => StubVerdict::pass("the Borrow Checker stirs. \"...acceptable. for now.\""),
            Err(e) => StubVerdict::fail(format!("the Borrow Checker is silent — {e}")),
        },
        "mut_binding" => {
            if !source.contains("let mut") {
                StubVerdict::fail("the Smith eyes you — missing required: `let mut`")
            } else if !source.contains("+= 1") && !source.contains("+=1") {
                StubVerdict::fail("the Smith eyes you — missing required: `+= 1`")
            } else {
                StubVerdict::pass("the Smith strikes the work. \"good — that one bends.\"")
            }
        }
        "if_else_sign" => {
            if !source.contains("if ") {
                StubVerdict::fail("the Cartographer frowns — missing required: `if `")
            } else if !source.contains("else") {
                StubVerdict::fail("the Cartographer frowns — missing required: `else`")
            } else if !source.contains("< 0") && !source.contains("<0") {
                StubVerdict::fail("the Cartographer frowns — missing required: `< 0`")
            } else {
                StubVerdict::pass("the Cartographer nods. \"three roads diverge — well chosen.\"")
            }
        }
        "loop_break" => {
            if !source.contains("loop") {
                StubVerdict::fail("the Bellringer waits — missing required: `loop`")
            } else if !source.contains("break") {
                StubVerdict::fail("the Bellringer waits — missing required: `break`")
            } else if !source.contains("*= 2") && !source.contains("*=2") {
                StubVerdict::fail(
                    "the Bellringer waits — the value should double each iteration: `*= 2`",
                )
            } else {
                StubVerdict::pass(
                    "the Bellringer pulls the rope — the bell sounds. \"the loop returned its number.\"",
                )
            }
        }
        "match_option" => {
            if !source.contains("match") {
                StubVerdict::fail("the Oracle's eyes are closed — missing required: `match`")
            } else if !source.contains("Some(") {
                StubVerdict::fail("the Oracle's eyes are closed — missing required: `Some(`")
            } else if !source.contains("None") {
                StubVerdict::fail("the Oracle's eyes are closed — missing required: `None`")
            } else {
                StubVerdict::pass(
                    "the Oracle exhales. \"both paths walked. nothing slips through.\"",
                )
            }
        }
        "vec_iter" => {
            if !source.contains("vec!") {
                StubVerdict::fail("the Cooper waits — missing required: `vec!`")
            } else if !source.contains(".iter()") {
                StubVerdict::fail("the Cooper waits — call `.iter()` on the vector")
            } else if !source.contains(".sum") {
                StubVerdict::fail("the Cooper waits — finish with `.sum()` to reduce")
            } else {
                StubVerdict::pass(
                    "the Cooper hoops the barrel. \"every stave counted, sum sealed.\"",
                )
            }
        }
        "tuple_destructure" => {
            if !source.contains("let (") {
                StubVerdict::fail("the Twin waits — missing required: `let (`")
            } else if !source.contains(",") {
                StubVerdict::fail("the Twin waits — a tuple needs a comma between its parts")
            } else if !source.contains(") =") && !source.contains(")=") {
                StubVerdict::fail("the Twin waits — close the pattern with `) =`")
            } else {
                StubVerdict::pass("the Twin grins. \"two names, one breath. cleanly split.\"")
            }
        }
        "while_loop" => {
            if !source.contains("while ") {
                StubVerdict::fail("the Tinker frowns — missing required: `while `")
            } else if !source.contains("> 0") && !source.contains(">0") {
                StubVerdict::fail("the Tinker frowns — the predicate should test `> 0`")
            } else if !source.contains("-= 1") && !source.contains("-=1") {
                StubVerdict::fail("the Tinker frowns — decrement with `-= 1` each iteration")
            } else {
                StubVerdict::pass("the Tinker pockets the spring. \"counted down, gear by gear.\"")
            }
        }
        "struct_basic" => {
            if !source.contains("struct Knight") {
                StubVerdict::fail("the Herald squints — missing required: `struct Knight`")
            } else if !source.contains("name:") {
                StubVerdict::fail("the Herald squints — Knight needs a `name:` field")
            } else if !source.contains("hp:") {
                StubVerdict::fail("the Herald squints — Knight needs an `hp:` field")
            } else if !source.contains(".name") {
                StubVerdict::fail("the Herald squints — read the field with `.name`")
            } else {
                StubVerdict::pass(
                    "the Herald reads the tabard. \"by name and by number — both fields announced.\"",
                )
            }
        }
        "borrow_mut" => {
            if !source.contains("fn bump") {
                StubVerdict::fail("the Forgewright glares — missing required: `fn bump`")
            } else if !source.contains("&mut i32") {
                StubVerdict::fail(
                    "the Forgewright glares — the parameter must be `&mut i32` (exclusive borrow)",
                )
            } else if !source.contains("*x") {
                StubVerdict::fail(
                    "the Forgewright glares — write through the reference with a `*x` deref",
                )
            } else {
                StubVerdict::pass(
                    "the Forgewright nods. \"one writer at the forge — the borrow holds.\"",
                )
            }
        }
        "string_vs_str" => {
            if !source.contains("fn greet") {
                StubVerdict::fail("the Linguist tilts her head — missing required: `fn greet`")
            } else if !source.contains("&str") {
                StubVerdict::fail(
                    "the Linguist tilts her head — the parameter type should be `&str`",
                )
            } else if !source.contains("String::from") {
                StubVerdict::fail(
                    "the Linguist tilts her head — call `greet` once with a `String::from(...)` value",
                )
            } else {
                StubVerdict::pass(
                    "the Linguist smiles. \"one tongue, two voices — &str carries both.\"",
                )
            }
        }
        "option_unwrap_or" => {
            if !source.contains("Option<") {
                StubVerdict::fail("the Pilgrim shakes his head — missing required: `Option<`")
            } else if !source.contains(".unwrap_or(") {
                StubVerdict::fail(
                    "the Pilgrim shakes his head — collapse the absent case with `.unwrap_or(default)`",
                )
            } else {
                StubVerdict::pass(
                    "the Pilgrim raises his lantern. \"the absent path lit a default.\"",
                )
            }
        }
        "for_in_range" => {
            if !source.contains("for ") {
                StubVerdict::fail("the Drillmaster barks — missing required: `for `")
            } else if !source.contains(" in ") {
                StubVerdict::fail("the Drillmaster barks — `for` needs a binding `in` an iterable")
            } else if !source.contains("0..10") && !source.contains("0 .. 10") {
                StubVerdict::fail("the Drillmaster barks — iterate the exact range `0..10`")
            } else {
                StubVerdict::pass("the Drillmaster claps once. \"ten paces, exact and ordered.\"")
            }
        }
        "closure_basic" => {
            if !source.contains("let add") {
                StubVerdict::fail("the Reckoner's quill hovers — missing required: `let add`")
            } else if !source.contains("= |") {
                StubVerdict::fail(
                    "the Reckoner's quill hovers — bind a closure literal with `= |...|`",
                )
            } else if !source.contains("+ b") && !source.contains("+b") {
                StubVerdict::fail("the Reckoner's quill hovers — the body should compute `a + b`")
            } else {
                StubVerdict::pass("the Reckoner inks the ledger. \"summed in a single stroke.\"")
            }
        }
        "slice_basic" => {
            if !source.contains("fn sum_slice") {
                StubVerdict::fail("the Quartermaster squints — missing required: `fn sum_slice`")
            } else if !source.contains("&[i32]") {
                StubVerdict::fail(
                    "the Quartermaster squints — the parameter must be a slice `&[i32]`",
                )
            } else if !source.contains(".iter()") {
                StubVerdict::fail("the Quartermaster squints — walk the slice with `.iter()`")
            } else {
                StubVerdict::pass(
                    "the Quartermaster ticks the manifest. \"a window into the stores — counted, not claimed.\"",
                )
            }
        }
        "result_question_mark" => {
            if !source.contains("Result<") {
                StubVerdict::fail("the Auditor's pen taps — missing required: `Result<`")
            } else if !source.contains(".parse") {
                StubVerdict::fail("the Auditor's pen taps — call `.parse` on a string source")
            } else if !source.contains("?;") && !source.contains("?\n") {
                StubVerdict::fail(
                    "the Auditor's pen taps — propagate the error with the `?` operator",
                )
            } else {
                StubVerdict::pass(
                    "the Auditor marks the tally. \"errors travel upward — the books balance.\"",
                )
            }
        }
        "derive_debug" => {
            if !source.contains("#[derive(Debug)]") {
                StubVerdict::fail(
                    "the Chronicler shakes his head — missing required: `#[derive(Debug)]`",
                )
            } else if !source.contains("struct Item") {
                StubVerdict::fail("the Chronicler shakes his head — define `struct Item`")
            } else if !source.contains(":?") {
                StubVerdict::fail(
                    "the Chronicler shakes his head — print with the debug formatter `{:?}`",
                )
            } else {
                StubVerdict::pass("the Chronicler dips his quill. \"derived, not authored.\"")
            }
        }
        "iter_map_collect" => {
            if !source.contains(".map(") {
                StubVerdict::fail("the Alchemist stirs — missing required: `.map(`")
            } else if !source.contains(".collect") {
                StubVerdict::fail(
                    "the Alchemist stirs — finish with `.collect` to gather the result",
                )
            } else if !source.contains("|x|") && !source.contains("|x |") {
                StubVerdict::fail(
                    "the Alchemist stirs — the closure should bind one element as `|x|`",
                )
            } else {
                StubVerdict::pass("the Alchemist decants the flask. \"every drop transmuted.\"")
            }
        }
        "enum_match" => {
            if !source.contains("enum Direction") {
                StubVerdict::fail("the Heraldic Sage frowns — missing required: `enum Direction`")
            } else if !source.contains("match ") {
                StubVerdict::fail(
                    "the Heraldic Sage frowns — inspect the variant with a `match ` expression",
                )
            } else if !source.contains("Direction::") {
                StubVerdict::fail(
                    "the Heraldic Sage frowns — name a variant with the `Direction::` path",
                )
            } else {
                StubVerdict::pass(
                    "the Heraldic Sage walks the sash. \"every variant a panel, every arm a blazon read.\"",
                )
            }
        }
        // ── Act 3 (Guildhall Quarter). Flavor MUST stay byte-identical to
        // compile-api/src/grader.rs — contract::server_and_stub_flavor_agree
        // enforces it.
        "impl_method" => {
            if !source.contains("impl") {
                StubVerdict::fail("the Guildmaster waits — missing required: `impl`")
            } else if !source.contains("&self") {
                StubVerdict::fail(
                    "the Guildmaster waits — a method borrows the instance with `&self`",
                )
            } else if !source.contains("self.") {
                StubVerdict::fail("the Guildmaster waits — read the instance's data with `self.`")
            } else {
                StubVerdict::pass(
                    "the Guildmaster nods. \"duties bound to a name — that is a method.\"",
                )
            }
        }
        "assoc_new" => {
            if !source.contains("fn new") {
                StubVerdict::fail("the Recruiter shakes her head — missing required: `fn new`")
            } else if !source.contains("Self") {
                StubVerdict::fail("the Recruiter shakes her head — a constructor returns `Self`")
            } else if !source.contains("::new") {
                StubVerdict::fail("the Recruiter shakes her head — call it with the `::new` path")
            } else {
                StubVerdict::pass(
                    "the Recruiter stamps the roll. \"Self::new — a member forged from a name.\"",
                )
            }
        }
        "if_let" => {
            if !source.contains("if let") {
                StubVerdict::fail("the Locksmith frowns — missing required: `if let`")
            } else if !source.contains("Some(") {
                StubVerdict::fail("the Locksmith frowns — match the present case with `Some(`")
            } else {
                StubVerdict::pass(
                    "the Locksmith turns the one key that fits. \"Some — and only Some — opens.\"",
                )
            }
        }
        "while_let" => {
            if !source.contains("while let") {
                StubVerdict::fail("the Porter sets the crate down — missing required: `while let`")
            } else if !source.contains(".pop(") {
                StubVerdict::fail("the Porter sets the crate down — drain the stack with `.pop()`")
            } else {
                StubVerdict::pass(
                    "the Porter empties the cart, crate by crate. \"while there is a Some, keep unloading.\"",
                )
            }
        }
        "tuple_struct" => {
            if !source.contains("struct Meters(") {
                StubVerdict::fail(
                    "the Surveyor squints down the line — missing required: `struct Meters(`",
                )
            } else if !source.contains(".0") {
                StubVerdict::fail(
                    "the Surveyor squints down the line — read the wrapped value with `.0`",
                )
            } else {
                StubVerdict::pass(
                    "the Surveyor reads the rod. \"a bare number, now named Meters — a newtype.\"",
                )
            }
        }
        "enum_data_match" => {
            if !source.contains("enum Item") {
                StubVerdict::fail("the Armorer taps the anvil — missing required: `enum Item`")
            } else if !source.contains("Weapon") {
                StubVerdict::fail("the Armorer taps the anvil — give Item a `Weapon` variant")
            } else if !source.contains("match") {
                StubVerdict::fail("the Armorer taps the anvil — inspect the item with `match`")
            } else if !source.contains("=>") {
                StubVerdict::fail("the Armorer taps the anvil — each variant needs a `=>` arm")
            } else {
                StubVerdict::pass(
                    "the Armorer lays out the rack. \"weapon or potion — name every kind, miss none.\"",
                )
            }
        }
        // ── Act 4 (Trait Mage's Tower). Flavor MUST stay byte-identical to
        // compile-api/src/grader.rs.
        "trait_def" => {
            if !source.contains("trait ") {
                StubVerdict::fail("Vexis waits — missing required: a `trait ` definition")
            } else if !source.contains("impl ") {
                StubVerdict::fail("Vexis waits — implement it with an `impl ` block")
            } else if !source.contains(" for ") {
                StubVerdict::fail("Vexis waits — bind the trait to a type: `impl Trait for Type`")
            } else {
                StubVerdict::pass(
                    "Vexis lowers his staff. \"a capability named once, granted to a type — that is a trait.\"",
                )
            }
        }
        "generic_fn" => {
            if !source.contains("<T") {
                StubVerdict::fail("the Wandwright frowns — missing required: a type parameter `<T`")
            } else if !source.contains("PartialOrd") {
                StubVerdict::fail(
                    "the Wandwright frowns — bound the parameter so it can be compared: `T: PartialOrd`",
                )
            } else {
                StubVerdict::pass(
                    "the Wandwright sights down the blank. \"one wand, any element — bounded by what it can compare.\"",
                )
            }
        }
        "generic_struct" => {
            if !source.contains("struct ") {
                StubVerdict::fail("the Conjurer waits — missing required: a `struct ` definition")
            } else if !source.contains("<T>") {
                StubVerdict::fail("the Conjurer waits — make it generic with a `<T>` parameter")
            } else if !source.contains(": T") {
                StubVerdict::fail("the Conjurer waits — give it a field of the generic type `: T`")
            } else {
                StubVerdict::pass(
                    "the Conjurer cups two matching lights. \"a vessel for any type — so long as both halves agree.\"",
                )
            }
        }
        "dyn_trait" => {
            if !source.contains("Box<dyn") {
                StubVerdict::fail("the Familiar flickers — missing required: `Box<dyn ...>`")
            } else if !source.contains("Box::new") {
                StubVerdict::fail(
                    "the Familiar flickers — box each value onto the heap with `Box::new`",
                )
            } else {
                StubVerdict::pass(
                    "the Familiar shifts through its forms. \"many shapes, one cage — dispatched at a touch.\"",
                )
            }
        }
        "lifetimes" => {
            if !source.contains("<'a>") {
                StubVerdict::fail(
                    "the Lanternkeeper waits — missing required: a lifetime parameter `<'a>`",
                )
            } else if !source.contains("&'a") {
                StubVerdict::fail("the Lanternkeeper waits — annotate the references with `&'a`")
            } else {
                StubVerdict::pass(
                    "the Lanternkeeper keeps the flame. \"the borrow lives exactly as long as 'a — no shorter.\"",
                )
            }
        }
        "assoc_type" => {
            if !source.contains("type Output") {
                StubVerdict::fail("the Loremaster turns the page — missing required: `type Output`")
            } else if !source.contains("Self::Output") {
                StubVerdict::fail(
                    "the Loremaster turns the page — refer to it as `Self::Output` in the method",
                )
            } else {
                StubVerdict::pass(
                    "the Loremaster turns the page. \"each producer names its own yield — the type follows the trait.\"",
                )
            }
        }
        // ── Act 5 (Tavern of Tribulations). Flavor MUST stay byte-identical
        // to compile-api/src/grader.rs.
        "result_match" => {
            if !source.contains("match ") {
                StubVerdict::fail("the Barkeep waits — missing required: `match `")
            } else if !source.contains("Ok(") {
                StubVerdict::fail("the Barkeep waits — handle the success arm `Ok(`")
            } else if !source.contains("Err(") {
                StubVerdict::fail("the Barkeep waits — handle the failure arm `Err(`")
            } else {
                StubVerdict::pass(
                    "the Barkeep nods. \"poured or spilled — you accounted for both.\"",
                )
            }
        }
        "custom_error" => {
            if !source.contains("enum ") {
                StubVerdict::fail(
                    "the Bouncer crosses his arms — missing required: an error `enum `",
                )
            } else if !source.contains("Result<") {
                StubVerdict::fail(
                    "the Bouncer crosses his arms — return a `Result<...>` from check",
                )
            } else if !source.contains("Err(") {
                StubVerdict::fail(
                    "the Bouncer crosses his arms — produce a failure with `Err(...)`",
                )
            } else {
                StubVerdict::pass(
                    "the Bouncer grunts. \"every kind of trouble, named and on the list.\"",
                )
            }
        }
        "from_error" => {
            if !source.contains("impl From<") {
                StubVerdict::fail("the Interpreter pauses — missing required: `impl From<...>`")
            } else if !source.contains(" for ") {
                StubVerdict::fail("the Interpreter pauses — implement it `for` your error type")
            } else if !source.contains("fn from") {
                StubVerdict::fail("the Interpreter pauses — the conversion is a `fn from` method")
            } else {
                StubVerdict::pass(
                    "the Interpreter inclines her head. \"one tongue's failure, spoken in another.\"",
                )
            }
        }
        "option_map" => {
            if !source.contains("Option<") {
                StubVerdict::fail("the Mixologist waits — keep it an `Option<...>`")
            } else if !source.contains(".map(") {
                StubVerdict::fail("the Mixologist waits — transform the inside with `.map(`")
            } else {
                StubVerdict::pass(
                    "the Mixologist swirls the glass. \"changed within — if there was anything to change.\"",
                )
            }
        }
        "and_then" => {
            if !source.contains("Option") {
                StubVerdict::fail("the Tabkeeper waits — the steps should yield an `Option`")
            } else if !source.contains(".and_then(") {
                StubVerdict::fail(
                    "the Tabkeeper waits — chain the fallible steps with `.and_then(`",
                )
            } else {
                StubVerdict::pass(
                    "the Tabkeeper closes the ledger. \"one failed round and the tab is cut.\"",
                )
            }
        }
        "unwrap_or_else" => {
            if !source.contains(".unwrap_or_else(") {
                StubVerdict::fail("the Cellarer waits — missing required: `.unwrap_or_else(`")
            } else if !source.contains("||") {
                StubVerdict::fail("the Cellarer waits — the lazy default is a closure `|| ...`")
            } else {
                StubVerdict::pass(
                    "the Cellarer taps the cask. \"a fresh draught — drawn only when the old runs dry.\"",
                )
            }
        }
        // ── Act 6 (Iterator Forge). Flavor MUST stay byte-identical to
        // compile-api/src/grader.rs.
        "hashmap_basic" => {
            if !source.contains("HashMap") {
                StubVerdict::fail("the Keymaster waits — missing required: `HashMap`")
            } else if !source.contains(".insert(") {
                StubVerdict::fail("the Keymaster waits — store a value with `.insert(`")
            } else if !source.contains(".get(") {
                StubVerdict::fail("the Keymaster waits — look it up with `.get(`")
            } else {
                StubVerdict::pass(
                    "the Keymaster turns the ring. \"every store under its own key.\"",
                )
            }
        }
        "iter_filter" => {
            if !source.contains(".filter(") {
                StubVerdict::fail("the Sifter shakes the screen — missing required: `.filter(`")
            } else if !source.contains(".collect") {
                StubVerdict::fail(
                    "the Sifter shakes the screen — gather the survivors with `.collect`",
                )
            } else {
                StubVerdict::pass(
                    "the Sifter shakes the screen. \"only what fits the mesh falls through.\"",
                )
            }
        }
        "iter_fold" => {
            if !source.contains(".fold(") {
                StubVerdict::fail("the Smelter waits — missing required: `.fold(`")
            } else {
                StubVerdict::pass("the Smelter tips the crucible. \"many ores, one ingot.\"")
            }
        }
        "iter_enumerate" => {
            if !source.contains(".enumerate(") {
                StubVerdict::fail("the Tallywright waits — missing required: `.enumerate(`")
            } else {
                StubVerdict::pass(
                    "the Tallywright stamps the row. \"each item numbered as it passes.\"",
                )
            }
        }
        "iter_zip" => {
            if !source.contains(".zip(") {
                StubVerdict::fail("the Riveter waits — missing required: `.zip(`")
            } else {
                StubVerdict::pass(
                    "the Riveter drives the rivet. \"two plates, aligned pair by pair.\"",
                )
            }
        }
        "closure_move" => {
            if !source.contains("move |") {
                StubVerdict::fail(
                    "the Bondsmith waits — missing required: a `move` closure (`move |...|`)",
                )
            } else {
                StubVerdict::pass(
                    "the Bondsmith seals the charge. \"owned, and carried where I go.\"",
                )
            }
        }
        // ── Act 7 (Concurrent Coast). Flavor MUST stay byte-identical to
        // compile-api/src/grader.rs.
        "thread_spawn" => {
            if !source.contains("thread::spawn") {
                StubVerdict::fail("the Dockmaster waits — missing required: `thread::spawn`")
            } else if !source.contains(".join(") {
                StubVerdict::fail("the Dockmaster waits — wait for the worker with `.join(`")
            } else {
                StubVerdict::pass(
                    "the Dockmaster waves them off. \"sent to work — and met again at the gate.\"",
                )
            }
        }
        "arc_mutex" => {
            if !source.contains("Arc") {
                StubVerdict::fail("the Lighthouse Keeper waits — share ownership with `Arc`")
            } else if !source.contains("Mutex") {
                StubVerdict::fail("the Lighthouse Keeper waits — guard the value with `Mutex`")
            } else if !source.contains(".lock(") {
                StubVerdict::fail(
                    "the Lighthouse Keeper waits — take exclusive access with `.lock(`",
                )
            } else {
                StubVerdict::pass(
                    "the Lighthouse Keeper turns the key. \"one hand on the lamp at a time.\"",
                )
            }
        }
        "mpsc_channel" => {
            if !source.contains("mpsc::channel") {
                StubVerdict::fail("the Signaler waits — missing required: `mpsc::channel`")
            } else if !source.contains(".send(") {
                StubVerdict::fail("the Signaler waits — push a value in with `.send(`")
            } else if !source.contains(".recv(") {
                StubVerdict::fail("the Signaler waits — read it at the far end with `.recv(`")
            } else {
                StubVerdict::pass(
                    "the Signaler flashes the lamp. \"sent down the coast, read at the next tower.\"",
                )
            }
        }
        "atomic" => {
            if !source.contains("Atomic") {
                StubVerdict::fail("the Tidewatch waits — use an `Atomic` type")
            } else if !source.contains(".fetch_add(") {
                StubVerdict::fail("the Tidewatch waits — bump it atomically with `.fetch_add(`")
            } else {
                StubVerdict::pass(
                    "the Tidewatch clicks the gauge. \"one notch up — no gatekeeper.\"",
                )
            }
        }
        "thread_scope" => {
            if !source.contains("thread::scope") {
                StubVerdict::fail("the Harbormaster waits — missing required: `thread::scope`")
            } else if !source.contains(".spawn(") {
                StubVerdict::fail("the Harbormaster waits — launch a scoped thread with `.spawn(`")
            } else {
                StubVerdict::pass(
                    "the Harbormaster rings the bell. \"every boat in before the harbor closes.\"",
                )
            }
        }
        "async_fn" => {
            if !source.contains("async fn") {
                StubVerdict::fail("the Tideforecaster waits — missing required: `async fn`")
            } else if !source.contains(".await") {
                StubVerdict::fail("the Tideforecaster waits — drive the future with `.await`")
            } else {
                StubVerdict::pass(
                    "the Tideforecaster lowers the glass. \"the tide will come — awaited, not forced.\"",
                )
            }
        }
        _ => return None,
    };
    Some(v)
}

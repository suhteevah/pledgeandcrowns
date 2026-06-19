// SPDX-License-Identifier: AGPL-3.0-or-later
//! Per-encounter source-code grader. Pure function, fully unit-tested.
//!
//! v0 is pattern-matching; v1 will swap the body of `grade` for a
//! real `cargo build --offline` + wasmtime invocation per
//! `design/05-tech-architecture.md` §2 — the test suite below stays
//! valid because it only exercises the public contract.

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

fn requires_all(source: &str, needles: &[&str]) -> Result<(), String> {
    for n in needles {
        if !source.contains(n) {
            return Err(format!("missing required: `{n}`"));
        }
    }
    Ok(())
}

pub fn grade(encounter_id: &str, source: &str) -> Verdict {
    match encounter_id {
        "intro_let_binding" => match requires_all(source, &["let answer", "42"]) {
            Ok(()) => Verdict::pass("answer bound. Ferris nods approvingly."),
            Err(e) => Verdict::fail(format!("not yet — {e}")),
        },
        "double_function" => match requires_all(source, &["fn double", "i32", "* 2"])
            .or_else(|_| requires_all(source, &["fn double", "i32", "*2"]))
        {
            Ok(()) => {
                Verdict::pass("the Trait Mage's staff hums. \"named once, doubled twice over.\"")
            }
            Err(e) => Verdict::fail(format!("not yet — {e}")),
        },
        "borrow_preview" => match requires_all(source, &["&value", "println!"]) {
            Ok(()) => Verdict::pass("the Borrow Checker stirs. \"...acceptable. for now.\""),
            Err(e) => Verdict::fail(format!("the Borrow Checker is silent — {e}")),
        },
        // ── Mission 4: declare a mutable binding and increment it.
        "mut_binding" => {
            if !source.contains("let mut") {
                Verdict::fail("the Smith eyes you — missing required: `let mut`")
            } else if !source.contains("+= 1") && !source.contains("+=1") {
                Verdict::fail("the Smith eyes you — missing required: `+= 1`")
            } else {
                Verdict::pass("the Smith strikes the work. \"good — that one bends.\"")
            }
        }
        // ── Mission 5: branch on i32 sign with if/else.
        "if_else_sign" => {
            if !source.contains("if ") {
                Verdict::fail("the Cartographer frowns — missing required: `if `")
            } else if !source.contains("else") {
                Verdict::fail("the Cartographer frowns — missing required: `else`")
            } else if !source.contains("< 0") && !source.contains("<0") {
                Verdict::fail("the Cartographer frowns — missing required: `< 0`")
            } else {
                Verdict::pass("the Cartographer nods. \"three roads diverge — well chosen.\"")
            }
        }
        // ── Mission 6: `loop` with `break value;` to return from a loop.
        "loop_break" => {
            if !source.contains("loop") {
                Verdict::fail("the Bellringer waits — missing required: `loop`")
            } else if !source.contains("break") {
                Verdict::fail("the Bellringer waits — missing required: `break`")
            } else if !source.contains("*= 2") && !source.contains("*=2") {
                Verdict::fail(
                    "the Bellringer waits — the value should double each iteration: `*= 2`",
                )
            } else {
                Verdict::pass(
                    "the Bellringer pulls the rope — the bell sounds. \"the loop returned its number.\"",
                )
            }
        }
        // ── Mission 7: match on Option<i32>, handle Some/None.
        "match_option" => {
            if !source.contains("match") {
                Verdict::fail("the Oracle's eyes are closed — missing required: `match`")
            } else if !source.contains("Some(") {
                Verdict::fail("the Oracle's eyes are closed — missing required: `Some(`")
            } else if !source.contains("None") {
                Verdict::fail("the Oracle's eyes are closed — missing required: `None`")
            } else {
                Verdict::pass("the Oracle exhales. \"both paths walked. nothing slips through.\"")
            }
        }
        // ── Mission 9: iterate a Vec and reduce. Teaches `iter`, `sum`.
        "vec_iter" => {
            if !source.contains("vec!") {
                Verdict::fail("the Cooper waits — missing required: `vec!`")
            } else if !source.contains(".iter()") {
                Verdict::fail("the Cooper waits — call `.iter()` on the vector")
            } else if !source.contains(".sum") {
                Verdict::fail("the Cooper waits — finish with `.sum()` to reduce")
            } else {
                Verdict::pass("the Cooper hoops the barrel. \"every stave counted, sum sealed.\"")
            }
        }
        // ── Mission 10: destructure a tuple in a let binding.
        "tuple_destructure" => {
            if !source.contains("let (") {
                Verdict::fail("the Twin waits — missing required: `let (`")
            } else if !source.contains(",") {
                Verdict::fail("the Twin waits — a tuple needs a comma between its parts")
            } else if !source.contains(") =") && !source.contains(")=") {
                Verdict::fail("the Twin waits — close the pattern with `) =`")
            } else {
                Verdict::pass("the Twin grins. \"two names, one breath. cleanly split.\"")
            }
        }
        // ── Mission 11: `while` loop with a decrementing counter.
        "while_loop" => {
            if !source.contains("while ") {
                Verdict::fail("the Tinker frowns — missing required: `while `")
            } else if !source.contains("> 0") && !source.contains(">0") {
                Verdict::fail("the Tinker frowns — the predicate should test `> 0`")
            } else if !source.contains("-= 1") && !source.contains("-=1") {
                Verdict::fail("the Tinker frowns — decrement with `-= 1` each iteration")
            } else {
                Verdict::pass("the Tinker pockets the spring. \"counted down, gear by gear.\"")
            }
        }
        // ── Mission 12: take `&mut T`, mutate through the deref. Act 2:
        // exclusive borrow + interior mutation through a reference.
        "borrow_mut" => {
            if !source.contains("fn bump") {
                Verdict::fail("the Forgewright glares — missing required: `fn bump`")
            } else if !source.contains("&mut i32") {
                Verdict::fail(
                    "the Forgewright glares — the parameter must be `&mut i32` (exclusive borrow)",
                )
            } else if !source.contains("*x") {
                Verdict::fail(
                    "the Forgewright glares — write through the reference with a `*x` deref",
                )
            } else {
                Verdict::pass(
                    "the Forgewright nods. \"one writer at the forge — the borrow holds.\"",
                )
            }
        }
        // ── Mission 13: function takes `&str`, callable with both
        // `String` (via `&s`) and a string literal. Act 2: deref coercion
        // and the `String` vs `&str` distinction.
        "string_vs_str" => {
            if !source.contains("fn greet") {
                Verdict::fail("the Linguist tilts her head — missing required: `fn greet`")
            } else if !source.contains("&str") {
                Verdict::fail("the Linguist tilts her head — the parameter type should be `&str`")
            } else if !source.contains("String::from") {
                Verdict::fail(
                    "the Linguist tilts her head — call `greet` once with a `String::from(...)` value",
                )
            } else {
                Verdict::pass(
                    "the Linguist smiles. \"one tongue, two voices — &str carries both.\"",
                )
            }
        }
        // ── Mission 14: handle a missing `Option<T>` with `.unwrap_or`.
        // Act 2: idiomatic non-`match` Option handling.
        "option_unwrap_or" => {
            if !source.contains("Option<") {
                Verdict::fail("the Pilgrim shakes his head — missing required: `Option<`")
            } else if !source.contains(".unwrap_or(") {
                Verdict::fail(
                    "the Pilgrim shakes his head — collapse the absent case with `.unwrap_or(default)`",
                )
            } else {
                Verdict::pass("the Pilgrim raises his lantern. \"the absent path lit a default.\"")
            }
        }
        // ── Mission 15: `for i in 0..N` range iteration. Act 2: the
        // for-loop is the iterator-protocol entry point.
        "for_in_range" => {
            if !source.contains("for ") {
                Verdict::fail("the Drillmaster barks — missing required: `for `")
            } else if !source.contains(" in ") {
                Verdict::fail("the Drillmaster barks — `for` needs a binding `in` an iterable")
            } else if !source.contains("0..10") && !source.contains("0 .. 10") {
                Verdict::fail("the Drillmaster barks — iterate the exact range `0..10`")
            } else {
                Verdict::pass("the Drillmaster claps once. \"ten paces, exact and ordered.\"")
            }
        }
        // ── Mission 16: bind a two-argument closure to a name and
        // invoke it. Act 2 entry to closures + `Fn` traits.
        "closure_basic" => {
            if !source.contains("let add") {
                Verdict::fail("the Reckoner's quill hovers — missing required: `let add`")
            } else if !source.contains("= |") {
                Verdict::fail("the Reckoner's quill hovers — bind a closure literal with `= |...|`")
            } else if !source.contains("+ b") && !source.contains("+b") {
                Verdict::fail("the Reckoner's quill hovers — the body should compute `a + b`")
            } else {
                Verdict::pass("the Reckoner inks the ledger. \"summed in a single stroke.\"")
            }
        }
        // ── Mission 8: define a struct with named fields and read one.
        "struct_basic" => {
            if !source.contains("struct Knight") {
                Verdict::fail("the Herald squints — missing required: `struct Knight`")
            } else if !source.contains("name:") {
                Verdict::fail("the Herald squints — Knight needs a `name:` field")
            } else if !source.contains("hp:") {
                Verdict::fail("the Herald squints — Knight needs an `hp:` field")
            } else if !source.contains(".name") {
                Verdict::fail("the Herald squints — read the field with `.name`")
            } else {
                Verdict::pass(
                    "the Herald reads the tabard. \"by name and by number — both fields announced.\"",
                )
            }
        }
        // ── Mission 17: take a `&[i32]` slice. Act 2: borrowing a
        // contiguous view without owning the buffer.
        "slice_basic" => {
            if !source.contains("fn sum_slice") {
                Verdict::fail("the Quartermaster squints — missing required: `fn sum_slice`")
            } else if !source.contains("&[i32]") {
                Verdict::fail("the Quartermaster squints — the parameter must be a slice `&[i32]`")
            } else if !source.contains(".iter()") {
                Verdict::fail("the Quartermaster squints — walk the slice with `.iter()`")
            } else {
                Verdict::pass(
                    "the Quartermaster ticks the manifest. \"a window into the stores — counted, not claimed.\"",
                )
            }
        }
        // ── Mission 18: propagate a `Result` with the `?` operator.
        // Act 2: error short-circuiting without `match`.
        "result_question_mark" => {
            if !source.contains("Result<") {
                Verdict::fail("the Auditor's pen taps — missing required: `Result<`")
            } else if !source.contains(".parse") {
                Verdict::fail("the Auditor's pen taps — call `.parse` on a string source")
            } else if !source.contains("?;") && !source.contains("?\n") {
                Verdict::fail("the Auditor's pen taps — propagate the error with the `?` operator")
            } else {
                Verdict::pass(
                    "the Auditor marks the tally. \"errors travel upward — the books balance.\"",
                )
            }
        }
        // ── Mission 19: derive Debug on a struct, then `{:?}` print it.
        "derive_debug" => {
            if !source.contains("#[derive(Debug)]") {
                Verdict::fail(
                    "the Chronicler shakes his head — missing required: `#[derive(Debug)]`",
                )
            } else if !source.contains("struct Item") {
                Verdict::fail("the Chronicler shakes his head — define `struct Item`")
            } else if !source.contains(":?") {
                Verdict::fail(
                    "the Chronicler shakes his head — print with the debug formatter `{:?}`",
                )
            } else {
                Verdict::pass("the Chronicler dips his quill. \"derived, not authored.\"")
            }
        }
        // ── Mission 20: iter().map().collect() into a Vec.
        "iter_map_collect" => {
            if !source.contains(".map(") {
                Verdict::fail("the Alchemist stirs — missing required: `.map(`")
            } else if !source.contains(".collect") {
                Verdict::fail("the Alchemist stirs — finish with `.collect` to gather the result")
            } else if !source.contains("|x|") && !source.contains("|x |") {
                Verdict::fail("the Alchemist stirs — the closure should bind one element as `|x|`")
            } else {
                Verdict::pass("the Alchemist decants the flask. \"every drop transmuted.\"")
            }
        }
        // ── Mission 21: define an enum, match on its variants.
        "enum_match" => {
            if !source.contains("enum Direction") {
                Verdict::fail("the Heraldic Sage frowns — missing required: `enum Direction`")
            } else if !source.contains("match ") {
                Verdict::fail(
                    "the Heraldic Sage frowns — inspect the variant with a `match ` expression",
                )
            } else if !source.contains("Direction::") {
                Verdict::fail(
                    "the Heraldic Sage frowns — name a variant with the `Direction::` path",
                )
            } else {
                Verdict::pass(
                    "the Heraldic Sage walks the sash. \"every variant a panel, every arm a blazon read.\"",
                )
            }
        }
        // ── Act 3 (Guildhall Quarter): methods/impl, associated fns,
        // if let, while let, tuple structs, enum-with-data match.
        "impl_method" => {
            if !source.contains("impl") {
                Verdict::fail("the Guildmaster waits — missing required: `impl`")
            } else if !source.contains("&self") {
                Verdict::fail("the Guildmaster waits — a method borrows the instance with `&self`")
            } else if !source.contains("self.") {
                Verdict::fail("the Guildmaster waits — read the instance's data with `self.`")
            } else {
                Verdict::pass(
                    "the Guildmaster nods. \"duties bound to a name — that is a method.\"",
                )
            }
        }
        "assoc_new" => {
            if !source.contains("fn new") {
                Verdict::fail("the Recruiter shakes her head — missing required: `fn new`")
            } else if !source.contains("Self") {
                Verdict::fail("the Recruiter shakes her head — a constructor returns `Self`")
            } else if !source.contains("::new") {
                Verdict::fail("the Recruiter shakes her head — call it with the `::new` path")
            } else {
                Verdict::pass(
                    "the Recruiter stamps the roll. \"Self::new — a member forged from a name.\"",
                )
            }
        }
        "if_let" => {
            if !source.contains("if let") {
                Verdict::fail("the Locksmith frowns — missing required: `if let`")
            } else if !source.contains("Some(") {
                Verdict::fail("the Locksmith frowns — match the present case with `Some(`")
            } else {
                Verdict::pass(
                    "the Locksmith turns the one key that fits. \"Some — and only Some — opens.\"",
                )
            }
        }
        "while_let" => {
            if !source.contains("while let") {
                Verdict::fail("the Porter sets the crate down — missing required: `while let`")
            } else if !source.contains(".pop(") {
                Verdict::fail("the Porter sets the crate down — drain the stack with `.pop()`")
            } else {
                Verdict::pass(
                    "the Porter empties the cart, crate by crate. \"while there is a Some, keep unloading.\"",
                )
            }
        }
        "tuple_struct" => {
            if !source.contains("struct Meters(") {
                Verdict::fail(
                    "the Surveyor squints down the line — missing required: `struct Meters(`",
                )
            } else if !source.contains(".0") {
                Verdict::fail(
                    "the Surveyor squints down the line — read the wrapped value with `.0`",
                )
            } else {
                Verdict::pass(
                    "the Surveyor reads the rod. \"a bare number, now named Meters — a newtype.\"",
                )
            }
        }
        "enum_data_match" => {
            if !source.contains("enum Item") {
                Verdict::fail("the Armorer taps the anvil — missing required: `enum Item`")
            } else if !source.contains("Weapon") {
                Verdict::fail("the Armorer taps the anvil — give Item a `Weapon` variant")
            } else if !source.contains("match") {
                Verdict::fail("the Armorer taps the anvil — inspect the item with `match`")
            } else if !source.contains("=>") {
                Verdict::fail("the Armorer taps the anvil — each variant needs a `=>` arm")
            } else {
                Verdict::pass(
                    "the Armorer lays out the rack. \"weapon or potion — name every kind, miss none.\"",
                )
            }
        }
        // ── Act 4 (Trait Mage's Tower): traits, generics, dyn, lifetimes,
        // associated types.
        "trait_def" => {
            if !source.contains("trait ") {
                Verdict::fail("Vexis waits — missing required: a `trait ` definition")
            } else if !source.contains("impl ") {
                Verdict::fail("Vexis waits — implement it with an `impl ` block")
            } else if !source.contains(" for ") {
                Verdict::fail("Vexis waits — bind the trait to a type: `impl Trait for Type`")
            } else {
                Verdict::pass(
                    "Vexis lowers his staff. \"a capability named once, granted to a type — that is a trait.\"",
                )
            }
        }
        "generic_fn" => {
            if !source.contains("<T") {
                Verdict::fail("the Wandwright frowns — missing required: a type parameter `<T`")
            } else if !source.contains("PartialOrd") {
                Verdict::fail(
                    "the Wandwright frowns — bound the parameter so it can be compared: `T: PartialOrd`",
                )
            } else {
                Verdict::pass(
                    "the Wandwright sights down the blank. \"one wand, any element — bounded by what it can compare.\"",
                )
            }
        }
        "generic_struct" => {
            if !source.contains("struct ") {
                Verdict::fail("the Conjurer waits — missing required: a `struct ` definition")
            } else if !source.contains("<T>") {
                Verdict::fail("the Conjurer waits — make it generic with a `<T>` parameter")
            } else if !source.contains(": T") {
                Verdict::fail("the Conjurer waits — give it a field of the generic type `: T`")
            } else {
                Verdict::pass(
                    "the Conjurer cups two matching lights. \"a vessel for any type — so long as both halves agree.\"",
                )
            }
        }
        "dyn_trait" => {
            if !source.contains("Box<dyn") {
                Verdict::fail("the Familiar flickers — missing required: `Box<dyn ...>`")
            } else if !source.contains("Box::new") {
                Verdict::fail(
                    "the Familiar flickers — box each value onto the heap with `Box::new`",
                )
            } else {
                Verdict::pass(
                    "the Familiar shifts through its forms. \"many shapes, one cage — dispatched at a touch.\"",
                )
            }
        }
        "lifetimes" => {
            if !source.contains("<'a>") {
                Verdict::fail(
                    "the Lanternkeeper waits — missing required: a lifetime parameter `<'a>`",
                )
            } else if !source.contains("&'a") {
                Verdict::fail("the Lanternkeeper waits — annotate the references with `&'a`")
            } else {
                Verdict::pass(
                    "the Lanternkeeper keeps the flame. \"the borrow lives exactly as long as 'a — no shorter.\"",
                )
            }
        }
        "assoc_type" => {
            if !source.contains("type Output") {
                Verdict::fail("the Loremaster turns the page — missing required: `type Output`")
            } else if !source.contains("Self::Output") {
                Verdict::fail(
                    "the Loremaster turns the page — refer to it as `Self::Output` in the method",
                )
            } else {
                Verdict::pass(
                    "the Loremaster turns the page. \"each producer names its own yield — the type follows the trait.\"",
                )
            }
        }
        // ── Act 5 (Tavern of Tribulations): Result/Option methods, custom
        // errors, From conversion.
        "result_match" => {
            if !source.contains("match ") {
                Verdict::fail("the Barkeep waits — missing required: `match `")
            } else if !source.contains("Ok(") {
                Verdict::fail("the Barkeep waits — handle the success arm `Ok(`")
            } else if !source.contains("Err(") {
                Verdict::fail("the Barkeep waits — handle the failure arm `Err(`")
            } else {
                Verdict::pass("the Barkeep nods. \"poured or spilled — you accounted for both.\"")
            }
        }
        "custom_error" => {
            if !source.contains("enum ") {
                Verdict::fail("the Bouncer crosses his arms — missing required: an error `enum `")
            } else if !source.contains("Result<") {
                Verdict::fail("the Bouncer crosses his arms — return a `Result<...>` from check")
            } else if !source.contains("Err(") {
                Verdict::fail("the Bouncer crosses his arms — produce a failure with `Err(...)`")
            } else {
                Verdict::pass(
                    "the Bouncer grunts. \"every kind of trouble, named and on the list.\"",
                )
            }
        }
        "from_error" => {
            if !source.contains("impl From<") {
                Verdict::fail("the Interpreter pauses — missing required: `impl From<...>`")
            } else if !source.contains(" for ") {
                Verdict::fail("the Interpreter pauses — implement it `for` your error type")
            } else if !source.contains("fn from") {
                Verdict::fail("the Interpreter pauses — the conversion is a `fn from` method")
            } else {
                Verdict::pass(
                    "the Interpreter inclines her head. \"one tongue's failure, spoken in another.\"",
                )
            }
        }
        "option_map" => {
            if !source.contains("Option<") {
                Verdict::fail("the Mixologist waits — keep it an `Option<...>`")
            } else if !source.contains(".map(") {
                Verdict::fail("the Mixologist waits — transform the inside with `.map(`")
            } else {
                Verdict::pass(
                    "the Mixologist swirls the glass. \"changed within — if there was anything to change.\"",
                )
            }
        }
        "and_then" => {
            if !source.contains("Option") {
                Verdict::fail("the Tabkeeper waits — the steps should yield an `Option`")
            } else if !source.contains(".and_then(") {
                Verdict::fail("the Tabkeeper waits — chain the fallible steps with `.and_then(`")
            } else {
                Verdict::pass(
                    "the Tabkeeper closes the ledger. \"one failed round and the tab is cut.\"",
                )
            }
        }
        "unwrap_or_else" => {
            if !source.contains(".unwrap_or_else(") {
                Verdict::fail("the Cellarer waits — missing required: `.unwrap_or_else(`")
            } else if !source.contains("||") {
                Verdict::fail("the Cellarer waits — the lazy default is a closure `|| ...`")
            } else {
                Verdict::pass(
                    "the Cellarer taps the cask. \"a fresh draught — drawn only when the old runs dry.\"",
                )
            }
        }
        // ── Act 6 (Iterator Forge): collections, iterator adapters,
        // closure capture.
        "hashmap_basic" => {
            if !source.contains("HashMap") {
                Verdict::fail("the Keymaster waits — missing required: `HashMap`")
            } else if !source.contains(".insert(") {
                Verdict::fail("the Keymaster waits — store a value with `.insert(`")
            } else if !source.contains(".get(") {
                Verdict::fail("the Keymaster waits — look it up with `.get(`")
            } else {
                Verdict::pass("the Keymaster turns the ring. \"every store under its own key.\"")
            }
        }
        "iter_filter" => {
            if !source.contains(".filter(") {
                Verdict::fail("the Sifter shakes the screen — missing required: `.filter(`")
            } else if !source.contains(".collect") {
                Verdict::fail("the Sifter shakes the screen — gather the survivors with `.collect`")
            } else {
                Verdict::pass(
                    "the Sifter shakes the screen. \"only what fits the mesh falls through.\"",
                )
            }
        }
        "iter_fold" => {
            if !source.contains(".fold(") {
                Verdict::fail("the Smelter waits — missing required: `.fold(`")
            } else {
                Verdict::pass("the Smelter tips the crucible. \"many ores, one ingot.\"")
            }
        }
        "iter_enumerate" => {
            if !source.contains(".enumerate(") {
                Verdict::fail("the Tallywright waits — missing required: `.enumerate(`")
            } else {
                Verdict::pass(
                    "the Tallywright stamps the row. \"each item numbered as it passes.\"",
                )
            }
        }
        "iter_zip" => {
            if !source.contains(".zip(") {
                Verdict::fail("the Riveter waits — missing required: `.zip(`")
            } else {
                Verdict::pass("the Riveter drives the rivet. \"two plates, aligned pair by pair.\"")
            }
        }
        "closure_move" => {
            if !source.contains("move |") {
                Verdict::fail(
                    "the Bondsmith waits — missing required: a `move` closure (`move |...|`)",
                )
            } else {
                Verdict::pass("the Bondsmith seals the charge. \"owned, and carried where I go.\"")
            }
        }
        // ── Act 7 (Concurrent Coast): threads, Arc/Mutex, channels,
        // atomics, scoped threads, async.
        "thread_spawn" => {
            if !source.contains("thread::spawn") {
                Verdict::fail("the Dockmaster waits — missing required: `thread::spawn`")
            } else if !source.contains(".join(") {
                Verdict::fail("the Dockmaster waits — wait for the worker with `.join(`")
            } else {
                Verdict::pass(
                    "the Dockmaster waves them off. \"sent to work — and met again at the gate.\"",
                )
            }
        }
        "arc_mutex" => {
            if !source.contains("Arc") {
                Verdict::fail("the Lighthouse Keeper waits — share ownership with `Arc`")
            } else if !source.contains("Mutex") {
                Verdict::fail("the Lighthouse Keeper waits — guard the value with `Mutex`")
            } else if !source.contains(".lock(") {
                Verdict::fail("the Lighthouse Keeper waits — take exclusive access with `.lock(`")
            } else {
                Verdict::pass(
                    "the Lighthouse Keeper turns the key. \"one hand on the lamp at a time.\"",
                )
            }
        }
        "mpsc_channel" => {
            if !source.contains("mpsc::channel") {
                Verdict::fail("the Signaler waits — missing required: `mpsc::channel`")
            } else if !source.contains(".send(") {
                Verdict::fail("the Signaler waits — push a value in with `.send(`")
            } else if !source.contains(".recv(") {
                Verdict::fail("the Signaler waits — read it at the far end with `.recv(`")
            } else {
                Verdict::pass(
                    "the Signaler flashes the lamp. \"sent down the coast, read at the next tower.\"",
                )
            }
        }
        "atomic" => {
            if !source.contains("Atomic") {
                Verdict::fail("the Tidewatch waits — use an `Atomic` type")
            } else if !source.contains(".fetch_add(") {
                Verdict::fail("the Tidewatch waits — bump it atomically with `.fetch_add(`")
            } else {
                Verdict::pass("the Tidewatch clicks the gauge. \"one notch up — no gatekeeper.\"")
            }
        }
        "thread_scope" => {
            if !source.contains("thread::scope") {
                Verdict::fail("the Harbormaster waits — missing required: `thread::scope`")
            } else if !source.contains(".spawn(") {
                Verdict::fail("the Harbormaster waits — launch a scoped thread with `.spawn(`")
            } else {
                Verdict::pass(
                    "the Harbormaster rings the bell. \"every boat in before the harbor closes.\"",
                )
            }
        }
        "async_fn" => {
            if !source.contains("async fn") {
                Verdict::fail("the Tideforecaster waits — missing required: `async fn`")
            } else if !source.contains(".await") {
                Verdict::fail("the Tideforecaster waits — drive the future with `.await`")
            } else {
                Verdict::pass(
                    "the Tideforecaster lowers the glass. \"the tide will come — awaited, not forced.\"",
                )
            }
        }
        // ── Act 8 (Vault of Pointers): Box, Rc, RefCell, Cell, Weak.
        "box_basic" => {
            if !source.contains("Box<") {
                Verdict::fail("the Vaultwright waits — missing required: `Box<...>`")
            } else if !source.contains("Box::new") {
                Verdict::fail("the Vaultwright waits — put the value on the heap with `Box::new`")
            } else {
                Verdict::pass(
                    "the Vaultwright seals the box. \"on the heap it holds what the stack cannot.\"",
                )
            }
        }
        "rc_basic" => {
            if !source.contains("Rc::new") {
                Verdict::fail("the Sharekeeper waits — missing required: `Rc::new`")
            } else if !source.contains("Rc::clone") {
                Verdict::fail("the Sharekeeper waits — hand out a second handle with `Rc::clone`")
            } else {
                Verdict::pass("the Sharekeeper tallies the claims. \"one hoard, many holders.\"")
            }
        }
        "refcell" => {
            if !source.contains("RefCell") {
                Verdict::fail("the Warden waits — missing required: `RefCell`")
            } else if !source.contains(".borrow_mut(") {
                Verdict::fail("the Warden waits — mutate the inner value with `.borrow_mut(`")
            } else {
                Verdict::pass(
                    "the Warden checks the ledger. \"changed within — by the runtime's leave.\"",
                )
            }
        }
        "cell" => {
            if !source.contains("Cell::new") {
                Verdict::fail("the Swapwarden waits — missing required: `Cell::new`")
            } else if !source.contains(".set(") {
                Verdict::fail("the Swapwarden waits — swap the whole value with `.set(`")
            } else {
                Verdict::pass("the Swapwarden trades the coin. \"the whole value, swapped clean.\"")
            }
        }
        "rc_refcell" => {
            if !source.contains("Rc::new") {
                Verdict::fail("the Strongbox waits — share ownership with `Rc::new`")
            } else if !source.contains("RefCell") {
                Verdict::fail("the Strongbox waits — make it mutable inside with `RefCell`")
            } else if !source.contains(".borrow_mut(") {
                Verdict::fail("the Strongbox waits — open and change it via `.borrow_mut(`")
            } else {
                Verdict::pass(
                    "the Strongbox turns the keys. \"many owners, and the box still opens.\"",
                )
            }
        }
        "weak_ref" => {
            if !source.contains("Weak") {
                Verdict::fail("the Ghostkeeper waits — missing required: a `Weak` reference")
            } else if !source.contains("downgrade") {
                Verdict::fail("the Ghostkeeper waits — make the weak handle with `Rc::downgrade`")
            } else if !source.contains(".upgrade(") {
                Verdict::fail("the Ghostkeeper waits — reclaim it (or not) with `.upgrade(`")
            } else {
                Verdict::pass(
                    "the Ghostkeeper tugs the tether. \"a hold that holds nothing alive.\"",
                )
            }
        }
        _ => Verdict::pass(format!(
            "[freeform] received {} bytes. encounter `{encounter_id}` has no grader yet.",
            source.len()
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

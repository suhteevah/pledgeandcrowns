// SPDX-License-Identifier: MIT
//! Mission registry + the F-press dialogue/editor handoff.
//!
//! A mission is a pairing of (npc_prompt, starter_code, encounter_id).
//! When the player stands by an NPC and presses F, this plugin loads
//! the mission's starter code into the editor and pops it open. The
//! editor's existing Compile button (wired in `compile_client`) sends
//! `encounter_id` to the API so server-side validation can branch on
//! which mission is being graded.

use crate::plugins::editor::EditorState;
use crate::plugins::npc::NearbyNpc;
use crate::plugins::progress::MissionProgress;
use crate::plugins::state::{GameState, PauseState};
use crate::plugins::trial;
use crate::plugins::ui_theme;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

#[derive(Clone)]
pub struct Mission {
    pub id: &'static str,
    pub npc_name: &'static str,
    /// One-sentence brief shown above the NPC's interaction prompt.
    pub prompt: &'static str,
    /// Multi-section tutorial shown in the editor side panel once the
    /// mission is active. Plain text with `## Section` headers; the
    /// renderer styles them. Keep each mission ~100-200 words — the
    /// audience is dev-curious adults, not absolute beginners.
    pub tutorial: &'static str,
    /// Mission id that must be cleared before this one is unlocked.
    ///
    /// Every literal in `MissionRegistry::default` writes `prereq: None`;
    /// that is a placeholder. After the list is built, a loop sets each
    /// mission's prereq to the mission listed immediately before it, so
    /// progression is strict-linear in registry order and only the first
    /// mission ends up with `None`. Registry tests enforce no cycles, no
    /// dangling references and full reachability;
    /// `tests/mission_text.rs` proves N+1 stays locked until N is cleared.
    pub prereq: Option<&'static str>,
    /// False when this build does not include the mission (the `demo`
    /// feature stops after [`DEMO_LAST_MISSION_ID`]). Like `prereq`, the
    /// literals all write `true` and `MissionRegistry::default` sets it.
    pub available: bool,
    pub starter_code: &'static str,
}

/// The last mission of Act 2: the final mission playable in the free demo
/// (`--features demo`). It is the last stage of the Borrow Checker's
/// trial, the Act 2 boss, so the demo ends on the boss (design/01 and
/// design/07: "Acts 1-2 together are the MVP vertical slice"). Acts 1-2
/// are the first 24 missions in the registry (21 lessons plus the three
/// trial stages); Act 3 opens with `impl_method`.
pub const DEMO_LAST_MISSION_ID: &str = "bridge_lantern";

/// Where the end-of-demo panel will point players for the full game.
/// Deliberately empty and unused: the store, price and name are blocked
/// on the trademark search and store setup (CLAUDE.md hard rule 6).
pub const FULL_GAME_URL: Option<&str> = None;

impl Mission {
    /// True when the player can start this mission: it is in this build
    /// and its prereq (if any) is cleared. This does not look at whether
    /// the mission itself is cleared; callers check that separately so
    /// the completion-view revisit flow works.
    pub fn is_unlocked(&self, progress: &MissionProgress) -> bool {
        let unlocked = self.available
            && match self.prereq {
                None => true,
                Some(prev) => progress.is_cleared(prev),
            };
        tracing::trace!(
            mission = self.id,
            available = self.available,
            prereq = ?self.prereq,
            unlocked,
            "is_unlocked"
        );
        unlocked
    }
}

#[derive(Resource)]
pub struct MissionRegistry {
    pub missions: Vec<Mission>,
}

impl MissionRegistry {
    /// First not-yet-cleared mission in registry order, among the
    /// missions this build includes. Drives the "next mission" hint in
    /// the HUD so players know where to head. `None` once everything
    /// available is cleared.
    pub fn next_locked(&self, progress: &MissionProgress) -> Option<&Mission> {
        self.missions
            .iter()
            .find(|m| m.available && !progress.is_cleared(m.id))
    }

    pub fn find(&self, id: &str) -> Option<&Mission> {
        self.missions.iter().find(|m| m.id == id)
    }

    /// True when `id` names a mission this build includes. Unknown ids
    /// are unavailable.
    pub fn is_available(&self, id: &str) -> bool {
        self.find(id).is_some_and(|m| m.available)
    }

    /// True in a `demo` build once the last demo mission is cleared.
    /// Always false in the full game.
    pub fn demo_complete(&self, progress: &MissionProgress) -> bool {
        cfg!(feature = "demo") && progress.is_cleared(DEMO_LAST_MISSION_ID)
    }
}

impl Default for MissionRegistry {
    fn default() -> Self {
        let mut missions = vec![
            // Starters are deliberately neutral templates — they
            // must NOT contain the grader's required tokens, or
            // the player could submit them unchanged and pass.
            // The contract suite (game/tests/contract.rs) enforces
            // this invariant.
            Mission {
                id: "intro_let_binding",
                npc_name: "Ferris",
                prompt: "Bind the integer forty-two to a variable named `answer`.",
                tutorial: "## Concept\n\
Rust is statically typed: every value has a type known at compile time. \
You give a value a name with `let`. Bindings are immutable by default — \
once a name points at a value, that name can't be reassigned. This is \
the opposite of Python or JS, where `x = 1` and later `x = 2` is normal.\n\n\
## Syntax\n\
```\nlet name = value;\nlet name: Type = value;  // explicit type\n```\n\
The type annotation is usually optional; the compiler infers from the \
value. `42` is an `i32` by default.\n\n\
## Task\n\
Replace the placeholder `_todo` binding with a binding named `answer` \
holding the value `42`. Update the `println!` accordingly.\n\n\
## Hint\n\
The grader is looking for a `let answer` binding and the literal `42`. \
A type annotation is allowed but not required.",
                prereq: None,
                available: true,
                starter_code: "fn main() {\n    let _todo = 0;\n    println!(\"{_todo}\");\n}\n",
            },
            Mission {
                id: "double_function",
                npc_name: "Trait Mage",
                prompt: "Define `double(n: i32) -> i32` that returns `n` multiplied by two.",
                tutorial: "## Concept\n\
The Trait Mage frames a function as a *named spell* — same inputs, same \
output, no surprises. Functions in Rust are declared with `fn`. \
Parameters carry an explicit type — there is no implicit casting between \
numeric types. The return type follows `->`. The last expression in the \
body is the return value when there's no trailing semicolon (Rust calls \
this an *implicit return*).\n\n\
## Syntax\n\
```\nfn add(a: i32, b: i32) -> i32 {\n    a + b   // no semicolon = return value\n}\n```\n\
Adding a semicolon after `a + b` would turn it into a statement and \
return `()` (the unit type), which would not match the declared `i32` return.\n\n\
## Task\n\
Define a function `double` that takes a single `i32` and returns it \
multiplied by two. Then call it from `main` with `21` and print the \
result.\n\n\
## Hint\n\
The grader needs to see `fn double`, the type `i32`, and the expression \
`* 2`. `n * 2` is the canonical body.",
                prereq: None,
                available: true,
                starter_code: "fn _todo() {}\n\nfn main() {\n    // call your function with 21 and print the result\n}\n",
            },
            Mission {
                id: "borrow_preview",
                npc_name: "The Borrow Checker",
                prompt: "Take an immutable reference to `value` and pass it to a print macro.",
                tutorial: "## Concept\n\
Ownership is Rust's defining feature. Each value has exactly one owner; \
when the owner goes out of scope, the value is dropped. To use a value \
without taking ownership, you *borrow* it via a reference: `&value` \
(immutable) or `&mut value` (exclusive, mutable). The Borrow Checker \
enforces the rules at compile time — no runtime cost.\n\n\
## Syntax\n\
```\nlet s = String::from(\"hi\");\nlet r = &s;          // borrow\nprintln!(\"{r}\");    // r is still valid; s still owns the data\n```\n\
The macros `println!` and `format!` accept references directly via the \
`{name}` capture syntax.\n\n\
## Task\n\
Bind `value` (already provided), then create an immutable reference to \
it and print the reference. Do not move the value — just borrow it.\n\n\
## Hint\n\
The grader looks for `&value` and `println!`. The simplest answer is \
`let r = &value; println!(\"{r}\");`",
                prereq: None,
                available: true,
                starter_code: "fn main() {\n    let value = String::from(\"sample\");\n    // build a reference to the binding above and print it\n}\n",
            },
            Mission {
                id: "mut_binding",
                npc_name: "The Smith",
                prompt: "Declare a mutable variable, then increment it by one.",
                tutorial: "## Concept\n\
`let` bindings are immutable by default — that's a deliberate Rust \
choice that catches a class of bugs. To allow reassignment, opt in with \
`let mut`. The `mut` keyword makes mutability visible at the binding \
site, so a reader can tell what changes and what doesn't just by \
scanning the code.\n\n\
## Syntax\n\
```\nlet mut counter = 0;\ncounter += 1;       // compound assignment\ncounter = counter + 1;  // also fine\n```\n\
Compound-assignment operators (`+=`, `-=`, `*=`, `/=`) work the same \
way as in C/Python.\n\n\
## Task\n\
Make `x` mutable, then increment it by one using the compound-assignment \
operator.\n\n\
## Hint\n\
The grader requires both `let mut` and `+= 1` to appear in the source. \
The Smith's rule: name what changes. The `mut` keyword *is* that name.",
                prereq: None,
                available: true,
                starter_code: "fn main() {\n    let x = 0;\n    // make x mutable, then add one\n    println!(\"{x}\");\n}\n",
            },
            Mission {
                id: "if_else_sign",
                npc_name: "The Cartographer",
                prompt: "Branch on the sign of an `i32` — negative, zero, positive.",
                tutorial: "## Concept\n\
`if` in Rust is an *expression*, not just a statement — it produces a \
value, so you can put it on the right-hand side of `let` or use it as \
the last expression in a function. Each branch must produce the same \
type. `else if` chains let you cover several cases.\n\n\
## Syntax\n\
```\nlet label = if n < 0 {\n    \"negative\"\n} else if n == 0 {\n    \"zero\"\n} else {\n    \"positive\"\n};\n```\n\
Note: no parentheses around the condition. That's a Rust convention — \
the braces around the bodies do the visual grouping.\n\n\
## Task\n\
Implement `sign(n: i32) -> &'static str` so it returns `\"neg\"`, \
`\"zero\"`, or `\"pos\"` depending on the sign of `n`.\n\n\
## Hint\n\
The Cartographer's compass picks one road at the fork; your `if` does \
the same with the sign of `n`. The grader requires `if `, `else`, and a \
literal comparison `< 0` to appear. The canonical body is a three-arm \
`if` / `else if` / `else`.",
                prereq: None,
                available: true,
                starter_code: "fn sign(_n: i32) -> &'static str {\n    // three branches, one per sign\n    \"replace me\"\n}\n\nfn main() {\n    println!(\"{}\", sign(-3));\n}\n",
            },
            Mission {
                id: "loop_break",
                npc_name: "The Bellringer",
                prompt: "Use `loop` to find the first power of two at or above one hundred.",
                tutorial: "## Concept\n\
The Bellringer's image: the rope is the loop body, the bell above is \
the threshold, the sound is the `break` that yields a value. Rust has \
three loop constructs: `loop` (infinite), `while` (predicate), and \
`for` (iterator). The infinite `loop` is the only one that can return \
a value: `break expr;` exits the loop and yields `expr` as the loop's \
value. That makes `let result = loop { ... };` an idiomatic accumulator \
pattern when the exit condition isn't easily expressed as a `while`.\n\n\
## Syntax\n\
```\nlet first_big = loop {\n    if n >= threshold { break n; }\n    n *= 2;\n};\n```\n\
The semicolon after the closing brace is required because `let` is a \
statement.\n\n\
## Task\n\
Starting from `1`, double the value each iteration until it reaches at \
least `100`, then break and return that value.\n\n\
## Hint\n\
The grader looks for `loop`, `break`, and `*= 2` (the doubling step).",
                prereq: None,
                available: true,
                starter_code: "fn main() {\n    let mut _n = 1;\n    // loop until _n >= 100, doubling each step, then break the value\n    println!(\"{_n}\");\n}\n",
            },
            Mission {
                id: "match_option",
                npc_name: "The Oracle",
                prompt: "Pattern-match an `Option<i32>` — return the inner value or zero.",
                tutorial: "## Concept\n\
The Oracle holds an orb whose top half glows and bottom half stays dark — \
the answer present and the answer absent, both real, both seen. \
`Option<T>` is Rust's standard \"maybe a value\" type. Instead of `null`, \
the language forces you to handle the absent case explicitly: `Some(T)` \
holds a value, `None` doesn't. `match` is the exhaustive way to inspect \
it — the compiler refuses to compile if you forget an arm.\n\n\
## Syntax\n\
```\nmatch maybe {\n    Some(n) => n,\n    None => 0,\n}\n```\n\
Match arms produce values; the whole `match` is an expression. Each arm \
must produce the same type. Use `=>` (fat arrow), not `->`.\n\n\
## Task\n\
Implement `unwrap_or_zero(x: Option<i32>) -> i32` using a `match` with \
both `Some` and `None` arms.\n\n\
## Hint\n\
The grader requires `match`, `Some(`, and `None` to all appear. \
Don't shortcut with `.unwrap_or(0)` — this mission is teaching `match`.",
                prereq: None,
                available: true,
                starter_code: "fn unwrap_or_zero(_x: Option<i32>) -> i32 {\n    // pattern-match and return the inner value or 0\n    -1\n}\n\nfn main() {\n    println!(\"{}\", unwrap_or_zero(Some(7)));\n}\n",
            },
            Mission {
                id: "struct_basic",
                npc_name: "The Herald",
                prompt: "Define a `Knight` struct with `name` and `hp`, then read the name back.",
                tutorial: "## Concept\n\
The Herald wears a tabard of three banded fields and reads each one aloud — \
that is what a `struct` is. \
A `struct` groups related values under one type with named fields. \
Fields are private outside the defining module by default; you'd add \
`pub` to expose them. Construction uses the `TypeName { field: value }` \
literal, one `field: value` per field; access uses `instance.field`.\n\n\
## Syntax\n\
```\nstruct Point { x: i32, y: i32 }\nlet p = Point { x: 3, y: 4 };\nprintln!(\"{}\", p.x);\n```\n\
Fields are reordering-tolerant in the literal — name them, don't position \
them. There is no implicit constructor; you list every field unless you \
write a `fn new(...) -> Self` impl.\n\n\
## Task\n\
Define `struct Knight` with a `name: String` and `hp: i32`. In `main`, \
build one and `println!` its name.\n\n\
## Hint\n\
The grader looks for `struct Knight`, `name:`, `hp:`, and `.name` in \
the source.",
                prereq: None,
                available: true,
                starter_code: "// define Knight here\n\nfn main() {\n    // build a Knight and print its name\n}\n",
            },
            Mission {
                id: "vec_iter",
                npc_name: "The Cooper",
                prompt: "Sum the elements of a `Vec<i32>` using `.iter().sum()`.",
                tutorial: "## Concept\n\
The Cooper's barrel is the lesson: open the lid, walk every stave, do \
something with each, close the lid. A `Vec<T>` is the heap-allocated \
growable list — Rust's analogue of Python `list` or JS `Array`. The \
standard way to walk it is to call `.iter()` for an immutable borrow of \
each element, then chain an adapter (`.map`, `.filter`) and a consumer \
(`.sum`, `.collect`, `.count`). Iterators are zero-cost: the compiler \
usually fuses them into the same machine code a hand-written loop \
would produce.\n\n\
## Syntax\n\
```\nlet v = vec![1, 2, 3];\nlet total: i32 = v.iter().sum();\n```\n\
The type annotation on the binding (`: i32`) is what tells `sum` which \
numeric type to add into. Without it the compiler can't pick.\n\n\
## Task\n\
Build a `Vec<i32>` holding `10`, `20` and `30`, sum it via \
`.iter().sum()`, and print the total. Your program should print `60`.\n\n\
## Hint\n\
The grader looks for `vec!`, `.iter()`, and `.sum`. The annotation on \
the binding (or a turbofish on `sum`) is what lets the code compile at \
all: without it `sum` cannot tell which number type to produce.",
                prereq: None,
                available: true,
                starter_code: "fn main() {\n    // build a Vec<i32> of 10, 20 and 30, sum it via .iter().sum(), print the total\n    println!(\"todo\");\n}\n",
            },
            Mission {
                id: "tuple_destructure",
                npc_name: "The Twin",
                prompt: "Destructure a 2-tuple in a single `let` binding.",
                tutorial: "## Concept\n\
The Twin is one body bound to two names — left half and right half, each \
with its own ring. Destructuring is the same trick at the binding site. \
Tuples bundle a fixed number of values of possibly-different types. \
Pattern syntax in `let` lets you peel them apart in one statement, \
giving each component its own name. This generalises to nested tuples, \
references, and structs — the same syntax shape works everywhere a \
pattern is allowed.\n\n\
## Syntax\n\
```\nlet (x, y) = (3, 4);\nlet (a, b, _) = (1, 2, 99);   // _ ignores\n```\n\
The pattern on the left and the value on the right must have matching \
shape, or the compiler refuses.\n\n\
## Task\n\
Bind the two halves of `(3, 4)` to `a` and `b` in one `let` and print \
both.\n\n\
## Hint\n\
The grader requires `let (` (the pattern open), a `,` (separator), \
and `) =` (pattern close + assign).",
                prereq: None,
                available: true,
                starter_code: "fn main() {\n    let _pair = (3, 4);\n    // destructure into a and b in a single let, then print both\n}\n",
            },
            Mission {
                id: "borrow_mut",
                npc_name: "The Forgewright",
                prompt: "Take an `&mut i32` and write through the deref.",
                tutorial: "## Concept\n\
An exclusive (mutable) borrow `&mut T` lets a function change the value \
the caller still owns. While the `&mut` is live, no other borrow — read \
or write — may exist; the Borrow Checker enforces this at compile time. \
That single-writer rule is what makes Rust data races a compile error \
rather than a runtime gremlin.\n\n\
## Syntax\n\
```\nfn set_to(x: &mut i32, v: i32) {\n    *x = v;   // deref to write\n}\nlet mut n = 0;\nset_to(&mut n, 7);\n```\n\
The `*x` deref is required to assign through the reference; without it \
the compiler rejects the assignment (E0308): `x` is a `&mut i32`, not an \
`i32`.\n\n\
## Task\n\
Write `fn bump(x: &mut i32)` that sets `*x` to a new value. Then call it \
from `main` against a mutable local.\n\n\
## Hint\n\
The grader requires `fn bump`, the parameter type `&mut i32`, and a \
`*x` deref in the body.",
                prereq: None,
                available: true,
                starter_code: "fn _todo(_x: i32) {}\n\nfn main() {\n    let mut _n = 0;\n    // call your function with a mutable borrow of _n\n    println!(\"{_n}\");\n}\n",
            },
            Mission {
                id: "string_vs_str",
                npc_name: "The Linguist",
                prompt: "Write a function that takes `&str` and call it with both a `String` and a literal.",
                tutorial: "## Concept\n\
Rust has two string types: `String` (owned, growable, heap) and `&str` \
(borrowed slice, often backed by a literal in the binary). A function \
that accepts `&str` is the most general signature: callers pass string \
literals directly, and `String` values coerce automatically when borrowed \
with `&` (via `Deref<Target=str>`). Reach for `String` only when you \
need ownership — otherwise prefer `&str`.\n\n\
## Syntax\n\
```\nfn greet(name: &str) {\n    println!(\"hi {name}\");\n}\nlet s = String::from(\"world\");\ngreet(&s);       // String → &str by deref coercion\ngreet(\"static\"); // &'static str literal\n```\n\
The same `greet` body serves both call sites with zero allocation.\n\n\
## Task\n\
Write `fn greet(name: &str)`. From `main`, build a `String` and call \
`greet` against it, then call `greet` once more with a string literal.\n\n\
## Hint\n\
The grader requires `fn greet`, the type `&str`, and a `String::from` \
caller in `main`.",
                prereq: None,
                available: true,
                starter_code: "fn _todo(_n: i32) {}\n\nfn main() {\n    // define a function above that takes a string slice,\n    // then call it once with a String and once with a literal\n}\n",
            },
            Mission {
                id: "option_unwrap_or",
                npc_name: "The Pilgrim",
                prompt: "Collapse an `Option<i32>` to `i32` with `.unwrap_or(default)`.",
                tutorial: "## Concept\n\
`Option<T>` carries a value or nothing, and Rust forces you to handle \
both. A `match` is exhaustive but verbose; the standard library ships a \
family of *combinators* — `.unwrap_or`, `.unwrap_or_else`, `.map`, \
`.and_then` — that collapse common cases into one line. `.unwrap_or(d)` \
returns the inner value if `Some`, otherwise `d`. Different idiom from \
`match`; both are fluent Rust.\n\n\
## Syntax\n\
```\nfn safe(x: Option<i32>) -> i32 {\n    x.unwrap_or(0)\n}\nassert_eq!(safe(Some(5)), 5);\nassert_eq!(safe(None), 0);\n```\n\
`.unwrap_or_else(|| compute())` defers the default computation when it's \
expensive — but for a constant, plain `.unwrap_or` is the right call.\n\n\
## Task\n\
Implement `fn safe(x: Option<i32>) -> i32` using `.unwrap_or` to return \
`0` when `x` is `None`.\n\n\
## Hint\n\
The grader requires both `Option<` and `.unwrap_or(`. Don't reach for \
`match` on this one — that's the previous mission.",
                prereq: None,
                available: true,
                starter_code: "fn safe(_x: i32) -> i32 {\n    // collapse the absent case to a default with a combinator\n    -1\n}\n\nfn main() {\n    println!(\"{}\", safe(0));\n}\n",
            },
            Mission {
                id: "for_in_range",
                npc_name: "The Drillmaster",
                prompt: "Iterate the half-open range `0..10` with a `for` loop.",
                tutorial: "## Concept\n\
`for` in Rust is sugar over the iterator protocol — under the hood it \
calls `IntoIterator::into_iter` on the right-hand side and walks the \
result. The simplest iterable is the half-open range `a..b`, which \
yields each integer from `a` (inclusive) up to `b` (exclusive). Prefer \
`for` over hand-rolled `while` whenever you're walking a known sequence.\n\n\
## Syntax\n\
```\nfor i in 0..10 {\n    println!(\"{i}\");   // prints 0 through 9\n}\nfor x in vec![1, 2, 3] { /* ... */ }\n```\n\
Use `0..=10` (with `=`) for an inclusive upper bound. The loop variable \
is a fresh binding each iteration; you don't need `mut`.\n\n\
## Task\n\
Write a `main` that iterates `i` over `0..10` and prints each value.\n\n\
## Hint\n\
The grader requires `for `, ` in `, and the literal range `0..10`.",
                prereq: None,
                available: true,
                starter_code: "fn main() {\n    // walk the range 0..10 with a for loop, printing each step\n    println!(\"todo\");\n}\n",
            },
            Mission {
                id: "closure_basic",
                npc_name: "The Reckoner",
                prompt: "Bind a two-argument closure to `add` and call it.",
                tutorial: "## Concept\n\
A *closure* is an anonymous function that can capture variables from its \
defining scope. Rust closures look like `|args| expr` and behave like \
values — you bind one to a name with `let`, pass it to higher-order \
functions, or return it from a function. The compiler infers parameter \
and return types from how the closure is used.\n\n\
## Syntax\n\
```\nlet add = |a, b| a + b;\nassert_eq!(add(2, 3), 5);\nlet greet = |name: &str| println!(\"hi {name}\");\n```\n\
A multi-line body uses braces: `|x| { let y = x + 1; y * 2 }`. Closures \
that capture by move use the `move` keyword; we'll meet those later.\n\n\
## Task\n\
Bind a closure named `add` that takes two parameters `a` and `b` and \
returns their sum. Then call `add(19, 23)` from `main` and print the \
result. Your program should print `42`.\n\n\
## Hint\n\
The grader requires `let add`, the closure-literal opener `= |`, and \
a body that adds the two parameters, like `a + b`.",
                prereq: None,
                available: true,
                starter_code: "fn main() {\n    let _todo = 0;\n    // bind a closure named `add` that returns a + b, then print add(19, 23)\n    println!(\"{_todo}\");\n}\n",
            },
            Mission {
                id: "slice_basic",
                npc_name: "The Quartermaster",
                prompt: "Write `sum_slice(xs: &[i32]) -> i32` that sums a borrowed slice.",
                tutorial: "## Concept\n\
A slice `&[T]` is a borrowed view into a contiguous run of `T`s — a \
pointer plus a length. It owns nothing; the buffer behind it lives \
elsewhere (a `Vec`, an array, a string). Functions that read sequence \
data should take `&[T]`, not `&Vec<T>` — `&Vec<T>` is strictly less \
flexible because every `&Vec<T>` coerces to `&[T]` but not the reverse. \
Same lesson as `&str` vs `String`: prefer the borrowed view.\n\n\
## Syntax\n\
```\nfn first(xs: &[i32]) -> Option<&i32> { xs.iter().next() }\nlet v = vec![1, 2, 3];\nfirst(&v);          // Vec<i32> -> &[i32]\nfirst(&[10, 20]);   // array -> &[i32]\n```\n\
The `.iter()` method on a slice yields `&T`s; chain it with `.sum`, \
`.max`, `.filter`, etc.\n\n\
## Task\n\
Define `fn sum_slice(xs: &[i32]) -> i32` that returns the sum of the \
slice, then call it from `main` against a small literal array.\n\n\
## Hint\n\
The grader requires `fn sum_slice`, the slice type `&[i32]`, and a \
`.iter()` call somewhere in the body.",
                prereq: None,
                available: true,
                starter_code: "fn _todo(_n: i32) -> i32 { 0 }\n\nfn main() {\n    // define sum_slice above with a slice parameter, then call it\n}\n",
            },
            Mission {
                id: "result_question_mark",
                npc_name: "The Auditor",
                prompt: "Parse a string into an integer and propagate the error with `?`.",
                tutorial: "## Concept\n\
`Result<T, E>` is the standard error-carrying type — `Ok(T)` on success, \
`Err(E)` on failure. The `?` operator at the end of a fallible expression \
either unwraps the `Ok` and continues, or short-circuits the function \
with the `Err` value (after running it through `From` if needed). It \
turns ten lines of `match` into one — the idiomatic way to write \
fallible code in Rust.\n\n\
## Syntax\n\
```\nfn parse_pair(a: &str, b: &str) -> Result<(i32, i32), String> {\n    let x = a.parse::<i32>().map_err(|e| e.to_string())?;\n    let y = b.parse::<i32>().map_err(|e| e.to_string())?;\n    Ok((x, y))\n}\n```\n\
`?` works on `Option` too — `None` short-circuits the function, returning \
`None`. The function's return type must match.\n\n\
## Task\n\
Write `fn parse_int(s: &str) -> Result<i32, String>` that uses `.parse` \
and `?` to short-circuit on failure, returning `Ok(n)` on success.\n\n\
## Hint\n\
The grader requires `Result<`, `.parse`, and a `?` operator (followed by \
`;` or a newline) to all appear.",
                prereq: None,
                available: true,
                starter_code: "fn parse_int(_s: &str) -> i32 {\n    // change the signature to return Result, then propagate the parse error with ?\n    0\n}\n\nfn main() {\n    let _ = parse_int(\"42\");\n}\n",
            },
            Mission {
                id: "derive_debug",
                npc_name: "The Chronicler",
                prompt: "Derive `Debug` on `struct Item` and print one with `{:?}`.",
                tutorial: "## Concept\n\
Rust's `Debug` trait is the developer-facing formatting trait — what you \
get from `{:?}` and `{:#?}`. You almost never write a `Debug` impl by \
hand: the `#[derive(Debug)]` attribute generates one mechanically from \
the struct's fields, as long as every field is itself `Debug`. The \
parallel `Display` trait is the user-facing one (`{}`); that one you \
*do* write manually because the format is a UX choice.\n\n\
## Syntax\n\
```\n#[derive(Debug)]\nstruct Point { x: i32, y: i32 }\nlet p = Point { x: 3, y: 4 };\nprintln!(\"{p:?}\");      // Point { x: 3, y: 4 }\nprintln!(\"{p:#?}\");     // pretty-printed across lines\n```\n\
You can derive several traits at once: `#[derive(Debug, Clone, PartialEq)]`.\n\n\
## Task\n\
Define `struct Item` with at least one field, derive `Debug`, build one \
in `main`, and print it with the debug formatter.\n\n\
## Hint\n\
The grader requires `#[derive(Debug)]`, `struct Item`, and `:?` (the \
debug formatter) to all appear.",
                prereq: None,
                available: true,
                starter_code: "// add a derive attribute, then define struct Item\nstruct Item { name: String }\n\nfn main() {\n    let _item = Item { name: String::from(\"ring\") };\n    // print _item using the debug formatter\n}\n",
            },
            Mission {
                id: "iter_map_collect",
                npc_name: "The Alchemist",
                prompt: "Map a Vec through `|x| x * 2` and `collect` into a new Vec.",
                tutorial: "## Concept\n\
The iterator triple — `iter` / `map` / `collect` — is Rust's most common \
data-shaping idiom. `.iter()` produces a stream of `&T`. `.map(f)` \
transforms each item lazily — no allocation yet, just a chain of \
adapters. `.collect()` is the consumer that materialises the chain into \
a concrete container; the target type (often `Vec<_>`) is inferred from \
the binding's annotation or specified with a turbofish.\n\n\
## Syntax\n\
```\nlet v = vec![1, 2, 3];\nlet doubled: Vec<i32> = v.iter().map(|x| x * 2).collect();\n// or with a turbofish:\nlet doubled = v.iter().map(|x| x * 2).collect::<Vec<_>>();\n```\n\
The closure receives `&i32` here. std implements arithmetic operators for \
`&i32`, so `x * 2` works without an explicit `*x`; comparisons like \
`x == 2` still need `*x`.\n\n\
## Task\n\
Build a `Vec<i32>`, map each element through a `|x|` closure that \
doubles it, and collect the result into a new `Vec`.\n\n\
## Hint\n\
The grader requires `.map(`, `.collect`, and a single-element closure \
binder `|x|`.",
                prereq: None,
                available: true,
                starter_code: "fn main() {\n    let v = vec![1, 2, 3];\n    // chain iter / map / collect into a new Vec\n    let _ = v;\n}\n",
            },
            Mission {
                id: "enum_match",
                npc_name: "The Heraldic Sage",
                prompt: "Define `enum Direction` with four compass variants and `match` each to its bearing.",
                tutorial: "## Concept\n\
The Heraldic Sage's sash carries one stitched panel per variant — oak, \
river, sun, azure — and a `match` arm walks left to right across the chest. \
A Rust `enum` is a *sum type*: a value is exactly one of a fixed set of \
named variants. Variants can be unit (`North`), tuple (`Move(i32, i32)`), \
or struct-like (`Hit { damage: u32 }`). `match` is the way to inspect \
one: the compiler verifies the arms cover every variant, so adding a \
new variant later forces you to update every match site — the language \
prevents you from silently forgetting a case.\n\n\
## Syntax\n\
```\nenum Shape { Circle(f32), Square(f32) }\nfn area(s: Shape) -> f32 {\n    match s {\n        Shape::Circle(r) => 3.14 * r * r,\n        Shape::Square(side) => side * side,\n    }\n}\n```\n\
Use the path syntax `EnumName::Variant` to construct or match a variant. \
You can also `use Shape::*;` inside a function to drop the prefix.\n\n\
## Task\n\
Define `enum Direction` with the variants `North`, `East`, `South` and \
`West`. Write `fn degrees(d: Direction) -> i32` that uses `match` to give \
each its compass bearing: `0`, `90`, `180` and `270`, in that order. In \
`main`, print `degrees` of all four, in the same order, one per line. Your \
program should print `0`, `90`, `180`, `270`.\n\n\
## Hint\n\
The grader requires `enum Direction`, a `match ` expression, and at \
least one `Direction::` variant path. The mission checker also calls \
your `degrees` with each variant, so the signature must match exactly.",
                prereq: None,
                available: true,
                starter_code: "// define enum Direction with North, East, South and West\n\nfn main() {\n    // print the compass bearing of each Direction, one per line\n}\n",
            },
            Mission {
                id: "while_loop",
                npc_name: "The Tinker",
                prompt: "Use a `while` loop to count down from five to zero.",
                tutorial: "## Concept\n\
The Tinker tightens, checks, tightens again — pouch by pouch along the \
belt — until the work is right. That cycle is `while`. \
It runs its body as long as a predicate stays true. It's the right \
choice when the exit condition is simple and side-effect-driven; when \
you're walking a collection prefer `for x in iter`, when you need to \
return a value prefer `loop` with `break value`.\n\n\
## Syntax\n\
```\nlet mut n = 5;\nwhile n > 0 {\n    n -= 1;\n}\n```\n\
Forgetting to mutate the predicate variable is the classic infinite-loop \
bug — Rust won't save you from logic errors, only from memory unsafety.\n\n\
## Task\n\
Starting from `5`, count down to `0` using a `while` loop with a `-= 1` \
decrement.\n\n\
## Hint\n\
The grader requires `while `, a `> 0` predicate, and `-= 1` for the \
decrement step.",
                prereq: None,
                available: true,
                starter_code: "fn main() {\n    let mut _n = 5;\n    // count down with a while loop and a -= 1 step\n    println!(\"{_n}\");\n}\n",
            },
            // ── Act 2 boss: the Borrow Checker's trial, The Bridge ─────
            // Three stages owned by the Borrow Checker (`trial::TRIALS`),
            // after his prologue `borrow_preview`: move vs borrow, shared
            // vs exclusive borrows, a lifetime on a returned slice. Every
            // starter fails with the error the stage is about, and every
            // compiler error he names is compiled in
            // `tests/mission_text.rs` (hard rule 3).
            Mission {
                id: "bridge_toll",
                npc_name: "The Borrow Checker",
                prompt: "Make `weigh` borrow the cargo, so `main` still owns it after the toll.",
                tutorial: "## Concept\n\
The first toll of the Bridge. Passing a `String` to a function by value \
*moves* it: ownership goes to the parameter, and here `weigh` drops the \
cargo when it returns. The name `cargo` is still in scope in `main`, but it \
no longer owns anything, so the next use is rejected with E0382, \
\"borrow of moved value: `cargo`\". A function that only reads should \
*borrow*: take `&str`, and the caller lends with `&`. The owner keeps it.\n\n\
## Syntax\n\
```\nfn len_of(s: &str) -> usize {\n    s.len()\n}\nlet name = String::from(\"gate\");\nlet n = len_of(&name);   // lend: &String coerces to &str\nprintln!(\"{name} {n}\"); // name is still the owner\n```\n\
A borrow ends when its last use is over; it never takes the value away.\n\n\
## Task\n\
The starter does not compile: `weigh` takes the cargo, then `main` prints \
it. Change `weigh` to `fn weigh(cargo: &str) -> usize` and call it with \
`&cargo`. Your program should print `toll: 11`, then `cargo: lantern oil`.\n\n\
## Hint\n\
The grader needs `fn weigh(cargo: &str)` and a call that lends, \
`weigh(&cargo)`. Calling `weigh(cargo.clone())` also compiles, but it pays \
the toll with a copy: the Borrow Checker wants a loan, not a gift.",
                prereq: None,
                available: true,
                starter_code: "fn weigh(cargo: String) -> usize {\n    cargo.len()\n}\n\nfn main() {\n    let cargo = String::from(\"lantern oil\");\n    // this won't compile yet: weigh takes the cargo, then main reaches for it\n    let toll = weigh(cargo);\n    println!(\"toll: {toll}\");\n    println!(\"cargo: {cargo}\");\n}\n",
            },
            Mission {
                id: "bridge_quill",
                npc_name: "The Borrow Checker",
                prompt: "Let both readers finish, then write on the scroll through an `&mut` borrow.",
                tutorial: "## Concept\n\
The second toll is the rule itself: at any moment a value may have many \
shared borrows (`&T`) or one exclusive borrow (`&mut T`), never both. A \
borrow lasts until its last use, not to the end of the block. Two readers \
may hold `&scroll[0]` together, but `scroll.push(...)` needs `&mut scroll`, \
and while a reader is still used later the compiler refuses with E0502, \
\"cannot borrow `scroll` as mutable because it is also borrowed as \
immutable\".\n\n\
## Syntax\n\
```\nfn add(list: &mut Vec<i32>, n: i32) {\n    list.push(n);\n}\nlet mut v = vec![1];\nlet first = &v[0];\nprintln!(\"{first}\");   // last use of the shared borrow\nadd(&mut v, 2);         // now the exclusive borrow is allowed\n```\n\
The `&mut` at the call site says, out loud, who may write.\n\n\
## Task\n\
Write `fn inscribe(scroll: &mut Vec<String>, entry: &str)` that pushes the \
entry onto the scroll. In `main`, let both readers print first, then write \
with `inscribe(&mut scroll, \"east road\")` and print the scroll. Your \
program should print `reader one: north road`, `reader two: north road`, \
then `scroll: north road, east road`.\n\n\
## Hint\n\
The grader needs `fn inscribe(scroll: &mut Vec<String>, entry: &str)`, a \
`push` onto the scroll inside it, and a call `inscribe(&mut scroll, ...)`. \
`entry.to_string()` turns the `&str` into the `String` the scroll holds. \
Moving the two reader lines above the write is all `main` needs.",
                prereq: None,
                available: true,
                starter_code: "fn main() {\n    let mut scroll = vec![String::from(\"north road\")];\n    let first = &scroll[0];\n    let second = &scroll[0];\n    // won't compile yet: the quill writes while readers hold the scroll\n    scroll.push(String::from(\"east road\"));\n    println!(\"reader one: {first}\");\n    println!(\"reader two: {second}\");\n    println!(\"scroll: {}\", scroll.join(\", \"));\n}\n",
            },
            Mission {
                id: "bridge_lantern",
                npc_name: "The Borrow Checker",
                prompt: "Give `before` a lifetime `'a` that ties its answer to `line` alone.",
                tutorial: "## Concept\n\
The last toll. A function that returns a reference must say which input \
it borrows from, so the caller knows how long the answer may live. With \
one reference parameter the compiler works it out; with two (and no \
`self`) it will not guess, and stops with E0106, \"missing lifetime \
specifier\". A lifetime parameter `'a` names the link. Tie the answer to \
the input it really comes from. The compiler's help ties both inputs to \
`'a`. Do that here and the answer is held to `sep` as well, so a `sep` \
dropped early fails with E0597, \"`sep` does not live long enough\".\n\n\
## Syntax\n\
```\nfn after<'a>(line: &'a str, sep: &str) -> &'a str {\n    match line.find(sep) {\n        Some(i) => &line[i + sep.len()..],\n        None => \"\",\n    }\n}\n```\n\
`'a` appears on `line` and on the return type only: `sep` is merely read.\n\n\
## Task\n\
`before` returns the part of `line` before the first `sep`, or all of \
`line` if there is none. The starter fails with E0106. Add `'a` so the \
answer borrows from `line` only, and leave `sep` a plain `&str`. Keep \
`main` as it is: `sep` is dropped at the end of the inner block, before \
`part` is printed. Your program should print `first: oil`, then \
`whole: glass`.\n\n\
## Hint\n\
The grader needs `fn before<'a>(line: &'a str, sep: &str) -> &'a str` \
(another lifetime name is fine). If you write `sep: &'a str` as well, \
`main` fails with E0597: the answer would be tied to `sep`, which is \
dropped at the inner `}`.",
                prereq: None,
                available: true,
                starter_code: "// this won't compile yet: say which input the answer borrows from\nfn before(line: &str, sep: &str) -> &str {\n    match line.find(sep) {\n        Some(i) => &line[..i],\n        None => line,\n    }\n}\n\nfn main() {\n    let lantern = String::from(\"oil:wick:glass\");\n    let part;\n    {\n        let sep = String::from(\":\");\n        part = before(&lantern, &sep);\n    }\n    println!(\"first: {part}\");\n    println!(\"whole: {}\", before(\"glass\", \":\"));\n}\n",
            },
            // ── Act 3: The Guildhall Quarter ──────────────────────────
            // Fills the doc's Act 3 gaps the prelude skipped: methods/impl,
            // associated fns, if let, while let, tuple structs, enum-with-data
            // match. See design/01-curriculum.md §Act 3 and
            // docs/superpowers/specs/2026-06-18-act3-guildhall-missions-design.md.
            Mission {
                id: "impl_method",
                npc_name: "The Guildmaster",
                prompt: "Give the `Hero` struct a method `power(&self) -> i32` and call it.",
                tutorial: "## Concept\n\
A struct holds data; an `impl` block gives it behaviour. A *method* is a \
function declared inside `impl TypeName { ... }` whose first parameter is \
`&self` (a shared borrow of the instance), `&mut self` (exclusive), or \
`self` (by value). Inside, `self.field` reads the instance's data. You call \
a method with dot syntax: `instance.method()`.\n\n\
## Syntax\n\
```\nstruct Hero { level: i32 }\nimpl Hero {\n    fn power(&self) -> i32 { self.level * 10 }\n}\nlet h = Hero { level: 3 };\nprintln!(\"{}\", h.power());\n```\n\
`&self` borrows the hero, so calling `power` doesn't consume it — you can \
call it again.\n\n\
## Task\n\
The `Hero` struct is given. Add an `impl Hero` block with a method \
`power(&self) -> i32`, then call it in `main` and print the result.\n\n\
## Hint\n\
The grader looks for `impl`, `&self`, and a `self.` field access. The \
Guildmaster's rule: a member is defined by their duties.",
                prereq: None,
                available: true,
                starter_code: "struct Hero { level: i32 }\n\nfn main() {\n    let _h = Hero { level: 3 };\n    // give Hero a `power` method (it borrows the hero) returning its power, then call it\n}\n",
            },
            Mission {
                id: "assoc_new",
                npc_name: "The Recruiter",
                prompt: "Add an associated function `Hero::new(level)` that returns `Self`.",
                tutorial: "## Concept\n\
Not every function in an `impl` block takes `self`. An *associated \
function* belongs to the type, not an instance — the canonical one is a \
constructor named `new`. Its return type is `Self` (an alias for the type \
being implemented), and you call it with path syntax `TypeName::new(...)`. \
Rust has no `constructor` keyword; this is the idiom.\n\n\
## Syntax\n\
```\nimpl Hero {\n    fn new(level: i32) -> Self {\n        Self { level }\n    }\n}\nlet h = Hero::new(5);\n```\n\
`Self { level }` is shorthand for `Hero { level: level }`: `Self` names \
the type, and the field name can stand alone when the local shares it.\n\n\
## Task\n\
Give `Hero` an associated function `new(level: i32) -> Self`, then build a \
hero with `Hero::new(5)` and print its level.\n\n\
## Hint\n\
The grader needs `fn new`, `Self`, and a `::new` call. The Recruiter forges \
a member from nothing but a name.",
                prereq: None,
                available: true,
                starter_code: "struct Hero { level: i32 }\n\nfn main() {\n    // add a constructor associated function that builds a Hero from a level, then call it\n    let _ = Hero { level: 1 };\n}\n",
            },
            Mission {
                id: "if_let",
                npc_name: "The Locksmith",
                prompt: "Use `if let Some(x) =` to act only when an `Option` is present.",
                tutorial: "## Concept\n\
A full `match` on an `Option` has two arms, but often you only care about \
the `Some` case. `if let` is the concise form: it runs its block only when \
the value matches the pattern, binding the inner value. It's sugar for a \
`match` with one real arm and a `_ => {}` catch-all — you trade \
exhaustiveness for brevity.\n\n\
## Syntax\n\
```\nlet maybe: Option<i32> = Some(7);\nif let Some(n) = maybe {\n    println!(\"{n}\");\n}\n```\n\
You may add an `else` for the non-matching case, but unlike `match` you're \
not forced to.\n\n\
## Task\n\
`maybe` is provided. Use `if let Some(n) = maybe` to print the inner value \
only when it's present.\n\n\
## Hint\n\
The grader needs `if let` and `Some(`. The Locksmith tries one key: if it \
fits, the door opens; if not, nothing happens.",
                prereq: None,
                available: true,
                starter_code: "fn main() {\n    let maybe: Option<i32> = Some(7);\n    // print the inner value only when the option actually holds one\n    let _ = maybe;\n}\n",
            },
            Mission {
                id: "while_let",
                npc_name: "The Porter",
                prompt: "Drain a stack with `while let Some(x) = stack.pop()`.",
                tutorial: "## Concept\n\
`while let` is the looping cousin of `if let`: it runs its body as long as \
the pattern keeps matching, then stops. The classic use is draining a \
collection — `Vec::pop()` returns `Some(value)` until the vec is empty, \
then `None`, which ends the loop.\n\n\
## Syntax\n\
```\nlet mut stack = vec![1, 2, 3];\nwhile let Some(top) = stack.pop() {\n    println!(\"{top}\");\n}\n```\n\
Each iteration removes and binds the last element; when `pop` returns \
`None`, the loop exits cleanly.\n\n\
## Task\n\
A `stack` is provided. Use `while let Some(top) = stack.pop()` to print \
every element as you remove it.\n\n\
## Hint\n\
The grader needs `while let` and `.pop(`. The Porter empties the cart \
crate by crate until there's nothing left to lift.",
                prereq: None,
                available: true,
                starter_code: "fn main() {\n    let mut stack = vec![1, 2, 3];\n    // remove and print each element until the stack is empty\n    let _ = &mut stack;\n}\n",
            },
            Mission {
                id: "tuple_struct",
                npc_name: "The Surveyor",
                prompt: "Wrap a raw `f64` in a `Meters(f64)` tuple struct and read it back.",
                tutorial: "## Concept\n\
A *tuple struct* is a struct whose fields are positional rather than named: \
`struct Meters(f64);`. The single-field form is the *newtype* pattern — it \
wraps a raw value in a distinct type so the compiler can tell `Meters` from \
a bare `f64` (or from `Seconds(f64)`). You build one like a function call, \
`Meters(3.5)`, and read fields by position: `.0`, `.1`, … A field-less \
`struct Marker;` is a *unit struct*, used as a zero-size tag.\n\n\
## Syntax\n\
```\nstruct Meters(f64);\nlet d = Meters(3.5);\nprintln!(\"{}\", d.0);\n```\n\
`d.0` is the first (and here only) field.\n\n\
## Task\n\
Define a `Meters(f64)` tuple struct, wrap the raw value below in it, then \
print the wrapped number via `.0`.\n\n\
## Hint\n\
The grader needs `struct Meters(` and a `.0` field access. The Surveyor \
reads the rod: a bare number, now named.",
                prereq: None,
                available: true,
                starter_code: "fn main() {\n    let _raw: f64 = 3.5;\n    // wrap that raw f64 in a one-field `Meters` newtype, then read the wrapped value back out\n}\n",
            },
            Mission {
                id: "enum_data_match",
                npc_name: "The Armorer",
                prompt: "Define an `Item` enum whose variants carry data, then `match` exhaustively.",
                tutorial: "## Concept\n\
Enum variants can carry data. A struct-like variant names its fields: \
`Weapon { damage: i32 }`. To read that data you `match` and bind it in the \
pattern: `Item::Weapon { damage } => damage`. The match must be exhaustive \
— every variant handled — or the code won't compile. This is the Armorer's \
discipline: name every kind of thing, miss none.\n\n\
## Syntax\n\
```\nenum Item {\n    Weapon { damage: i32 },\n    Potion { heal: i32 },\n}\nfn value(item: Item) -> i32 {\n    match item {\n        Item::Weapon { damage } => damage,\n        Item::Potion { heal } => heal,\n    }\n}\n```\n\
Each arm binds the variant's field and returns it.\n\n\
## Task\n\
Define `enum Item` with `Weapon { damage: i32 }` and `Potion { heal: i32 }`, \
then change `value` to take an `Item` and `match` it exhaustively, returning \
the inner number.\n\n\
## Hint\n\
The grader needs `enum Item`, `Weapon`, a `match`, and a `=>` arm. Handle \
both kinds — a non-exhaustive match won't compile.",
                prereq: None,
                available: true,
                starter_code: "fn value(item: i32) -> i32 {\n    // define an `Item` type with two kinds (a weapon carrying damage, a potion carrying heal),\n    // take an Item here instead of an i32, and handle every kind to return its number\n    item\n}\n\nfn main() {\n    let _ = value(0);\n}\n",
            },
            // ── Act 3 boss: the Guildmaster's trial, The Variant Wraith ─
            // Three stages owned by the Guildmaster (`trial::TRIALS`), after
            // his prologue `impl_method`: name every variant in a `match`,
            // open variants that carry data, then a new variant makes the
            // old match non-exhaustive. Every compiler error he names is
            // compiled in `tests/mission_text.rs` (hard rule 3).
            Mission {
                id: "wraith_forms",
                npc_name: "The Guildmaster",
                prompt: "Give every form of the Wraith its own arm, so `counter` answers each one.",
                tutorial: "## Concept\n\
The Variant Wraith shifts between three forms, and the Guild answers each \
one by name. A `match` on an enum must cover every variant or it does not \
compile. A `_` arm covers \"everything else\", so it meets that rule \
without naming anything: here it answers the Shadow with steel, which \
passes straight through it. Take the `_` arm away without naming the \
Shadow and the compiler refuses with E0004, \"non-exhaustive patterns: \
`Wraith::Shadow` not covered\". Name every form, and the compiler checks \
the list for you.\n\n\
## Syntax\n\
```\nenum Gate { Open, Shut }\nfn sign(g: Gate) -> &'static str {\n    match g {\n        Gate::Open => \"pass\",\n        Gate::Shut => \"halt\",\n    }\n}\n```\n\
One arm per variant: no `_`, nothing left unnamed.\n\n\
## Task\n\
`counter` answers the Shadow through `_ => \"steel\"`. Replace that arm \
with `Wraith::Shadow => \"light\"`. Your program should print \
`Solid: steel`, `Mist: wind`, then `Shadow: light`.\n\n\
## Hint\n\
The grader needs an arm that starts `Wraith::Shadow =>`. The Shadow is \
driven off by light alone: any other answer fails the check.",
                prereq: None,
                available: true,
                starter_code: "#[derive(Debug, Clone, Copy)]\nenum Wraith {\n    Solid,\n    Mist,\n    Shadow,\n}\n\nfn counter(form: Wraith) -> &'static str {\n    match form {\n        Wraith::Solid => \"steel\",\n        Wraith::Mist => \"wind\",\n        // this runs, but `_` answers a form it never looked at\n        _ => \"steel\",\n    }\n}\n\nfn main() {\n    let forms = [Wraith::Solid, Wraith::Mist, Wraith::Shadow];\n    for form in forms {\n        println!(\"{form:?}: {}\", counter(form));\n    }\n}\n",
            },
            Mission {
                id: "wraith_essence",
                npc_name: "The Guildmaster",
                prompt: "Open each form in its `match` arm and use the data it carries.",
                tutorial: "## Concept\n\
Now the forms carry data: `Solid { armor: u32 }` and \
`Mist { density: u32 }`. A pattern for a struct-like variant opens it with \
braces and can bind each field to a name: in the arm \
`Wraith::Solid { armor } => ...`, `armor` is a local. Written alone, \
`Wraith::Solid` is a pattern for a unit variant, so the compiler stops \
with E0533, \"expected unit struct, unit variant or constant, found struct \
variant `Wraith::Solid`\".\n\n\
## Syntax\n\
```\nenum Coin { Gold { weight: u32 }, Copper }\nfn worth(c: Coin) -> u32 {\n    match c {\n        Coin::Gold { weight } => weight * 10,\n        Coin::Copper => 1,\n    }\n}\n```\n\
`Coin::Gold { weight: w }` binds the same field to another name.\n\n\
## Task\n\
`damage` takes a form and the strength of a blow. Solid armor soaks part \
of it: `blow - armor`. Mist spreads it thin: `blow / density`. The Shadow \
takes `0`. The starter fails with E0533. Open each form and use what it \
carries. Your program should print `Solid { armor: 3 } takes 7`, \
`Mist { density: 2 } takes 5`, then `Shadow takes 0`.\n\n\
## Hint\n\
The grader needs `Wraith::Solid { armor } => blow - armor` and \
`Wraith::Mist { density } => blow / density` (other binding names are \
fine). The compiler's help writes `Wraith::Solid { armor: _ }`. That \
compiles, but `_` throws the armor away: the program then prints \
`Solid { armor: 3 } takes 10`.",
                prereq: None,
                available: true,
                starter_code: "#[derive(Debug, Clone, Copy)]\nenum Wraith {\n    Solid { armor: u32 },\n    Mist { density: u32 },\n    Shadow,\n}\n\nfn damage(form: Wraith, blow: u32) -> u32 {\n    // this won't compile yet: open each form to reach what it carries\n    match form {\n        Wraith::Solid => blow,\n        Wraith::Mist => blow,\n        Wraith::Shadow => 0,\n    }\n}\n\nfn main() {\n    let forms = [\n        Wraith::Solid { armor: 3 },\n        Wraith::Mist { density: 2 },\n        Wraith::Shadow,\n    ];\n    for form in forms {\n        println!(\"{form:?} takes {}\", damage(form, 10));\n    }\n}\n",
            },
            Mission {
                id: "wraith_possessed",
                npc_name: "The Guildmaster",
                prompt: "The Wraith has a fourth form. Give `Possessed` its own arm.",
                tutorial: "## Concept\n\
Mid-fight the Wraith takes a fourth form, `Possessed { will: u32 }`. A new \
variant changes every `match` on the enum, and the compiler finds each \
one: `damage` now stops with E0004, \"non-exhaustive patterns: \
`Wraith::Possessed { .. }` not covered\". That error is the reward for \
naming every form. A blanket `_ => 0` arm compiles too, and so would the \
next new form after it: `_` answers it without anyone naming it.\n\n\
## Syntax\n\
```\nenum Gate { Open, Shut, Barred { bolts: u32 } }\nmatch g {\n    Gate::Open => 0,\n    Gate::Shut => 1,\n    Gate::Barred { bolts } => 1 + bolts,\n}\n```\n\
## Task\n\
A possessed Wraith's host fights back, so it takes `blow + will`. The \
starter fails with E0004. Add an arm for `Wraith::Possessed { will }` and \
keep the other three as they are. Your program should print \
`Solid { armor: 3 } takes 7`, `Mist { density: 2 } takes 5`, \
`Shadow takes 0`, then `Possessed { will: 4 } takes 14`.\n\n\
## Hint\n\
The grader needs `Wraith::Possessed { will } => blow + will` (another \
name for `will` is fine). A `_ => 0` arm compiles, but the program then \
prints `Possessed { will: 4 } takes 0`: the possessed form is struck for \
nothing.",
                prereq: None,
                available: true,
                starter_code: "#[derive(Debug, Clone, Copy)]\nenum Wraith {\n    Solid { armor: u32 },\n    Mist { density: u32 },\n    Shadow,\n    Possessed { will: u32 },\n}\n\nfn damage(form: Wraith, blow: u32) -> u32 {\n    // this won't compile yet: the new form has no arm\n    match form {\n        Wraith::Solid { armor } => blow - armor,\n        Wraith::Mist { density } => blow / density,\n        Wraith::Shadow => 0,\n    }\n}\n\nfn main() {\n    let forms = [\n        Wraith::Solid { armor: 3 },\n        Wraith::Mist { density: 2 },\n        Wraith::Shadow,\n        Wraith::Possessed { will: 4 },\n    ];\n    for form in forms {\n        println!(\"{form:?} takes {}\", damage(form, 10));\n    }\n}\n",
            },
            // ── Act 4: The Trait Mage's Tower ─────────────────────────
            // Generics + traits + dyn + lifetimes + associated types. See
            // design/01-curriculum.md §Act 4 and
            // docs/superpowers/specs/2026-06-18-act4-trait-tower-missions-design.md.
            Mission {
                id: "trait_def",
                npc_name: "Vexis the Archmage",
                prompt: "Define a `trait`, implement it for a type, and call the method.",
                tutorial: "## Concept\n\
A *trait* is a named set of behaviour a type can promise to provide — \
Rust's version of an interface. You declare it with `trait Name { ... }`, \
listing method signatures. A type opts in with `impl Trait for Type`, \
supplying the bodies. Any type that implements the trait can then be used \
wherever that behaviour is required. (Traits can also carry *default* \
method bodies and require other traits as *supertraits* — later floors of \
the tower.)\n\n\
## Syntax\n\
```\ntrait Element {\n    fn name(&self) -> &str;\n}\nstruct Fire;\nimpl Element for Fire {\n    fn name(&self) -> &str { \"fire\" }\n}\nFire.name();\n```\n\
The signature in the trait ends with `;`; the body lives in the `impl`.\n\n\
## Task\n\
`struct Fire` is given. Define a `trait Element` with a `name(&self) -> \
&str` method, implement it `for Fire`, then call `.name()` in `main`.\n\n\
## Hint\n\
The grader needs `trait `, `impl `, and ` for `. Vexis: a capability named \
once, granted to a type.",
                prereq: None,
                available: true,
                starter_code: "struct Fire;\n\nfn main() {\n    let _f = Fire;\n    // define an Element capability (a name() method), grant it to Fire, then call name()\n}\n",
            },
            Mission {
                id: "generic_fn",
                npc_name: "The Wandwright",
                prompt: "Write a generic `larger<T: PartialOrd>(a, b)` that returns the bigger.",
                tutorial: "## Concept\n\
A *generic* function works over many types instead of one. You declare a \
type parameter in angle brackets — `fn f<T>(...)` — and the body must work \
for every `T`. To do anything with a `T` (compare it, add it) you add a \
*trait bound* saying which capabilities `T` must have: `<T: PartialOrd>` \
means \"any `T` that can be ordered\". The compiler generates a specialised \
copy per concrete type (*monomorphisation*) — zero runtime cost.\n\n\
## Syntax\n\
```\nfn larger<T: PartialOrd>(a: T, b: T) -> T {\n    if a > b { a } else { b }\n}\nlarger(3, 7);       // T = i32\nlarger(1.5, 0.5);   // T = f64\n```\n\
Without the `PartialOrd` bound, `a > b` won't compile — the compiler \
can't assume an arbitrary `T` is comparable.\n\n\
## Task\n\
Write `larger<T: PartialOrd>(a: T, b: T) -> T` returning the larger of the \
two, then call it on two integers in `main`.\n\n\
## Hint\n\
The grader needs the type parameter `<T` and the bound `PartialOrd`. One \
wand, any element — bounded by what it can compare.",
                prereq: None,
                available: true,
                starter_code: "fn main() {\n    // write a generic `larger` function returning the bigger of two comparable values,\n    // then call it on two integers\n    println!(\"{}\", 0);\n}\n",
            },
            Mission {
                id: "generic_struct",
                npc_name: "The Conjurer",
                prompt: "Define a generic `struct Pair<T>` holding two values of one type.",
                tutorial: "## Concept\n\
Structs can be generic too. `struct Pair<T> { a: T, b: T }` is a container \
that holds two values of *the same* type `T` — `Pair<i32>`, \
`Pair<String>`, whatever the caller supplies. The type parameter goes \
after the name in angle brackets and is then usable as a field type. The \
standard library's `Vec<T>`, `Option<T>`, and `HashMap<K, V>` are all just \
generic structs/enums.\n\n\
## Syntax\n\
```\nstruct Pair<T> {\n    a: T,\n    b: T,\n}\nlet p = Pair { a: 1, b: 2 };   // Pair<i32>, inferred\nlet _ = (p.a, p.b);\n```\n\
The compiler infers `T` from the values you pass; you can also annotate \
`Pair::<i32> { a: 1, b: 2 }` explicitly.\n\n\
## Task\n\
Define `struct Pair<T>` with two fields of type `T`, build one from two \
integers, and read both fields.\n\n\
## Hint\n\
The grader needs `struct `, the parameter `<T>`, and a field typed `: T`. \
A vessel for any type — so long as both halves agree.",
                prereq: None,
                available: true,
                starter_code: "fn main() {\n    // define a generic Pair that holds two values of one shared type,\n    // build one from two integers, then read both fields\n    let _ = (1, 2);\n}\n",
            },
            Mission {
                id: "dyn_trait",
                npc_name: "The Familiar",
                prompt: "Collect differently-built `Element`s into one `Vec<Box<dyn Element>>`.",
                tutorial: "## Concept\n\
Generics give *static* dispatch — one type per call site, resolved at \
compile time. Sometimes you need a single collection holding *different* \
concrete types that share a trait. That's a *trait object*: `dyn Trait`, \
usually behind a pointer like `Box<dyn Trait>`. Calls through it are \
*dynamically* dispatched (a vtable lookup at runtime). The tradeoff: \
flexibility (mix types in one `Vec`) for a small indirection cost.\n\n\
## Syntax\n\
```\nlet zoo: Vec<Box<dyn Element>> = vec![\n    Box::new(Fire),\n    Box::new(Water),\n];\nfor e in &zoo {\n    println!(\"{}\", e.name());\n}\n```\n\
`Box::new` moves each value to the heap; the `Vec<Box<dyn Element>>` \
annotation then coerces each `Box<Fire>` / `Box<Water>` to \
`Box<dyn Element>`, erasing the concrete type.\n\n\
## Task\n\
`Element`, `Fire`, and `Water` are provided. Build a \
`Vec<Box<dyn Element>>` holding one of each, then print every `name()`.\n\n\
## Hint\n\
The grader needs `Box<dyn` and `Box::new`. Many shapes, one cage — dispatched \
at a touch.",
                prereq: None,
                available: true,
                starter_code: "trait Element {\n    fn name(&self) -> &str;\n}\nstruct Fire;\nimpl Element for Fire {\n    fn name(&self) -> &str { \"fire\" }\n}\nstruct Water;\nimpl Element for Water {\n    fn name(&self) -> &str { \"water\" }\n}\n\nfn main() {\n    // collect a Fire and a Water into one Vec of trait objects, then print each name()\n    let _ = (Fire, Water);\n}\n",
            },
            Mission {
                id: "lifetimes",
                npc_name: "The Lanternkeeper",
                prompt: "Add the lifetime `<'a>` so `longest` can return a borrow of its inputs.",
                tutorial: "## Concept\n\
When a function returns a reference, the compiler must know *which* input \
that reference borrows from — so it can guarantee the borrow doesn't \
outlive its source. A *lifetime parameter* `'a` names that relationship. \
`fn longest<'a>(x: &'a str, y: &'a str) -> &'a str` says: the returned \
reference lives as long as the shorter of `x` and `y`. Lifetimes are not \
runtime values — they're compile-time annotations the borrow checker uses \
to reject dangling references.\n\n\
## Syntax\n\
```\nfn longest<'a>(x: &'a str, y: &'a str) -> &'a str {\n    if x.len() > y.len() { x } else { y }\n}\n```\n\
Without `'a`, the compiler can't tell whether the result borrows `x` or \
`y`, so it refuses to compile.\n\n\
## Task\n\
The `longest` stub won't compile. Add a lifetime `'a`, tie both `&str` \
parameters and the return to it, and it will.\n\n\
## Hint\n\
The grader needs the lifetime parameter `<'a>` and an annotated reference \
`&'a`. The borrow lives exactly as long as `'a`.",
                prereq: None,
                available: true,
                starter_code: "fn longest(x: &str, y: &str) -> &str {\n    // this won't compile yet: add a lifetime 'a tying the inputs and the return together\n    if x.len() > y.len() { x } else { y }\n}\n\nfn main() {\n    let _ = longest(\"aa\", \"b\");\n}\n",
            },
            Mission {
                id: "assoc_type",
                npc_name: "The Loremaster",
                prompt: "Give the `Producer` trait an associated type `Output`.",
                tutorial: "## Concept\n\
An *associated type* lets a trait name a type that each implementor fills \
in — `type Output;` in the trait, `type Output = i32;` in the impl. \
Inside the trait, methods refer to it as `Self::Output`. It's how \
`Iterator` declares `type Item`: every iterator chooses what it yields, \
but the trait is written once over `Self::Item`. Use an associated type \
(not a generic parameter) when there's exactly *one* sensible choice per \
implementor.\n\n\
## Syntax\n\
```\ntrait Producer {\n    type Output;\n    fn make(&self) -> Self::Output;\n}\nimpl Producer for Coiner {\n    type Output = i32;\n    fn make(&self) -> Self::Output { 7 }\n}\n```\n\
`Self::Output` resolves to `i32` for `Coiner`, to whatever each other \
implementor declares.\n\n\
## Task\n\
Give `trait Producer` an associated `type Output`, have `make` return \
`Self::Output`, then implement it for a `Coiner` whose `Output` is `i32`.\n\n\
## Hint\n\
The grader needs `type Output` and `Self::Output`. Each producer names \
its own yield — the type follows the trait.",
                prereq: None,
                available: true,
                starter_code: "trait Producer {\n    fn make(&self) -> i32;\n}\n\nfn main() {\n    // change Producer so each implementor names its own output type (an associated type),\n    // then implement it for a Coiner that makes an i32\n    let _ = ();\n}\n",
            },
            // ── Act 4 boss: Vexis's trial, The Chimera of Vexis ────────
            // Three stages owned by Vexis (`trial::TRIALS`), after his
            // prologue `trait_def`: a trait implemented per head type, a
            // generic fn with a trait bound, then `&[Box<dyn Enemy>]`
            // dispatch that a fourth head joins without new arms. Every
            // compiler error he names is compiled in
            // `tests/mission_text.rs` (hard rule 3).
            Mission {
                id: "chimera_heads",
                npc_name: "Vexis the Archmage",
                prompt: "Implement `Enemy` for the Frost and Storm heads.",
                tutorial: "## Concept\n\
The Chimera of Vexis has three heads, and each breathes a different \
element. A trait names the behaviour they share. Each head is its own \
type, and it gains the trait's methods only through its own \
`impl Enemy for ...` block. Until then the methods do not exist for that \
type: `Frost.name()` stops with E0599, \"no method named `name` found for \
struct `Frost` in the current scope\".\n\n\
## Syntax\n\
```\ntrait Spell {\n    fn power(&self) -> u32;\n}\nstruct Spark;\nimpl Spell for Spark {\n    fn power(&self) -> u32 {\n        3\n    }\n}\n```\n\
Every method the trait declares must appear in the `impl`.\n\n\
## Task\n\
`Fire` already implements `Enemy`. Implement it for `Frost` (name \
`\"frost\"`, breath `5`) and for `Storm` (name `\"storm\"`, breath `9`). \
Your program should print `fire breathes 7`, `frost breathes 5`, then \
`storm breathes 9`.\n\n\
## Hint\n\
The grader needs `impl Enemy for Frost` and `impl Enemy for Storm`. The \
same methods in a plain `impl Frost` block print the same lines, but then \
Frost is still not an `Enemy`, and the checker asks for one.",
                prereq: None,
                available: true,
                starter_code: "trait Enemy {\n    fn name(&self) -> &'static str;\n    fn breath(&self) -> u32;\n}\n\nstruct Fire;\nstruct Frost;\nstruct Storm;\n\nimpl Enemy for Fire {\n    fn name(&self) -> &'static str {\n        \"fire\"\n    }\n    fn breath(&self) -> u32 {\n        7\n    }\n}\n\nfn main() {\n    // this won't compile yet: only one head has learned the trait\n    println!(\"{} breathes {}\", Fire.name(), Fire.breath());\n    println!(\"{} breathes {}\", Frost.name(), Frost.breath());\n    println!(\"{} breathes {}\", Storm.name(), Storm.breath());\n}\n",
            },
            Mission {
                id: "chimera_bound",
                npc_name: "Vexis the Archmage",
                prompt: "Bound `volley`'s type parameter with `T: Enemy`.",
                tutorial: "## Concept\n\
A generic function is written once for many types, but inside it `T` \
could be anything: a number, a door, a `String`. So the compiler allows \
only what every `T` can do, and `head.name()` stops with E0599, \"no \
method named `name` found for reference `&T` in the current scope\". A \
trait bound, `<T: Enemy>`, narrows `T` to types that implement `Enemy`, \
and their methods become callable. Each call is compiled for its own \
concrete type: static dispatch.\n\n\
## Syntax\n\
```\nfn power_of<T: Spell>(s: &T) -> u32 {\n    s.power()\n}\npower_of(&Spark);   // T = Spark\n```\n\
`fn power_of(s: &impl Spell)` is a shorter way to write the same \
bound.\n\n\
## Task\n\
`volley` reports one head breathing for several rounds. The starter fails \
with E0599. Add the bound so `volley` accepts any `Enemy`. Your program \
should print `fire x3 = 21`, `frost x3 = 15`, then `storm x3 = 27`.\n\n\
## Hint\n\
The grader needs `fn volley<T: Enemy>` (a `where T: Enemy` clause or \
`head: &impl Enemy` also counts). The compiler's help suggests the same \
bound. One `volley` per head type would not be generic: the checker calls \
`volley` with a head of its own.",
                prereq: None,
                available: true,
                starter_code: "trait Enemy {\n    fn name(&self) -> &'static str;\n    fn breath(&self) -> u32;\n}\n\nstruct Fire;\nstruct Frost;\nstruct Storm;\n\nimpl Enemy for Fire {\n    fn name(&self) -> &'static str { \"fire\" }\n    fn breath(&self) -> u32 { 7 }\n}\nimpl Enemy for Frost {\n    fn name(&self) -> &'static str { \"frost\" }\n    fn breath(&self) -> u32 { 5 }\n}\nimpl Enemy for Storm {\n    fn name(&self) -> &'static str { \"storm\" }\n    fn breath(&self) -> u32 { 9 }\n}\n\n// this won't compile yet: nothing promises that T can breathe\nfn volley<T>(head: &T, rounds: u32) -> String {\n    format!(\"{} x{rounds} = {}\", head.name(), head.breath() * rounds)\n}\n\nfn main() {\n    println!(\"{}\", volley(&Fire, 3));\n    println!(\"{}\", volley(&Frost, 3));\n    println!(\"{}\", volley(&Storm, 3));\n}\n",
            },
            Mission {
                id: "chimera_plasma",
                npc_name: "Vexis the Archmage",
                prompt: "Rewrite `battle` to take `&[Box<dyn Enemy>]`, so the fourth head can join.",
                tutorial: "## Concept\n\
The Chimera grows a fourth head, `Plasma`. The old `battle` matched on an \
enum of the heads it knew, so every new head meant a new variant and a \
new arm. A trait object, `Box<dyn Enemy>`, holds any type that implements \
`Enemy`, and each call through it finds that type's method at run time: \
dynamic dispatch. The heads now arrive as a `Vec<Box<dyn Enemy>>`, and \
`battle(&heads)` stops with E0308, \"mismatched types\": it expects \
`&[Head]`.\n\n\
## Syntax\n\
```\nfn total(spells: &[Box<dyn Spell>]) -> u32 {\n    let mut sum = 0;\n    for s in spells {\n        sum += s.power();\n    }\n    sum\n}\n```\n\
A `&Vec<Box<dyn Spell>>` is accepted where `&[Box<dyn Spell>]` is \
expected.\n\n\
## Task\n\
Change `battle` to `fn battle(heads: &[Box<dyn Enemy>]) -> u32`: print \
`{name} breathes {breath}` for each head and return the total. The `Head` \
enum is then unused; delete it. Your program should print \
`fire breathes 7`, `frost breathes 5`, `storm breathes 9`, \
`plasma breathes 12`, then `total: 33`.\n\n\
## Hint\n\
The grader needs `fn battle(heads: &[Box<dyn Enemy>])`. Matching on \
concrete head types cannot keep up: the checker sends `battle` a head it \
has never seen.",
                prereq: None,
                available: true,
                starter_code: "trait Enemy {\n    fn name(&self) -> &'static str;\n    fn breath(&self) -> u32;\n}\n\nstruct Fire;\nstruct Frost;\nstruct Storm;\nstruct Plasma;\n\nimpl Enemy for Fire {\n    fn name(&self) -> &'static str { \"fire\" }\n    fn breath(&self) -> u32 { 7 }\n}\nimpl Enemy for Frost {\n    fn name(&self) -> &'static str { \"frost\" }\n    fn breath(&self) -> u32 { 5 }\n}\nimpl Enemy for Storm {\n    fn name(&self) -> &'static str { \"storm\" }\n    fn breath(&self) -> u32 { 9 }\n}\nimpl Enemy for Plasma {\n    fn name(&self) -> &'static str { \"plasma\" }\n    fn breath(&self) -> u32 { 12 }\n}\n\n// The old plan: one arm per head the Chimera had yesterday.\nenum Head {\n    Fire,\n    Frost,\n    Storm,\n}\n\nfn battle(heads: &[Head]) -> u32 {\n    let mut total = 0;\n    for head in heads {\n        let (name, breath) = match head {\n            Head::Fire => (Fire.name(), Fire.breath()),\n            Head::Frost => (Frost.name(), Frost.breath()),\n            Head::Storm => (Storm.name(), Storm.breath()),\n        };\n        println!(\"{name} breathes {breath}\");\n        total += breath;\n    }\n    total\n}\n\nfn main() {\n    let heads: Vec<Box<dyn Enemy>> = vec![\n        Box::new(Fire),\n        Box::new(Frost),\n        Box::new(Storm),\n        Box::new(Plasma),\n    ];\n    // this won't compile yet: battle only knows yesterday's heads\n    println!(\"total: {}\", battle(&heads));\n}\n",
            },
            // ── Act 5: The Tavern of Tribulations ─────────────────────
            // Error handling: Result/Option methods, custom errors, From
            // conversion. See design/01-curriculum.md §Act 5 and
            // docs/superpowers/specs/2026-06-18-act5-tavern-missions-design.md.
            Mission {
                id: "result_match",
                npc_name: "The Barkeep",
                prompt: "Handle a `Result` with `match` — the value on `Ok`, a fallback on `Err`.",
                tutorial: "## Concept\n\
`Result<T, E>` is `Ok(T)` on success or `Err(E)` on failure — Rust's \
recoverable-error type. The most explicit way to handle one is `match`, \
which forces you to cover *both* arms: the compiler won't let you forget \
the error case. You did this for `Option` (Some/None) earlier; `Result` is \
the same shape with `Ok`/`Err` and an error payload you can inspect.\n\n\
## Syntax\n\
```\nmatch result {\n    Ok(v) => v,\n    Err(e) => {\n        eprintln!(\"{e}\");\n        -1\n    }\n}\n```\n\
Both arms must produce the same type. Bind the inner value with `Ok(v)` / \
`Err(e)`.\n\n\
## Task\n\
`describe` takes a `Result<i32, String>`. `match` on it: return the value \
on `Ok`, and `-1` on `Err`.\n\n\
## Hint\n\
The grader needs `match `, `Ok(`, and `Err(`. The Barkeep serves what's \
poured — or mops up what's spilled.",
                prereq: None,
                available: true,
                starter_code: "fn describe(r: Result<i32, String>) -> i32 {\n    // handle both outcomes: the value on success, -1 on failure\n    let _ = r;\n    0\n}\n\nfn main() {\n    let _ = describe(Ok(5));\n}\n",
            },
            Mission {
                id: "custom_error",
                npc_name: "The Bouncer",
                prompt: "Define a custom error `enum` and return it inside a `Result`.",
                tutorial: "## Concept\n\
Real programs define their *own* error types so each failure has a name. \
The simplest is an `enum` with one variant per failure mode; a fallible \
function then returns `Result<T, YourError>` and produces `Err(Your\
Error::Variant)` when things go wrong. (In production you'd usually derive \
the boilerplate with the `thiserror` crate — but understanding the bare \
`enum` version first is the point. The grading sandbox is dependency-free, \
so we write it by hand.)\n\n\
## Syntax\n\
```\nenum BrewError { TooHot, TooCold }\nfn check(t: i32) -> Result<i32, BrewError> {\n    if t > 100 { Err(BrewError::TooHot) }\n    else { Ok(t) }\n}\n```\n\
The error type can carry data too (`enum E { Bad(String) }`), just like any \
enum.\n\n\
## Task\n\
Define an error `enum` with at least two failure kinds, then change `check` \
to return `Result<i32, YourError>` and return an `Err` when `temp` is out \
of range.\n\n\
## Hint\n\
The grader needs `enum `, `Result<`, and an `Err(`. The Bouncer has a name \
for every kind of trouble.",
                prereq: None,
                available: true,
                starter_code: "fn check(temp: i32) -> i32 {\n    // define an error type with named failure kinds, change this to return a Result,\n    // and produce a failure when temp is out of range\n    temp\n}\n\nfn main() {\n    let _ = check(50);\n}\n",
            },
            Mission {
                id: "from_error",
                npc_name: "The Interpreter",
                prompt: "Implement `From<ParseFail> for AppError` so `?` can auto-convert.",
                tutorial: "## Concept\n\
The `?` operator's quiet superpower: when a fallible call returns a \
*different* error type than your function, `?` converts it automatically — \
if there's a `From` impl. Implementing `From<LowError> for HighError` lets \
`?` lift a low-level error into your application's error type with no \
boilerplate at the call site. It's how one function can propagate parse \
errors, IO errors, and your own errors through a single `Result<_, AppError>`.\n\n\
## Syntax\n\
```\nimpl From<ParseFail> for AppError {\n    fn from(e: ParseFail) -> Self {\n        AppError::Parse\n    }\n}\n// now `some_call()?` converts ParseFail -> AppError for free\n```\n\
`From` gives you `Into` for free, so `ParseFail.into()` also yields an \
`AppError`.\n\n\
## Task\n\
`ParseFail` and `AppError` are given. Implement `From<ParseFail>` for \
`AppError`, mapping the failure to a variant.\n\n\
## Hint\n\
The grader needs `impl From<`, ` for `, and `fn from`. The Interpreter \
turns one tongue's complaint into another's.",
                prereq: None,
                available: true,
                starter_code: "struct ParseFail;\nenum AppError {\n    Parse,\n}\n\nfn main() {\n    // make AppError convertible from ParseFail: implement the standard conversion\n    // trait so that `ParseFail.into()` yields an AppError\n    let _ = ParseFail;\n}\n",
            },
            Mission {
                id: "option_map",
                npc_name: "The Mixologist",
                prompt: "Use `.map()` to transform the inside of an `Option` without unwrapping.",
                tutorial: "## Concept\n\
`Option::map` applies a function to the inner value *if there is one*, and \
passes `None` straight through. It lets you transform a `Some(x)` into a \
`Some(f(x))` without a `match` and without unwrapping — the absence case is \
handled for you. This is the combinator style: keep the value wrapped, \
describe the transformation, and let the type carry the maybe-ness.\n\n\
## Syntax\n\
```\nlet n: Option<i32> = Some(3);\nlet plus: Option<i32> = n.map(|x| x + 1);  // Some(4)\nNone::<i32>.map(|x| x + 1);                 // still None\n```\n\
`Result` has the same `.map` (transforms `Ok`, leaves `Err`).\n\n\
## Task\n\
`add_one` takes an `Option<i32>`. Use `.map` to return the value plus one, \
still wrapped — don't unwrap, don't `match`.\n\n\
## Hint\n\
The grader needs `Option<` and `.map(`. The Mixologist changes what's in \
the glass — only if the glass holds something.",
                prereq: None,
                available: true,
                starter_code: "fn add_one(o: Option<i32>) -> Option<i32> {\n    // if there's a value, return it plus one — still wrapped — without unwrapping\n    o\n}\n\nfn main() {\n    let _ = add_one(Some(3));\n}\n",
            },
            Mission {
                id: "and_then",
                npc_name: "The Tabkeeper",
                prompt: "Chain fallible steps with `.and_then()` — the first `None` short-circuits.",
                tutorial: "## Concept\n\
When each step itself returns an `Option` (or `Result`), `.map` would give \
you a nested `Option<Option<_>>`. `.and_then` flattens it: it runs the next \
fallible step only if the previous one produced a value, and threads the \
failure straight through. Chaining `.and_then` is how you express \"do this, \
then this, then this — stop at the first failure\" without a pyramid of \
`match`es.\n\n\
## Syntax\n\
```\nfn half(n: i32) -> Option<i32> {\n    if n % 2 == 0 { Some(n / 2) } else { None }\n}\nSome(8).and_then(half).and_then(half);  // Some(2)\nSome(8).and_then(half).and_then(half).and_then(half);  // Some(1)\nSome(8).and_then(half).and_then(half).and_then(half).and_then(half);  // None (1 is odd)\n```\n\
`Result::and_then` works the same way over `Ok`/`Err`.\n\n\
## Task\n\
`half` is given. Chain it on `Some(8)` with `.and_then` twice and print \
the result with `{:?}`, then chain it four times and print that too. \
Your program should print `Some(2)`, then `None`: in the four-step \
chain the odd intermediate (1) short-circuits.\n\n\
## Hint\n\
The grader needs `Option` and `.and_then(`. The Tabkeeper runs the tab — \
one unpaid round and the whole chain is cut.",
                prereq: None,
                available: true,
                starter_code: "fn half(n: i32) -> Option<i32> {\n    if n % 2 == 0 { Some(n / 2) } else { None }\n}\n\nfn main() {\n    // chain `half` on Some(8) twice and print it with {:?}, then four times and print that\n    let _ = half(8);\n}\n",
            },
            Mission {
                id: "unwrap_or_else",
                npc_name: "The Cellarer",
                prompt: "Provide a lazily-computed default with `.unwrap_or_else(|| …)`.",
                tutorial: "## Concept\n\
`.unwrap_or(d)` returns the inner value or a default `d` — but `d` is \
computed *eagerly*, even when it isn't needed. `.unwrap_or_else(|| ...)` \
takes a *closure* and only runs it on the `None`/`Err` path, so an expensive \
fallback costs nothing in the common case. Reach for `_else` whenever the \
default is more than a cheap constant.\n\n\
## Syntax\n\
```\nlet n: Option<i32> = None;\nn.unwrap_or(0);              // 0, but 0 was built regardless\nn.unwrap_or_else(|| expensive());  // expensive() runs only because n is None\n```\n\
`Result::unwrap_or_else` receives the error value: `r.unwrap_or_else(|e| ...)`.\n\n\
## Task\n\
`value` takes an `Option<i32>`. Return the inner value, or compute a \
fallback lazily with `.unwrap_or_else(|| ...)`.\n\n\
## Hint\n\
The grader needs `.unwrap_or_else(` and a closure `||`. The Cellarer broaches \
a fresh cask only when the old one runs dry.",
                prereq: None,
                available: true,
                starter_code: "fn value(o: Option<i32>) -> i32 {\n    // unwrap the option, computing the fallback lazily with a closure (only if needed)\n    let _ = o;\n    0\n}\n\nfn main() {\n    let _ = value(None);\n}\n",
            },
            // ── Act 5 boss: the Barkeep's trial, The Tavernkeeper's Wager ─
            // Three stages owned by the Barkeep (`trial::TRIALS`), after his
            // prologue `result_match`: `?` needs a function that returns
            // `Result`, `From` impls let `?` convert errors, then a
            // multi-step quest chained with `?` (no `match`, `unwrap`,
            // `expect` or `panic!`). Every compiler error he names is
            // compiled in `tests/mission_text.rs` (hard rule 3).
            Mission {
                id: "wager_stake",
                npc_name: "The Barkeep",
                prompt: "Let `double_stake` return a `Result`, so `?` can hand a bad stake back.",
                tutorial: "## Concept\n\
The Tavernkeeper's Wager opens with the stake. `read_stake` returns a \
`Result`: `Ok` with the number, or `Err` when the text is not one. Inside \
a function, `?` unwraps an `Ok`, and on an `Err` it returns that error to \
the caller at once. That only works in a function that itself returns a \
`Result` (or an `Option`). In a function that returns `u32`, the compiler \
stops with E0277, \"the `?` operator can only be used in a function that \
returns `Result` or `Option`\".\n\n\
## Syntax\n\
```\nfn half(text: &str) -> Result<u32, ParseIntError> {\n    let n = text.parse::<u32>()?; // an Err returns here\n    Ok(n / 2)\n}\n```\n\
The success value goes back wrapped in `Ok`.\n\n\
## Task\n\
Change `double_stake` to return `Result<u32, ParseIntError>` and give \
back `Ok(stake * 2)`. Your program should print `Ok(40)`, then \
`Err(ParseIntError { kind: InvalidDigit })`.\n\n\
## Hint\n\
The grader needs `fn double_stake(text: &str) -> Result<u32, \
ParseIntError>` and an `Ok(` around the doubled stake. Swapping `?` for \
`.unwrap()` compiles with the old signature, but a stake of `twenty` then \
panics: the Barkeep wants the error handed back, not a crash.",
                prereq: None,
                available: true,
                starter_code: "use std::num::ParseIntError;\n\nfn read_stake(text: &str) -> Result<u32, ParseIntError> {\n    text.trim().parse::<u32>()\n}\n\nfn double_stake(text: &str) -> u32 {\n    // this won't compile yet: `?` needs a function that can fail\n    let stake = read_stake(text)?;\n    stake * 2\n}\n\nfn main() {\n    println!(\"{:?}\", double_stake(\"20\"));\n    println!(\"{:?}\", double_stake(\"twenty\"));\n}\n",
            },
            Mission {
                id: "wager_from",
                npc_name: "The Barkeep",
                prompt: "Implement `From` for each step's error, so `?` can turn it into a `WagerError`.",
                tutorial: "## Concept\n\
`place_bet` has two steps that fail in different ways: parsing fails with \
a `ParseIntError`, paying fails with `Short`. The function promises one \
error type, `WagerError`. On an `Err`, `?` converts the error with \
`From::from`, so each step's error needs an `impl From<...> for \
WagerError`. Without them the compiler stops with E0277, \"`?` couldn't \
convert the error to `WagerError`\".\n\n\
## Syntax\n\
```\nenum DiceError { Lost }\nstruct Spilled;\nimpl From<Spilled> for DiceError {\n    fn from(_: Spilled) -> Self {\n        DiceError::Lost\n    }\n}\n```\n\
With this impl, `?` on a `Result<_, Spilled>` works in a function that \
returns `Result<_, DiceError>`.\n\n\
## Task\n\
Write `impl From<ParseIntError> for WagerError` (giving \
`WagerError::BadStake`) and `impl From<Short> for WagerError` (giving \
`WagerError::TooPoor`). Leave `place_bet` as it is. Your program should \
print `Ok(35)`, `Err(BadStake)`, then `Err(TooPoor)`.\n\n\
## Hint\n\
The grader needs `impl From<ParseIntError> for WagerError` and \
`impl From<Short> for WagerError`. A `.map_err(|_| WagerError::BadStake)` \
before each `?` also compiles, but then the conversion is repeated at \
every call instead of written once.",
                prereq: None,
                available: true,
                starter_code: "use std::num::ParseIntError;\n\n#[derive(Debug)]\nstruct Short;\n\nfn pay(purse: u32, stake: u32) -> Result<u32, Short> {\n    if stake > purse { Err(Short) } else { Ok(purse - stake) }\n}\n\n#[derive(Debug)]\nenum WagerError {\n    BadStake,\n    TooPoor,\n}\n\nfn place_bet(text: &str, purse: u32) -> Result<u32, WagerError> {\n    // this won't compile yet: `?` has no way to make a WagerError\n    let stake = text.trim().parse::<u32>()?;\n    let left = pay(purse, stake)?;\n    Ok(left)\n}\n\nfn main() {\n    println!(\"{:?}\", place_bet(\"15\", 50));\n    println!(\"{:?}\", place_bet(\"lots\", 50));\n    println!(\"{:?}\", place_bet(\"80\", 50));\n}\n",
            },
            Mission {
                id: "wager_quest",
                npc_name: "The Barkeep",
                prompt: "Chain the quest's three steps with `?`: no `match`, `unwrap`, `expect` or `panic!`.",
                tutorial: "## Concept\n\
The wager itself: a quest of three steps, each of which can fail with its \
own error. The `From` impls already turn each one into a `QuestError`. \
With `?` after each step the function reads top to bottom: every step \
either gives its value or ends the quest with its error, and the first \
failure wins. `.unwrap()` does the opposite: on an `Err` it panics, and \
the whole program stops, the quests after it included.\n\n\
## Syntax\n\
```\nfn trip(a: &str, b: &str) -> Result<u32, TripError> {\n    let x = step_one(a)?;\n    let y = step_two(b)?;\n    Ok(x + y)\n}\n```\n\
No nested `match`: each `?` handles its own step's failure.\n\n\
## Task\n\
Replace each `.unwrap()` in `quest` with `?`. Use no `match`, `unwrap`, \
`expect` or `panic!` anywhere in the program: the Tavernkeeper checks. \
Your program should print `Ok(22)`, `Err(BadMap)`, `Err(Locked)`, then \
`Err(Broke)`.\n\n\
## Hint\n\
The grader needs `read_map(map)?`, `open_vault(key)?` and \
`pay_guide(gold, leagues)?`. As written, the starter prints `Ok(22)` and \
then panics on the second quest: called `Result::unwrap()` on an `Err` \
value.",
                prereq: None,
                available: true,
                starter_code: "use std::num::ParseIntError;\n\n#[derive(Debug)]\nstruct WrongKey;\n\n#[derive(Debug)]\nstruct Short;\n\n#[derive(Debug)]\nstruct Reward {\n    gold: u32,\n}\n\n#[derive(Debug)]\nenum QuestError {\n    BadMap,\n    Locked,\n    Broke,\n}\n\nimpl From<ParseIntError> for QuestError {\n    fn from(_: ParseIntError) -> Self {\n        QuestError::BadMap\n    }\n}\nimpl From<WrongKey> for QuestError {\n    fn from(_: WrongKey) -> Self {\n        QuestError::Locked\n    }\n}\nimpl From<Short> for QuestError {\n    fn from(_: Short) -> Self {\n        QuestError::Broke\n    }\n}\n\n// Step 1: the map says how many leagues to walk.\nfn read_map(map: &str) -> Result<u32, ParseIntError> {\n    map.trim().parse::<u32>()\n}\n\n// Step 2: only the brass key opens the vault. It holds 30 gold.\nfn open_vault(key: &str) -> Result<u32, WrongKey> {\n    if key == \"brass\" { Ok(30) } else { Err(WrongKey) }\n}\n\n// Step 3: the guide charges 2 gold a league.\nfn pay_guide(gold: u32, leagues: u32) -> Result<u32, Short> {\n    let fee = leagues * 2;\n    if fee > gold { Err(Short) } else { Ok(gold - fee) }\n}\n\nfn quest(map: &str, key: &str) -> Result<Reward, QuestError> {\n    // this runs, but the first failed step brings the tavern down\n    let leagues = read_map(map).unwrap();\n    let gold = open_vault(key).unwrap();\n    let left = pay_guide(gold, leagues).unwrap();\n    Ok(Reward { gold: left })\n}\n\nfn main() {\n    println!(\"{:?}\", quest(\"4\", \"brass\").map(|r| r.gold));\n    println!(\"{:?}\", quest(\"far\", \"brass\").map(|r| r.gold));\n    println!(\"{:?}\", quest(\"4\", \"bone\").map(|r| r.gold));\n    println!(\"{:?}\", quest(\"20\", \"brass\").map(|r| r.gold));\n}\n",
            },
            // ── Act 6: The Iterator Forge ─────────────────────────────
            // Collections (HashMap), iterator adapters, closure capture.
            // See design/01-curriculum.md §Act 6 and
            // docs/superpowers/specs/2026-06-18-act6-iterator-forge-missions-design.md.
            Mission {
                id: "hashmap_basic",
                npc_name: "The Keymaster",
                prompt: "Store a value under a key in a `HashMap`, then look it up.",
                tutorial: "## Concept\n\
A `HashMap<K, V>` stores values keyed by a lookup key — Rust's analogue of \
a Python `dict` or JS object/Map. Unlike `Vec`, it lives in \
`std::collections`, so you `use` it first. `.insert(k, v)` adds or \
replaces; `.get(&k)` returns an `Option<&V>` (the key might be absent — the \
type system makes you handle that).\n\n\
## Syntax\n\
```\nuse std::collections::HashMap;\nlet mut m = HashMap::new();\nm.insert(\"ace\", 1);\nlet score = m.get(\"ace\");   // Some(&1)\n```\n\
The map's `K`/`V` types are inferred from the first `insert`. `.get` borrows \
the key; it returns `None` for a missing one.\n\n\
## Task\n\
Bring `HashMap` into scope and build one. `insert` the value `7` under \
the key `\"vault\"`, then print `.get(\"vault\")` and `.get(\"cellar\")` \
with `{:?}`, one per line. Your program should print `Some(7)`, then \
`None`: nothing was ever stored under the cellar key.\n\n\
## Hint\n\
The grader needs `HashMap`, `.insert(`, and `.get(`. The Keymaster files \
every store under its own key.",
                prereq: None,
                available: true,
                starter_code: "fn main() {\n    // store 7 under the key vault in a map, then print what looking up vault and cellar gives\n    let _ = ();\n}\n",
            },
            Mission {
                id: "iter_filter",
                npc_name: "The Sifter",
                prompt: "Keep only the even numbers with `.filter()`, then `.collect()`.",
                tutorial: "## Concept\n\
`.filter(pred)` is the iterator adapter that keeps only the elements for \
which the predicate returns `true`, dropping the rest. Like `.map`, it's \
lazy — nothing happens until a consumer like `.collect` or a `for` loop \
drives it. The predicate is a closure receiving a *reference* to each \
element.\n\n\
## Syntax\n\
```\nlet v = vec![1, 2, 3, 4];\nlet evens: Vec<&i32> = v.iter()\n    .filter(|x| **x % 2 == 0)\n    .collect();\n```\n\
`.iter()` yields `&i32`, and `.filter`'s closure gets a `&&i32`, so you \
deref twice (`**x`) to compare the value.\n\n\
## Task\n\
Build a `Vec<i32>`, then keep only the even elements with `.filter` and \
gather the survivors with `.collect`.\n\n\
## Hint\n\
The grader needs `.filter(` and `.collect`. The Sifter shakes the screen — \
only what fits the mesh falls through.",
                prereq: None,
                available: true,
                starter_code: "fn main() {\n    let v = vec![1, 2, 3, 4];\n    // keep only the even numbers, gathering the survivors into a new collection\n    let _ = &v;\n}\n",
            },
            Mission {
                id: "iter_fold",
                npc_name: "The Smelter",
                prompt: "Reduce a `Vec` to a single total with `.fold(0, |acc, x| …)`.",
                tutorial: "## Concept\n\
`.fold(init, f)` is the general reducer: it threads an *accumulator* \
through the iterator, starting from `init` and combining each element via \
the closure `|acc, x| ...`, returning the final accumulator. `.sum()` and \
`.product()` are special cases of fold; reach for `fold` when the \
combination is custom (min, max-by, building a string, …).\n\n\
## Syntax\n\
```\nlet v = vec![1, 2, 3];\nlet total = v.iter().fold(0, |acc, x| acc + x);  // 6\nlet joined = v.iter().fold(String::new(), |s, x| s + &x.to_string());\n```\n\
The accumulator's type is whatever `init` is; each step returns the next \
accumulator.\n\n\
## Task\n\
Build a `Vec<i32>` and reduce it to its sum with `.fold` (not `.sum()` — \
this mission is about fold).\n\n\
## Hint\n\
The grader needs `.fold(`. The Smelter pours many ores into one crucible — \
one ingot comes out.",
                prereq: None,
                available: true,
                starter_code: "fn main() {\n    let v = vec![1, 2, 3];\n    // reduce the vec to a single total by accumulating across it (don't use .sum())\n    let _ = &v;\n}\n",
            },
            Mission {
                id: "iter_enumerate",
                npc_name: "The Tallywright",
                prompt: "Iterate with `.enumerate()` to get each item's index alongside it.",
                tutorial: "## Concept\n\
`.enumerate()` adapts an iterator of `T` into an iterator of `(usize, T)` — \
each element paired with its position, starting at 0. It's the idiomatic \
way to get an index in a `for` loop without a manual counter. Destructure \
the tuple in the loop pattern: `for (i, x) in ...`.\n\n\
## Syntax\n\
```\nlet v = vec![10, 20, 30];\nfor (i, x) in v.iter().enumerate() {\n    println!(\"{i}: {x}\");   // 0: 10, 1: 20, 2: 30\n}\n```\n\
The index is a `usize`; `x` is whatever the underlying iterator yielded \
(here `&i32`).\n\n\
## Task\n\
Walk a `Vec` with `.enumerate()` and a `for (i, x)` pattern so each step \
has both the position and the value.\n\n\
## Hint\n\
The grader needs `.enumerate(`. The Tallywright numbers every item as it \
passes the bench.",
                prereq: None,
                available: true,
                starter_code: "fn main() {\n    let v = vec![10, 20, 30];\n    // walk the vec so each step gives you both the position and the value\n    let _ = &v;\n}\n",
            },
            Mission {
                id: "iter_zip",
                npc_name: "The Riveter",
                prompt: "Pair two `Vec`s element-by-element with `.zip()`.",
                tutorial: "## Concept\n\
`.zip(other)` walks two iterators in lockstep, yielding pairs \
`(a, b)` until the *shorter* one runs out. It's how you process two \
sequences together — names with scores, keys with values, this row with \
that row. The result is an iterator of tuples you can `for`-destructure, \
`map` over, or `collect`.\n\n\
## Syntax\n\
```\nlet a = vec![1, 2];\nlet b = vec![3, 4];\nfor (x, y) in a.iter().zip(b.iter()) {\n    println!(\"{x},{y}\");   // 1,3  then  2,4\n}\n```\n\
If the vecs differ in length, `zip` stops at the shorter — no panic.\n\n\
## Task\n\
Build two `Vec`s and walk them together with `.zip()`, pairing the nth of \
one with the nth of the other.\n\n\
## Hint\n\
The grader needs `.zip(`. The Riveter joins two plates rivet by rivet — \
aligned, pair by pair.",
                prereq: None,
                available: true,
                starter_code: "fn main() {\n    let a = vec![1, 2];\n    let b = vec![3, 4];\n    // walk both vecs together, pairing the nth of one with the nth of the other\n    let _ = (&a, &b);\n}\n",
            },
            Mission {
                id: "closure_move",
                npc_name: "The Bondsmith",
                prompt: "Capture a value by value into a `move` closure, then call it.",
                tutorial: "## Concept\n\
A closure normally *borrows* the variables it captures. Prefix it with \
`move` and it takes them *by value* instead — the closure owns its \
captures. This is essential when the closure outlives the current scope \
(returned from a function, or `spawn`ed onto a thread): the data has to \
move with it, not be borrowed from a stack frame that's about to vanish.\n\n\
## Syntax\n\
```\nlet name = String::from(\"Garin\");\nlet greet = move || println!(\"hail, {name}\");\ngreet();   // `name` was moved into `greet`\n// `name` is no longer usable here — the closure owns it\n```\n\
Without `move`, `greet` would borrow `name`; with `move`, it owns it.\n\n\
## Task\n\
Bind a `String`, then make a `move` closure that captures it by value and \
prints it, and call the closure.\n\n\
## Hint\n\
The grader needs a `move` closure — the literal `move |`. The Bondsmith \
seals the charge into himself; it travels where he goes.",
                prereq: None,
                available: true,
                starter_code: "fn main() {\n    let name = String::from(\"Garin\");\n    // capture `name` BY VALUE into a closure (so the closure owns it), then call it\n    let _ = &name;\n}\n",
            },
            // ── Act 7: The Concurrent Coast ───────────────────────────
            // Threads, Arc<Mutex>, channels, atomics, scoped threads, async.
            // See design/01-curriculum.md §Act 7 and
            // docs/superpowers/specs/2026-06-18-act7-concurrent-coast-missions-design.md.
            Mission {
                id: "thread_spawn",
                npc_name: "The Dockmaster",
                prompt: "Run work on another thread with `thread::spawn`, then `.join()` it.",
                tutorial: "## Concept\n\
`std::thread::spawn(closure)` runs the closure on a *new* OS thread, \
immediately returning a `JoinHandle`. The new thread runs concurrently with \
the one that spawned it. Calling `.join()` on the handle blocks until that \
thread finishes and hands back its return value (wrapped in a `Result`, \
since a thread can panic). This is the foundation of parallelism in Rust.\n\n\
## Syntax\n\
```\nuse std::thread;\nlet handle = thread::spawn(|| {\n    21 * 2   // runs on the new thread\n});\nlet answer = handle.join().unwrap();   // waits, then gets 42\n```\n\
The spawned closure must be `'static` and `Send` — it can't borrow \
short-lived locals (use `move`, or `thread::scope`).\n\n\
## Task\n\
Spawn a thread that computes a small value, then `.join()` the handle to \
wait for it.\n\n\
## Hint\n\
The grader needs `thread::spawn` and `.join(`. The Dockmaster sends a \
worker off, then waits at the gate for them to return.",
                prereq: None,
                available: true,
                starter_code: "fn main() {\n    // run a small computation on another thread, then wait for it to finish\n    let _ = ();\n}\n",
            },
            Mission {
                id: "arc_mutex",
                npc_name: "The Lighthouse Keeper",
                prompt: "Share mutable state behind an `Arc<Mutex<T>>` and `.lock()` it.",
                tutorial: "## Concept\n\
To *share* data across threads you need two things: shared ownership and \
safe mutation. `Arc<T>` (Atomically Reference-Counted) gives many threads \
joint ownership of the same value. `Mutex<T>` guards the value so only one \
thread holds it at a time. Together, `Arc<Mutex<T>>` is the canonical \
shared-mutable-state pattern: clone the `Arc` to each thread, `.lock()` to \
get exclusive access.\n\n\
## Syntax\n\
```\nuse std::sync::{Arc, Mutex};\nlet shared = Arc::new(Mutex::new(0));\n{\n    let mut guard = shared.lock().unwrap();\n    *guard += 1;          // exclusive access while the guard lives\n}   // lock released here\n```\n\
`.lock()` returns a `Result` (it errors if another thread panicked holding \
the lock); the guard derefs to the inner value.\n\n\
## Task\n\
Wrap `0` in `Arc::new(Mutex::new(...))` and make a second handle with \
`Arc::clone`. Add `5` through the second handle's `.lock()` guard, then \
lock the first handle and print the value. Your program should print \
`5`: both handles share one value. (This sandbox runs a single thread, \
so both handles stay on it; the clone is what you would move into a \
spawned thread.)\n\n\
## Hint\n\
The grader needs `Arc`, `Mutex`, and `.lock(`. The Keeper holds the one key \
to the lamp — only one hand on it at a time.",
                prereq: None,
                available: true,
                starter_code: "fn main() {\n    // wrap 0 so two handles can share it and mutate it one-at-a-time behind a guard; add 5 through one, print through the other\n    let _ = 0;\n}\n",
            },
            Mission {
                id: "mpsc_channel",
                npc_name: "The Signaler",
                prompt: "Send a value down an `mpsc::channel` and receive it.",
                tutorial: "## Concept\n\
A channel is a one-way pipe between threads: the *sender* pushes values in, \
the *receiver* pulls them out. `std::sync::mpsc::channel()` returns a \
`(Sender, Receiver)` pair (mpsc = multi-producer, single-consumer — you can \
clone the sender). `tx.send(v)` queues a value; `rx.recv()` blocks until \
one arrives. Channels let threads communicate by *passing messages* instead \
of sharing memory — often simpler to reason about than locks.\n\n\
## Syntax\n\
```\nuse std::sync::mpsc;\nlet (tx, rx) = mpsc::channel();\ntx.send(7).unwrap();\nlet got = rx.recv().unwrap();   // 7\n```\n\
Both `send` and `recv` return `Result` — `send` errors if the receiver is \
gone, `recv` if all senders are.\n\n\
## Task\n\
Open a channel, `send` the value `7` and then `11` through the sender, \
and `recv` twice from the receiver, printing each value on its own line. \
Your program should print `7`, then `11`: a channel hands values out in \
the order they went in.\n\n\
## Hint\n\
The grader needs `mpsc::channel`, `.send(`, and `.recv(`. The Signaler \
flashes the message down the coast; the next tower reads it.",
                prereq: None,
                available: true,
                starter_code: "fn main() {\n    // open a channel, push 7 then 11 in at one end, and pull both out at the other, printing each\n    let _ = ();\n}\n",
            },
            Mission {
                id: "atomic",
                npc_name: "The Tidewatch",
                prompt: "Increment a shared `AtomicUsize` with `.fetch_add()` — no lock.",
                tutorial: "## Concept\n\
For simple shared counters, a `Mutex` is overkill. The `std::sync::atomic` \
types (`AtomicUsize`, `AtomicBool`, …) support *lock-free* updates: the \
hardware guarantees the operation happens as one indivisible step, so many \
threads can hit it at once without tearing. `.fetch_add(n, ordering)` adds \
`n` and returns the previous value, atomically. The `Ordering` argument \
controls how this operation synchronizes with others; `SeqCst` is the \
simplest, strongest choice.\n\n\
## Syntax\n\
```\nuse std::sync::atomic::{AtomicUsize, Ordering};\nlet counter = AtomicUsize::new(0);\ncounter.fetch_add(1, Ordering::SeqCst);\nlet now = counter.load(Ordering::SeqCst);   // 1\n```\n\
No `.lock()`, no guard — the atomic *is* the synchronization.\n\n\
## Task\n\
Make an `AtomicUsize` holding `10`. Bump it with `.fetch_add(1, \
Ordering::SeqCst)` and print the value that call returns, then print \
`.load(Ordering::SeqCst)`. Your program should print `10`, then `11`: \
`fetch_add` returns the value from *before* the add.\n\n\
## Hint\n\
The grader needs `Atomic` and `.fetch_add(`. The Tidewatch's gauge clicks \
up one notch — no gatekeeper needed.",
                prereq: None,
                available: true,
                starter_code: "fn main() {\n    // make a shared counter holding 10 that can be bumped without a lock; bump it by one, print what that returned, then the new value\n    let _ = 0;\n}\n",
            },
            Mission {
                id: "thread_scope",
                npc_name: "The Harbormaster",
                prompt: "Use `thread::scope` to run threads that borrow a local.",
                tutorial: "## Concept\n\
`thread::spawn` requires `'static` closures — they can't borrow locals, \
because the thread might outlive them. `thread::scope` solves this: it \
guarantees every thread spawned inside it finishes before the scope \
returns, so those threads *can* safely borrow stack data. You spawn with \
`s.spawn(...)` on the scope handle; the scope joins them all automatically \
at the closing brace.\n\n\
## Syntax\n\
```\nuse std::thread;\nlet data = vec![1, 2, 3];\nthread::scope(|s| {\n    s.spawn(|| {\n        println!(\"{:?}\", &data);   // borrows `data` — allowed!\n    });\n});   // all scoped threads joined here\n```\n\
Because the scope blocks until its threads end, the borrow is sound — no \
`Arc`, no `'static` needed.\n\n\
## Task\n\
Inside `thread::scope`, `s.spawn` a thread that borrows a local `Vec`.\n\n\
## Hint\n\
The grader needs `thread::scope` and `.spawn(`. The Harbormaster won't \
close the harbor until every boat is back in.",
                prereq: None,
                available: true,
                starter_code: "fn main() {\n    let data = vec![1, 2, 3];\n    // run threads that borrow `data` and are guaranteed to finish before this function returns\n    let _ = &data;\n}\n",
            },
            Mission {
                id: "async_fn",
                npc_name: "The Tideforecaster",
                prompt: "Write an `async fn` and `.await` another async call.",
                tutorial: "## Concept\n\
An `async fn` doesn't run its body when called — it returns a *future*, a \
value representing work that can be paused and resumed. You drive a future \
to completion by `.await`-ing it inside another async context; `.await` \
yields control while the work is pending, letting other tasks run. This is \
how Rust does cooperative, non-blocking concurrency.\n\n\
**Note:** a future does nothing until an *executor* polls it — normally \
provided by a runtime crate like `tokio` (`#[tokio::main]`). This grading \
sandbox is dependency-free, so the code here *compiles* and is graded, but \
the futures are never actually run. The concept is the lesson.\n\n\
## Syntax\n\
```\nasync fn double(x: i32) -> i32 { x * 2 }\nasync fn run() -> i32 {\n    double(21).await   // await another async fn\n}\n// with tokio: #[tokio::main] async fn main() { run().await; }\n```\n\
## Task\n\
Define an `async fn` that doubles a number, and another `async fn` that \
calls it with `.await`.\n\n\
## Hint\n\
The grader needs `async fn` and `.await`. The Tideforecaster reads the \
almanac and waits for a tide that hasn't come in yet.",
                prereq: None,
                available: true,
                starter_code: "fn main() {\n    // define an async function that doubles a number, and another that calls it and waits on the result\n    // (it won't actually run here without a runtime like tokio - that's expected)\n    let _ = ();\n}\n",
            },
            // ── Act 8: The Vault of Pointers ──────────────────────────
            // Box, Rc, RefCell, Cell, Rc<RefCell>, Weak. See
            // design/01-curriculum.md §Act 8 and
            // docs/superpowers/specs/2026-06-19-act8-vault-pointers-missions-design.md.
            Mission {
                id: "box_basic",
                npc_name: "The Vaultwright",
                prompt: "Use `Box<T>` to put a recursive `Node`'s child on the heap.",
                tutorial: "## Concept\n\
A `Box<T>` is the simplest smart pointer: it owns a single value stored on \
the *heap* instead of the stack. The headline use is *recursive types* — a \
`struct Node { next: Node }` has infinite size and won't compile, but \
`next: Option<Box<Node>>` is fine: the `Box` is a fixed-size pointer, so the \
type's size is known. `Box` is also how you store a value when you only have \
a `dyn Trait` (unsized) or want to move a big value cheaply.\n\n\
## Syntax\n\
```\nstruct Node {\n    value: i32,\n    next: Option<Box<Node>>,\n}\nlet list = Node {\n    value: 1,\n    next: Some(Box::new(Node { value: 2, next: None })),\n};\n```\n\
`Box::new(x)` moves `x` to the heap; the `Box` derefs to `x` transparently.\n\n\
## Task\n\
A `Node` is given with a non-recursive `next`. Make `next` an \
`Option<Box<Node>>` and build a two-node list with `Box::new`.\n\n\
## Hint\n\
The grader needs `Box<` and `Box::new`. The Vaultwright builds a box on the \
heap to hold what the stack cannot.",
                prereq: None,
                available: true,
                starter_code: "struct Node {\n    value: i32,\n    next: Option<i32>,\n}\n\nfn main() {\n    // make `next` able to hold another Node on the heap behind a pointer, then build a 2-node list\n    let _ = Node { value: 1, next: None };\n}\n",
            },
            Mission {
                id: "rc_basic",
                npc_name: "The Sharekeeper",
                prompt: "Give two owners a shared `Rc` handle with `Rc::new` + `Rc::clone`.",
                tutorial: "## Concept\n\
`Rc<T>` (Reference Counted) lets *several* owners share one value on the \
heap, in a single thread. Each `Rc::clone` bumps a count and hands out \
another handle to the *same* allocation (it does NOT deep-copy the value); \
when the last `Rc` drops, the value is freed. Use it for shared, read-mostly \
data with no single clear owner — a graph node referenced from many places, \
say. (`Arc` is the thread-safe version you met on the Coast.)\n\n\
## Syntax\n\
```\nuse std::rc::Rc;\nlet original = Rc::new(String::from(\"hoard\"));\nlet shared = Rc::clone(&original);   // count is now 2, same allocation\nprintln!(\"{}\", Rc::strong_count(&original));   // 2\n```\n\
`Rc::clone(&x)` is the idiomatic spelling (cheap, makes the count bump \
explicit) — prefer it over `x.clone()`.\n\n\
## Task\n\
Wrap `String::from(\"hoard\")` in `Rc::new` and print \
`Rc::strong_count(&original)`. Make a second handle with `Rc::clone` and \
print the count again, then `drop` that handle and print it a third time. \
Your program should print `1`, `2`, then `1`, one per line.\n\n\
## Hint\n\
The grader needs `Rc::new` and `Rc::clone`. The Sharekeeper hands out claims \
to one hoard — and keeps count. Cloning the `String` itself and wrapping \
the copy in a new `Rc` makes a second hoard, and the count stays at `1`.",
                prereq: None,
                available: true,
                starter_code: "fn main() {\n    // give two owners a shared, reference-counted handle to the same value (single-threaded),\n    // printing the owner count before the second handle, with it, and after dropping it\n    let _ = String::from(\"hoard\");\n}\n",
            },
            Mission {
                id: "refcell",
                npc_name: "The Warden",
                prompt: "Mutate through a shared reference with `RefCell` + `.borrow_mut()`.",
                tutorial: "## Concept\n\
Normally Rust enforces the borrow rules at *compile* time. `RefCell<T>` \
moves that check to *runtime*: it lets you mutate the inner value through a \
shared `&` reference (\"interior mutability\"). `.borrow()` hands out a shared \
read guard, `.borrow_mut()` an exclusive write guard — and if you violate \
the one-writer-XOR-many-readers rule at runtime, it *panics* instead of \
failing to compile. Pair it with `Rc` for shared-mutable single-threaded \
data.\n\n\
## Syntax\n\
```\nuse std::cell::RefCell;\nlet guarded = RefCell::new(0);\n*guarded.borrow_mut() += 1;   // exclusive write\nlet r = guarded.borrow();     // shared read\n```\n\
The guards release when they drop; holding a `borrow_mut` while taking \
another borrow panics.\n\n\
## Task\n\
Put `10` in `RefCell::new` and take a plain shared reference (`&`) to it. \
Print the value through that reference with `.borrow()`, add `5` through \
the same reference with `.borrow_mut()`, then print it again. Your \
program should print `10`, then `15`.\n\n\
## Hint\n\
The grader needs `RefCell` and `.borrow_mut(`. The Warden lets you change \
what's in the box — but checks the ledger as you do. Copying the number \
out (`let mut n = *r.borrow_mut();`) and changing the copy leaves the box \
untouched.",
                prereq: None,
                available: true,
                starter_code: "fn main() {\n    // hold 10 in a value you can mutate through a shared reference, with the borrow rules checked at runtime;\n    // print it, add 5 through the shared reference, print it again\n    let _ = 0;\n}\n",
            },
            Mission {
                id: "cell",
                npc_name: "The Swapwarden",
                prompt: "Swap a `Copy` value through a shared reference with `Cell` + `.set()`.",
                tutorial: "## Concept\n\
`Cell<T>` is the simpler sibling of `RefCell`, most useful for small `Copy` \
types. It gives interior mutability with no borrow guards and no runtime \
checks: through a shared `&Cell` you can't get a reference *into* it, only \
`.get()` a copy out (that needs `T: Copy`) or `.set()` / `.replace()` a \
whole new value in (any `T`). Because you never hold a reference to the \
interior, there's nothing to check — it's cheaper than `RefCell` when the \
value is small and `Copy`.\n\n\
## Syntax\n\
```\nuse std::cell::Cell;\nlet slot = Cell::new(1);\nslot.set(5);             // swap the whole value\nlet now = slot.get();    // copy it out: 5\n```\n\
No `.borrow()` — `Cell` deals in whole values, not references.\n\n\
## Task\n\
Put `1` in `Cell::new` and take a shared reference (`&`) to it. Print \
`.get()`, then `.set(5)` through the shared reference, then print \
`.get()` again. Your program should print `1`, then `5`.\n\n\
## Hint\n\
The grader needs `Cell::new` and `.set(`. The Swapwarden trades the whole \
coin in the slot — no borrowing, just a swap. `.get()` hands you a copy, \
so changing that copy changes nothing in the slot.",
                prereq: None,
                available: true,
                starter_code: "fn main() {\n    // hold 1 as a single Copy value you can swap out through a shared reference (no borrow guard);\n    // print it, swap in 5 through the shared reference, print it again\n    let _ = 1;\n}\n",
            },
            Mission {
                id: "rc_refcell",
                npc_name: "The Strongbox",
                prompt: "Combine `Rc<RefCell<T>>` for shared *and* mutable single-thread data.",
                tutorial: "## Concept\n\
`Rc` gives shared ownership but only immutable access; `RefCell` gives \
interior mutability but a single owner. Compose them — `Rc<RefCell<T>>` — \
and you get the workhorse pattern for single-threaded shared-mutable state: \
clone the `Rc` to every owner, and any of them can `.borrow_mut()` the \
shared value. It's how you build mutable graphs, shared caches, and \
observer lists without threads. (Across threads, the analogue is \
`Arc<Mutex<T>>`.)\n\n\
## Syntax\n\
```\nuse std::cell::RefCell;\nuse std::rc::Rc;\nlet shared = Rc::new(RefCell::new(0));\nlet clone = Rc::clone(&shared);   // both point at the same RefCell\n*clone.borrow_mut() += 1;          // ...and either can mutate it\n```\n\
The `Rc` shares the allocation; the `RefCell` guards the mutation.\n\n\
## Task\n\
Build an `Rc::new(RefCell::new(0))` and clone the `Rc`. Add `10` through \
`.borrow_mut()` on the clone and `5` through `.borrow_mut()` on the \
original, then print the value. Your program should print `15`: both \
handles open the same box.\n\n\
## Hint\n\
The grader needs `Rc::new`, `RefCell`, and `.borrow_mut(`. The Strongbox: \
many keys to one box, and the box can be opened and changed. If you see \
`10` or `5`, you built two boxes instead of cloning the `Rc`.",
                prereq: None,
                available: true,
                starter_code: "fn main() {\n    // make a value starting at 0 that two owners share AND can mutate (the single-threaded shared-mutable pattern);\n    // add 10 through one owner, 5 through the other, then print it\n    let _ = 0;\n}\n",
            },
            Mission {
                id: "weak_ref",
                npc_name: "The Ghostkeeper",
                prompt: "Make a non-owning `Weak` reference with `Rc::downgrade`, then `.upgrade()`.",
                tutorial: "## Concept\n\
Two `Rc`s pointing at each other would never reach count 0 — a *reference \
cycle* that leaks. `Weak<T>` breaks it: it's a non-owning handle to an `Rc`'s \
value that does NOT keep it alive. You make one with `Rc::downgrade(&rc)`. \
Because the value might already be gone, you can't use a `Weak` directly — \
you call `.upgrade()`, which returns `Option<Rc<T>>`: `Some` if the value's \
still alive, `None` if it's been dropped. Use `Weak` for back-pointers \
(child → parent) so the parent owns the child, not vice versa.\n\n\
## Syntax\n\
```\nuse std::rc::{Rc, Weak};\nlet strong = Rc::new(5);\nlet weak: Weak<i32> = Rc::downgrade(&strong);   // doesn't bump the strong count\nif let Some(rc) = weak.upgrade() { /* still alive */ }\n```\n\
`downgrade` creates the weak handle; `upgrade` safely tries to reclaim a \
strong one.\n\n\
## Task\n\
From `Rc::new(5)`, make a `Weak` with `Rc::downgrade` and print \
`weak.upgrade()` with `{:?}`. Then `drop` the `Rc` and print \
`weak.upgrade()` again. Your program should print `Some(5)`, then `None`.\n\n\
## Hint\n\
The grader needs `Weak`, `downgrade`, and `.upgrade(`. The Ghostkeeper holds \
a tether to a hoard that may already have vanished. Don't keep the first \
upgraded `Rc` in a variable past the `drop`: it is a strong owner too, \
and would keep the value alive.",
                prereq: None,
                available: true,
                starter_code: "fn main() {\n    // make a non-owning handle to an Rc holding 5 (one that won't keep it alive, and might find it gone);\n    // print what accessing it gives, drop the Rc, then print it again\n    let _ = ();\n}\n",
            },
        ];
        // Strict-linear progression: each mission's prereq is the one
        // listed immediately before it. Shape decision logged in HANDOFF
        // (2026-05-03 What's Next item 1) — branching can come later
        // when the registry has Acts 3+ to organize.
        for i in 1..missions.len() {
            missions[i].prereq = Some(missions[i - 1].id);
        }
        // Demo build: everything after the Act 2 boundary is unavailable.
        // If the boundary id ever stops resolving, keep every mission
        // available rather than shipping an empty game; the
        // `demo_boundary_is_the_last_act_2_mission` test catches it.
        if cfg!(feature = "demo") {
            match missions.iter().position(|m| m.id == DEMO_LAST_MISSION_ID) {
                Some(last) => {
                    for m in &mut missions[last + 1..] {
                        m.available = false;
                    }
                    tracing::info!(
                        last_free = DEMO_LAST_MISSION_ID,
                        free = last + 1,
                        total = missions.len(),
                        "demo build: missions after the Act 2 boundary are unavailable"
                    );
                }
                None => tracing::error!(
                    boundary = DEMO_LAST_MISSION_ID,
                    "demo boundary mission not in registry; demo gate disabled"
                ),
            }
        }
        Self { missions }
    }
}

/// Emitted when the player presses F against an NPC whose mission is
/// still locked (its prereq isn't cleared). Carries the attempted
/// mission id so consumers can log/branch. The audio plugin reads this
/// to fire the `mission_locked` sting — keeping the locked-attempt
/// detection here (the one place that already owns the F-press +
/// nearby-NPC + progress reads) instead of re-deriving it in audio.rs.
#[derive(Message, Debug, Clone)]
pub struct MissionLockedAttempt {
    pub mission_id: String,
}

pub struct MissionPlugin;

impl Plugin for MissionPlugin {
    fn build(&self, app: &mut App) {
        tracing::debug!("MissionPlugin::build");
        app.init_resource::<MissionRegistry>()
            .init_resource::<ActiveMission>()
            .init_resource::<CompletionView>()
            .init_resource::<EpilogueView>()
            .init_resource::<trial::TrialView>()
            .add_message::<MissionLockedAttempt>()
            .add_systems(
                Update,
                (
                    handle_interact_key,
                    trial::trial_panel_keys,
                    dismiss_completion_on_escape,
                    dismiss_epilogue_on_escape,
                    trial::watch_stage_clears,
                )
                    .run_if(in_state(GameState::InGame)),
            )
            .add_systems(
                EguiPrimaryContextPass,
                (
                    draw_interaction_prompt,
                    draw_next_pin,
                    draw_completion_panel,
                    draw_epilogue_panel,
                )
                    .run_if(in_state(GameState::InGame)),
            );
        #[cfg(feature = "demo")]
        app.init_resource::<DemoEndView>()
            .add_systems(
                Update,
                (
                    reopen_demo_end_on_locked_attempt,
                    dismiss_demo_end_on_escape,
                )
                    .run_if(in_state(GameState::InGame)),
            )
            .add_systems(
                EguiPrimaryContextPass,
                draw_demo_end_panel.run_if(in_state(GameState::InGame)),
            );
    }
}

/// End-of-demo panel state (`demo` feature only). The panel auto-shows
/// once [`DEMO_LAST_MISSION_ID`] is cleared; Esc hides it for the
/// session, and trying to start a full-game mission brings it back.
#[derive(Resource, Default)]
pub struct DemoEndView {
    pub dismissed_this_session: bool,
}

#[derive(Resource, Default)]
pub struct ActiveMission {
    pub current: Option<Mission>,
}

/// Shown after a player F-talks to a cleared NPC. Recaps the lesson
/// before letting them re-enter the editor to revisit. Cleared by Esc
/// or by pressing F again (which falls through to opening the editor).
#[derive(Resource, Default)]
pub struct CompletionView {
    pub mission_id: Option<String>,
}

/// Win-condition flag. The epilogue panel auto-shows once
/// `cleared_count == registry.missions.len()`. After Esc-dismissal in a
/// session the player can revisit it via any cleared NPC's completion
/// flow — this state just suppresses the auto-pop until they earn it
/// again (which only matters if a future patch invalidates clears).
#[derive(Resource, Default)]
pub struct EpilogueView {
    pub dismissed_this_session: bool,
}

#[allow(clippy::too_many_arguments)]
fn handle_interact_key(
    keys: Res<ButtonInput<KeyCode>>,
    handoff: Option<ResMut<crate::plugins::dialogue::DialogueHandoff>>,
    nearby: Res<NearbyNpc>,
    registry: Res<MissionRegistry>,
    progress: Res<MissionProgress>,
    mut editor: ResMut<EditorState>,
    mut active: ResMut<ActiveMission>,
    mut completion: ResMut<CompletionView>,
    mut locked_attempts: MessageWriter<MissionLockedAttempt>,
) {
    // Same input-bleed class as WASD — F is gameplay, not text input.
    // If the editor is open, the player is typing their solution and
    // an F keystroke means "the letter f", not "talk to NPC".
    if editor.open {
        return;
    }
    // With DialoguePlugin mounted, F opens a conversation (and is consumed
    // there); its last line raises the handoff. Bare F remains for
    // fixtures that mount MissionPlugin alone.
    let handed_off = handoff.is_some_and(|mut h| h.take());
    if !handed_off && !keys.just_pressed(KeyCode::KeyF) {
        return;
    }
    let Some(entry) = nearby.current.as_ref() else {
        return;
    };
    // A recap open on one of this NPC's own missions (a trial stage picked
    // on the trial panel, say) is what F revisits; otherwise F is about the
    // NPC's current mission (see `trial::current_mission`).
    let viewing = completion
        .mission_id
        .as_deref()
        .filter(|id| trial::owned_missions(entry.mission_id).contains(id))
        .and_then(|id| registry.find(id));
    let Some(mission) = viewing
        .or_else(|| trial::current_mission(&registry, &progress, entry.mission_id))
        .cloned()
    else {
        tracing::warn!(
            "nearby NPC {} references unknown mission {}",
            entry.name,
            entry.mission_id
        );
        return;
    };

    // Not in this build (demo): never open the editor, even for a
    // mission a full-game save already cleared.
    if !mission.available {
        tracing::info!(
            "F-press on {} which is not in this build (demo ends after {})",
            mission.id,
            DEMO_LAST_MISSION_ID
        );
        locked_attempts.write(MissionLockedAttempt {
            mission_id: mission.id.to_string(),
        });
        return;
    }

    let cleared = progress.is_cleared(mission.id);
    let unlocked = mission.is_unlocked(&progress);
    let viewing_this = completion
        .mission_id
        .as_deref()
        .is_some_and(|id| id == mission.id);

    // Locked: prereq mission not cleared yet. Don't open the editor —
    // the prompt panel already renders the locked hint, so a no-op here
    // is the entire interaction (player gets visual feedback, no
    // gameplay state change). Cleared missions are always treated as
    // unlocked, so completion-revisit still works.
    if !cleared && !unlocked {
        tracing::debug!(
            "F-press on locked mission {} (prereq {:?} not cleared)",
            mission.id,
            mission.prereq
        );
        locked_attempts.write(MissionLockedAttempt {
            mission_id: mission.id.to_string(),
        });
        return;
    }

    // Cleared NPC + first F-press → completion recap, do NOT reopen
    // editor yet. Player must press F again (or Esc to dismiss) before
    // we drop them back into the code.
    if cleared && !viewing_this {
        tracing::info!("showing completion view for {}", mission.id);
        completion.mission_id = Some(mission.id.to_string());
        active.current = Some(mission);
        return;
    }

    // Either: not cleared (regular flow) OR cleared and player pressed
    // F a second time while viewing the completion (revisit flow).
    completion.mission_id = None;

    tracing::info!(
        "starting mission {} from {} ({}, revisit={})",
        mission.id,
        mission.npc_name,
        entry.name,
        cleared
    );
    start_mission(&mut editor, &mut active, mission);
}

/// Load `mission`'s starter into the editor, open it and make the mission
/// active (the F-press flow, and a trial's "next stage" key).
pub fn start_mission(editor: &mut EditorState, active: &mut ActiveMission, mission: Mission) {
    if let Some((t, i)) = trial::trial_of(mission.id) {
        tracing::debug!(
            trial = t.id,
            stage = mission.id,
            stage_no = i + 1,
            of = t.stages.len(),
            "trial stage opened in the editor"
        );
    }
    editor.source = mission.starter_code.to_string();
    editor.last_compile_result = None;
    editor.encounter_id = mission.id.to_string();
    editor.open = true;
    active.current = Some(mission);
}

fn dismiss_completion_on_escape(
    keys: Res<ButtonInput<KeyCode>>,
    mut completion: ResMut<CompletionView>,
) {
    if keys.just_pressed(KeyCode::Escape) && completion.mission_id.is_some() {
        tracing::debug!("completion view dismissed via Escape");
        completion.mission_id = None;
    }
}

fn dismiss_epilogue_on_escape(
    keys: Res<ButtonInput<KeyCode>>,
    mut epilogue: ResMut<EpilogueView>,
    progress: Res<MissionProgress>,
    registry: Res<MissionRegistry>,
    completion: Res<CompletionView>,
    editor: Res<EditorState>,
) {
    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }
    // Only dismiss when the epilogue is actually visible. Otherwise a
    // pre-victory Esc-press would silently set the flag and the player
    // would never see the epilogue when they clear the last mission.
    let all_cleared = progress.cleared_count() == registry.missions.len();
    let epilogue_visible = all_cleared
        && !epilogue.dismissed_this_session
        && completion.mission_id.is_none()
        && !editor.open;
    if epilogue_visible {
        epilogue.dismissed_this_session = true;
        tracing::info!("epilogue dismissed via Escape");
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_interaction_prompt(
    mut contexts: EguiContexts,
    nearby: Res<NearbyNpc>,
    editor: Res<EditorState>,
    active: Res<ActiveMission>,
    progress: Res<MissionProgress>,
    completion: Res<CompletionView>,
    registry: Res<MissionRegistry>,
    dialogue: Option<Res<crate::plugins::dialogue::Dialogue>>,
) {
    // Don't double up — the completion panel, editor and dialogue box
    // each own the screen real estate for their own flows.
    if editor.open || completion.mission_id.is_some() || dialogue.is_some_and(|d| d.is_open()) {
        return;
    }
    let Some(entry) = nearby.current.as_ref() else {
        return;
    };
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let mission = trial::current_mission(&registry, &progress, entry.mission_id);
    let current_id = mission.map_or(entry.mission_id, |m| m.id);
    let available = registry.is_available(current_id);
    let cleared = available && progress.is_cleared(current_id);
    let unlocked = mission.is_some_and(|m| m.is_unlocked(&progress));
    // A trial NPC shows where the trial stands (stage label + pips).
    let strip = trial::StageStrip::for_mission(current_id, &progress);
    let prereq_npc_name = mission
        .and_then(|m| m.prereq)
        .and_then(|pid| registry.find(pid))
        .map(|m| m.npc_name);

    let body = egui::FontId::proportional(ui_theme::BODY_SIZE);
    ui_theme::panel_area(
        "interact",
        egui::Align2::CENTER_BOTTOM,
        egui::Vec2::new(0.0, -24.0),
        egui::Order::Middle,
    )
    .show(ctx, |ui| {
        ui_theme::framed_panel(ui, |ui| {
            ui.set_max_width(PROMPT_MAX_WIDTH);
            ui.horizontal(|ui| {
                let (status, line) = if !available {
                    (
                        ui_theme::Status::Locked,
                        format!("{} waits beyond the demo (full game)", entry.name),
                    )
                } else if cleared {
                    (ui_theme::Status::Cleared, format!("talk to {}", entry.name))
                } else if !unlocked {
                    let hint = prereq_npc_name.unwrap_or("the previous mission");
                    (
                        ui_theme::Status::Locked,
                        format!("locked: clear {hint} first"),
                    )
                } else {
                    (ui_theme::Status::Next, format!("talk to {}", entry.name))
                };
                ui_theme::status_mark(ui, status, ui_theme::BODY_SIZE);
                if available && (cleared || unlocked) {
                    ui.label(
                        egui::RichText::new("[F]")
                            .font(body.clone())
                            .color(ui_theme::OXBLOOD)
                            .strong(),
                    );
                }
                ui.label(
                    egui::RichText::new(line)
                        .font(body.clone())
                        .color(ui_theme::TEXT),
                );
            });
            if let Some(strip) = strip.as_ref().filter(|_| available) {
                strip.show(ui, ui_theme::SMALL_SIZE);
            }
            if let Some(m) = active.current.as_ref().filter(|m| m.id == current_id) {
                ui_theme::markdown::show_inline(ui, m.prompt, ui_theme::BODY_SIZE);
            }
            ui_theme::caption(
                ui,
                format!("missions cleared: {}", progress.cleared_count()),
            );
        });
    });
}

/// Widest the NPC prompt panel grows before wrapping.
const PROMPT_MAX_WIDTH: f32 = 520.0;

/// When the "next mission" HUD pin shows: always while walking around in
/// game, hidden while anything that owns the screen is open (pause menu,
/// editor, dialogue, recap). A new player needs the pointer most when
/// they have wandered away from every NPC, which is exactly when the old
/// "only near an NPC" behaviour hid it.
pub fn next_pin_visible(
    paused: bool,
    editor_open: bool,
    dialogue_open: bool,
    recap_open: bool,
) -> bool {
    !(paused || editor_open || dialogue_open || recap_open)
}

/// Persistent "next mission" HUD pin, top-left. Visibility rule:
/// [`next_pin_visible`]. Disappears once every mission in the build is
/// cleared (the epilogue / end-of-demo panel takes over).
#[allow(clippy::too_many_arguments)]
fn draw_next_pin(
    mut contexts: EguiContexts,
    pause: Option<Res<State<PauseState>>>,
    editor: Res<EditorState>,
    dialogue: Option<Res<crate::plugins::dialogue::Dialogue>>,
    completion: Res<CompletionView>,
    progress: Res<MissionProgress>,
    registry: Res<MissionRegistry>,
) {
    let visible = next_pin_visible(
        pause.is_some_and(|p| *p.get() == PauseState::Paused),
        editor.open,
        dialogue.is_some_and(|d| d.is_open()),
        completion.mission_id.is_some(),
    );
    if !visible {
        return;
    }
    let Some(next) = registry.next_locked(&progress) else {
        return;
    };
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    ui_theme::panel_area(
        "next_mission",
        egui::Align2::LEFT_TOP,
        egui::Vec2::new(10.0, 10.0),
        egui::Order::Middle,
    )
    .show(ctx, |ui| {
        ui_theme::framed_panel(ui, |ui| {
            ui.horizontal(|ui| {
                ui_theme::status_mark(ui, ui_theme::Status::Next, ui_theme::BODY_SIZE);
                ui.label(
                    egui::RichText::new("Next:")
                        .size(ui_theme::BODY_SIZE)
                        .color(ui_theme::OXBLOOD)
                        .strong(),
                );
                ui.label(
                    egui::RichText::new(format!("find {}", next.npc_name))
                        .size(ui_theme::BODY_SIZE)
                        .color(ui_theme::TEXT),
                );
            });
            // The mission's goal line (its one-sentence prompt, with inline
            // code chips), never its snake_case id.
            ui.set_max_width(NEXT_PIN_MAX_WIDTH);
            ui_theme::markdown::show_inline(ui, next.prompt, ui_theme::SMALL_SIZE);
        });
    });
}

/// Widest the "next mission" pin grows before its goal line wraps.
const NEXT_PIN_MAX_WIDTH: f32 = 340.0;

/// A centred modal panel: framed, at least `min_width` wide.
fn modal_panel(ctx: &egui::Context, id: &str, min_width: f32, add: impl FnOnce(&mut egui::Ui)) {
    let max_width = (ctx.content_rect().width() - 48.0).max(min_width);
    ui_theme::panel_area(
        id,
        egui::Align2::CENTER_CENTER,
        egui::Vec2::ZERO,
        egui::Order::Foreground,
    )
    .show(ctx, |ui| {
        ui_theme::framed_panel(ui, |ui| {
            ui.set_min_width(min_width);
            ui.set_max_width(max_width.min(min_width.max(560.0)));
            add(ui);
        });
    });
}

/// Window height the recap keeps for its frame, heading, goal line and key
/// hints; the tutorial excerpt scrolls in what is left.
const RECAP_CHROME_HEIGHT: f32 = 260.0;

fn draw_completion_panel(
    mut contexts: EguiContexts,
    completion: Res<CompletionView>,
    registry: Res<MissionRegistry>,
    progress: Res<MissionProgress>,
) {
    let Some(mission_id) = completion.mission_id.as_deref() else {
        return;
    };
    let Some(mission) = registry.missions.iter().find(|m| m.id == mission_id) else {
        return;
    };
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    if let Some((t, index)) = trial::trial_of(mission.id) {
        draw_trial_panel(ctx, t, index, mission, &registry, &progress);
        return;
    }

    modal_panel(ctx, "mission complete", 520.0, |ui| {
        ui.horizontal(|ui| {
            ui_theme::status_mark(ui, ui_theme::Status::Cleared, ui_theme::HEADING_SIZE);
            ui_theme::heading(ui, mission.npc_name);
        });
        // The goal line below says what was cleared; no raw mission id.
        ui_theme::caption(ui, "encounter cleared");
        ui_theme::rule(ui);
        ui_theme::markdown::show_inline(ui, mission.prompt, ui_theme::BODY_SIZE);
        ui_theme::rule(ui);
        // Recap: the Concept section (what players most often re-read)
        // and the Syntax section (the code shape to remember). Task and
        // Hint come back when they reopen the editor. Scrolls if a long
        // tutorial would not fit the window.
        let max_h = (ui.ctx().content_rect().height() - RECAP_CHROME_HEIGHT).max(120.0);
        egui::ScrollArea::vertical()
            .id_salt("recap_scroll")
            .max_height(max_h)
            .show(ui, |ui| {
                ui_theme::markdown::show(ui, &recap_text(mission.tutorial));
            });
        ui_theme::rule(ui);
        ui_theme::key_hint(ui, &[("F", "revisit"), ("Esc", "close")]);
    });
}

/// The completion panel of a trial stage. Two shapes, both with the stage
/// label and pips:
///
/// - a stage cleared, trial not finished: "stage i cleared", the recap and
///   `[N] next stage` (the next stage opens straight in the editor) or
///   `[Esc]` to walk away and come back later;
/// - every stage cleared: the trial-complete panel, distinct from a normal
///   clear: the owner's closing line, every stage selectable with its
///   digit key so any stage can be re-read, `[F]` revisits the one on view.
fn draw_trial_panel(
    ctx: &egui::Context,
    t: &'static trial::Trial,
    index: usize,
    mission: &Mission,
    registry: &MissionRegistry,
    progress: &MissionProgress,
) {
    let complete = trial::is_complete(t, progress);
    let strip = trial::StageStrip {
        label: trial::stage_label(t, index),
        pips: trial::pips(t, progress, mission.id),
    };
    let next = t
        .stages
        .get(index + 1)
        .and_then(|id| registry.find(id))
        .filter(|m| m.is_unlocked(progress) && !progress.is_cleared(m.id));
    let id = if complete {
        "trial complete"
    } else {
        "trial stage cleared"
    };
    modal_panel(ctx, id, 560.0, |ui| {
        ui.horizontal(|ui| {
            ui_theme::status_mark(ui, ui_theme::Status::Cleared, ui_theme::HEADING_SIZE);
            if complete {
                ui_theme::heading(ui, format!("Trial complete: {}", t.name));
            } else {
                ui_theme::heading(ui, mission.npc_name);
            }
        });
        if complete {
            ui.label(
                egui::RichText::new(format!("{} - {}", mission.npc_name, t.complete_line))
                    .size(ui_theme::BODY_SIZE)
                    .color(ui_theme::TEXT)
                    .italics(),
            );
            ui_theme::caption(ui, format!("all {} stages cleared", t.stages.len()));
        } else {
            ui_theme::caption(ui, format!("stage {} cleared", index + 1));
        }
        ui_theme::rule(ui);
        strip.show(ui, ui_theme::BODY_SIZE);
        ui_theme::markdown::show_inline(ui, mission.prompt, ui_theme::BODY_SIZE);
        ui_theme::rule(ui);
        let max_h = (ui.ctx().content_rect().height() - RECAP_CHROME_HEIGHT - 40.0).max(120.0);
        egui::ScrollArea::vertical()
            .id_salt("trial_recap_scroll")
            .max_height(max_h)
            .show(ui, |ui| {
                ui_theme::markdown::show(ui, &recap_text(mission.tutorial));
            });
        ui_theme::rule(ui);
        let stage_keys = format!("1-{}", t.stages.len());
        let hints: Vec<(&str, &str)> = match (complete, next) {
            (true, _) => vec![
                (stage_keys.as_str(), "re-read a stage"),
                ("F", "revisit"),
                ("Esc", "close"),
            ],
            (false, Some(_)) => vec![("N", "next stage"), ("F", "revisit"), ("Esc", "walk away")],
            (false, None) => vec![("F", "revisit"), ("Esc", "close")],
        };
        ui_theme::key_hint(ui, &hints);
    });
}

/// Win-condition epilogue. Auto-shows when the player has cleared every
/// mission in the registry. Suppressed while the editor is open or a
/// per-mission completion panel is up so the two flows don't collide,
/// and one Esc dismisses it for the rest of the session. Voice is the
/// realm itself addressing the player — short, heraldic, no spoiler
/// for future acts since none exist yet.
fn draw_epilogue_panel(
    mut contexts: EguiContexts,
    progress: Res<MissionProgress>,
    registry: Res<MissionRegistry>,
    completion: Res<CompletionView>,
    editor: Res<EditorState>,
    epilogue: Res<EpilogueView>,
) {
    if editor.open
        || completion.mission_id.is_some()
        || epilogue.dismissed_this_session
        || progress.cleared_count() != registry.missions.len()
        // A demo build ends with its own panel, even on a full-game save.
        || registry.missions.iter().any(|m| !m.available)
    {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    modal_panel(ctx, "epilogue", 520.0, |ui| {
        ui_theme::heading(ui, "The realm pledges its crown.");
        ui_theme::rule(ui);
        ui.label(
            "You have walked every road of the village and answered every \
             sworn question. The compiler bows. The Borrow Checker stands \
             down. The banners come in.",
        );
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui_theme::status_mark(ui, ui_theme::Status::Cleared, ui_theme::BODY_SIZE);
            // Missions, not NPCs: a trial NPC owns several missions.
            ui.label(format!(
                "{} of {} missions cleared.",
                progress.cleared_count(),
                registry.missions.len()
            ));
        });
        ui.add_space(6.0);
        ui_theme::caption(
            ui,
            "More acts are being forged. For now: revisit any NPC with [F] \
             to re-read a lesson, or wander the village.",
        );
        ui_theme::rule(ui);
        ui_theme::key_hint(ui, &[("Esc", "close")]);
    });
}

/// What the full game adds after the demo, one line per act. Only acts
/// that are built and in the registry belong here (Acts 3-8); do not
/// list planned acts or features that do not exist yet.
#[cfg(feature = "demo")]
const FULL_GAME_ACTS: &[(&str, &str)] = &[
    (
        "Act 3 - The Guildhall Quarter",
        "methods, associated functions, if let, while let, tuple structs, enums that carry data",
    ),
    (
        "Act 4 - The Trait Mage's Tower",
        "traits, generics, trait objects, lifetimes, associated types",
    ),
    (
        "Act 5 - The Tavern of Tribulations",
        "matching on Result, custom error types, From with ?, map, and_then, unwrap_or_else",
    ),
    (
        "Act 6 - The Iterator Forge",
        "HashMap, filter, fold, enumerate, zip, move closures",
    ),
    (
        "Act 7 - The Concurrent Coast",
        "threads, Arc<Mutex<T>>, channels, atomics, scoped threads, async fn",
    ),
    (
        "Act 8 - The Vault of Pointers",
        "Box, Rc, RefCell, Cell, Rc<RefCell<T>>, Weak",
    ),
];

#[cfg(feature = "demo")]
fn reopen_demo_end_on_locked_attempt(
    mut attempts: MessageReader<MissionLockedAttempt>,
    registry: Res<MissionRegistry>,
    progress: Res<MissionProgress>,
    mut view: ResMut<DemoEndView>,
) {
    for attempt in attempts.read() {
        if !registry.is_available(&attempt.mission_id) && registry.demo_complete(&progress) {
            tracing::info!(
                mission = attempt.mission_id,
                "full-game mission attempted in demo; showing end-of-demo panel"
            );
            view.dismissed_this_session = false;
        }
    }
}

#[cfg(feature = "demo")]
fn dismiss_demo_end_on_escape(
    keys: Res<ButtonInput<KeyCode>>,
    registry: Res<MissionRegistry>,
    progress: Res<MissionProgress>,
    completion: Res<CompletionView>,
    editor: Res<EditorState>,
    mut view: ResMut<DemoEndView>,
) {
    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }
    // Same guard as the epilogue: only an Esc that actually closes the
    // visible panel sets the flag.
    let visible = registry.demo_complete(&progress)
        && !view.dismissed_this_session
        && completion.mission_id.is_none()
        && !editor.open;
    if visible {
        view.dismissed_this_session = true;
        tracing::info!("end-of-demo panel dismissed via Escape");
    }
}

/// End-of-demo panel. Same visual language as the completion and
/// epilogue panels. No price, store, link or email capture: those are
/// blocked on the trademark search and store setup (see
/// [`FULL_GAME_URL`]).
#[cfg(feature = "demo")]
fn draw_demo_end_panel(
    mut contexts: EguiContexts,
    progress: Res<MissionProgress>,
    registry: Res<MissionRegistry>,
    completion: Res<CompletionView>,
    editor: Res<EditorState>,
    dialogue: Option<Res<crate::plugins::dialogue::Dialogue>>,
    view: Res<DemoEndView>,
) {
    if editor.open
        || completion.mission_id.is_some()
        || view.dismissed_this_session
        || dialogue.is_some_and(|d| d.active.is_some())
        || !registry.demo_complete(&progress)
    {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    let free = registry.missions.iter().filter(|m| m.available).count();
    let more = registry.missions.len() - free;

    modal_panel(ctx, "demo end", 560.0, |ui| {
        ui_theme::heading(ui, "The demo ends here.");
        ui_theme::rule(ui);
        ui.label(format!(
            "You have cleared all {free} missions of Acts 1 and 2, the Borrow \
             Checker's trial among them. The road past the Bridge continues in \
             the full game."
        ));
        ui.add_space(6.0);
        ui.label(format!(
            "The full game has {more} more missions across Acts 3 to 8:"
        ));
        ui.add_space(2.0);
        for (act, topics) in FULL_GAME_ACTS {
            ui.label(
                egui::RichText::new(*act)
                    .size(ui_theme::BODY_SIZE)
                    .color(ui_theme::OXBLOOD)
                    .strong(),
            );
            ui_theme::caption(ui, *topics);
        }
        ui.add_space(6.0);
        ui_theme::caption(
            ui,
            "Revisit any NPC with [F] to re-read a lesson, or wander the village.",
        );
        ui_theme::rule(ui);
        ui_theme::key_hint(ui, &[("Esc", "close")]);
    });
}

/// Pull the body of a `## Heading` section out of a tutorial string —
/// inclusive of the heading line, exclusive of the next `## ` line. If
/// the section is missing, the full tutorial is returned (better to
/// over-show than to silently drop the recap).
/// The recap's tutorial excerpt: the Concept and Syntax sections, each
/// only if the tutorial has it (never the whole tutorial).
fn recap_text(tutorial: &str) -> String {
    ["## Concept", "## Syntax"]
        .iter()
        .filter(|h| tutorial.contains(**h))
        .map(|h| extract_section(tutorial, h))
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn extract_section(tutorial: &str, heading: &str) -> String {
    let Some(start) = tutorial.find(heading) else {
        return tutorial.to_string();
    };
    let after = &tutorial[start..];
    if let Some(rel_next) = after[heading.len()..].find("## ") {
        after[..heading.len() + rel_next].trim_end().to_string()
    } else {
        after.trim_end().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_pin_shows_while_walking_and_hides_under_screens() {
        assert!(next_pin_visible(false, false, false, false));
        assert!(!next_pin_visible(true, false, false, false));
        assert!(!next_pin_visible(false, true, false, false));
        assert!(!next_pin_visible(false, false, true, false));
        assert!(!next_pin_visible(false, false, false, true));
    }

    #[test]
    fn recap_is_concept_plus_syntax_for_every_mission() {
        let reg = MissionRegistry::default();
        for m in &reg.missions {
            let r = recap_text(m.tutorial);
            assert!(r.starts_with("## Concept"), "{}", m.id);
            assert!(r.contains("## Syntax"), "{}: no syntax section", m.id);
            assert!(!r.contains("## Task") && !r.contains("## Hint"), "{}", m.id);
        }
        assert_eq!(recap_text("no headings"), "");
    }

    #[test]
    fn extract_section_pulls_only_the_named_section() {
        let tut = "## Concept\nfirst body\n\n## Syntax\nsecond body\n\n## Task\nthird";
        let got = extract_section(tut, "## Concept");
        assert!(got.contains("first body"));
        assert!(!got.contains("second body"));
        assert!(!got.contains("third"));
    }

    #[test]
    fn extract_section_returns_tail_when_heading_is_last() {
        let tut = "## Concept\nfirst\n\n## Hint\nlast hint body";
        let got = extract_section(tut, "## Hint");
        assert!(got.contains("last hint body"));
        assert!(!got.contains("first"));
    }

    #[test]
    fn extract_section_falls_back_to_full_tutorial_when_heading_missing() {
        let tut = "no headings at all in this string";
        let got = extract_section(tut, "## Concept");
        assert_eq!(got, tut);
    }

    #[test]
    fn extract_section_works_against_real_mission_tutorials() {
        let reg = MissionRegistry::default();
        for m in &reg.missions {
            let concept = extract_section(m.tutorial, "## Concept");
            assert!(
                concept.contains("## Concept"),
                "mission {} concept extraction lost its heading",
                m.id
            );
            // Mustn't bleed into the next section.
            assert!(
                !concept.contains("## Syntax"),
                "mission {} concept section bled into Syntax",
                m.id
            );
        }
    }
}

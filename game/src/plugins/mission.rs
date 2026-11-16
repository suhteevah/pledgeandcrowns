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
`expect` or `panic!` anywhere in the program: the Barkeep checks. \
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
            // ── Act 6 boss: the Keymaster's trial, The Forge-Golem ─────
            // Three stages owned by the Keymaster (`trial::TRIALS`), after
            // his prologue `hashmap_basic`: filter before map, tell `sum`
            // its type, then one chain (filter, map, fold) with no loop
            // and no `mut`. Every compiler error he names is compiled in
            // `tests/mission_text.rs` (hard rule 3).
            Mission {
                id: "golem_parts",
                npc_name: "The Keymaster",
                prompt: "Filter out the armoured parts before you map them to labels.",
                tutorial: "## Concept\n\
The Forge-Golem is built of parts, and only the unarmoured ones can be \
hurt. An iterator chain sends each item through its steps in order: \
`.filter(..)` keeps or drops an item, `.map(..)` turns it into something \
else. So order matters. Once `.map` has turned a `Part` into a `String` \
label, the steps after it see only the label, and \
`.filter(|p| !p.armoured)` stops with E0609, \"no field `armoured` on \
type `&String`\".\n\n\
## Syntax\n\
```\nlet ores = vec![(\"iron\", true), (\"slag\", false)];\nlet good: Vec<String> = ores\n    .iter()\n    .filter(|o| o.1)\n    .map(|o| o.0.to_uppercase())\n    .collect();\n```\n\
`.filter` sees the whole item; `.map` decides what comes out.\n\n\
## Task\n\
Swap the two steps in `weak_points` so the chain filters first and maps \
after. Your program should print \
`weak points: [\"arm (3)\", \"knee (4)\", \"eye (6)\"]`.\n\n\
## Hint\n\
The grader needs `.filter(..)` followed by `.map(..)`. Dropping the \
chest and the head by name would print the same line here, but the \
checker sends `weak_points` parts it has never seen.",
                prereq: None,
                available: true,
                starter_code: "struct Part {\n    name: &'static str,\n    armoured: bool,\n    weakness: u32,\n}\n\nfn golem() -> Vec<Part> {\n    vec![\n        Part { name: \"arm\", armoured: false, weakness: 3 },\n        Part { name: \"chest\", armoured: true, weakness: 8 },\n        Part { name: \"knee\", armoured: false, weakness: 4 },\n        Part { name: \"head\", armoured: true, weakness: 5 },\n        Part { name: \"eye\", armoured: false, weakness: 6 },\n    ]\n}\n\n// The parts a blow can reach, with how weak each one is.\nfn weak_points(parts: &[Part]) -> Vec<String> {\n    // this won't compile yet: after `map`, only the labels are left\n    parts\n        .iter()\n        .map(|p| format!(\"{} ({})\", p.name, p.weakness))\n        .filter(|p| !p.armoured)\n        .collect()\n}\n\nfn main() {\n    let parts = golem();\n    println!(\"weak points: {:?}\", weak_points(&parts));\n}\n",
            },
            Mission {
                id: "golem_sum",
                npc_name: "The Keymaster",
                prompt: "Tell `sum` what kind of number to make, then double the total.",
                tutorial: "## Concept\n\
The Golem's core takes the weakness of every weak point, added up and \
doubled. `.sum()` adds up what an iterator yields, and it can make many \
kinds of number: `u32`, `u64`, `f64` and more. It learns which one from \
where the result goes. In `let total = ...sum();` nothing names a type, \
and `total * 2` does not settle it either, so the compiler stops with \
E0283, \"type annotations needed\". Name the type and the chain \
compiles. A `.fold(0, |acc, w| acc + w)` in place of `.sum()` also \
compiles.\n\n\
## Syntax\n\
```\nlet weights = vec![2u32, 5, 9];\nlet heavy: u32 = weights.iter().filter(|w| **w > 3).sum();\nlet light = weights.iter().filter(|w| **w <= 3).sum::<u32>();\n```\n\
A type on the variable, or a turbofish `::<u32>` on `sum`.\n\n\
## Task\n\
The starter fails with E0283. Give `total` a type so `core_damage` \
returns the weak points' total, doubled. Your program should print \
`golem: arm chest knee head eye`, then `core damage: 26`.\n\n\
## Hint\n\
The grader needs `let total: u32 = ...` (or `.sum::<u32>()`, or a \
`.fold(`). rustc's help writes `let total: u32 = parts`. Leave the \
filter in: summing every part, armour and all, prints \
`core damage: 52`.",
                prereq: None,
                available: true,
                starter_code: "struct Part {\n    name: &'static str,\n    armoured: bool,\n    weakness: u32,\n}\n\nfn golem() -> Vec<Part> {\n    vec![\n        Part { name: \"arm\", armoured: false, weakness: 3 },\n        Part { name: \"chest\", armoured: true, weakness: 8 },\n        Part { name: \"knee\", armoured: false, weakness: 4 },\n        Part { name: \"head\", armoured: true, weakness: 5 },\n        Part { name: \"eye\", armoured: false, weakness: 6 },\n    ]\n}\n\n// The core takes every weak point's weakness, added up and doubled.\nfn core_damage(parts: &[Part]) -> u32 {\n    // this won't compile yet: `sum` can make many kinds of number\n    let total = parts\n        .iter()\n        .filter(|p| !p.armoured)\n        .map(|p| p.weakness)\n        .sum();\n    total * 2\n}\n\nfn main() {\n    let parts = golem();\n    let names: Vec<&str> = parts.iter().map(|p| p.name).collect();\n    println!(\"golem: {}\", names.join(\" \"));\n    println!(\"core damage: {}\", core_damage(&parts));\n}\n",
            },
            Mission {
                id: "golem_core",
                npc_name: "The Keymaster",
                prompt: "One chain: filter, map, fold. No loop and no `mut`.",
                tutorial: "## Concept\n\
The final blow. Every unarmoured part takes the blow times its weakness, \
and the Golem falls to the total. The starter's loop `break`s at the \
first armoured part, so the chest stops the count before the knee and \
the eye are reached. In a chain, `.filter` skips an item and goes on to \
the next; `.map` turns each weak part into its hit; \
`.fold(0, |total, hit| total + hit)` gathers the hits into one number. \
No loop and no mutable total: each step hands its result to the next.\n\n\
## Syntax\n\
```\nlet ores = vec![4u32, 0, 7, 0, 2];\nlet worth = ores\n    .iter()\n    .filter(|o| **o > 0)\n    .map(|o| o * 10)\n    .fold(0, |sum, v| sum + v);\n```\n\
`.sum()` is a ready-made fold for adding.\n\n\
## Task\n\
Rewrite `total_damage` as one chain over `parts`. Use no `for`, \
`while`, `loop` or `mut` anywhere in the program: the Keymaster checks. \
Your program should print `golem: arm chest knee head eye`, \
`blow 1: 13`, then `blow 3: 39`.\n\n\
## Hint\n\
The grader needs `.filter(` and a `.fold(` (or `.sum`). As written, the \
starter prints `blow 1: 3` and `blow 3: 9`: only the arm is counted. \
`.take_while(..)` in place of `.filter(..)` makes the same mistake, \
because it stops at the first armoured part.",
                prereq: None,
                available: true,
                starter_code: "struct Part {\n    name: &'static str,\n    armoured: bool,\n    weakness: u32,\n}\n\nfn golem() -> Vec<Part> {\n    vec![\n        Part { name: \"arm\", armoured: false, weakness: 3 },\n        Part { name: \"chest\", armoured: true, weakness: 8 },\n        Part { name: \"knee\", armoured: false, weakness: 4 },\n        Part { name: \"head\", armoured: true, weakness: 5 },\n        Part { name: \"eye\", armoured: false, weakness: 6 },\n    ]\n}\n\n// Every weak point takes the blow, multiplied by its weakness.\nfn total_damage(parts: &[Part], blow: u32) -> u32 {\n    // this runs, but the first armoured part ends the count\n    let mut total = 0;\n    for p in parts {\n        if p.armoured {\n            break;\n        }\n        total += blow * p.weakness;\n    }\n    total\n}\n\nfn main() {\n    let parts = golem();\n    let names: Vec<&str> = parts.iter().map(|p| p.name).collect();\n    println!(\"golem: {}\", names.join(\" \"));\n    println!(\"blow 1: {}\", total_damage(&parts, 1));\n    println!(\"blow 3: {}\", total_damage(&parts, 3));\n}\n",
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
            // ── Act 7 boss: the Dockmaster's trial, The Tide Hydra ─────
            // Three stages owned by the Dockmaster (`trial::TRIALS`), after
            // his prologue `thread_spawn`: one sender clone per head (one
            // thread), one `move` thread per head (sorted reports), then
            // wait for every head's `Down` before striking. The server has
            // no threads: stage 2 is compile-only there, stage 3 checks
            // `await_heads` with checker-sent reports and does not run
            // `main`. Every compiler error he names is compiled in
            // `tests/mission_text.rs` (hard rule 3).
            Mission {
                id: "hydra_senders",
                npc_name: "The Dockmaster",
                prompt: "Give each head its own clone of the sender.",
                tutorial: "## Concept\n\
The Tide Hydra's heads report on one channel. `fight` takes its \
`Sender` by value, so the loop's first call moves the only sender away \
and the next call has none: E0382, \"use of moved value: `tx`\", with \
\"value moved here, in previous iteration of loop\". `tx.clone()` makes \
another sender into the same channel, so every head can hold its own. \
While any sender is alive, the receiver waits for more: \
`rx.recv_timeout(..)` on an empty channel gives `Err(Timeout)`. Once \
the last sender is dropped it gives `Err(Disconnected)`, and \
`rx.iter()` ends. That is why `hunt` drops `tx` before it collects.\n\n\
## Syntax\n\
```\nlet (tx, rx) = mpsc::channel();\nlet tx2 = tx.clone();\ntx.send(1).unwrap();\ntx2.send(2).unwrap();\ndrop(tx);\ndrop(tx2);\nlet all: Vec<i32> = rx.iter().collect();\n```\n\
Both senders feed the one receiver.\n\n\
## Task\n\
Pass each head a clone of the sender, `tx.clone()`. Your program should \
print `fang down after 3 blows`, `venom down after 2 blows`, \
`brine down after 4 blows`, then `3 heads down`.\n\n\
## Hint\n\
The grader needs a `.clone()` of the sender; rustc's help suggests \
`fight(head, hp, tx.clone());`. Lending `&tx` to a `fight` that takes \
`&Sender<String>` also compiles here, but in the next stage each head \
fights on its own thread and needs a sender of its own.",
                prereq: None,
                available: true,
                starter_code: "use std::sync::mpsc::{self, Sender};\n\n// A head fights until it falls, then reports on the sender it holds.\nfn fight(head: &'static str, hp: u32, tx: Sender<String>) {\n    let blows = hp.div_ceil(4);\n    let report = format!(\"{head} down after {blows} blows\");\n    tx.send(report).expect(\"receiver dropped\");\n}\n\nfn hunt(heads: &[(&'static str, u32)]) -> Vec<String> {\n    let (tx, rx) = mpsc::channel();\n    for &(head, hp) in heads {\n        // this won't compile yet: the first head takes the only sender\n        fight(head, hp, tx);\n    }\n    // No sender left: the receiver knows no more reports are coming.\n    drop(tx);\n    rx.iter().collect()\n}\n\nfn main() {\n    let reports = hunt(&[(\"fang\", 9), (\"venom\", 6), (\"brine\", 13)]);\n    for report in &reports {\n        println!(\"{report}\");\n    }\n    println!(\"{} heads down\", reports.len());\n}\n",
            },
            Mission {
                id: "hydra_threads",
                npc_name: "The Dockmaster",
                prompt: "Spawn one thread per head, and `move` what it needs into it.",
                tutorial: "## Concept\n\
Now each head is fought on its own thread. A thread started with \
`thread::spawn` can outlive the function that started it, so its \
closure must own everything it uses. Without `move`, the closure only \
borrows `hp`, and the compiler stops with E0373, \"closure may outlive \
the current function, but it borrows `hp`, which is owned by the current \
function\". With `move`, the closure takes `head`, `hp` and its own \
sender along. Nothing in Rust promises which thread reports first, so \
`hunt` sorts the reports before it returns them.\n\n\
## Syntax\n\
```\nuse std::thread;\nlet crew = vec![\"rope\", \"sail\"];\nlet handle = thread::spawn(move || crew.len());\nprintln!(\"{}\", handle.join().unwrap());\n```\n\
`move` hands `crew` to the thread for good.\n\n\
## Task\n\
Add `move` to the closure passed to `thread::spawn`. Your program should \
print `brine down after 4 blows`, `fang down after 3 blows`, \
`venom down after 2 blows`, then `3 heads down`. (This server has no \
threads: it builds your code and looks for `spawn(`, `move` and a \
`.sort`, but it cannot run it.)\n\n\
## Hint\n\
The grader needs `spawn(move ||` and the `.sort`. rustc's help says the \
same: use the `move` keyword. Without the sort, nothing fixes the order \
of the lines.",
                prereq: None,
                available: true,
                starter_code: "use std::sync::mpsc::{self, Sender};\nuse std::thread;\n\n// A head fights until it falls, then reports on the sender it holds.\nfn fight(head: &'static str, hp: u32, tx: Sender<String>) {\n    let blows = hp.div_ceil(4);\n    let report = format!(\"{head} down after {blows} blows\");\n    tx.send(report).expect(\"receiver dropped\");\n}\n\nfn hunt(heads: &[(&'static str, u32)]) -> Vec<String> {\n    let (tx, rx) = mpsc::channel();\n    for &(head, hp) in heads {\n        let tx = tx.clone();\n        // this won't compile yet: the thread borrows what the loop owns\n        thread::spawn(|| fight(head, hp, tx));\n    }\n    drop(tx);\n    // Threads finish in no promised order: sort what comes back.\n    let mut reports: Vec<String> = rx.iter().collect();\n    reports.sort();\n    reports\n}\n\nfn main() {\n    let reports = hunt(&[(\"fang\", 9), (\"venom\", 6), (\"brine\", 13)]);\n    for report in &reports {\n        println!(\"{report}\");\n    }\n    println!(\"{} heads down\", reports.len());\n}\n",
            },
            Mission {
                id: "hydra_body",
                npc_name: "The Dockmaster",
                prompt: "Wait until every head reports `Down`, then strike the body.",
                tutorial: "## Concept\n\
Each head's thread sends a `Report::Hit` for every blow it takes, then a \
`Report::Down` when it falls. The body may be struck only once every \
head is down. The starter's `await_heads` leaves its loop at the first \
`Down`, so it can strike while other heads still fight. Count the \
`Down` reports and return once the count reaches `heads`. A channel \
keeps each sender's messages in the order they were sent, so a head's \
hits arrive before its `Down`: once every head is down, every hit has \
been counted.\n\n\
## Syntax\n\
```\nlet mut done = 0;\nwhile done < 2 {\n    match rx.recv() {\n        Ok(Signal::Done) => done += 1,\n        Ok(Signal::Work) => {}\n        Err(_) => break,\n    }\n}\n```\n\
`rx.recv()` hands over the next message, or an `Err`.\n\n\
## Task\n\
Fix `await_heads`: count the `Down` reports and return the hits once \
`heads` heads are down. Your program should print \
`3 heads down after 9 hits`, then `strike the body`. (This server has \
no threads, so it checks `await_heads` alone: the checker sends the \
reports itself, on one thread, and does not run your `main`.)\n\n\
## Hint\n\
The grader needs `await_heads` to compare its count with `heads`, for \
example `while down < heads`. The checker rejects a `Down` counted as a \
hit, and a count that stops one head early.",
                prereq: None,
                available: true,
                starter_code: "use std::sync::mpsc::{self, Receiver};\nuse std::thread;\n\nenum Report {\n    Hit,\n    Down,\n}\n\n// Count the hits, and return only once every head has reported down.\nfn await_heads(rx: &Receiver<Report>, heads: usize) -> u32 {\n    // this runs, but it strikes as soon as the first head falls\n    let _ = heads;\n    let mut hits = 0;\n    for report in rx.iter() {\n        match report {\n            Report::Hit => hits += 1,\n            Report::Down => break,\n        }\n    }\n    hits\n}\n\nfn main() {\n    // The blows each head takes: fang, venom, brine.\n    let blows = [3, 2, 4];\n    let (tx, rx) = mpsc::channel();\n    for n in blows {\n        let tx = tx.clone();\n        thread::spawn(move || {\n            for _ in 0..n {\n                tx.send(Report::Hit).expect(\"receiver dropped\");\n            }\n            tx.send(Report::Down).expect(\"receiver dropped\");\n        });\n    }\n    drop(tx);\n    let hits = await_heads(&rx, blows.len());\n    println!(\"{} heads down after {hits} hits\", blows.len());\n    println!(\"strike the body\");\n}\n",
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
            // ── Act 8 boss: the Vaultwright's trial, The Memory Warden ─
            // Three stages owned by the Vaultwright (`trial::TRIALS`),
            // after his prologue `box_basic`, ending the registry: `Box`
            // for a recursive enum, `Rc<RefCell<_>>` for a shared vault,
            // then a ring of debts linked with `Weak` and walked without
            // leaking. Every compiler error he names is compiled in
            // `tests/mission_text.rs` (hard rule 3).
            Mission {
                id: "warden_box",
                npc_name: "The Vaultwright",
                prompt: "Put the rest of the ledger in a `Box`, so `Ledger` has a size.",
                tutorial: "## Concept\n\
The Memory Warden keeps his ledger as a chain: each entry holds an \
amount and the rest of the ledger. Written as `Entry(u32, Ledger)`, a \
`Ledger` holds a whole `Ledger`, which holds another, without end, so \
the compiler cannot give it a size: E0072, \"recursive type `Ledger` has \
infinite size\". A `Box<Ledger>` is a pointer of fixed size to a \
`Ledger` on the heap, so `Entry(u32, Box<Ledger>)` has a size. rustc's \
help says so too: \"insert some indirection (e.g., a `Box`, `Rc`, or \
`&`) to break the cycle\".\n\n\
## Syntax\n\
```\nenum Chain {\n    Link(u32, Box<Chain>),\n    End,\n}\nlet c = Chain::Link(1, Box::new(Chain::End));\n```\n\
`{:?}` prints through a `Box` to the value inside.\n\n\
## Task\n\
Change `Entry` to hold a `Box<Ledger>`, and wrap `rest` in `Box::new` \
in `entry`. `total` works unchanged. Your program should print \
`Entry(5, Entry(7, Entry(3, Closed)))`, then `owed: 15`.\n\n\
## Hint\n\
The grader needs `Box<Ledger>` and `Box::new(`. `total(rest)` still \
compiles with the `Box`: a `&Box<Ledger>` is accepted where a \
`&Ledger` is expected.",
                prereq: None,
                available: true,
                starter_code: "// A ledger is either closed, or one entry and the rest of the ledger.\n#[derive(Debug)]\nenum Ledger {\n    // this won't compile yet: a Ledger can't hold a whole Ledger inline\n    Entry(u32, Ledger),\n    Closed,\n}\n\nfn entry(amount: u32, rest: Ledger) -> Ledger {\n    Ledger::Entry(amount, rest)\n}\n\nfn total(ledger: &Ledger) -> u32 {\n    match ledger {\n        Ledger::Entry(amount, rest) => amount + total(rest),\n        Ledger::Closed => 0,\n    }\n}\n\nfn main() {\n    let ledger = entry(5, entry(7, entry(3, Ledger::Closed)));\n    println!(\"{ledger:?}\");\n    println!(\"owed: {}\", total(&ledger));\n}\n",
            },
            Mission {
                id: "warden_shared",
                npc_name: "The Vaultwright",
                prompt: "Share the vault as an `Rc<RefCell<u32>>`, so any debtor can pay into it.",
                tutorial: "## Concept\n\
Two debtors pay into one vault, and each holds a handle to it. `Rc` \
gives that shared ownership, but only to read: \
`*debtor.vault += coins` stops with E0594, \"cannot assign to data in an \
`Rc`\". A `RefCell` inside the `Rc` lends its value out while the \
program runs: `.borrow_mut()` gives one writer at a time, through any \
handle. `Rc::strong_count` counts the handles: the vault's own and one \
per debtor.\n\n\
## Syntax\n\
```\nuse std::cell::RefCell;\nuse std::rc::Rc;\nlet till = Rc::new(RefCell::new(0));\nlet clerk = Rc::clone(&till);\n*clerk.borrow_mut() += 4;\nprintln!(\"{}\", till.borrow());\n```\n\
`till` and `clerk` open the same `RefCell`.\n\n\
## Task\n\
Make `vault` an `Rc<RefCell<u32>>` in `Debtor` and in `main`, and add \
the coins with `.borrow_mut()` in `pay`. Your program should print \
`miller and smith paid`, `vault holds 12`, then `handles: 3`.\n\n\
## Hint\n\
The grader needs `Rc<RefCell<u32>>` and `.borrow_mut()`. The borrow \
rules still hold, only at run time: keep a `vault.borrow()` alive \
across a `pay` and the program panics with RefCell already borrowed.",
                prereq: None,
                available: true,
                starter_code: "use std::rc::Rc;\n\n// Every debtor holds a handle to the one vault.\nstruct Debtor {\n    name: &'static str,\n    vault: Rc<u32>,\n}\n\nfn pay(debtor: &Debtor, coins: u32) {\n    // this won't compile yet: an Rc shares its value, but only to read\n    *debtor.vault += coins;\n}\n\nfn main() {\n    let vault = Rc::new(0);\n    let miller = Debtor { name: \"miller\", vault: Rc::clone(&vault) };\n    let smith = Debtor { name: \"smith\", vault: Rc::clone(&vault) };\n    pay(&miller, 5);\n    pay(&smith, 7);\n    println!(\"{} and {} paid\", miller.name, smith.name);\n    println!(\"vault holds {vault}\");\n    println!(\"handles: {}\", Rc::strong_count(&vault));\n}\n",
            },
            Mission {
                id: "warden_weak",
                npc_name: "The Vaultwright",
                prompt: "Link the ring of debts with `Weak`, and walk it once without a leak.",
                tutorial: "## Concept\n\
The Warden owes you, you owe the teller, and the teller owes the \
Warden. With `Rc` links each debtor keeps the next alive, so the ring \
keeps itself alive: in the starter `Rc::strong_count(&warden)` is 2, \
and after `main` drops all three handles the probe still finds the \
Warden. That is a leak. A `Weak` link, made with `Rc::downgrade`, does \
not count as an owner. `.upgrade()` gives `Some` while the debtor is \
alive and `None` once it is gone. `circle` follows the links with \
`.upgrade()` and stops when it is back at `start` (`Rc::ptr_eq`) or \
finds no one.\n\n\
## Syntax\n\
```\nuse std::cell::RefCell;\nuse std::rc::{Rc, Weak};\nstruct Node {\n    next: RefCell<Weak<Node>>,\n}\nlet a = Rc::new(Node { next: RefCell::new(Weak::new()) });\nlet b = Rc::new(Node { next: RefCell::new(Rc::downgrade(&a)) });\nlet back = b.next.borrow().upgrade();\n```\n\
`back` is an `Option<Rc<Node>>`.\n\n\
## Task\n\
Change `owes` to `RefCell<Weak<Debtor>>`: start it as `Weak::new()`, \
link with `Rc::downgrade`, and follow a link in `circle` with \
`.upgrade()`. Your program should print \
`circle: [\"warden\", \"you\", \"teller\"]`, \
`warden: strong 1, weak 1`, then `after the ledger closes: None`.\n\n\
## Hint\n\
The grader needs `Weak<Debtor>`, `downgrade(` and `.upgrade()`. As \
written, the starter prints `warden: strong 2, weak 0` and \
`after the ledger closes: Some(\"warden\")`: the ring outlives every \
handle in `main`. Keep the stop at `start`: a ring has no end of its \
own.",
                prereq: None,
                available: true,
                starter_code: "use std::cell::RefCell;\nuse std::rc::Rc;\n\nstruct Debtor {\n    name: &'static str,\n    // this runs, but a strong link keeps its creditor alive\n    owes: RefCell<Option<Rc<Debtor>>>,\n}\n\nfn debtor(name: &'static str) -> Rc<Debtor> {\n    let owes = RefCell::new(None);\n    Rc::new(Debtor { name, owes })\n}\n\n// Follow the debts from `start` until they come back round, or end.\nfn circle(start: &Rc<Debtor>) -> Vec<&'static str> {\n    let mut names = vec![start.name];\n    let mut at = Rc::clone(start);\n    loop {\n        let next = at.owes.borrow().clone();\n        match next {\n            Some(d) if !Rc::ptr_eq(&d, start) => {\n                names.push(d.name);\n                at = d;\n            }\n            _ => return names,\n        }\n    }\n}\n\nfn main() {\n    let warden = debtor(\"warden\");\n    let you = debtor(\"you\");\n    let teller = debtor(\"teller\");\n    // The Warden owes you, you owe the teller, the teller owes him.\n    *warden.owes.borrow_mut() = Some(Rc::clone(&you));\n    *you.owes.borrow_mut() = Some(Rc::clone(&teller));\n    *teller.owes.borrow_mut() = Some(Rc::clone(&warden));\n    println!(\"circle: {:?}\", circle(&warden));\n    let strong = Rc::strong_count(&warden);\n    let weak = Rc::weak_count(&warden);\n    println!(\"warden: strong {strong}, weak {weak}\");\n    let probe = Rc::downgrade(&warden);\n    drop((warden, you, teller));\n    let left = probe.upgrade().map(|d| d.name);\n    println!(\"after the ledger closes: {left:?}\");\n}\n",
            },
            // ── Act 9: The Forbidden Library ──────────────────────────
            // `unsafe`, FFI, `macro_rules!`, `Drop`, `Send`/`Sync` and
            // `MaybeUninit`, then the Librarian's trial. Design:
            // docs/superpowers/specs/2026-11-14-act9-forbidden-library.md.
            // Hard rule 1: `unsafe` appears here only inside string
            // literals (player-facing text). The player's program runs in
            // the wasm sandbox, never in the game. Every compiler error and
            // runtime behaviour this act names is compiled or run in
            // `tests/mission_text.rs` (hard rule 3).
            Mission {
                id: "raw_pointer",
                npc_name: "The Shelfmark",
                prompt: "Make a raw pointer from a reference and dereference it inside `unsafe`.",
                tutorial: "## Concept\n\
A raw pointer (`*const T`, `*mut T`) is an address with no promises: it \
may be null, dangling, unaligned, or alias a `&mut`. Making one is safe. \
Reading or writing through one is not, so it goes inside an `unsafe` \
block. `unsafe` turns nothing off: types and the borrow checker still \
apply inside it (two `&mut` to one value is still E0499). It unlocks \
five extra operations (dereference a raw pointer, call an `unsafe` fn, \
access a `static mut`, implement an `unsafe` trait, read a union field) \
and makes you responsible for the rules the compiler can no longer \
check. Break them and the program has undefined behaviour: no defined \
meaning at all, not merely a wrong answer.\n\n\
## Syntax\n\
```\nlet p: *mut i32 = &mut x;   // safe: just an address\nunsafe { *p += 1; }          // you promise p is valid\nlet r = &raw const x;        // a *const i32, no reference made\n```\n\n\
## Task\n\
Add 5 to `gold` through `write_ptr`, then read it back through a \
`*const i32` and print three lines: `pointer is null: false`, \
`gold through the pointer: 15`, then `gold by name: 15`.\n\n\
## Hint\n\
E0133, \"dereference of raw pointer is unsafe and requires unsafe \
block\": the write must sit inside `unsafe { ... }`. Make \
`let read_ptr: *const i32 = &gold;` after the write, and put a \
`// SAFETY:` comment above each `unsafe` block saying why the pointer \
is valid.",
                prereq: None,
                available: true,
                starter_code: "fn main() {\n    let mut gold: i32 = 10;\n    let write_ptr: *mut i32 = &mut gold;\n    // Add 5 to gold through write_ptr, then read gold back through a\n    // read-only raw pointer and print both lines.\n    *write_ptr += 5;\n    println!(\"gold by name: {gold}\");\n}\n",
            },
            Mission {
                id: "extern_c",
                npc_name: "The Envoy",
                prompt: "Export a function with the C ABI and call it back through an `unsafe extern \"C\"` declaration.",
                tutorial: "## Concept\n\
FFI is how Rust calls, and is called by, code in other languages. \
`extern \"C\"` picks the C calling convention: where arguments and the \
return value travel. `#[unsafe(no_mangle)]` exports a function under \
its plain name so a C linker can find it. `#[repr(C)]` lays a struct \
out by C's rules (declared field order, C padding); Rust's default \
layout is unspecified. An `unsafe extern \"C\" { ... }` block declares a \
foreign function. The compiler cannot check that declaration against \
the real code, so each call is unsafe. Real C needs a C compiler, a \
`build.rs` and `#[link]` (`bindgen` writes the declarations for you); \
this sandbox has none of them, so you play both sides in one program: \
export `crown_seal` as a C library would, then call it back through a \
foreign declaration.\n\n\
## Syntax\n\
```\n#[repr(C)]\nstruct Rune { glyph: u8, power: u32, tier: u8 }\n#[unsafe(no_mangle)]\npub extern \"C\" fn crown_seal(r: Rune) -> u32 { /* ... */ }\nunsafe extern \"C\" { fn crown_seal(r: Rune) -> u32; }\nlet v = unsafe { crown_seal(r) };\n```\n\
Edition 2024 wants both `unsafe`s: a plain `extern \"C\" {` block is \
\"extern blocks must be unsafe\", and a plain `#[no_mangle]` is \
\"unsafe attribute used without unsafe\".\n\n\
## Task\n\
Give `Rune` a C layout and `#[derive(Clone, Copy)]`, export \
`crown_seal` with the C ABI under its plain name, and call it through \
`foreign::crown_seal` inside `unsafe`. Print the size line, then the \
seal three ways: directly, through the extern block, and through an \
`extern \"C\" fn(Rune) -> u32` pointer: `Rune: 12 bytes, align 4`, \
`direct call: 44`, `through the extern block: 44`, then \
`through a fn pointer: 44`.\n\n\
## Hint\n\
E0133: anything declared in an extern block is unsafe to call, \
because only you know the declaration is true. With `repr(C)`, \
`u8, u32, u8` takes 12 bytes: 1, 3 of padding, 4, 1, 3 of tail \
padding, aligned to 4. Without it Rust may reorder the fields, so the \
size line can come out different.",
                prereq: None,
                available: true,
                starter_code: "pub struct Rune {\n    pub glyph: u8,\n    pub power: u32,\n    pub tier: u8,\n}\n\n// Give Rune a C layout, and export this with the C calling convention\n// under its plain name so the foreign declaration below can find it.\npub fn crown_seal(rune: Rune) -> u32 {\n    rune.power * 2 + rune.tier as u32 + rune.glyph as u32\n}\n\nmod foreign {\n    unsafe extern \"C\" {\n        pub fn crown_seal(rune: super::Rune) -> u32;\n    }\n}\n\nfn main() {\n    let rune = Rune { glyph: 3, power: 20, tier: 1 };\n    let sealed = foreign::crown_seal(rune);\n    println!(\"through the extern block: {sealed}\");\n}\n",
            },
            Mission {
                id: "macro_rules",
                npc_name: "The Macro Monk",
                prompt: "Define a `macro_rules!` macro that takes one or more expressions.",
                tutorial: "## Concept\n\
`macro_rules!` defines a macro by example: each rule is a pattern over \
tokens and a template pasted in its place. Macros expand before type \
checking, so unlike functions they can take any number of arguments. \
`$x:expr` captures an expression; `$( ... ),+` repeats a sub-pattern \
one or more times, separated by commas (`*` for zero or more). Pasting \
is literal: write `$x` twice and the expression runs twice, so bind it \
to a local once. Locals made inside the template cannot clash with the \
caller's names (macro hygiene). A macro must be defined above its first \
use in the file. Procedural macros, such as `#[derive]`, are Rust \
programs that run at compile time; they need a crate of their own, so \
they cannot be written here.\n\n\
## Syntax\n\
```\nmacro_rules! shout {\n    ($($x:expr),+ $(,)?) => {{\n        $( println!(\"{}!\", $x); )+\n    }};\n}\n```\n\n\
## Task\n\
Define `chant!` above `main` so the three calls compile. Each verse \
prints as an indented line, and the call evaluates to the sum. The \
program prints `  verse: 1`, `first chant: 1`, `  verse: 2`, \
`  verse: 3`, `  verse: 4`, `second chant: 9`, `  verse: 10`, \
`  verse: 20`, then `third chant: 30`.\n\n\
## Hint\n\
\"cannot find macro `chant` in this scope\": define it with \
`macro_rules!` above `main`. Match the list with \
`($($verse:expr),+ $(,)?)`: the `$(,)?` accepts the trailing comma in \
`chant!(10, 20,)`. Wrap the template in `{{ ... }}` so the expansion is \
one block that ends with `total`.",
                prereq: None,
                available: true,
                starter_code: "// Define a declarative macro named `chant` above main. It should accept\n// one or more comma-separated expressions, print each as \"  verse: N\",\n// and evaluate to their sum.\n\nfn main() {\n    let first = chant!(1);\n    println!(\"first chant: {first}\");\n    let second = chant!(2, 3, 4);\n    println!(\"second chant: {second}\");\n    let third = chant!(10, 20,);\n    println!(\"third chant: {third}\");\n}\n",
            },
            Mission {
                id: "drop_trait",
                npc_name: "The Sexton",
                prompt: "Implement `Drop`, drop one value early, and keep another from ever dropping.",
                tutorial: "## Concept\n\
`Drop::drop` runs automatically when a value's owner goes out of scope. \
Locals drop in reverse order of declaration. You may not call \
`x.drop()` yourself: E0040, \"explicit use of destructor method\". To end \
a value early, move it into `drop(x)`: `std::mem::drop` is an ordinary \
function that takes ownership. `let _ = Soul { .. };` binds nothing, so \
that new value is dropped at the end of the statement. Drop is not \
guaranteed to run: `std::mem::forget` skips it (this is safe; a leak is \
not undefined behaviour), and `std::process::exit` ends the program \
without running any destructor.\n\n\
## Syntax\n\
```\nimpl Drop for Soul {\n    fn drop(&mut self) {\n        println!(\"{} is released\", self.name);\n    }\n}\ndrop(early);            // ends `early` now\nstd::mem::forget(kept); // its drop never runs\n```\n\n\
## Task\n\
Release `early` before \"the ritual begins\", drop an unnamed soul, let \
`_inner` end with its own scope, keep `bound` from ever being released, \
and print \"main ends\". The program prints `early is released`, \
`the ritual begins`, `unnamed is released`, `inner scope ends`, \
`inner is released`, `main ends`, `second is released`, then \
`first is released`.\n\n\
## Hint\n\
E0040: destructors are called for you. `drop(early);` moves the value \
into `std::mem::drop`, which ends it there. `std::mem::forget(bound);` \
takes ownership and never runs `drop`. A leading underscore on a name \
only silences the unused-variable warning: `let _inner = ...` still \
lives to the end of its block.",
                prereq: None,
                available: true,
                starter_code: "struct Soul {\n    name: &'static str,\n}\n\nimpl Drop for Soul {\n    fn drop(&mut self) {\n        println!(\"{} is released\", self.name);\n    }\n}\n\nfn main() {\n    let _first = Soul { name: \"first\" };\n    let _second = Soul { name: \"second\" };\n    let early = Soul { name: \"early\" };\n    let _bound = Soul { name: \"bound\" };\n\n    // Release `early` now, before the ritual begins.\n    early.drop();\n    println!(\"the ritual begins\");\n\n    // Then: an unnamed soul, an inner scope, keep `bound` from ever being\n    // released, and print \"main ends\".\n}\n",
            },
            Mission {
                id: "send_sync",
                npc_name: "The Courier",
                prompt: "Promise, with `unsafe impl`, that `Tome` is `Send` and `Sync`, and justify both.",
                tutorial: "## Concept\n\
`Send` means a value can be moved to another thread. `Sync` means a \
`&T` can be shared between threads; exactly, `T` is `Sync` when `&T` is \
`Send`. Both are auto traits: a struct has them when all its fields do. \
Raw pointers have neither, so `Tome` has neither; `Rc` has neither \
either (its count is not atomic). When you know more than the \
compiler, `unsafe impl Send for T {}` is a promise. If it is false, \
safe code can cause a data race, which is undefined behaviour. This \
sandbox has no threads (`thread::spawn` panics on wasm32-wasip1), so the \
Courier checks the promise where Rust always checks it: at compile \
time, through `T: Send` and `T: Sync` bounds.\n\n\
## Syntax\n\
```\n// SAFETY: why this is true\nunsafe impl Send for Tome {}\nunsafe impl Sync for Tome {}\nfn needs_send<T: Send>(x: T) -> T { x }\n```\n\n\
## Task\n\
Make `Tome` acceptable to `courier_accepts` and `readers_accept`, with \
a `// SAFETY:` comment for each promise. The program prints \
`the courier accepts the tome`, `page reads 7`, then \
`the page is returned to the shelf`.\n\n\
## Hint\n\
E0277, \"`*mut u32` cannot be sent between threads safely\": the raw \
pointer inside `Tome` is the culprit. A `Tome` owns its page alone \
(made by `Box::into_raw`, freed once in `Drop`), like a `Box<u32>`, so \
handing it to another thread is fine. `Sync` holds because nothing can \
change the page through a `&Tome`.",
                prereq: None,
                available: true,
                starter_code: "struct Tome {\n    page: *mut u32,\n}\n\nimpl Tome {\n    fn new(value: u32) -> Self {\n        Tome {\n            page: Box::into_raw(Box::new(value)),\n        }\n    }\n\n    fn read(&self) -> u32 {\n        unsafe { *self.page }\n    }\n}\n\nimpl Drop for Tome {\n    fn drop(&mut self) {\n        unsafe { drop(Box::from_raw(self.page)) };\n        println!(\"the page is returned to the shelf\");\n    }\n}\n\n// The courier and the readers refuse the tome. Decide whether it is sound\n// to promise that a Tome may move to, and be shared between, threads.\n\nfn courier_accepts<T: Send>(item: T) -> T {\n    item\n}\n\nfn readers_accept<T: Sync>(item: &T) -> &T {\n    item\n}\n\nfn main() {\n    let tome = Tome::new(7);\n    let carried = courier_accepts(tome);\n    println!(\"the courier accepts the tome\");\n    let shared = readers_accept(&carried);\n    println!(\"page reads {}\", shared.read());\n}\n",
            },
            Mission {
                id: "maybe_uninit",
                npc_name: "The Bookbinder",
                prompt: "Fill a `MaybeUninit<[u32; 4]>` through a raw pointer, then `assume_init`.",
                tutorial: "## Concept\n\
Safe Rust never lets you read memory that has not been initialised \
(E0381). `MaybeUninit<T>` is the opt-out: storage for a `T` that may \
not hold a valid `T` yet, about which the compiler assumes nothing. \
You fill it through a raw pointer (or the safe `MaybeUninit::write`), \
then call `assume_init()`, which is `unsafe`: you promise every byte is \
initialised and forms a valid `T`. Calling it early is undefined \
behaviour even for plain integers; an uninitialised `u32` is not \"some \
random number\". Fill with `ptr.write(v)`, not `*ptr = v`: assignment \
first drops the old value, and dropping uninitialised memory is \
undefined behaviour for any type with a destructor. A `MaybeUninit` \
never drops what it holds.\n\n\
## Syntax\n\
```\nlet mut slot = MaybeUninit::<[u32; 4]>::uninit();\nlet p: *mut u32 = slot.as_mut_ptr().cast::<u32>();\nunsafe { p.add(i).write(v) };            // i < 4\nlet arr = unsafe { slot.assume_init() };  // after every write\n```\n\n\
## Task\n\
Fill all four pages of a `MaybeUninit<[u32; PAGES]>` through a raw \
pointer, page `i` holding `(i + 1) * 10`, then `assume_init` and print \
`bound pages: [10, 20, 30, 40]` and `total ink: 100`.\n\n\
## Hint\n\
E0381, \"used binding `quire` isn't initialized\": rustc points at the \
read on the `println!` line, because safe Rust cannot prove the loop \
set every element. Take `let first = quire.as_mut_ptr().cast::<u32>();`, \
write each page with `unsafe { first.add(i).write(..) }`, and call \
`assume_init` once, after the loop, never before.",
                prereq: None,
                available: true,
                starter_code: "const PAGES: usize = 4;\n\nfn main() {\n    // Declare the quire now, fill each page in the loop, read it after.\n    let quire: [u32; PAGES];\n    for i in 0..PAGES {\n        quire[i] = (i as u32 + 1) * 10;\n    }\n    println!(\"bound pages: {quire:?}\");\n    println!(\"total ink: {}\", quire.iter().sum::<u32>());\n}\n",
            },
            // ── Act 9 boss: the Librarian's trial ──────────────────────
            // Three stages owned by the Librarian (`trial::TRIALS`). He
            // has no prologue: his roster mission is stage 1. A safe door
            // over an `unsafe fn`, a recursive `macro_rules!` with an
            // accumulator, then a type that owns a raw pointer and frees
            // it in `Drop`. The Librarian ends the registry.
            Mission {
                id: "librarian_index",
                npc_name: "The Librarian",
                prompt: "Make `fetch` safe for every index: check bounds, then call the `unsafe fn`.",
                tutorial: "## Concept\n\
An `unsafe fn` has a contract the compiler cannot check, written in a \
`# Safety` doc section. Calling it is `unsafe`: the caller promises to \
keep the contract. The art of unsafe Rust is to wrap such a function \
in a safe one whose own checks keep the contract for every possible \
input. That wrapper is sound if no caller, using only safe code, can \
cause undefined behaviour through it. Pointer arithmetic is where this \
matters: `ptr.add(i)` is only defined while the result stays inside \
the same allocation.\n\n\
## Syntax\n\
```\n/// # Safety\n/// `p` must point at a live `u32`.\nunsafe fn read(p: *const u32) -> u32 {\n    unsafe { *p }\n}\nlet x = 5;\nlet v = unsafe { read(&x) };\n```\n\n\
## Task\n\
Make `fetch` safe to call with any index: check it against the shelf's \
length, and only then call `volume_at` inside `unsafe`. The program \
prints `index 0: volume 11`, `index 3: volume 44`, \
`index 4: no such volume`, then `index 99: no such volume`.\n\n\
## Hint\n\
E0133, \"call to unsafe function `volume_at` is unsafe and requires \
unsafe block\". Wrapping the call in `unsafe` alone compiles, but then \
index 99 reads past the shelf: undefined behaviour. Write \
`if index < shelf.len() { ... } else { None }`, with a `// SAFETY:` \
comment that names the check.",
                prereq: None,
                available: true,
                starter_code: "/// Returns the volume `index` places past `shelf`.\n///\n/// # Safety\n///\n/// `shelf` must point into a live array of initialised `u32`s, and\n/// `shelf.add(index)` must still be inside that same array.\nunsafe fn volume_at(shelf: *const u32, index: usize) -> u32 {\n    unsafe { *shelf.add(index) }\n}\n\n// The Librarian wants a door that is safe to open with ANY index.\nfn fetch(shelf: &[u32], index: usize) -> Option<u32> {\n    Some(volume_at(shelf.as_ptr(), index))\n}\n\nfn main() {\n    let shelf = [11, 22, 33, 44];\n    for index in [0, 3, 4, 99] {\n        match fetch(&shelf, index) {\n            Some(volume) => println!(\"index {index}: volume {volume}\"),\n            None => println!(\"index {index}: no such volume\"),\n        }\n    }\n}\n",
            },
            Mission {
                id: "librarian_demand",
                npc_name: "The Librarian",
                prompt: "Grow `incant!` so `incant!(1, 2, 3, 4)` expands to the Librarian's exact demand.",
                tutorial: "## Concept\n\
A macro can call itself. To build a left-nested expression such as \
`seal(seal(seal(a, b), c), d)`, carry the part built so far in an \
accumulator: an internal rule that starts with a literal token like \
`@acc`, which callers never write. A literal `@` can never start an \
expression, so a caller's `incant!(1, 2)` never matches an `@acc` rule, \
and an `@acc` call never matches the entry rule, whichever order the \
rules are written in.\n\n\
## Syntax\n\
```\nmacro_rules! count {\n    (@acc $n:expr) => { $n };\n    (@acc $n:expr, $x:expr $(, $rest:expr)*) => {\n        count!(@acc $n + 1 $(, $rest)*)\n    };\n    ($($x:expr),*) => { count!(@acc 0 $(, $x)*) };\n}\n```\n\
`count!(5, 6, 7)` is `3`.\n\n\
## Task\n\
Make `incant!(a, b, c, d)` expand to \
`seal(seal(seal(page(a), page(b)), page(c)), page(d))`, for any number \
of arguments, one or more. The program prints `the demand: 78`, \
`incant!(1, 2, 3, 4): 78`, `the incantation matches: true`, \
`incant!(7): 21`, then `incant!(2, 9): 39`.\n\n\
## Hint\n\
\"no rules expected `,`\": your only rule takes exactly one expression. \
The entry rule is \
`($first:expr $(, $rest:expr)*) => { incant!(@acc page($first) $(, $rest)*) }`. \
`seal` is neither commutative nor associative, so folding from the \
right gives `48`, not `78`.",
                prereq: None,
                available: true,
                starter_code: "fn page(n: u32) -> u32 {\n    n * 3\n}\n\nfn seal(left: u32, right: u32) -> u32 {\n    left * 2 + right\n}\n\n// Grow this macro so incant!(a, b, c, d) expands to\n//     seal(seal(seal(page(a), page(b)), page(c)), page(d))\n// and works for any number of arguments, one or more.\nmacro_rules! incant {\n    ($first:expr) => {\n        page($first)\n    };\n}\n\nfn main() {\n    let demand = seal(seal(seal(page(1), page(2)), page(3)), page(4));\n    let answer = incant!(1, 2, 3, 4);\n    println!(\"the demand: {demand}\");\n    println!(\"incant!(1, 2, 3, 4): {answer}\");\n    println!(\"the incantation matches: {}\", demand == answer);\n    println!(\"incant!(7): {}\", incant!(7));\n    println!(\"incant!(2, 9): {}\", incant!(2, 9));\n}\n",
            },
            Mission {
                id: "librarian_seal",
                npc_name: "The Librarian",
                prompt: "Give `Codex` a bounds-checked `get` and a `Drop` that returns its pages.",
                tutorial: "## Concept\n\
`Box::into_raw` turns a box into a raw pointer and gives up ownership: \
nothing frees that memory unless you rebuild the same box with \
`Box::from_raw` (same address, same type, same length for a slice) and \
let it drop. Do it twice and you free memory twice; never, and it \
leaks. A raw pointer owns nothing; the struct that holds it must. That \
is what `Vec` and `Box` do inside the standard library.\n\n\
## Syntax\n\
```\nlet b: Box<[u32]> = vec![1, 2].into_boxed_slice();\nlet n = b.len();\nlet p: *mut u32 = Box::into_raw(b).cast::<u32>();\nlet whole = std::ptr::slice_from_raw_parts_mut(p, n);\ndrop(unsafe { Box::from_raw(whole) }); // freed, once\n```\n\n\
## Task\n\
Write `get` so it returns `Some(page)` for valid indexes and `None` \
otherwise without reading outside the codex, and implement `Drop` so \
the pages go back to the heap and the codex prints its seal line. The \
program prints `page 0: Some(5)`, `page 2: Some(13)`, `page 3: None`, \
`the Librarian closes the book`, then \
`the codex is sealed (3 pages returned)`.\n\n\
## Hint\n\
As written, the starter compiles and prints `None` three times and the \
closing line, nothing more: `get` reads nothing and nothing frees the \
pages. In `get`, read `*self.ptr.add(index)` only when \
`index < self.len`. In `impl Drop for Codex`, rebuild the slice \
pointer with `std::ptr::slice_from_raw_parts_mut(self.ptr, self.len)`, \
then `Box::from_raw` it inside `unsafe`.",
                prereq: None,
                available: true,
                starter_code: "struct Codex {\n    ptr: *mut u32,\n    len: usize,\n}\n\nimpl Codex {\n    fn from_pages(pages: Vec<u32>) -> Self {\n        let boxed: Box<[u32]> = pages.into_boxed_slice();\n        let len = boxed.len();\n        let ptr = Box::into_raw(boxed).cast::<u32>();\n        Codex { ptr, len }\n    }\n\n    // Return the page at `index`, or None past the end, without ever\n    // reading outside the codex.\n    fn get(&self, _index: usize) -> Option<u32> {\n        None\n    }\n}\n\n// Box::into_raw handed the pages out. Nothing gives them back.\n\nfn main() {\n    let codex = Codex::from_pages(vec![5, 8, 13]);\n    println!(\"page 0: {:?}\", codex.get(0));\n    println!(\"page 2: {:?}\", codex.get(2));\n    println!(\"page 3: {:?}\", codex.get(3));\n    println!(\"the Librarian closes the book\");\n}\n",
            },
            // ── Act 10: The Throne ─────────────────────────────────────
            // Measuring allocations, `Cow`, compile-time evaluation, the
            // lane shape SIMD wants, `no_std`, Cargo's caret rule, then the
            // Compiler's trial. Design:
            // docs/superpowers/specs/2026-11-14-act10-throne.md. Nothing
            // here grades speed: every lesson is a deterministic output (an
            // allocation count, borrowed or copied, a value computed at
            // compile time). `unsafe` and `#[global_allocator]` appear here
            // only inside string literals (the player's program). Every
            // compiler error and run-time behaviour this act names is
            // compiled or run in `tests/mission_text.rs` (hard rule 3).
            Mission {
                id: "prealloc",
                npc_name: "The Assayer",
                prompt: "Build the 1000-coin ledger with exactly one allocation.",
                tutorial: "## Concept\n\
Before you optimise, measure. Here the instrument is a counting \
allocator: a `#[global_allocator]` that wraps `std::alloc::System` and \
adds one to an atomic counter on every allocation and reallocation. \
(Writing one takes `unsafe impl GlobalAlloc`: you promise to keep the \
allocator's contract, which passing every call on to `System` does.) A \
`Vec` that starts empty has to reallocate as it grows, and std does not \
promise how often. Give it the final size up front with \
`Vec::with_capacity(n)` and std does promise that `push` never \
reallocates while the length is below the capacity. On a real machine \
you would reach for a profiler (`cargo flamegraph`, `perf`, DHAT), and \
crates such as `smallvec` keep small collections off the heap.\n\n\
## Syntax\n\
```\nlet before = ALLOCATIONS.load(Ordering::Relaxed);\nlet mut v = Vec::with_capacity(1000);\n// ... push 1000 times ...\nlet after = ALLOCATIONS.load(Ordering::Relaxed);\n```\n\n\
## Task\n\
Change how the ledger is created so building it costs exactly one \
allocation. The allocator and the measurement are already written. The \
program prints `coins: 1000`, then `allocations: 1`.\n\n\
## Hint\n\
The scale reads more than 1: every count above 1 is the `Vec` \
outgrowing its buffer and asking for a bigger one. You know the final \
length before the loop starts, so ask for it once: \
`Vec::with_capacity(1000)`. (`(0..1000u32).collect::<Vec<_>>()` \
measures 1 here too.)",
                prereq: None,
                available: true,
                starter_code: "use std::alloc::{GlobalAlloc, Layout, System};\nuse std::sync::atomic::{AtomicUsize, Ordering};\n\n/// The Assayer's scale: wraps the system allocator and counts every\n/// allocation and reallocation.\nstruct CountingAlloc;\n\nstatic ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);\n\nunsafe impl GlobalAlloc for CountingAlloc {\n    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {\n        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);\n        unsafe { System.alloc(layout) }\n    }\n\n    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {\n        unsafe { System.dealloc(ptr, layout) }\n    }\n\n    unsafe fn realloc(\n        &self,\n        ptr: *mut u8,\n        layout: Layout,\n        new_size: usize,\n    ) -> *mut u8 {\n        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);\n        unsafe { System.realloc(ptr, layout, new_size) }\n    }\n}\n\n#[global_allocator]\nstatic GLOBAL: CountingAlloc = CountingAlloc;\n\nfn main() {\n    let before = ALLOCATIONS.load(Ordering::Relaxed);\n\n    // The Assayer wants this ledger built with a single allocation.\n    let mut ledger = Vec::new();\n    for coin in 0..1000u32 {\n        ledger.push(coin);\n    }\n\n    let after = ALLOCATIONS.load(Ordering::Relaxed);\n    println!(\"coins: {}\", ledger.len());\n    println!(\"allocations: {}\", after - before);\n}\n",
            },
            Mission {
                id: "cow_borrow",
                npc_name: "The Copyist",
                prompt: "Return `Cow<'_, str>`: borrow titles that need no change, copy only the rest.",
                tutorial: "## Concept\n\
`Cow<'a, str>` (\"clone on write\") is either `Cow::Borrowed(&'a str)` or \
`Cow::Owned(String)`. It dereferences to `&str`, so callers use it like \
any string. A function that only sometimes changes its input can hand \
back the input itself, borrowed, when nothing changes, and build a new \
`String` only when something does. The common case then copies \
nothing. Call `.into_owned()` if you later need a `String`: it copies \
only if the value was borrowed.\n\n\
## Syntax\n\
```\nfn tidy(s: &str) -> Cow<'_, str> {\n    if needs_change(s) {\n        Cow::Owned(fix(s))\n    } else {\n        Cow::Borrowed(s)\n    }\n}\n```\n\n\
## Task\n\
Make `shelf_name` return `Cow<'_, str>`, borrowing titles without \
spaces, and print each name with `(borrowed)` or `(copied)`. The \
program prints `Grimoire (borrowed)`, `Book_of_Crowns (copied)`, then \
`Ledger (borrowed)`.\n\n\
## Hint\n\
`replace` always builds a new `String`, even when there is nothing to \
replace. Return `Cow::Owned(title.replace(' ', \"_\"))` only when \
`title.contains(' ')`, and `Cow::Borrowed(title)` otherwise. In `main`, \
`match &name { Cow::Borrowed(_) => \"borrowed\", Cow::Owned(_) => \"copied\" }`.",
                prereq: None,
                available: true,
                starter_code: "/// Shelf names use underscores instead of spaces. This version builds a\n/// brand-new String for every title, even ones with nothing to change.\nfn shelf_name(title: &str) -> String {\n    title.replace(' ', \"_\")\n}\n\nfn main() {\n    for title in [\"Grimoire\", \"Book of Crowns\", \"Ledger\"] {\n        let name = shelf_name(title);\n        // The Copyist wants each line to end with (borrowed) or\n        // (copied).\n        println!(\"{name}\");\n    }\n}\n",
            },
            Mission {
                id: "const_eval",
                npc_name: "The Astronomer",
                prompt: "Make `star_table` a `const fn`, compute `STARS` at compile time, and check it with a compile-time `assert!`.",
                tutorial: "## Concept\n\
A `const` item's value is computed by the compiler and built into the \
program; no work at run time builds it. Only `const fn`s may be called \
there. A `const fn` is an ordinary function that can also run inside \
the compiler, so its body is limited: no `for` loops (they call \
`Iterator::next`, which is not a `const fn`), but `while`, `if`, \
`match`, arithmetic and array indexing are fine. \
`const _: () = assert!(cond, \"msg\");` checks `cond` while compiling: \
if it is false the build fails with E0080, \"evaluation panicked: \
msg\". A test that runs before the program exists. (`#[inline]` is \
another way to ask the compiler for speed; this lesson sticks to what \
you can see in the output.)\n\n\
## Syntax\n\
```\nconst fn square(n: u32) -> u32 {\n    n * n\n}\nconst NINE: u32 = square(3);\nconst _: () = assert!(NINE == 9, \"bad square\");\n```\n\n\
## Task\n\
Make `star_table` a `const fn` so `STARS` is computed at compile time, \
and add a compile-time check that the last star is 50. The program \
prints `star chart: [1, 2, 5, 10, 17, 26, 37, 50]`, then \
`brightest: 50`.\n\n\
## Hint\n\
E0015, \"cannot call non-const function `star_table` in constants\": a \
`const` can only call `const fn`s. Adding `const` alone is not enough: \
the `for` loop is a second E0015, \"cannot use `for` loop on \
`std::ops::Range<usize>` in constant functions\". Count with \
`let mut i = 0; while i < 8 { ...; i += 1; }`, then add \
`const _: () = assert!(STARS[7] == 50, \"the last star is misplaced\");` \
at the top level.",
                prereq: None,
                available: true,
                starter_code: "fn star_table() -> [u32; 8] {\n    let mut table = [0u32; 8];\n    for i in 0..8 {\n        table[i] = (i as u32) * (i as u32) + 1;\n    }\n    table\n}\n\n// The Astronomer wants the chart drawn before the night begins:\n// computed by the compiler, and checked by it too.\nconst STARS: [u32; 8] = star_table();\n\nfn main() {\n    println!(\"star chart: {STARS:?}\");\n    println!(\"brightest: {}\", STARS[7]);\n}\n",
            },
            Mission {
                id: "chunks_exact",
                npc_name: "The Choirmaster",
                prompt: "Sum in four lanes with `chunks_exact(4)` and don't drop the remainder.",
                tutorial: "## Concept\n\
SIMD hardware adds several numbers in one instruction, in parallel \
\"lanes\". Code that suits it works in fixed-width chunks with \
independent accumulators, so no step waits on the one before. \
`chunks_exact(4)` yields only full 4-element slices; the std docs note \
that because every chunk has exactly that size, the compiler can often \
optimise the loop better than with `chunks`. The 0 to 3 elements left \
over are not yielded: they wait in `.remainder()`, and forgetting them \
is the classic bug. Portable SIMD (`std::simd`) is unstable: on stable \
Rust, `use std::simd` is E0658, \"use of unstable library feature \
`portable_simd`\". A grader cannot check speed, so the Choirmaster \
checks the answer.\n\n\
## Syntax\n\
```\nlet mut chunks = data.chunks_exact(4);\nfor chunk in &mut chunks { /* 4 elements */ }\nlet rest = chunks.remainder(); // 0..=3 elements\n```\n\n\
## Task\n\
Fix `choir_sum` so it also adds the voices that do not fill a whole \
chunk. The program prints `choir sum: 55`, `plain sum: 55`, then \
`empty choir: 0`.\n\n\
## Hint\n\
1 + 2 + ... + 10 is 55, but the choir sings 36: voices 9 and 10 are \
never heard. Keep the iterator in a variable \
(`let mut chunks = voices.chunks_exact(4);`), loop over `&mut chunks`, \
then add `chunks.remainder().iter().sum::<u32>()` to the lanes. \
(`chunks(4)`, whose last chunk may be short, is also correct, but gives \
up the exact-width shape.)",
                prereq: None,
                available: true,
                starter_code: "/// Sums in four independent lanes. The Choirmaster says some voices are\n/// never heard.\nfn choir_sum(voices: &[u32]) -> u32 {\n    let mut lanes = [0u32; 4];\n    for chunk in voices.chunks_exact(4) {\n        for (lane, voice) in lanes.iter_mut().zip(chunk) {\n            *lane += voice;\n        }\n    }\n    lanes.iter().sum::<u32>()\n}\n\nfn main() {\n    let voices: Vec<u32> = (1..=10).collect();\n    println!(\"choir sum: {}\", choir_sum(&voices));\n    println!(\"plain sum: {}\", voices.iter().sum::<u32>());\n    println!(\"empty choir: {}\", choir_sum(&[]));\n}\n",
            },
            Mission {
                id: "no_std",
                npc_name: "The Hermit",
                prompt: "In a `#![no_std]` crate, format the beacon line into a fixed buffer with `core::fmt::Write`.",
                tutorial: "## Concept\n\
`#![no_std]` at the crate root drops the standard library and links \
only `core`: no heap, no files, no threads, and no `println!` (\"cannot \
find macro `println` in this scope\"). `String` and `format!` live in \
`alloc` and `std`, so they are gone too. What stays is everything that \
needs no operating system: integers, slices, `Option`, `Result`, \
iterators, `core::fmt`. Firmware lives here. (`extern crate alloc;` \
brings `String` and `Vec` back when the device has a global \
allocator.) This sandbox still needs std to print, so the starter links \
it under another name, `extern crate std as host;`, for `main` alone. \
The `firmware` module must not use it.\n\n\
## Syntax\n\
```\nuse core::fmt::{self, Write};\nstruct Buf { bytes: [u8; 32], len: usize }\nimpl Write for Buf {\n    fn write_str(&mut self, s: &str) -> fmt::Result {\n        // copy s in, or return Err(fmt::Error)\n    }\n}\nwrite!(buf, \"tower {n}\")?; // no heap needed\n```\n\n\
## Task\n\
Rewrite `firmware::signal` to format into a 32-byte stack buffer \
through `core::fmt::Write`, refusing (returning `Err`) instead of \
overflowing. Then, in `main`, try to write a message longer than 32 \
bytes into a fresh buffer and report whether it was refused. The \
program prints `tower 7: lit (sent: true)`, then \
`overflow refused: true`.\n\n\
## Hint\n\
E0425, \"cannot find type `String` in this scope\", and \"cannot find \
macro `format` in this scope\": both live in `alloc`/`std`. rustc \
suggests `use crate::host::format;`. That compiles, and it is exactly \
what the outpost does not have. Give `firmware` a \
`Beacon { bytes: [u8; 32], len: usize }` that implements \
`core::fmt::Write` (one method, `write_str`): return `Err(fmt::Error)` \
when `self.len + s.len()` would pass 32, otherwise `copy_from_slice` \
the bytes in and advance `len`. `write!(beacon, ...)` then works \
without a heap, and `core::str::from_utf8(&self.bytes[..self.len])` \
gives the text back.",
                prereq: None,
                available: true,
                starter_code: "#![no_std]\n\nextern crate std as host;\n\nmod firmware {\n    // The Hermit's outpost has no heap. Rewrite `signal` so it formats\n    // into a fixed-size buffer instead of a String.\n    pub fn signal(tower: u8, lit: bool) -> String {\n        format!(\"tower {tower}: {}\", if lit { \"lit\" } else { \"dark\" })\n    }\n}\n\nfn main() {\n    let line = firmware::signal(7, true);\n    host::println!(\"{line} (sent: true)\");\n}\n",
            },
            Mission {
                id: "semver_caret",
                npc_name: "The Registrar",
                prompt: "Fix `caret_allows` so it follows Cargo's caret rule below 1.0 too.",
                tutorial: "## Concept\n\
In `Cargo.toml`, `serde = \"1.2.3\"` means the caret requirement \
`^1.2.3`: any version from 1.2.3 up to, but not including, 2.0.0. \
Cargo assumes semver: a change to the leftmost non-zero part may break \
you, anything to its right should not. Below 1.0 that moves right: \
`^0.2.3` allows 0.2.3 up to 0.3.0, and `^0.0.3` allows only 0.0.3. \
`Cargo.lock` records the exact versions chosen; `cargo update` picks \
newer ones the requirements allow. The rest of the ecosystem: `rustup` \
installs toolchains, `cargo fmt` (rustfmt) formats, `cargo clippy` \
lints, `cargo doc` builds docs, and docs.rs hosts them for every crate \
on crates.io. None of those run in this sandbox; the caret rule does.\n\n\
## Syntax\n\
```\n[dependencies]\nserde = \"1.2.3\" # >= 1.2.3, < 2.0.0\nrand = \"0.8.5\"  # >= 0.8.5, < 0.9.0\ntiny = \"0.0.3\"  # exactly 0.0.3\n```\n\n\
## Task\n\
Make `caret_allows(required, candidate)` follow the caret rule for \
1.x, 0.x and 0.0.x requirements (pre-release tags are out of scope). \
The program prints `^1.2.3 accepts 1.9.0: true`, \
`^1.2.3 accepts 2.0.0: false`, `^1.2.3 accepts 1.2.2: false`, \
`^0.2.3 accepts 0.2.9: true`, `^0.2.3 accepts 0.3.0: false`, \
`^0.0.3 accepts 0.0.3: true`, `^0.0.3 accepts 0.0.4: false`, then \
`^1.2.3 vs 1.x.0: not a version`.\n\n\
## Hint\n\
The rule is right for `^1.2.3` and wrong for `^0.2.3`: it lets 0.3.0 \
through. After rejecting `candidate < required`, find the leftmost \
non-zero part of `required`: the candidate must match `required` up to \
and including it. \
`if required.major > 0 { ... } else if required.minor > 0 { ... } else { ... }`.",
                prereq: None,
                available: true,
                starter_code: "#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]\nstruct Version {\n    major: u64,\n    minor: u64,\n    patch: u64,\n}\n\nfn parse(text: &str) -> Option<Version> {\n    let mut parts = text.split('.');\n    let major = parts.next()?.parse().ok()?;\n    let minor = parts.next()?.parse().ok()?;\n    let patch = parts.next()?.parse().ok()?;\n    if parts.next().is_some() {\n        return None;\n    }\n    Some(Version { major, minor, patch })\n}\n\n/// The Registrar says this rule is right for 1.x and wrong below it.\nfn caret_allows(required: Version, candidate: Version) -> bool {\n    candidate >= required && candidate.major == required.major\n}\n\nfn main() {\n    let checks = [\n        (\"1.2.3\", \"1.9.0\"),\n        (\"1.2.3\", \"2.0.0\"),\n        (\"1.2.3\", \"1.2.2\"),\n        (\"0.2.3\", \"0.2.9\"),\n        (\"0.2.3\", \"0.3.0\"),\n        (\"0.0.3\", \"0.0.3\"),\n        (\"0.0.3\", \"0.0.4\"),\n        (\"1.2.3\", \"1.x.0\"),\n    ];\n    for (req, cand) in checks {\n        match (parse(req), parse(cand)) {\n            (Some(r), Some(c)) => println!(\"^{req} accepts {cand}: {}\", caret_allows(r, c)),\n            _ => println!(\"^{req} vs {cand}: not a version\"),\n        }\n    }\n}\n",
            },
            // ── Act 10 boss: the Compiler's trial ──────────────────────
            // Three stages owned by the Compiler (`trial::TRIALS`), no
            // prologue: his roster mission is stage 1, as for the
            // Librarian. One allocation, an honest lifetime, the weakest
            // bound that works. The Compiler ends the registry: clearing
            // `compiler_bound` clears every mission, which shows the
            // epilogue. His authored words quote no compiler output; the
            // real stderr is in the editor's result pane.
            Mission {
                id: "compiler_alloc",
                npc_name: "The Compiler",
                prompt: "Build the court's proclamation with exactly one allocation.",
                tutorial: "## Concept\n\
Every `format!` builds a new `String`, so building text with `format!` \
in a loop asks for memory at every step, and copies everything written \
so far each time. When you can work out the final length first, \
`String::with_capacity(len)` allocates once, and `push_str` never \
reallocates while the text fits. The Compiler measures with the same \
counting allocator the Assayer used.\n\n\
## Syntax\n\
```\nlet mut line = String::with_capacity(11);\nline.push_str(\"crown\");\nline.push_str(\", \");\nline.push_str(\"seal\"); // 11 bytes: it fits\n```\n\n\
## Task\n\
Make `proclaim` cost exactly one allocation. The output text must not \
change. The program prints \
`Ferris, the Borrow Checker, Vexis, the Librarian`, then \
`allocations: 1`.\n\n\
## Hint\n\
Read the count your program prints: every `format!` is a new `String`. \
The final length is the names' lengths added up, plus 2 bytes \
(`\", \"`) between each pair. \
`String::with_capacity(letters + separators)`, then `push_str` the \
separator (when `i > 0`) and each name. (`names.join(\", \")` also \
measures 1 here.)",
                prereq: None,
                available: true,
                starter_code: "use std::alloc::{GlobalAlloc, Layout, System};\nuse std::sync::atomic::{AtomicUsize, Ordering};\n\n/// The Assayer's scale: wraps the system allocator and counts every\n/// allocation and reallocation.\nstruct CountingAlloc;\n\nstatic ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);\n\nunsafe impl GlobalAlloc for CountingAlloc {\n    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {\n        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);\n        unsafe { System.alloc(layout) }\n    }\n\n    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {\n        unsafe { System.dealloc(ptr, layout) }\n    }\n\n    unsafe fn realloc(\n        &self,\n        ptr: *mut u8,\n        layout: Layout,\n        new_size: usize,\n    ) -> *mut u8 {\n        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);\n        unsafe { System.realloc(ptr, layout, new_size) }\n    }\n}\n\n#[global_allocator]\nstatic GLOBAL: CountingAlloc = CountingAlloc;\n\n/// The Compiler demands this proclamation cost exactly one allocation.\nfn proclaim(names: &[&str]) -> String {\n    let mut line = String::new();\n    for (i, name) in names.iter().enumerate() {\n        if i > 0 {\n            line = format!(\"{line}, \");\n        }\n        line = format!(\"{line}{name}\");\n    }\n    line\n}\n\nfn main() {\n    let court = [\n        \"Ferris\",\n        \"the Borrow Checker\",\n        \"Vexis\",\n        \"the Librarian\",\n    ];\n    let before = ALLOCATIONS.load(Ordering::Relaxed);\n    let line = proclaim(&court);\n    let after = ALLOCATIONS.load(Ordering::Relaxed);\n    println!(\"{line}\");\n    println!(\"allocations: {}\", after - before);\n}\n",
            },
            Mission {
                id: "compiler_lifetime",
                npc_name: "The Compiler",
                prompt: "Give `herald` a lifetime that ties the result only to `name`.",
                tutorial: "## Concept\n\
With one reference input, Rust infers that the output borrows from it. \
With two, it cannot guess, and asks you to name a lifetime. Name it \
honestly: put `'a` only on the inputs the result may really borrow \
from. Here the result borrows `name` (the `Borrowed` case) and never \
`title` (the `Owned` case copies it into a new `String`). Tie `title` \
to the result too and the compiler will believe you, and then refuse \
any caller whose `title` is dropped first.\n\n\
## Syntax\n\
```\nfn pick<'a>(keep: &'a str, look: &str) -> &'a str {\n    if look.is_empty() { keep } else { keep.trim() }\n}\n```\n\n\
## Task\n\
Fix the signature of `herald` so the program compiles and prints \
`Ferris (borrowed)`, then `Sir Ferris (owned)`. `main` must stay as it \
is.\n\n\
## Hint\n\
E0106, \"missing lifetime specifier\": the signature does not say \
whether the result is borrowed from `name` or from `title`. rustc's \
help puts `'a` on both inputs; follow it and `main` fails instead, \
E0597, \"`title` does not live long enough\": `title` is dropped at \
the end of its block while `titled` lives on. Put `'a` on `name` and \
the result only: \
`fn herald<'a>(name: &'a str, title: &str) -> Cow<'a, str>`.",
                prereq: None,
                available: true,
                starter_code: "use std::borrow::Cow;\n\nfn herald(name: &str, title: &str) -> Cow<str> {\n    if title.is_empty() {\n        Cow::Borrowed(name)\n    } else {\n        Cow::Owned(format!(\"{title} {name}\"))\n    }\n}\n\nfn main() {\n    let name = String::from(\"Ferris\");\n    let plain = herald(&name, \"\");\n    let titled = {\n        let title = String::from(\"Sir\");\n        herald(&name, &title)\n    };\n    for line in [&plain, &titled] {\n        let how = match line {\n            Cow::Borrowed(_) => \"borrowed\",\n            Cow::Owned(_) => \"owned\",\n        };\n        println!(\"{line} ({how})\");\n    }\n}\n",
            },
            Mission {
                id: "compiler_bound",
                npc_name: "The Compiler",
                prompt: "Bound `T` so `strongest` works for integers, floats and strings.",
                tutorial: "## Concept\n\
A generic function may only do to `T` what its bounds allow. `>` comes \
from `PartialOrd`. `Ord` is the stronger promise of a total order, \
which `sort` and `Iterator::max` need. Floats implement `PartialOrd` \
but not `Ord`: NaN is not equal to itself and every comparison with it \
is false, so their order is only partial. Ask for the weakest bound \
that does the job, and the function serves the most types.\n\n\
## Syntax\n\
```\nfn bigger<T: PartialOrd>(a: T, b: T) -> T {\n    if a > b { a } else { b }\n}\n```\n\n\
## Task\n\
Add the bound that lets `strongest` compile for the integer, float and \
string calls in `main`. The program prints `Some(9)`, `Some(2.5)`, \
`Some(\"vexis\")`, then `None`.\n\n\
## Hint\n\
E0369, \"binary operation `>` cannot be applied to type `&T`\": read \
the help line under it. `Ord` with `.iter().max()` looks shorter; try \
it and the float call fails with E0277, \"the trait bound \
`{float}: Ord` is not satisfied\". \
`fn strongest<T: PartialOrd>(champions: &[T]) -> Option<&T>`, keeping \
the loop.",
                prereq: None,
                available: true,
                starter_code: "fn strongest<T>(champions: &[T]) -> Option<&T> {\n    let mut best = champions.first()?;\n    for champion in champions {\n        if champion > best {\n            best = champion;\n        }\n    }\n    Some(best)\n}\n\nfn main() {\n    println!(\"{:?}\", strongest(&[3, 9, 4]));\n    println!(\"{:?}\", strongest(&[0.5, 2.5, 1.0]));\n    println!(\"{:?}\", strongest(&[\"ferris\", \"vexis\", \"borrow\"]));\n    println!(\"{:?}\", strongest::<i32>(&[]));\n}\n",
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
    let epilogue_visible = epilogue_due(&progress, &registry)
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

/// Present only while a scripted capture scene wants the world without the
/// "next mission" pin (the district line-ups, whose top-left NPC the pin
/// would cover). Never inserted by gameplay; `screenshot::scenes` inserts
/// it for one scene and removes it in that scene's teardown.
#[derive(Resource, Debug, Default)]
pub struct HideNextPinForCapture;

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
    capture_hide: Option<Res<HideNextPinForCapture>>,
) {
    if capture_hide.is_some() {
        return;
    }
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
/// realm itself addressing the player, then a short credits block. All
/// ten acts are built, so it says the game is complete; it promises
/// nothing that does not exist. Copy: [`EPILOGUE_LINES`].
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
        || !epilogue_due(&progress, &registry)
    {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    modal_panel(ctx, "epilogue", 520.0, |ui| {
        ui_theme::heading(ui, EPILOGUE_LINES.heading);
        ui_theme::rule(ui);
        ui_theme::markdown::show_inline(ui, EPILOGUE_LINES.body, ui_theme::BODY_SIZE);
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui_theme::status_mark(ui, ui_theme::Status::Cleared, ui_theme::BODY_SIZE);
            // Missions, not NPCs: a trial NPC owns several missions.
            ui.label(format!(
                "{} of {} missions cleared, across all ten acts.",
                progress.cleared_count(),
                registry.missions.len()
            ));
        });
        ui.add_space(6.0);
        ui_theme::caption(ui, EPILOGUE_LINES.after);
        ui_theme::rule(ui);
        for line in EPILOGUE_LINES.credits {
            ui_theme::caption(ui, *line);
        }
        ui_theme::rule(ui);
        ui_theme::key_hint(ui, &[("Esc", "close")]);
    });
}

/// Whether this save has earned the epilogue: every mission in the
/// registry is cleared. Never in a demo build (it ends with its own
/// panel, even on a full-game save). The panel also waits while the
/// editor or a completion panel is open, and after one Esc.
pub fn epilogue_due(progress: &MissionProgress, registry: &MissionRegistry) -> bool {
    registry
        .missions
        .iter()
        .all(|m| m.available && progress.is_cleared(m.id))
}

/// The epilogue's copy, in one place so tests can read it.
pub struct EpilogueLines {
    pub heading: &'static str,
    /// Inline markdown (code spans render as chips).
    pub body: &'static str,
    pub after: &'static str,
    pub credits: &'static [&'static str],
}

/// What the epilogue says once every mission is cleared. Plain and true:
/// the game has ten acts and nothing more is promised. The owner is the
/// company; no personal names.
pub const EPILOGUE_LINES: EpilogueLines = EpilogueLines {
    heading: "The realm pledges its crown.",
    body: "You have answered every sworn question, from the first `let` in \
           Hearthstone Square to the Compiler's last proof on the Throne. \
           The Compiler rises. The banners come in.",
    after: "That is the whole road. Revisit any NPC with [F] to re-read a \
            lesson, or wander the realm.",
    credits: &[
        "Pledge & Crown, made by Chico Cell Repair LLC.",
        "Written in Rust with Bevy, bevy_egui, bevy_ecs_tilemap and bevy_kira_audio.",
        "Rust is a trademark of the Rust Foundation. This game is not \
         affiliated with or endorsed by the Rust Foundation or the Rust Project.",
    ],
};

/// What the full game adds after the demo, one line per act. Only acts
/// that are built and in the registry belong here (Acts 3-10, all of
/// them); do not list features that do not exist.
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
    (
        "Act 9 - The Forbidden Library",
        "unsafe and raw pointers, extern \"C\", macro_rules!, Drop, Send and Sync, MaybeUninit",
    ),
    (
        "Act 10 - The Throne",
        "measuring allocations, Cow, const fn, chunks_exact, no_std, Cargo's caret rule",
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
            "The full game has {more} more missions across Acts 3 to 10:"
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

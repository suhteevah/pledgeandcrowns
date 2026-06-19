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
use crate::plugins::state::GameState;
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
    /// Mission id that must be cleared before this one is unlocked. The
    /// first mission has `None`. Set linearly in `MissionRegistry::default`
    /// — registry tests enforce no cycles, no dangling references, full
    /// reachability from a no-prereq root.
    pub prereq: Option<&'static str>,
    pub starter_code: &'static str,
}

impl Mission {
    /// True when the player can start this mission. Cleared missions are
    /// always considered unlocked (so the completion-view revisit flow
    /// works regardless of progression rules).
    pub fn is_unlocked(&self, progress: &MissionProgress) -> bool {
        match self.prereq {
            None => true,
            Some(prev) => progress.is_cleared(prev),
        }
    }
}

#[derive(Resource)]
pub struct MissionRegistry {
    pub missions: Vec<Mission>,
}

impl MissionRegistry {
    /// First not-yet-cleared mission in registry order. Drives the "next
    /// mission" hint in the HUD so players know where to head.
    pub fn next_locked(&self, progress: &MissionProgress) -> Option<&Mission> {
        self.missions.iter().find(|m| !progress.is_cleared(m.id))
    }

    pub fn find(&self, id: &str) -> Option<&Mission> {
        self.missions.iter().find(|m| m.id == id)
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
`pub` to expose them. Construction uses the `TypeName { field: value, .. }` \
literal; access uses `instance.field`.\n\n\
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
Build a `Vec<i32>`, then sum it via `.iter().sum()` and print the total.\n\n\
## Hint\n\
The grader looks for `vec!`, `.iter()`, and `.sum`. The annotation on \
the binding (or a turbofish on `sum`) is required for the code to \
actually compile, but the grader only checks tokens.",
                prereq: None,
                starter_code: "fn main() {\n    // build a Vec<i32>, sum it via .iter().sum(), print the total\n    println!(\"todo\");\n}\n",
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
you'd be reassigning the local `x` binding (the reference itself), not \
the value it points at.\n\n\
## Task\n\
Write `fn bump(x: &mut i32)` that sets `*x` to a new value. Then call it \
from `main` against a mutable local.\n\n\
## Hint\n\
The grader requires `fn bump`, the parameter type `&mut i32`, and a \
`*x` deref in the body.",
                prereq: None,
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
returns their sum. Then call it from `main` and print the result.\n\n\
## Hint\n\
The grader requires `let add`, the closure-literal opener `= |`, and \
the body fragment `+ b`.",
                prereq: None,
                starter_code: "fn main() {\n    let _todo = 0;\n    // bind a closure named `add` that returns a + b, then call it\n    println!(\"{_todo}\");\n}\n",
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
The closure receives `&i32` here; arithmetic auto-derefs so `x * 2` \
works without an explicit `*x`.\n\n\
## Task\n\
Build a `Vec<i32>`, map each element through a `|x|` closure that \
doubles it, and collect the result into a new `Vec`.\n\n\
## Hint\n\
The grader requires `.map(`, `.collect`, and a single-element closure \
binder `|x|`.",
                prereq: None,
                starter_code: "fn main() {\n    let v = vec![1, 2, 3];\n    // chain iter / map / collect into a new Vec\n    let _ = v;\n}\n",
            },
            Mission {
                id: "enum_match",
                npc_name: "The Heraldic Sage",
                prompt: "Define `enum Direction` with several variants and match on it.",
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
Define `enum Direction` with at least two variants, then write a function \
that takes a `Direction` and uses `match` to map each variant to a value.\n\n\
## Hint\n\
The grader requires `enum Direction`, a `match ` expression, and at \
least one `Direction::` variant path.",
                prereq: None,
                starter_code: "// define enum Direction with several variants\n\nfn main() {\n    // match on a Direction value and produce something\n}\n",
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
                starter_code: "fn main() {\n    let mut _n = 5;\n    // count down with a while loop and a -= 1 step\n    println!(\"{_n}\");\n}\n",
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
`Self { level }` is shorthand for `Hero { level }` when the field and the \
local share a name.\n\n\
## Task\n\
Give `Hero` an associated function `new(level: i32) -> Self`, then build a \
hero with `Hero::new(5)` and print its level.\n\n\
## Hint\n\
The grader needs `fn new`, `Self`, and a `::new` call. The Recruiter forges \
a member from nothing but a name.",
                prereq: None,
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
                starter_code: "fn value(item: i32) -> i32 {\n    // define an `Item` type with two kinds (a weapon carrying damage, a potion carrying heal),\n    // take an Item here instead of an i32, and handle every kind to return its number\n    item\n}\n\nfn main() {\n    let _ = value(0);\n}\n",
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
`Pair::<i32> { .. }` explicitly.\n\n\
## Task\n\
Define `struct Pair<T>` with two fields of type `T`, build one from two \
integers, and read both fields.\n\n\
## Hint\n\
The grader needs `struct `, the parameter `<T>`, and a field typed `: T`. \
A vessel for any type — so long as both halves agree.",
                prereq: None,
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
`Box::new` moves each value to the heap and erases its concrete type down \
to `dyn Element`.\n\n\
## Task\n\
`Element`, `Fire`, and `Water` are provided. Build a \
`Vec<Box<dyn Element>>` holding one of each, then print every `name()`.\n\n\
## Hint\n\
The grader needs `Box<dyn` and `dyn `. Many shapes, one cage — dispatched \
at a touch.",
                prereq: None,
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
                starter_code: "trait Producer {\n    fn make(&self) -> i32;\n}\n\nfn main() {\n    // change Producer so each implementor names its own output type (an associated type),\n    // then implement it for a Coiner that makes an i32\n    let _ = ();\n}\n",
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
```\nfn half(n: i32) -> Option<i32> {\n    if n % 2 == 0 { Some(n / 2) } else { None }\n}\nSome(8).and_then(half).and_then(half);  // Some(2)\nSome(8).and_then(half).and_then(half).and_then(half);  // None (3 is odd)\n```\n\
`Result::and_then` works the same way over `Ok`/`Err`.\n\n\
## Task\n\
`half` is given. Chain it on `Some(8)` with `.and_then` (at least once) so \
an odd intermediate short-circuits to `None`.\n\n\
## Hint\n\
The grader needs `Option` and `.and_then(`. The Tabkeeper runs the tab — \
one unpaid round and the whole chain is cut.",
                prereq: None,
                starter_code: "fn half(n: i32) -> Option<i32> {\n    if n % 2 == 0 { Some(n / 2) } else { None }\n}\n\nfn main() {\n    // chain `half` on Some(8) so the first failure short-circuits to None\n    let _ = half(8);\n}\n",
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
                starter_code: "fn value(o: Option<i32>) -> i32 {\n    // unwrap the option, computing the fallback lazily with a closure (only if needed)\n    let _ = o;\n    0\n}\n\nfn main() {\n    let _ = value(None);\n}\n",
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
Bring `HashMap` into scope, build one, `insert` a value under a string key, \
then `get` it back.\n\n\
## Hint\n\
The grader needs `HashMap`, `.insert(`, and `.get(`. The Keymaster files \
every store under its own key.",
                prereq: None,
                starter_code: "fn main() {\n    // store a value under a string key in a map, then look it up by that key\n    let _ = ();\n}\n",
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
                starter_code: "fn main() {\n    let name = String::from(\"Garin\");\n    // capture `name` BY VALUE into a closure (so the closure owns it), then call it\n    let _ = &name;\n}\n",
            },
        ];
        // Strict-linear progression: each mission's prereq is the one
        // listed immediately before it. Shape decision logged in HANDOFF
        // (2026-05-03 What's Next item 1) — branching can come later
        // when the registry has Acts 3+ to organize.
        for i in 1..missions.len() {
            missions[i].prereq = Some(missions[i - 1].id);
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
            .add_message::<MissionLockedAttempt>()
            .add_systems(
                Update,
                (
                    handle_interact_key,
                    dismiss_completion_on_escape,
                    dismiss_epilogue_on_escape,
                )
                    .run_if(in_state(GameState::InGame)),
            )
            .add_systems(
                EguiPrimaryContextPass,
                (
                    draw_interaction_prompt,
                    draw_completion_panel,
                    draw_epilogue_panel,
                )
                    .run_if(in_state(GameState::InGame)),
            );
    }
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
    if !keys.just_pressed(KeyCode::KeyF) {
        return;
    }
    let Some(entry) = nearby.current.as_ref() else {
        return;
    };
    let Some(mission) = registry
        .missions
        .iter()
        .find(|m| m.id == entry.mission_id)
        .cloned()
    else {
        tracing::warn!(
            "nearby NPC {} references unknown mission {}",
            entry.name,
            entry.mission_id
        );
        return;
    };

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

fn draw_interaction_prompt(
    mut contexts: EguiContexts,
    nearby: Res<NearbyNpc>,
    editor: Res<EditorState>,
    active: Res<ActiveMission>,
    progress: Res<MissionProgress>,
    completion: Res<CompletionView>,
    registry: Res<MissionRegistry>,
) {
    // Don't double up — the completion panel and editor each own the
    // screen real estate for their own flows.
    if editor.open || completion.mission_id.is_some() {
        return;
    }
    let Some(entry) = nearby.current.as_ref() else {
        return;
    };
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let cleared = progress.is_cleared(entry.mission_id);
    let mission = registry.find(entry.mission_id);
    let unlocked = mission.is_some_and(|m| m.is_unlocked(&progress));
    let prereq_npc_name = mission
        .and_then(|m| m.prereq)
        .and_then(|pid| registry.find(pid))
        .map(|m| m.npc_name);

    egui::Window::new("interact")
        .title_bar(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_BOTTOM, egui::Vec2::new(0.0, -24.0))
        .show(ctx, |ui| {
            if cleared {
                ui.label(format!("[done] [F] talk to {}", entry.name));
            } else if !unlocked {
                let hint = prereq_npc_name.unwrap_or("the previous mission");
                ui.label(format!("[locked — clear {hint} first]"));
            } else {
                ui.label(format!("[F] talk to {}", entry.name));
            }
            if let Some(m) = active.current.as_ref().filter(|m| m.id == entry.mission_id) {
                ui.small(m.prompt);
            }
            ui.small(format!("missions cleared: {}", progress.cleared_count()));
        });

    // Persistent "next mission" HUD pin — visible regardless of which
    // NPC the player is standing next to, so they always know where to
    // head if they wander off the path.
    if let Some(next) = registry.next_locked(&progress) {
        egui::Window::new("next_mission")
            .title_bar(false)
            .resizable(false)
            .anchor(egui::Align2::LEFT_TOP, egui::Vec2::new(8.0, 8.0))
            .show(ctx, |ui| {
                ui.small(format!("next: {} · find {}", next.id, next.npc_name));
            });
    }
}

fn draw_completion_panel(
    mut contexts: EguiContexts,
    completion: Res<CompletionView>,
    registry: Res<MissionRegistry>,
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

    egui::Window::new("mission complete")
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .min_width(420.0)
        .show(ctx, |ui| {
            ui.heading(format!("✓ {}", mission.npc_name));
            ui.small(format!("encounter: {}", mission.id));
            ui.separator();
            ui.label(mission.prompt);
            ui.separator();
            // Show the Concept section of the tutorial as a recap. It's
            // the section players most often want to re-read; the others
            // (Syntax/Task/Hint) are revealed again when they reopen the
            // editor.
            ui.small(extract_section(mission.tutorial, "## Concept"));
            ui.separator();
            ui.label("[F] revisit  ·  [Esc] close");
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
    {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    egui::Window::new("epilogue")
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .min_width(520.0)
        .show(ctx, |ui| {
            ui.heading("The realm pledges its crown.");
            ui.separator();
            ui.label(
                "You have walked every road of the village and answered every \
                 sworn question. The compiler bows. The Borrow Checker stands \
                 down. The banners come in.",
            );
            ui.add_space(6.0);
            ui.label(format!(
                "{} of {} encounters cleared.",
                progress.cleared_count(),
                registry.missions.len()
            ));
            ui.add_space(6.0);
            ui.small(
                "More acts are being forged. For now: revisit any NPC with [F] \
                 to re-read a lesson, or wander the village.",
            );
            ui.separator();
            ui.label("[Esc] close");
        });
}

/// Pull the body of a `## Heading` section out of a tutorial string —
/// inclusive of the heading line, exclusive of the next `## ` line. If
/// the section is missing, the full tutorial is returned (better to
/// over-show than to silently drop the recap).
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

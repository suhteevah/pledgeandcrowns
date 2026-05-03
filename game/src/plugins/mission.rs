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
    pub starter_code: &'static str,
}

#[derive(Resource)]
pub struct MissionRegistry {
    pub missions: Vec<Mission>,
}

impl Default for MissionRegistry {
    fn default() -> Self {
        Self {
            missions: vec![
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
                    starter_code: "fn main() {\n    let mut _n = 5;\n    // count down with a while loop and a -= 1 step\n    println!(\"{_n}\");\n}\n",
                },
            ],
        }
    }
}

pub struct MissionPlugin;

impl Plugin for MissionPlugin {
    fn build(&self, app: &mut App) {
        tracing::debug!("MissionPlugin::build");
        app.init_resource::<MissionRegistry>()
            .init_resource::<ActiveMission>()
            .init_resource::<CompletionView>()
            .add_systems(
                Update,
                (handle_interact_key, dismiss_completion_on_escape)
                    .run_if(in_state(GameState::InGame)),
            )
            .add_systems(
                EguiPrimaryContextPass,
                (draw_interaction_prompt, draw_completion_panel)
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

fn handle_interact_key(
    keys: Res<ButtonInput<KeyCode>>,
    nearby: Res<NearbyNpc>,
    registry: Res<MissionRegistry>,
    progress: Res<MissionProgress>,
    mut editor: ResMut<EditorState>,
    mut active: ResMut<ActiveMission>,
    mut completion: ResMut<CompletionView>,
) {
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
    let viewing_this = completion
        .mission_id
        .as_deref()
        .is_some_and(|id| id == mission.id);

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

fn draw_interaction_prompt(
    mut contexts: EguiContexts,
    nearby: Res<NearbyNpc>,
    editor: Res<EditorState>,
    active: Res<ActiveMission>,
    progress: Res<MissionProgress>,
    completion: Res<CompletionView>,
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
    let mark = if cleared { "[done] " } else { "" };

    egui::Window::new("interact")
        .title_bar(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_BOTTOM, egui::Vec2::new(0.0, -24.0))
        .show(ctx, |ui| {
            ui.label(format!("{mark}[F] talk to {}", entry.name));
            if let Some(m) = active.current.as_ref().filter(|m| m.id == entry.mission_id) {
                ui.small(m.prompt);
            }
            ui.small(format!("missions cleared: {}", progress.cleared_count()));
        });
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

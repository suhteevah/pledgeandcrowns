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
Functions in Rust are declared with `fn`. Parameters carry an explicit \
type — there is no implicit casting between numeric types. The return \
type follows `->`. The last expression in the body is the return value \
when there's no trailing semicolon (Rust calls this an *implicit return*).\n\n\
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
The grader requires both `let mut` and `+= 1` to appear in the source.",
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
The grader requires `if `, `else`, and a literal comparison `< 0` to \
appear. The canonical body is a three-arm `if` / `else if` / `else`.",
                    starter_code: "fn sign(_n: i32) -> &'static str {\n    // three branches, one per sign\n    \"replace me\"\n}\n\nfn main() {\n    println!(\"{}\", sign(-3));\n}\n",
                },
                Mission {
                    id: "loop_break",
                    npc_name: "The Bellringer",
                    prompt: "Use `loop` to find the first power of two at or above one hundred.",
                    tutorial: "## Concept\n\
Rust has three loop constructs: `loop` (infinite), `while` (predicate), \
and `for` (iterator). The infinite `loop` is the only one that can return \
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
            .add_systems(
                Update,
                handle_interact_key.run_if(in_state(GameState::InGame)),
            )
            .add_systems(
                EguiPrimaryContextPass,
                draw_interaction_prompt.run_if(in_state(GameState::InGame)),
            );
    }
}

#[derive(Resource, Default)]
pub struct ActiveMission {
    pub current: Option<Mission>,
}

fn handle_interact_key(
    keys: Res<ButtonInput<KeyCode>>,
    nearby: Res<NearbyNpc>,
    registry: Res<MissionRegistry>,
    mut editor: ResMut<EditorState>,
    mut active: ResMut<ActiveMission>,
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

    tracing::info!(
        "starting mission {} from {} ({})",
        mission.id,
        mission.npc_name,
        entry.name
    );

    editor.source = mission.starter_code.to_string();
    editor.last_compile_result = None;
    editor.encounter_id = mission.id.to_string();
    editor.open = true;
    active.current = Some(mission);
}

fn draw_interaction_prompt(
    mut contexts: EguiContexts,
    nearby: Res<NearbyNpc>,
    editor: Res<EditorState>,
    active: Res<ActiveMission>,
    progress: Res<MissionProgress>,
) {
    if editor.open {
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

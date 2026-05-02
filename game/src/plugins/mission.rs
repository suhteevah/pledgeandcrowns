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
                Mission {
                    id: "vec_iter",
                    npc_name: "The Cooper",
                    prompt: "Sum the elements of a `Vec<i32>` using `.iter().sum()`.",
                    tutorial: "## Concept\n\
A `Vec<T>` is the heap-allocated growable list — Rust's analogue of \
Python `list` or JS `Array`. The standard way to walk it is to call \
`.iter()` for an immutable borrow of each element, then chain an \
adapter (`.map`, `.filter`) and a consumer (`.sum`, `.collect`, \
`.count`). Iterators are zero-cost: the compiler usually fuses them \
into the same machine code a hand-written loop would produce.\n\n\
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
                    id: "while_loop",
                    npc_name: "The Tinker",
                    prompt: "Use a `while` loop to count down from five to zero.",
                    tutorial: "## Concept\n\
`while` runs its body as long as a predicate stays true. It's the right \
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

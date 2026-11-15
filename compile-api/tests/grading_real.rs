// SPDX-License-Identifier: AGPL-3.0-or-later
//! Real-build tests for `/compile-real` grading ([`expectations`]).
//!
//! Slow (each case is a `cargo build --target wasm32-wasip1` plus a
//! sandboxed run), so `#[ignore]`d:
//!
//! ```text
//! cargo test -p pledge_compile_api --test grading_real -- --ignored
//! ```

use pledge_compile_api::expectations::{self, GradeState, GradedRun};
use pledge_compile_api::wasm_builder::RunLimits;
use pledge_compile_api::{CompileResponse, make_router, serve};

fn grade(id: &str, src: &str) -> GradedRun {
    expectations::grade(id, src, &RunLimits::default()).expect("grading setup")
}

/// rustc's primary `src/main.rs` lines in `stderr` (server numbering).
fn main_rs_error_lines(stderr: &str) -> Vec<usize> {
    stderr
        .lines()
        .filter_map(|l| l.trim_start().strip_prefix("--> "))
        .filter_map(|loc| {
            let loc = loc.replace('\\', "/");
            let mut parts = loc.rsplitn(3, ':');
            let _col = parts.next()?;
            let line: usize = parts.next()?.parse().ok()?;
            let file = parts.next()?;
            file.ends_with("src/main.rs").then_some(line)
        })
        .collect()
}

fn assert_no_checker_internals(text: &str) {
    for needle in ["__pnc", "pnc_entry", "PNC", "NONCE"] {
        assert!(!text.contains(needle), "leaked `{needle}`:\n{text}");
    }
}

#[test]
#[ignore = "slow: real wasm build"]
fn stdout_mission_passes_and_fails_on_wrong_output() {
    let pass = grade(
        "loop_break",
        "fn main() {\r\n    let mut n = 1;\r\n    let r = loop { if n >= 100 { break n; } n *= 2; };\r\n    println!(\"{r}   \");\r\n}\r\n",
    );
    assert_eq!(pass.state, GradeState::Passed, "{pass:?}");
    assert!(pass.ok());

    let wrong = grade(
        "loop_break",
        "fn main() { let mut n = 1; while n < 100 { n *= 3; } println!(\"{n}\"); }",
    );
    assert_eq!(wrong.state, GradeState::WrongOutput, "{wrong:?}");
    assert!(!wrong.ok());
    assert_eq!(wrong.expected.as_deref(), Some("a line reading `128`"));
    assert_eq!(wrong.actual.as_deref(), Some("243"));
    assert!(
        wrong
            .outcome
            .stderr
            .contains("Expected a line reading `128`")
    );
    assert!(wrong.outcome.stderr.contains("243"));
}

#[test]
#[ignore = "slow: real wasm build"]
fn harness_mission_passes_and_catches_swapped_labels() {
    let pass = grade(
        "if_else_sign",
        // No `fn main` at all: the checker does not need one here.
        "fn sign(n: i32) -> &'static str { if n < 0 { \"neg\" } else if n == 0 { \"zero\" } else { \"pos\" } }",
    );
    assert_eq!(pass.state, GradeState::Passed, "{pass:?}");

    let wrong = grade(
        "if_else_sign",
        "fn sign(n: i32) -> &'static str { if n < 0 { \"pos\" } else if n == 0 { \"zero\" } else { \"neg\" } }\nfn main() {}",
    );
    assert_eq!(wrong.state, GradeState::WrongOutput, "{wrong:?}");
    let expected = wrong.expected.clone().unwrap_or_default();
    let actual = wrong.actual.clone().unwrap_or_default();
    assert!(
        expected.contains("sign(-3) should give \"neg\""),
        "{expected}"
    );
    assert!(actual.contains("sign(-3) gave \"pos\""), "{actual}");
    assert_no_checker_internals(&wrong.outcome.stdout);
    assert_no_checker_internals(&wrong.outcome.stderr);
}

#[test]
#[ignore = "slow: real wasm build"]
fn compile_error_line_numbers_are_unchanged_in_both_layouts() {
    // Player line 3 holds the error. With the one-line preamble rustc
    // reports src/main.rs:4, which the client maps back with
    // SERVER_PREAMBLE_LINES = 1.
    let src = "fn main() {\n    let x: i32 = 1;\n    let y: i32 = \"nope\";\n}\n";
    let plain = grade("intro_let_binding", src);
    assert_eq!(plain.state, GradeState::CompileError);
    assert_eq!(
        main_rs_error_lines(&plain.outcome.stderr),
        vec![4],
        "{}",
        plain.outcome.stderr
    );

    let harness_src = "fn sign(n: i32) -> &'static str {\n    \"neg\"\n}\nfn broken() { let y: i32 = \"nope\"; }\n";
    let h = grade("if_else_sign", harness_src);
    assert_eq!(h.state, GradeState::CompileError);
    assert_eq!(
        main_rs_error_lines(&h.outcome.stderr),
        vec![5],
        "{}",
        h.outcome.stderr
    );
    assert_eq!(expectations::PLAYER_PREAMBLE_LINES, 1);
    assert_no_checker_internals(&h.outcome.stderr);
}

#[test]
#[ignore = "slow: real wasm build"]
fn player_breaking_the_checker_sees_the_contract_not_internals() {
    let g = grade(
        "if_else_sign",
        "fn sign(n: i64) -> String { String::from(\"neg\") }\nfn main() {}",
    );
    assert_eq!(g.state, GradeState::CompileError, "{g:?}");
    assert!(
        g.outcome
            .stderr
            .contains("the mission checker could not use your code"),
        "{}",
        g.outcome.stderr
    );
    assert!(g.outcome.stderr.contains("fn sign(n: i32) -> &'static str"));
    assert_no_checker_internals(&g.outcome.stderr);

    // Missing item entirely.
    let g = grade("match_option", "fn main() { println!(\"0\"); }");
    assert_eq!(g.state, GradeState::CompileError);
    assert!(g.outcome.stderr.contains("unwrap_or_zero"));
    assert_no_checker_internals(&g.outcome.stderr);
}

#[test]
#[ignore = "slow: real wasm build"]
fn harness_names_do_not_collide_with_player_names() {
    // The player defines main, and items named like checker internals.
    let src = r#"
macro_rules! println { ($($t:tt)*) => { ::std::print!("shadowed\n") }; }
fn emit(_: &str, _: i32) {}
fn mark(_: &str) {}
fn run() {}
const NONCE: &str = "guess";
mod harness { pub fn emit() {} }
fn double(n: i32) -> i32 { n * 2 }
fn main() { std::println!("{}", double(21)); }
"#;
    let g = grade("double_function", src);
    assert_eq!(g.state, GradeState::Passed, "{g:?}");
}

#[test]
#[ignore = "slow: real wasm build"]
fn reserved_prefix_and_forged_results_are_refused() {
    let g = grade(
        "double_function",
        "mod __pnc_harness {}\nfn double(n: i32) -> i32 { n * 2 }\nfn main() {}",
    );
    assert_eq!(g.state, GradeState::CompileError);
    assert!(g.outcome.stderr.contains("reserved"));

    // Printing the expected result lines does not count: only lines with
    // the per-request nonce do.
    let forged = grade(
        "double_function",
        "fn double(n: i32) -> i32 { println!(\"double(21) = 42\"); 0 }\nfn main() { println!(\"42\"); }",
    );
    assert_eq!(forged.state, GradeState::WrongOutput, "{forged:?}");

    // Exiting early stops the checker: not a pass.
    let exit = grade(
        "match_option",
        "fn unwrap_or_zero(x: Option<i32>) -> i32 { std::process::exit(0) }",
    );
    assert_eq!(exit.state, GradeState::WrongOutput, "{exit:?}");
    assert!(
        exit.outcome
            .stderr
            .contains("exited before the checker finished")
    );
}

#[test]
#[ignore = "slow: real wasm build"]
fn infinite_loop_in_checked_function_is_a_runtime_error() {
    let g = grade(
        "option_map",
        "fn add_one(o: Option<i32>) -> Option<i32> { loop {} }",
    );
    assert_eq!(g.state, GradeState::RuntimeError, "{g:?}");
    let e = g.outcome.stderr.to_lowercase();
    assert!(e.contains("fuel") || e.contains("timeout"), "{e}");
    assert_no_checker_internals(&g.outcome.stdout);
}

#[test]
#[ignore = "slow: real wasm build"]
fn thread_missions_compile_but_are_not_run_or_passed_as_checked() {
    let src = "use std::thread;\nfn main() { let h = thread::spawn(|| 21 * 2); println!(\"{}\", h.join().unwrap()); }";
    let g = grade("thread_spawn", src);
    assert_eq!(g.state, GradeState::CompiledNotChecked, "{g:?}");
    assert!(g.ok());
    assert!(!g.state.behaviour_checked());
    assert!(g.outcome.run_time.is_none(), "must not be run");
    assert!(g.note.contains("not checked"));

    let resp = pledge_compile_api::real_response(g);
    assert!(resp.ok);
    assert_eq!(resp.behaviour_checked, Some(false));
    assert!(resp.stdout.contains("-- not checked"), "{}", resp.stdout);
    assert!(
        !resp.stdout.contains("checked: does what"),
        "{}",
        resp.stdout
    );

    let no_join = grade(
        "thread_spawn",
        "use std::thread;\nfn main() { let _h = thread::spawn(|| 1); }",
    );
    assert_eq!(no_join.state, GradeState::StaticCheckFailed);
    assert!(!no_join.ok());

    let broken = grade(
        "thread_scope",
        "fn main() { std::thread::scope(|s| { s.spawn(|| x); }); }",
    );
    assert_eq!(broken.state, GradeState::CompileError);
}

#[test]
#[ignore = "slow: real wasm build"]
fn unknown_and_run_only_missions_say_output_was_not_checked() {
    let g = grade("no_such_mission", "fn main() { println!(\"hi\"); }");
    assert_eq!(g.state, GradeState::RanNotChecked);
    assert!(g.ok());
    assert!(g.note.contains("no output check"), "{}", g.note);

    // Run-only missions: since 6642ca4 every runnable mission pins its
    // output, so none may exist (this used `vec_iter`, which now checks
    // for `60`). Grade one if the registry ever gains one again.
    let run_only = expectations::all()
        .iter()
        .find(|m| matches!(m.check, expectations::Check::RunOnly { .. }));
    match run_only {
        Some(m) => {
            let g = grade(m.id, "fn main() { println!(\"hi\"); }");
            assert_eq!(g.state, GradeState::RanNotChecked, "{}", m.id);
            assert!(g.note.contains("output not checked"));
        }
        None => println!("no run-only mission in the registry; that half is vacuous"),
    }
}

#[tokio::test]
#[ignore = "slow: real wasm build"]
async fn http_response_carries_the_additive_fields() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { serve(listener, make_router()).await.unwrap() });
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    let client = reqwest::Client::new();
    let post = |id: &'static str, src: &'static str| {
        let client = client.clone();
        async move {
            client
                .post(format!("http://{addr}/compile-real"))
                .json(&serde_json::json!({"source": src, "encounter_id": id}))
                .send()
                .await
                .unwrap()
                .json::<serde_json::Value>()
                .await
                .unwrap()
        }
    };

    let v = post(
        "option_map",
        "fn add_one(o: Option<i32>) -> Option<i32> { o }",
    )
    .await;
    assert_eq!(v["ok"], false);
    assert_eq!(v["grade"], "wrong_output");
    assert_eq!(v["behaviour_checked"], true);
    assert!(v["expected"].as_str().unwrap().contains("Some(4)"), "{v}");
    assert!(v["stdout"].as_str().unwrap().starts_with("[real]"));

    let v = post(
        "option_map",
        "fn add_one(o: Option<i32>) -> Option<i32> { o.map(|x| x + 1) }",
    )
    .await;
    assert_eq!(v["ok"], true, "{v}");
    assert_eq!(v["grade"], "passed");
    assert!(v.get("expected").is_none(), "absent, not null: {v}");

    // Old clients deserialise the body unchanged.
    let typed: CompileResponse = serde_json::from_value(v).unwrap();
    assert!(typed.ok);
}

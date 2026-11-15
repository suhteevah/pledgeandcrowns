// SPDX-License-Identifier: AGPL-3.0-or-later
//! Per-mission checks for `/compile-real`, one entry per registry mission.
//!
//! Recorded results (`expected` lines, and the values in stdout matchers)
//! were produced by running each mission's canonical solution through the
//! real build + sandbox path (`compile-api/tests/expectations_proof.rs`,
//! `print_canonical_transcripts`), not written by hand. The same suite
//! proves, for every entry with a behaviour check, that the canonical
//! solution passes, the mission's starter code fails, and at least one
//! wrong-but-compiling variant fails.
//!
//! Only the DATA lives here. The canonical solutions are MIT game code and
//! are read only by that dev-only test, never at runtime.
//!
//! Classification (see `expectations.rs` for the mechanisms):
//! A = `Stdout` (pinned output of the player's own `main`),
//! B = `Harness` (the mission asks for an item; the checker calls it),
//! C = `CompileOnly` (cannot be run meaningfully on wasm32-wasip1).
//! No entry is `RunOnly`: every runnable mission's text pins its output.
//! Entries built with `mu` also carry construct tokens (`uses`) checked
//! after the behaviour matched; `muf` adds tokens the code must not
//! contain (`forbids`).

use super::Check::{CompileOnly, Harness as H, Stdout};
use super::Matcher::*;
use super::{Harness, MissionCheck};

const fn m(id: &'static str, check: super::Check) -> MissionCheck {
    MissionCheck {
        id,
        check,
        uses: &[],
        forbids: &[],
    }
}

/// A behaviour check plus the construct tokens that must also appear
/// ([`MissionCheck::uses`]), for missions whose short pinned output could
/// otherwise be printed as literals.
const fn mu(
    id: &'static str,
    check: super::Check,
    uses: &'static [&'static [&'static str]],
) -> MissionCheck {
    MissionCheck {
        id,
        check,
        uses,
        forbids: &[],
    }
}

/// [`mu`] plus tokens the code must not contain
/// ([`MissionCheck::forbids`]), for a mission whose text rules a construct
/// out.
const fn muf(
    id: &'static str,
    check: super::Check,
    uses: &'static [&'static [&'static str]],
    forbids: &'static [&'static str],
) -> MissionCheck {
    MissionCheck {
        id,
        check,
        uses,
        forbids,
    }
}

const NO_THREADS: &str = "wasm32-wasip1 has no threads (std::thread::spawn panics there), \
     so this program is compiled and statically checked but never run";

pub(super) const CHECKS: &[MissionCheck] = &[
    // ── Prelude ──────────────────────────────────────────────────────
    m(
        "intro_let_binding",
        Stdout {
            matchers: &[HasLine("42")],
        },
    ),
    m(
        "double_function",
        H(Harness {
            contract: "`fn double(n: i32) -> i32`",
            run_main: true,
            body: "    let _sig: fn(i32) -> i32 = super::double;
    emit(\"double(21)\", super::double(21));
    emit(\"double(-4)\", super::double(-4));
    emit(\"double(0)\", super::double(0));",
            expected: &["double(21) = 42", "double(-4) = -8", "double(0) = 0"],
            main_matchers: &[HasLine("42")],
        }),
    ),
    m(
        "borrow_preview",
        Stdout {
            matchers: &[HasLine("sample")],
        },
    ),
    m(
        "mut_binding",
        Stdout {
            matchers: &[HasLine("1")],
        },
    ),
    m(
        "if_else_sign",
        H(Harness {
            contract: "`fn sign(n: i32) -> &'static str`",
            run_main: false,
            body: "    let _sig: fn(i32) -> &'static str = super::sign;
    emit(\"sign(-3)\", super::sign(-3));
    emit(\"sign(-1)\", super::sign(-1));
    emit(\"sign(0)\", super::sign(0));
    emit(\"sign(1)\", super::sign(1));
    emit(\"sign(7)\", super::sign(7));",
            expected: &[
                "sign(-3) = \"neg\"",
                "sign(-1) = \"neg\"",
                "sign(0) = \"zero\"",
                "sign(1) = \"pos\"",
                "sign(7) = \"pos\"",
            ],
            main_matchers: &[],
        }),
    ),
    m(
        "loop_break",
        Stdout {
            matchers: &[HasLine("128")],
        },
    ),
    m(
        "match_option",
        H(Harness {
            contract: "`fn unwrap_or_zero(x: Option<i32>) -> i32`",
            run_main: false,
            body: "    let _sig: fn(Option<i32>) -> i32 = super::unwrap_or_zero;
    emit(\"unwrap_or_zero(Some(7))\", super::unwrap_or_zero(Some(7)));
    emit(\"unwrap_or_zero(None)\", super::unwrap_or_zero(None));
    emit(\"unwrap_or_zero(Some(-2))\", super::unwrap_or_zero(Some(-2)));",
            expected: &[
                "unwrap_or_zero(Some(7)) = 7",
                "unwrap_or_zero(None) = 0",
                "unwrap_or_zero(Some(-2)) = -2",
            ],
            main_matchers: &[],
        }),
    ),
    m(
        "struct_basic",
        H(Harness {
            contract: "`struct Knight { name: String, hp: i32 }` and a `fn main`",
            run_main: true,
            body: "    let k = super::Knight { name: ::std::string::String::from(\"Pnc\"), hp: 9 };
    let name: &::std::string::String = &k.name;
    let hp: i32 = k.hp;
    emit(\"Knight.name\", name);
    emit(\"Knight.hp\", hp);",
            expected: &["Knight.name = \"Pnc\"", "Knight.hp = 9"],
            main_matchers: &[],
        }),
    ),
    mu(
        "vec_iter",
        Stdout {
            matchers: &[HasLine("60")],
        },
        &[&[".iter()"], &[".sum"]],
    ),
    m(
        "tuple_destructure",
        Stdout {
            matchers: &[LineContainsAll(&["3", "4"])],
        },
    ),
    m(
        "borrow_mut",
        H(Harness {
            contract: "`fn bump(x: &mut i32)` and a `fn main`",
            run_main: true,
            body: "    let _sig: fn(&mut i32) = super::bump;
    let mut a = 5;
    super::bump(&mut a);
    emit(\"bump changed 5\", a != 5);
    let mut b = 0;
    super::bump(&mut b);
    emit(\"bump changed 0\", b != 0);",
            expected: &["bump changed 5 = true", "bump changed 0 = true"],
            main_matchers: &[],
        }),
    ),
    m(
        "string_vs_str",
        H(Harness {
            contract: "`fn greet(name: &str)` and a `fn main`",
            run_main: true,
            body: "    let _sig: fn(&str) = super::greet;
    let owned = ::std::string::String::from(\"Pnc\");
    super::greet(&owned);
    super::greet(\"pnc\");
    emit(\"greet takes a &String and a literal\", true);",
            expected: &["greet takes a &String and a literal = true"],
            main_matchers: &[],
        }),
    ),
    m(
        "option_unwrap_or",
        H(Harness {
            contract: "`fn safe(x: Option<i32>) -> i32`",
            run_main: false,
            body: "    let _sig: fn(Option<i32>) -> i32 = super::safe;
    emit(\"safe(Some(5))\", super::safe(Some(5)));
    emit(\"safe(None)\", super::safe(None));
    emit(\"safe(Some(-3))\", super::safe(Some(-3)));",
            expected: &["safe(Some(5)) = 5", "safe(None) = 0", "safe(Some(-3)) = -3"],
            main_matchers: &[],
        }),
    ),
    m(
        "for_in_range",
        Stdout {
            matchers: &[Exact("0\n1\n2\n3\n4\n5\n6\n7\n8\n9")],
        },
    ),
    mu(
        "closure_basic",
        Stdout {
            matchers: &[HasLine("42")],
        },
        &[&["let add"], &["|"]],
    ),
    m(
        "slice_basic",
        H(Harness {
            contract: "`fn sum_slice(xs: &[i32]) -> i32`",
            run_main: false,
            body: "    let _sig: fn(&[i32]) -> i32 = super::sum_slice;
    emit(\"sum_slice(&[1, 2, 3])\", super::sum_slice(&[1, 2, 3]));
    let empty: [i32; 0] = [];
    emit(\"sum_slice(&[])\", super::sum_slice(&empty));
    let v = ::std::vec![4, 5, 6];
    emit(\"sum_slice(&vec![4, 5, 6])\", super::sum_slice(&v));
    emit(\"sum_slice(&[-2, 2])\", super::sum_slice(&[-2, 2]));",
            expected: &[
                "sum_slice(&[1, 2, 3]) = 6",
                "sum_slice(&[]) = 0",
                "sum_slice(&vec![4, 5, 6]) = 15",
                "sum_slice(&[-2, 2]) = 0",
            ],
            main_matchers: &[],
        }),
    ),
    m(
        "result_question_mark",
        H(Harness {
            contract: "`fn parse_int(s: &str) -> Result<i32, String>`",
            run_main: false,
            body: "    let _sig: fn(&str) -> Result<i32, ::std::string::String> = super::parse_int;
    emit(\"parse_int(\\\"42\\\")\", super::parse_int(\"42\"));
    emit(\"parse_int(\\\"-7\\\")\", super::parse_int(\"-7\"));
    emit(\"parse_int(\\\"forty-two\\\") is Err\", super::parse_int(\"forty-two\").is_err());",
            expected: &[
                "parse_int(\"42\") = Ok(42)",
                "parse_int(\"-7\") = Ok(-7)",
                "parse_int(\"forty-two\") is Err = true",
            ],
            main_matchers: &[],
        }),
    ),
    m(
        "derive_debug",
        H(Harness {
            contract: "`struct Item` implementing `Debug`, and a `fn main` that prints one with `{:?}`",
            run_main: true,
            body: "    fn needs_debug<T: ::core::fmt::Debug>() {}
    needs_debug::<super::Item>();
    emit(\"Item implements Debug\", true);",
            expected: &["Item implements Debug = true"],
            main_matchers: &[LineContains("Item {")],
        }),
    ),
    m(
        "iter_map_collect",
        Stdout {
            matchers: &[LineContains("[2, 4, 6]")],
        },
    ),
    mu(
        "enum_match",
        H(Harness {
            contract: "`enum Direction { North, East, South, West }` and                        `fn degrees(d: Direction) -> i32`",
            run_main: true,
            body: "    let _sig: fn(super::Direction) -> i32 = super::degrees;
    emit(\"degrees(North)\", super::degrees(super::Direction::North));
    emit(\"degrees(East)\", super::degrees(super::Direction::East));
    emit(\"degrees(South)\", super::degrees(super::Direction::South));
    emit(\"degrees(West)\", super::degrees(super::Direction::West));",
            expected: &[
                "degrees(North) = 0",
                "degrees(East) = 90",
                "degrees(South) = 180",
                "degrees(West) = 270",
            ],
            main_matchers: &[InOrder(&["0", "90", "180", "270"])],
        }),
        &[&["match"]],
    ),
    m(
        "while_loop",
        Stdout {
            matchers: &[LastLine("0")],
        },
    ),
    // ── Act 2 boss: the Borrow Checker's trial, The Bridge ───────────
    // Each stage's signature is pinned by the `_sig` coercion, so a
    // solution that compiles only by cloning (stage 1), or that ties the
    // answer to both inputs (stage 3), fails the checker build.
    m(
        "bridge_toll",
        H(Harness {
            contract: "`fn weigh(cargo: &str) -> usize` and a `fn main`",
            run_main: true,
            body: "    let _sig: fn(&str) -> usize = super::weigh;
    emit(\"weigh(\\\"lantern oil\\\")\", super::weigh(\"lantern oil\"));
    let cargo = ::std::string::String::from(\"salt\");
    let toll = super::weigh(&cargo);
    emit(\"weigh(&cargo)\", toll);
    emit(\"cargo after the toll\", cargo);",
            expected: &[
                "weigh(\"lantern oil\") = 11",
                "weigh(&cargo) = 4",
                "cargo after the toll = \"salt\"",
            ],
            main_matchers: &[Exact("toll: 11\ncargo: lantern oil")],
        }),
    ),
    m(
        "bridge_quill",
        H(Harness {
            contract: "`fn inscribe(scroll: &mut Vec<String>, entry: &str)` and a `fn main`",
            run_main: true,
            body: "    let _sig: fn(&mut ::std::vec::Vec<::std::string::String>, &str) = super::inscribe;
    let mut scroll = ::std::vec![::std::string::String::from(\"a\")];
    super::inscribe(&mut scroll, \"b\");
    super::inscribe(&mut scroll, \"c\");
    emit(\"scroll after two entries\", scroll);",
            expected: &["scroll after two entries = [\"a\", \"b\", \"c\"]"],
            main_matchers: &[Exact(
                "reader one: north road\nreader two: north road\nscroll: north road, east road",
            )],
        }),
    ),
    m(
        "bridge_lantern",
        H(Harness {
            contract: "`fn before<'a>(line: &'a str, sep: &str) -> &'a str` and a `fn main`",
            run_main: true,
            body: "    let _sig: for<'a, 'b> fn(&'a str, &'b str) -> &'a str = super::before;
    emit(\"before(\\\"oil:wick\\\", \\\":\\\")\", super::before(\"oil:wick\", \":\"));
    emit(\"before(\\\"glass\\\", \\\":\\\")\", super::before(\"glass\", \":\"));
    emit(\"before(\\\"a--b\\\", \\\"--\\\")\", super::before(\"a--b\", \"--\"));
    let line = ::std::string::String::from(\"north:south\");
    let answer;
    {
        let sep = ::std::string::String::from(\":\");
        answer = super::before(&line, &sep);
    }
    emit(\"answer outlives sep\", answer);",
            expected: &[
                "before(\"oil:wick\", \":\") = \"oil\"",
                "before(\"glass\", \":\") = \"glass\"",
                "before(\"a--b\", \"--\") = \"a\"",
                "answer outlives sep = \"north\"",
            ],
            main_matchers: &[Exact("first: oil\nwhole: glass")],
        }),
    ),
    // ── Act 3: The Guildhall Quarter ─────────────────────────────────
    m(
        "impl_method",
        H(Harness {
            contract: "`impl Hero { fn power(&self) -> i32 }` and a `fn main`",
            run_main: true,
            body: "    let h = super::Hero { level: 3 };
    let first: i32 = h.power();
    let second: i32 = h.power();
    emit(\"power() borrows, so it can be called twice\", first == second);",
            expected: &["power() borrows, so it can be called twice = true"],
            main_matchers: &[],
        }),
    ),
    m(
        "assoc_new",
        H(Harness {
            contract: "`impl Hero { fn new(level: i32) -> Self }`",
            run_main: false,
            body: "    let h: super::Hero = super::Hero::new(5);
    emit(\"Hero::new(5).level\", h.level);
    emit(\"Hero::new(-2).level\", super::Hero::new(-2).level);",
            expected: &["Hero::new(5).level = 5", "Hero::new(-2).level = -2"],
            main_matchers: &[],
        }),
    ),
    m(
        "if_let",
        Stdout {
            matchers: &[HasLine("7")],
        },
    ),
    m(
        "while_let",
        Stdout {
            matchers: &[InOrder(&["3", "2", "1"])],
        },
    ),
    m(
        "tuple_struct",
        Stdout {
            matchers: &[HasLine("3.5")],
        },
    ),
    m(
        "enum_data_match",
        H(Harness {
            contract: "`enum Item { Weapon { damage: i32 }, Potion { heal: i32 } }` and \
                       `fn value(item: Item) -> i32`",
            run_main: false,
            body: "    let _sig: fn(super::Item) -> i32 = super::value;
    emit(\"value(Weapon { damage: 10 })\", super::value(super::Item::Weapon { damage: 10 }));
    emit(\"value(Potion { heal: 5 })\", super::value(super::Item::Potion { heal: 5 }));
    emit(\"value(Weapon { damage: -1 })\", super::value(super::Item::Weapon { damage: -1 }));",
            expected: &[
                "value(Weapon { damage: 10 }) = 10",
                "value(Potion { heal: 5 }) = 5",
                "value(Weapon { damage: -1 }) = -1",
            ],
            main_matchers: &[],
        }),
    ),
    // ── Act 3 boss: the Guildmaster's trial, The Variant Wraith ──────
    // The checker names every variant, so the enum's shape is pinned;
    // each stage pins `counter`/`damage` and calls it on fixed forms. In
    // stage 3 the Possessed result depends on its `will`, which a blanket
    // `_` arm cannot read.
    m(
        "wraith_forms",
        H(Harness {
            contract: "`enum Wraith { Solid, Mist, Shadow }`, `fn counter(form: Wraith) -> &'static str` and a `fn main`",
            run_main: true,
            body: "    let _sig: fn(super::Wraith) -> &'static str = super::counter;
    emit(\"counter(Wraith::Solid)\", super::counter(super::Wraith::Solid));
    emit(\"counter(Wraith::Mist)\", super::counter(super::Wraith::Mist));
    emit(\"counter(Wraith::Shadow)\", super::counter(super::Wraith::Shadow));",
            expected: &[
                "counter(Wraith::Solid) = \"steel\"",
                "counter(Wraith::Mist) = \"wind\"",
                "counter(Wraith::Shadow) = \"light\"",
            ],
            main_matchers: &[Exact("Solid: steel\nMist: wind\nShadow: light")],
        }),
    ),
    m(
        "wraith_essence",
        H(Harness {
            contract: "`enum Wraith { Solid { armor: u32 }, Mist { density: u32 }, Shadow }`, `fn damage(form: Wraith, blow: u32) -> u32` and a `fn main`",
            run_main: true,
            body: "    let _sig: fn(super::Wraith, u32) -> u32 = super::damage;
    emit(\"damage(Solid { armor: 3 }, 10)\", super::damage(super::Wraith::Solid { armor: 3 }, 10));
    emit(\"damage(Solid { armor: 1 }, 6)\", super::damage(super::Wraith::Solid { armor: 1 }, 6));
    emit(\"damage(Mist { density: 2 }, 10)\", super::damage(super::Wraith::Mist { density: 2 }, 10));
    emit(\"damage(Mist { density: 3 }, 12)\", super::damage(super::Wraith::Mist { density: 3 }, 12));
    emit(\"damage(Shadow, 99)\", super::damage(super::Wraith::Shadow, 99));",
            expected: &[
                "damage(Solid { armor: 3 }, 10) = 7",
                "damage(Solid { armor: 1 }, 6) = 5",
                "damage(Mist { density: 2 }, 10) = 5",
                "damage(Mist { density: 3 }, 12) = 4",
                "damage(Shadow, 99) = 0",
            ],
            main_matchers: &[Exact(
                "Solid { armor: 3 } takes 7\nMist { density: 2 } takes 5\nShadow takes 0",
            )],
        }),
    ),
    m(
        "wraith_possessed",
        H(Harness {
            contract: "`enum Wraith { Solid { armor: u32 }, Mist { density: u32 }, Shadow, Possessed { will: u32 } }`, `fn damage(form: Wraith, blow: u32) -> u32` and a `fn main`",
            run_main: true,
            body: "    let _sig: fn(super::Wraith, u32) -> u32 = super::damage;
    emit(\"damage(Possessed { will: 4 }, 10)\", super::damage(super::Wraith::Possessed { will: 4 }, 10));
    emit(\"damage(Possessed { will: 1 }, 2)\", super::damage(super::Wraith::Possessed { will: 1 }, 2));
    emit(\"damage(Solid { armor: 3 }, 10)\", super::damage(super::Wraith::Solid { armor: 3 }, 10));
    emit(\"damage(Mist { density: 2 }, 10)\", super::damage(super::Wraith::Mist { density: 2 }, 10));
    emit(\"damage(Shadow, 99)\", super::damage(super::Wraith::Shadow, 99));",
            expected: &[
                "damage(Possessed { will: 4 }, 10) = 14",
                "damage(Possessed { will: 1 }, 2) = 3",
                "damage(Solid { armor: 3 }, 10) = 7",
                "damage(Mist { density: 2 }, 10) = 5",
                "damage(Shadow, 99) = 0",
            ],
            main_matchers: &[Exact(
                "Solid { armor: 3 } takes 7\nMist { density: 2 } takes 5\nShadow takes 0\nPossessed { will: 4 } takes 14",
            )],
        }),
    ),
    // ── Act 4: The Trait Mage's Tower ────────────────────────────────
    m(
        "trait_def",
        H(Harness {
            contract: "`trait Element { fn name(&self) -> &str; }` implemented for the unit \
                       struct `Fire`, and a `fn main`",
            run_main: true,
            body: "    fn name_of<T: super::Element>(t: &T) -> &str { t.name() }
    let _n: &str = name_of(&super::Fire);
    emit(\"Fire implements Element\", true);",
            expected: &["Fire implements Element = true"],
            main_matchers: &[],
        }),
    ),
    m(
        "generic_fn",
        H(Harness {
            contract: "`fn larger<T: PartialOrd>(a: T, b: T) -> T`",
            run_main: false,
            body: "    emit(\"larger(3, 7)\", super::larger(3, 7));
    emit(\"larger(7, 3)\", super::larger(7, 3));
    emit(\"larger(1.5, 0.5)\", super::larger(1.5, 0.5));
    emit(\"larger('a', 'b')\", super::larger('a', 'b'));",
            expected: &[
                "larger(3, 7) = 7",
                "larger(7, 3) = 7",
                "larger(1.5, 0.5) = 1.5",
                "larger('a', 'b') = 'b'",
            ],
            main_matchers: &[],
        }),
    ),
    m(
        "generic_struct",
        H(Harness {
            contract: "`struct Pair<T> { a: T, b: T }`",
            run_main: false,
            body: "    let p = super::Pair { a: 1, b: 2 };
    emit(\"Pair { a: 1, b: 2 }.a\", p.a);
    emit(\"Pair { a: 1, b: 2 }.b\", p.b);
    let q = super::Pair { a: \"x\", b: \"y\" };
    emit(\"Pair { a: \\\"x\\\", b: \\\"y\\\" }.b\", q.b);",
            expected: &[
                "Pair { a: 1, b: 2 }.a = 1",
                "Pair { a: 1, b: 2 }.b = 2",
                "Pair { a: \"x\", b: \"y\" }.b = \"y\"",
            ],
            main_matchers: &[],
        }),
    ),
    m(
        "dyn_trait",
        Stdout {
            matchers: &[HasLine("fire"), HasLine("water")],
        },
    ),
    m(
        "lifetimes",
        H(Harness {
            contract: "`fn longest<'a>(x: &'a str, y: &'a str) -> &'a str`",
            run_main: false,
            body: "    let _sig: for<'a> fn(&'a str, &'a str) -> &'a str = super::longest;
    emit(\"longest(\\\"aa\\\", \\\"b\\\")\", super::longest(\"aa\", \"b\"));
    emit(\"longest(\\\"a\\\", \\\"bbb\\\")\", super::longest(\"a\", \"bbb\"));",
            expected: &[
                "longest(\"aa\", \"b\") = \"aa\"",
                "longest(\"a\", \"bbb\") = \"bbb\"",
            ],
            main_matchers: &[],
        }),
    ),
    m(
        "assoc_type",
        H(Harness {
            contract: "`trait Producer { type Output; fn make(&self) -> Self::Output; }` \
                       implemented for the unit struct `Coiner` with `Output = i32`",
            run_main: false,
            body: "    fn make_i32<P: super::Producer<Output = i32>>(p: &P) -> i32 { p.make() }
    let _v: i32 = make_i32(&super::Coiner);
    emit(\"Coiner is a Producer<Output = i32>\", true);",
            expected: &["Coiner is a Producer<Output = i32> = true"],
            main_matchers: &[],
        }),
    ),
    // ── Act 4 boss: Vexis's trial, The Chimera of Vexis ──────────────
    // Stage 1 calls the methods through `<Head as Enemy>`, so inherent
    // methods do not count. Stages 2 and 3 implement the player's trait
    // for a checker-owned head (`Wisp`) and pass it in: only a bounded
    // generic (stage 2) or trait-object code (stage 3) accepts a head the
    // player never saw.
    m(
        "chimera_heads",
        H(Harness {
            contract: "`trait Enemy { fn name(&self) -> &'static str; fn breath(&self) -> u32; }` implemented for `Fire`, `Frost` and `Storm`, and a `fn main`",
            run_main: true,
            body: "    let _frost: fn(&super::Frost) -> u32 = <super::Frost as super::Enemy>::breath;
    let _storm: fn(&super::Storm) -> u32 = <super::Storm as super::Enemy>::breath;
    emit(\"Frost name\", <super::Frost as super::Enemy>::name(&super::Frost));
    emit(\"Frost breath\", <super::Frost as super::Enemy>::breath(&super::Frost));
    emit(\"Storm name\", <super::Storm as super::Enemy>::name(&super::Storm));
    emit(\"Storm breath\", <super::Storm as super::Enemy>::breath(&super::Storm));
    emit(\"Fire breath\", <super::Fire as super::Enemy>::breath(&super::Fire));",
            expected: &[
                "Frost name = \"frost\"",
                "Frost breath = 5",
                "Storm name = \"storm\"",
                "Storm breath = 9",
                "Fire breath = 7",
            ],
            main_matchers: &[Exact("fire breathes 7\nfrost breathes 5\nstorm breathes 9")],
        }),
    ),
    m(
        "chimera_bound",
        H(Harness {
            contract: "`trait Enemy { fn name(&self) -> &'static str; fn breath(&self) -> u32; }`, a generic `fn volley<T: Enemy>(head: &T, rounds: u32) -> String` and a `fn main`",
            run_main: true,
            body: "    struct Wisp;
    impl super::Enemy for Wisp {
        fn name(&self) -> &'static str { \"wisp\" }
        fn breath(&self) -> u32 { 2 }
    }
    let _sig: fn(&super::Fire, u32) -> ::std::string::String = super::volley;
    emit(\"volley(&Fire, 3)\", super::volley(&super::Fire, 3));
    emit(\"volley(&Storm, 2)\", super::volley(&super::Storm, 2));
    emit(\"volley(&Wisp, 4)\", super::volley(&Wisp, 4));",
            expected: &[
                "volley(&Fire, 3) = \"fire x3 = 21\"",
                "volley(&Storm, 2) = \"storm x2 = 18\"",
                "volley(&Wisp, 4) = \"wisp x4 = 8\"",
            ],
            main_matchers: &[Exact("fire x3 = 21\nfrost x3 = 15\nstorm x3 = 27")],
        }),
    ),
    m(
        "chimera_plasma",
        H(Harness {
            contract: "`trait Enemy { fn name(&self) -> &'static str; fn breath(&self) -> u32; }`, `fn battle(heads: &[Box<dyn Enemy>]) -> u32` and a `fn main`",
            run_main: true,
            body: "    struct Wisp;
    impl super::Enemy for Wisp {
        fn name(&self) -> &'static str { \"wisp\" }
        fn breath(&self) -> u32 { 2 }
    }
    let _sig: fn(&[::std::boxed::Box<dyn super::Enemy>]) -> u32 = super::battle;
    let none: ::std::vec::Vec<::std::boxed::Box<dyn super::Enemy>> = ::std::vec::Vec::new();
    emit(\"battle(no heads)\", super::battle(&none));
    let mixed: ::std::vec::Vec<::std::boxed::Box<dyn super::Enemy>> = ::std::vec![
        ::std::boxed::Box::new(Wisp),
        ::std::boxed::Box::new(super::Plasma),
        ::std::boxed::Box::new(Wisp),
    ];
    emit(\"battle(wisp, plasma, wisp)\", super::battle(&mixed));",
            expected: &["battle(no heads) = 0", "battle(wisp, plasma, wisp) = 16"],
            main_matchers: &[Exact(
                "fire breathes 7\nfrost breathes 5\nstorm breathes 9\nplasma breathes 12\ntotal: 33",
            )],
        }),
    ),
    // ── Act 5: The Tavern of Tribulations ────────────────────────────
    m(
        "result_match",
        H(Harness {
            contract: "`fn describe(r: Result<i32, String>) -> i32`",
            run_main: false,
            body: "    let _sig: fn(Result<i32, ::std::string::String>) -> i32 = super::describe;
    emit(\"describe(Ok(5))\", super::describe(Ok(5)));
    emit(\"describe(Ok(-3))\", super::describe(Ok(-3)));
    emit(\"describe(Err(..))\", super::describe(Err(::std::string::String::from(\"x\"))));",
            expected: &[
                "describe(Ok(5)) = 5",
                "describe(Ok(-3)) = -3",
                "describe(Err(..)) = -1",
            ],
            main_matchers: &[],
        }),
    ),
    m(
        "custom_error",
        H(Harness {
            contract: "`fn check(temp: i32) -> Result<i32, YourError>`",
            run_main: false,
            body: "    let r: Result<i32, _> = super::check(50);
    let low = super::check(-1000).is_err();
    let high = super::check(1000).is_err();
    emit(\"check rejects an out-of-range temp\", low || high);",
            expected: &["check rejects an out-of-range temp = true"],
            main_matchers: &[],
        }),
    ),
    m(
        "from_error",
        H(Harness {
            contract: "`impl From<ParseFail> for AppError` mapping to `AppError::Parse`",
            run_main: false,
            body: "    let e: super::AppError = super::ParseFail.into();
    emit(\"ParseFail.into() is AppError::Parse\", ::core::matches!(e, super::AppError::Parse));",
            expected: &["ParseFail.into() is AppError::Parse = true"],
            main_matchers: &[],
        }),
    ),
    m(
        "option_map",
        H(Harness {
            contract: "`fn add_one(o: Option<i32>) -> Option<i32>`",
            run_main: false,
            body: "    let _sig: fn(Option<i32>) -> Option<i32> = super::add_one;
    emit(\"add_one(Some(3))\", super::add_one(Some(3)));
    emit(\"add_one(None)\", super::add_one(None));
    emit(\"add_one(Some(-1))\", super::add_one(Some(-1)));",
            expected: &[
                "add_one(Some(3)) = Some(4)",
                "add_one(None) = None",
                "add_one(Some(-1)) = Some(0)",
            ],
            main_matchers: &[],
        }),
    ),
    mu(
        "and_then",
        Stdout {
            matchers: &[InOrder(&["Some(2)", "None"])],
        },
        &[&[".and_then("]],
    ),
    m(
        "unwrap_or_else",
        H(Harness {
            contract: "`fn value(o: Option<i32>) -> i32`",
            run_main: false,
            body: "    let _sig: fn(Option<i32>) -> i32 = super::value;
    emit(\"value(Some(3))\", super::value(Some(3)));
    emit(\"value(Some(-8))\", super::value(Some(-8)));",
            expected: &["value(Some(3)) = 3", "value(Some(-8)) = -8"],
            main_matchers: &[],
        }),
    ),
    // ── Act 5 boss: the Barkeep's trial, The Tavernkeeper's Wager ────
    // Every stage calls the player's function on failing inputs too: an
    // `unwrap`/`expect`/`panic!` on a failing step crashes the checker. The
    // `From` impls of stage 2 are pinned by their own signatures. Stage 3
    // checks the step order (a bad map wins over a wrong key) and, once the
    // behaviour matched, needs a `?` and rules out `match`, `unwrap`,
    // `expect` and `panic!`, as its Task says.
    m(
        "wager_stake",
        H(Harness {
            contract: "`fn double_stake(text: &str) -> Result<u32, ParseIntError>` and a `fn main`",
            run_main: true,
            body: "    let _sig: fn(&str) -> ::core::result::Result<u32, ::std::num::ParseIntError> = super::double_stake;
    emit(\"double_stake(\\\"20\\\")\", super::double_stake(\"20\"));
    emit(\"double_stake(\\\" 7 \\\")\", super::double_stake(\" 7 \"));
    emit(\"double_stake(\\\"x\\\")\", super::double_stake(\"x\"));
    emit(\"double_stake(\\\"\\\")\", super::double_stake(\"\"));",
            expected: &[
                "double_stake(\"20\") = Ok(40)",
                "double_stake(\" 7 \") = Ok(14)",
                "double_stake(\"x\") = Err(ParseIntError { kind: InvalidDigit })",
                "double_stake(\"\") = Err(ParseIntError { kind: Empty })",
            ],
            main_matchers: &[Exact("Ok(40)\nErr(ParseIntError { kind: InvalidDigit })")],
        }),
    ),
    m(
        "wager_from",
        H(Harness {
            contract: "`enum WagerError { BadStake, TooPoor }`, `impl From<ParseIntError> for WagerError`, `impl From<Short> for WagerError`, `fn place_bet(text: &str, purse: u32) -> Result<u32, WagerError>` and a `fn main`",
            run_main: true,
            body: "    let _sig: fn(&str, u32) -> ::core::result::Result<u32, super::WagerError> = super::place_bet;
    let _parse: fn(::std::num::ParseIntError) -> super::WagerError = <super::WagerError as ::core::convert::From<::std::num::ParseIntError>>::from;
    let _short: fn(super::Short) -> super::WagerError = <super::WagerError as ::core::convert::From<super::Short>>::from;
    emit(\"WagerError::from(a ParseIntError)\", \"x\".parse::<u32>().err().map(_parse));
    emit(\"WagerError::from(Short)\", _short(super::Short));
    emit(\"place_bet(\\\"15\\\", 50)\", super::place_bet(\"15\", 50));
    emit(\"place_bet(\\\"50\\\", 50)\", super::place_bet(\"50\", 50));
    emit(\"place_bet(\\\"lots\\\", 50)\", super::place_bet(\"lots\", 50));
    emit(\"place_bet(\\\"80\\\", 50)\", super::place_bet(\"80\", 50));",
            expected: &[
                "WagerError::from(a ParseIntError) = Some(BadStake)",
                "WagerError::from(Short) = TooPoor",
                "place_bet(\"15\", 50) = Ok(35)",
                "place_bet(\"50\", 50) = Ok(0)",
                "place_bet(\"lots\", 50) = Err(BadStake)",
                "place_bet(\"80\", 50) = Err(TooPoor)",
            ],
            main_matchers: &[Exact("Ok(35)\nErr(BadStake)\nErr(TooPoor)")],
        }),
    ),
    muf(
        "wager_quest",
        H(Harness {
            contract: "`struct Reward { gold: u32 }`, `enum QuestError { BadMap, Locked, Broke }`, `fn quest(map: &str, key: &str) -> Result<Reward, QuestError>` and a `fn main`",
            run_main: true,
            body: "    let _sig: fn(&str, &str) -> ::core::result::Result<super::Reward, super::QuestError> = super::quest;
    emit(\"quest(\\\"4\\\", \\\"brass\\\")\", super::quest(\"4\", \"brass\").map(|r| r.gold));
    emit(\"quest(\\\"15\\\", \\\"brass\\\")\", super::quest(\"15\", \"brass\").map(|r| r.gold));
    emit(\"quest(\\\"far\\\", \\\"brass\\\")\", super::quest(\"far\", \"brass\").map(|r| r.gold));
    emit(\"quest(\\\"4\\\", \\\"bone\\\")\", super::quest(\"4\", \"bone\").map(|r| r.gold));
    emit(\"quest(\\\"20\\\", \\\"brass\\\")\", super::quest(\"20\", \"brass\").map(|r| r.gold));
    emit(\"quest(\\\"far\\\", \\\"bone\\\")\", super::quest(\"far\", \"bone\").map(|r| r.gold));
    emit(\"quest(\\\"99\\\", \\\"bone\\\")\", super::quest(\"99\", \"bone\").map(|r| r.gold));",
            expected: &[
                "quest(\"4\", \"brass\") = Ok(22)",
                "quest(\"15\", \"brass\") = Ok(0)",
                "quest(\"far\", \"brass\") = Err(BadMap)",
                "quest(\"4\", \"bone\") = Err(Locked)",
                "quest(\"20\", \"brass\") = Err(Broke)",
                "quest(\"far\", \"bone\") = Err(BadMap)",
                "quest(\"99\", \"bone\") = Err(Locked)",
            ],
            main_matchers: &[Exact("Ok(22)\nErr(BadMap)\nErr(Locked)\nErr(Broke)")],
        }),
        &[&["?"]],
        &["match", ".unwrap(", ".expect(", "panic!"],
    ),
    // ── Act 6: The Iterator Forge ────────────────────────────────────
    mu(
        "hashmap_basic",
        Stdout {
            matchers: &[InOrder(&["Some(7)", "None"])],
        },
        &[&[".insert("], &[".get("]],
    ),
    m(
        "iter_filter",
        Stdout {
            matchers: &[LineContains("[2, 4]")],
        },
    ),
    m(
        "iter_fold",
        Stdout {
            matchers: &[HasLine("6")],
        },
    ),
    m(
        "iter_enumerate",
        Stdout {
            matchers: &[
                LineContainsAll(&["0", "10"]),
                LineContainsAll(&["1", "20"]),
                LineContainsAll(&["2", "30"]),
            ],
        },
    ),
    m(
        "iter_zip",
        Stdout {
            matchers: &[LineContainsAll(&["1", "3"]), LineContainsAll(&["2", "4"])],
        },
    ),
    m(
        "closure_move",
        Stdout {
            matchers: &[LineContains("Garin")],
        },
    ),
    // ── Act 7: The Concurrent Coast ──────────────────────────────────
    m(
        "thread_spawn",
        CompileOnly {
            reason: NO_THREADS,
            required: &[&["spawn("], &[".join("]],
        },
    ),
    mu(
        "arc_mutex",
        Stdout {
            matchers: &[HasLine("5")],
        },
        &[&["Arc::clone", ".clone()"], &[".lock("]],
    ),
    mu(
        "mpsc_channel",
        Stdout {
            matchers: &[InOrder(&["7", "11"])],
        },
        &[&[".send("], &[".recv("]],
    ),
    mu(
        "atomic",
        Stdout {
            matchers: &[InOrder(&["10", "11"])],
        },
        &[&[".fetch_add("], &[".load("]],
    ),
    m(
        "thread_scope",
        CompileOnly {
            reason: NO_THREADS,
            required: &[&["scope("], &[".spawn("]],
        },
    ),
    m(
        "async_fn",
        CompileOnly {
            reason: "the mission does not name the async functions and no executor \
                     polls them, so there is no behaviour to observe",
            required: &[&["async fn"], &[".await"]],
        },
    ),
    // ── Act 8: The Vault of Pointers ─────────────────────────────────
    m(
        "box_basic",
        H(Harness {
            contract: "`struct Node { value: i32, next: Option<Box<Node>> }` and a `fn main`",
            run_main: true,
            body: "    let list = super::Node {
        value: 1,
        next: Some(::std::boxed::Box::new(super::Node { value: 2, next: None })),
    };
    let first: i32 = list.value;
    emit(\"list.value\", first);
    emit(\"list.next.value\", list.next.as_ref().map(|n| n.value));",
            expected: &["list.value = 1", "list.next.value = Some(2)"],
            main_matchers: &[],
        }),
    ),
    mu(
        "rc_basic",
        Stdout {
            matchers: &[InOrder(&["1", "2", "1"])],
        },
        &[&["Rc::clone"], &["strong_count"]],
    ),
    mu(
        "refcell",
        Stdout {
            matchers: &[InOrder(&["10", "15"])],
        },
        &[&[".borrow_mut("]],
    ),
    mu(
        "cell",
        Stdout {
            matchers: &[InOrder(&["1", "5"])],
        },
        &[&[".set("], &[".get("]],
    ),
    mu(
        "rc_refcell",
        Stdout {
            matchers: &[HasLine("15")],
        },
        &[&["Rc::clone", ".clone()"], &[".borrow_mut("]],
    ),
    mu(
        "weak_ref",
        Stdout {
            matchers: &[InOrder(&["Some(5)", "None"])],
        },
        &[&["downgrade"], &[".upgrade("]],
    ),
];

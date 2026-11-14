// SPDX-License-Identifier: AGPL-3.0-or-later
//! Characterisation test: replays every input stored in
//! `tests/snapshots/grader.json` through `grader::grade` and asserts the
//! verdict (ok + stdout + stderr) is byte-identical to the recorded one.
//!
//! The snapshot covers every shipped mission against its canonical
//! solution, its starter code, an empty string and single-token
//! near-misses, plus an unknown id (freeform fallback). It is generated
//! by `game/tests/contract.rs::stub_grader_matches_snapshot` with
//! `PNC_UPDATE_GRADER_SNAPSHOTS=1` (the game crate owns the mission
//! registry and canonical solutions; this crate cannot see them).

use pledge_compile_api::grader::grade;

#[test]
fn grader_matches_snapshot() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/snapshots/grader.json");
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    let rows: serde_json::Value = serde_json::from_str(&text).expect("snapshot is valid JSON");
    let rows = rows.as_array().expect("snapshot is a JSON array");
    assert!(rows.len() > 4 * 57, "snapshot suspiciously small");
    for row in rows {
        let id = row["id"].as_str().expect("id");
        let case = row["case"].as_str().expect("case");
        let input = row["input"].as_str().expect("input");
        let v = grade(id, input);
        let actual = serde_json::json!({ "ok": v.ok, "stdout": v.stdout, "stderr": v.stderr });
        assert_eq!(
            actual, row["verdict"],
            "server grader drift on `{id}` case `{case}`"
        );
    }
}

// SPDX-License-Identifier: MIT
// Refuse to bundle anything but the demo web build unless asked to: see
// web_gate.rs. Runs for every build of the shell (plain cargo,
// build-desktop.bat, `cargo tauri build`, `cargo tauri android build`).

#[path = "web_gate.rs"]
mod web_gate;

fn main() {
    let manifest_dir = std::path::PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR set by cargo"),
    );
    let stamp_path = manifest_dir.join("../../web/pkg/build-stamp.json");
    println!("cargo:rerun-if-changed={}", stamp_path.display());
    println!("cargo:rerun-if-env-changed={}", web_gate::ALLOW_FULL_VAR);
    let stamp = std::fs::read_to_string(&stamp_path).ok();
    let allow_full = std::env::var(web_gate::ALLOW_FULL_VAR).is_ok_and(|v| v.trim() == "1");
    match web_gate::check(stamp.as_deref(), allow_full) {
        Ok(flavour) => println!("cargo:warning=bundling the {flavour} web build from ../../web"),
        Err(why) => panic!("{why}"),
    }
    tauri_build::build()
}

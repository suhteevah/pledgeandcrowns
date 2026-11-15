// SPDX-License-Identifier: AGPL-3.0-or-later
//! Static checks on the shipped deploy files (security re-review P1-A,
//! P1-D). These cannot prove the units work on a real host (nothing here
//! runs systemd or bwrap); they pin the properties the review asked for so
//! an edit cannot silently undo them.

const API_UNIT: &str = include_str!("../deploy/pnc-compile-api.service");
const BUILDER_UNIT: &str = include_str!("../deploy/pnc-builder.service");
const VERIFY: &str = include_str!("../deploy/verify-sandbox.sh");
const README: &str = include_str!("../deploy/README.md");
const BUILDER_ENV: &str = include_str!("../deploy/builder.env.example");

/// Non-comment `Key=Value` lines.
fn directives(unit: &str) -> Vec<(&str, &str)> {
    unit.lines()
        .map(str::trim)
        .filter(|l| !l.starts_with('#') && !l.is_empty() && !l.starts_with('['))
        .filter_map(|l| l.split_once('='))
        .collect()
}

fn values<'a>(unit: &'a str, key: &str) -> Vec<&'a str> {
    directives(unit)
        .into_iter()
        .filter(|(k, _)| *k == key)
        .map(|(_, v)| v)
        .collect()
}

fn has(unit: &str, key: &str, value: &str) -> bool {
    values(unit, key).contains(&value)
}

#[test]
fn api_unit_holds_the_secrets_and_keeps_full_hardening() {
    assert_eq!(
        values(API_UNIT, "EnvironmentFile"),
        ["/etc/pnc-compile/env"]
    );
    assert!(has(
        API_UNIT,
        "ExecStart",
        "/usr/local/bin/pledge_compile_api serve"
    ));
    assert!(
        values(API_UNIT, "Environment")
            .iter()
            .any(|v| v.starts_with("PNC_BUILDER_SOCKET=/run/pnc-builder/")),
        "the API must hand builds to the builder"
    );
    for (k, v) in [
        ("RestrictNamespaces", "yes"),
        ("ProtectProc", "invisible"),
        ("ProcSubset", "pid"),
        ("ProtectKernelTunables", "yes"),
        ("ProtectKernelLogs", "yes"),
        ("MemoryDenyWriteExecute", "yes"),
        ("NoNewPrivileges", "yes"),
        ("SupplementaryGroups", "pnc-ipc"),
    ] {
        assert!(has(API_UNIT, k, v), "API unit must set {k}={v}");
    }
    let filters = values(API_UNIT, "SystemCallFilter");
    let allow: Vec<&str> = filters
        .iter()
        .filter(|v| !v.starts_with('~'))
        .flat_map(|v| v.split_whitespace())
        .collect();
    assert_eq!(allow, ["@system-service"], "API allow list: {allow:?}");
    let deny: Vec<&str> = filters
        .iter()
        .filter(|v| v.starts_with('~'))
        .flat_map(|v| v.trim_start_matches('~').split_whitespace())
        .collect();
    for group in ["@privileged", "@mount", "@debug", "@raw-io", "@module"] {
        assert!(deny.contains(&group), "API must deny {group}: {deny:?}");
    }
    let families = values(API_UNIT, "RestrictAddressFamilies").join(" ");
    assert!(!families.contains("AF_NETLINK"), "{families}");
}

#[test]
fn builder_unit_holds_no_secrets() {
    for v in values(BUILDER_UNIT, "EnvironmentFile") {
        assert!(
            !v.contains("/etc/pnc-compile"),
            "builder reads the secret env file: {v}"
        );
    }
    assert!(has(
        BUILDER_UNIT,
        "ExecStart",
        "/usr/local/bin/pledge_compile_api builder"
    ));
    for (_, v) in directives(BUILDER_UNIT) {
        assert!(!v.contains("PNC_AUTH_TOKENS") && !v.contains("PNC_METRICS_TOKENS"));
    }
    assert!(
        !BUILDER_ENV
            .lines()
            .any(|l| l.starts_with("PNC_AUTH_TOKENS") || l.starts_with("PNC_METRICS_TOKENS"))
    );
    assert!(has(BUILDER_UNIT, "RuntimeDirectory", "pnc-builder"));
    assert!(has(BUILDER_UNIT, "RuntimeDirectoryMode", "0750"));
    assert!(has(BUILDER_UNIT, "Group", "pnc-ipc"));
    assert!(has(BUILDER_UNIT, "OOMPolicy", "continue"));
}

/// The old unit allowed @mount and then subtracted @privileged, which
/// removes pivot_root, chroot, capset, setuid and setgroups again. The
/// builder's deny line must not subtract any group bwrap needs.
#[test]
fn builder_syscall_filter_does_not_subtract_what_bwrap_needs() {
    let filters = values(BUILDER_UNIT, "SystemCallFilter");
    let allow: Vec<&str> = filters
        .iter()
        .filter(|v| !v.starts_with('~'))
        .flat_map(|v| v.split_whitespace())
        .collect();
    for needed in ["@system-service", "@mount"] {
        assert!(
            allow.contains(&needed),
            "builder must allow {needed}: {allow:?}"
        );
    }
    let deny: Vec<&str> = filters
        .iter()
        .filter(|v| v.starts_with('~'))
        .flat_map(|v| v.trim_start_matches('~').split_whitespace())
        .collect();
    for forbidden in [
        "@privileged",
        "@mount",
        "@setuid",
        "@process",
        "@resources",
        "@network-io",
        "@system-service",
        "pivot_root",
        "chroot",
        "capset",
        "setuid",
        "setgroups",
        "unshare",
        "clone",
        "clone3",
        "mount",
        "umount2",
        "prctl",
    ] {
        assert!(
            !deny.contains(&forbidden),
            "builder deny line subtracts {forbidden}, which bwrap needs: {deny:?}"
        );
    }
    let families = values(BUILDER_UNIT, "RestrictAddressFamilies").join(" ");
    assert!(families.contains("AF_NETLINK") && families.contains("AF_UNIX"));
    assert!(
        !families.contains("AF_INET"),
        "builder needs no IP: {families}"
    );
    assert!(has(
        BUILDER_UNIT,
        "RestrictNamespaces",
        "user mnt pid net ipc uts cgroup"
    ));
    // Proc protections kept on the builder (no procfs mount in the sandbox).
    assert!(has(BUILDER_UNIT, "ProtectProc", "invisible"));
    assert!(has(BUILDER_UNIT, "ProtectKernelTunables", "yes"));
    assert!(
        values(BUILDER_UNIT, "ProcSubset").is_empty(),
        "see the unit comment"
    );
    assert!(BUILDER_UNIT.contains("systemd-analyze syscall-filter"));
}

#[test]
fn verify_script_requires_positive_evidence() {
    // Exact rustc texts, not just `"ok":false`.
    for needle in [
        "couldn't read",
        "No such file or directory",
        "file not found for module",
        // rustc 1.99's wording for a missing #[path] file (Linux run).
        "couldn't find file",
        "not defined at compile time",
    ] {
        assert!(
            VERIFY.contains(needle),
            "verify-sandbox.sh lacks `{needle}`"
        );
    }
    // Positive control through the same mechanism, run as the builder.
    assert!(VERIFY.contains("/build/Cargo.toml"));
    assert!(VERIFY.contains("sandbox-verify"));
    assert!(VERIFY.contains("runuser -u"));
    // Exact HTTP statuses and an explicit PASS gate.
    assert!(VERIFY.contains("expect_status"));
    assert!(VERIFY.contains("echo \"PASS"));
    // The old approach (deny-list off in the production unit, restart)
    // is gone.
    assert!(!VERIFY.contains("PNC_SOURCE_DENYLIST=false"));
    assert!(!VERIFY.contains("systemctl restart"));
}

#[test]
fn readme_is_an_operator_runbook_for_both_units() {
    for needle in [
        "pnc-builder.service",
        "pnc-compile-api.service",
        "0600",
        "apparmor_restrict_unprivileged_userns",
        "systemd-analyze security",
        "systemd-analyze syscall-filter",
        "verify-sandbox.sh",
        "cargo audit",
        "cargo deny",
        "pledge_compile_api toolchain",
        "9878",
        "PNC_BUILD_SANDBOX=none",
    ] {
        assert!(README.contains(needle), "README lacks `{needle}`");
    }
}

// SPDX-License-Identifier: AGPL-3.0-or-later
//! Filesystem sandbox for the player build: the primary control against
//! player source making rustc read host files.
//!
//! rustc evaluates `include_str!`, `include_bytes!`, `#[path]` and friends
//! with the filesystem view of the process that runs it. Run on the host,
//! that view includes the env file holding bearer tokens and
//! `/proc/<server pid>/environ`. So the build must run where those files
//! do not exist.
//!
//! # Modes (`PNC_BUILD_SANDBOX`)
//!
//! - `bwrap` (Linux, required for any exposed deployment): the build runs
//!   as
//!   `prlimit <rlimits> -- bwrap <namespaces + binds> -- <toolchain>/bin/cargo build ...`.
//!   Inside the sandbox the only things visible are:
//!   - `/usr` read-only (glibc and the dynamic loader rustc links
//!     against), with `/bin`, `/lib`, `/lib64` as symlinks into it;
//!   - the Rust toolchain dir (`PNC_TOOLCHAIN_DIR`) read-only;
//!   - the per-request job dir, bind-mounted read-write at `/build`, with
//!     its `Cargo.toml` and `src/` re-mounted read-only on top;
//!   - a fresh `/proc` (new PID namespace: only sandbox processes), a
//!     minimal `/dev`, and an empty tmpfs `/tmp`.
//!
//!   Not visible: `/`, `/etc` (so the env file), `/var`, `/home`, `/root`,
//!   the server's `/proc` entry. All namespaces are unshared, network
//!   included (`--unshare-all`), the environment is cleared both on the
//!   outer `bwrap` process and inside (`--clearenv`), and the sandbox
//!   dies with its parent. `prlimit` applies per-build rlimits (address
//!   space, CPU seconds, file size, open files, no core dumps) that every
//!   process in the build inherits.
//! - `none` (dev only): cargo runs directly on the host with an
//!   allow-listed environment. No filesystem isolation. Config refuses it
//!   for anything exposed (see `config.rs`).
//!
//! The argument vector is built by pure functions ([`bwrap_spec`],
//! [`cargo_build_spec`]) so its security properties are unit-tested on
//! every platform even though `bwrap` itself only runs on Linux.
//!
//! Not covered: `RLIMIT_NPROC` is per-*user* (it would count the server's
//! own threads), so process count is bounded by the unit's `TasksMax`
//! and the PID namespace instead.

use std::ffi::{OsStr, OsString};
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;

/// Where the job dir appears inside the sandbox. Fixed, so diagnostics
/// show `/build/src/main.rs` rather than a host temp path.
pub const SANDBOX_JOB_DIR: &str = "/build";

/// Rust target the player crate is built for.
pub const WASIP1_TARGET: &str = "wasm32-wasip1";

/// Default `bwrap` binary (Debian/Ubuntu package `bubblewrap`).
pub const DEFAULT_BWRAP_PATH: &str = "/usr/bin/bwrap";
/// Default `prlimit` binary (util-linux, essential on Debian/Ubuntu).
pub const DEFAULT_PRLIMIT_PATH: &str = "/usr/bin/prlimit";

/// Host paths that must never be exposed to the build, checked when a
/// toolchain dir is configured.
const FORBIDDEN_TOOLCHAIN_PREFIXES: &[&str] = &["/etc", "/proc", "/sys", "/dev", "/root", "/home"];

/// `PNC_BUILD_SANDBOX`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SandboxMode {
    /// Host build, dev only.
    #[default]
    None,
    /// bubblewrap + prlimit (Linux).
    Bwrap,
}

impl FromStr for SandboxMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "none" => Ok(Self::None),
            "bwrap" => Ok(Self::Bwrap),
            _ => Err("expected `bwrap` or `none`".to_string()),
        }
    }
}

impl fmt::Display for SandboxMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::None => "none",
            Self::Bwrap => "bwrap",
        })
    }
}

/// Per-build resource limits applied via `prlimit` in `bwrap` mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuildRlimits {
    /// `RLIMIT_AS`. Address space, not RSS: rustc maps far more than it
    /// touches, so this is a runaway backstop, set strictly below the
    /// unit's `MemoryMax`.
    pub address_space_bytes: u64,
    /// `RLIMIT_CPU`, seconds of CPU per process.
    pub cpu_secs: u64,
    /// `RLIMIT_FSIZE`, largest file any build process may write.
    pub file_size_bytes: u64,
    /// `RLIMIT_NOFILE`.
    pub nofile: u64,
}

/// Default address-space cap per build (MiB).
pub const DEFAULT_BUILD_ADDRESS_SPACE_MIB: u64 = 1280;
/// Smallest allowed address-space cap (MiB).
pub const MIN_BUILD_ADDRESS_SPACE_MIB: u64 = 512;
/// Largest allowed address-space cap (MiB). Strictly below the unit's
/// `MemoryMax=1536M`; `tests` in this module check the unit file.
pub const MAX_BUILD_ADDRESS_SPACE_MIB: u64 = 1408;

impl Default for BuildRlimits {
    fn default() -> Self {
        Self {
            address_space_bytes: DEFAULT_BUILD_ADDRESS_SPACE_MIB * 1024 * 1024,
            // Wall clock is 30 s; rustc runs codegen on several threads,
            // and RLIMIT_CPU counts all of a process's threads.
            cpu_secs: 90,
            file_size_bytes: 64 * 1024 * 1024,
            nofile: 1024,
        }
    }
}

/// Everything needed to construct a sandboxed build command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SandboxSettings {
    pub mode: SandboxMode,
    /// Toolchain root containing `bin/cargo`, `bin/rustc` and
    /// `lib/rustlib/` (e.g. `/opt/rust/rustup/toolchains/stable-x86_64-unknown-linux-gnu`).
    /// Required for `bwrap`; unused for `none` (cargo comes from `PATH`).
    pub toolchain_dir: Option<PathBuf>,
    pub bwrap_path: PathBuf,
    pub prlimit_path: PathBuf,
    pub rlimits: BuildRlimits,
}

impl Default for SandboxSettings {
    fn default() -> Self {
        Self {
            mode: SandboxMode::None,
            toolchain_dir: None,
            bwrap_path: PathBuf::from(DEFAULT_BWRAP_PATH),
            prlimit_path: PathBuf::from(DEFAULT_PRLIMIT_PATH),
            rlimits: BuildRlimits::default(),
        }
    }
}

/// Sandbox construction / probe failures.
#[derive(Debug, thiserror::Error)]
pub enum SandboxError {
    #[error("PNC_BUILD_SANDBOX=bwrap requires PNC_TOOLCHAIN_DIR")]
    MissingToolchain,
    #[error("invalid toolchain dir {path}: {reason}")]
    BadToolchain { path: PathBuf, reason: String },
    #[error("sandbox probe `{step}` failed: {detail}")]
    Probe { step: &'static str, detail: String },
}

/// A fully specified process launch: program, args, exact environment,
/// working directory. `env` is the *whole* environment (the inherited one
/// is always cleared).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSpec {
    pub program: OsString,
    pub args: Vec<OsString>,
    pub env: Vec<(OsString, OsString)>,
    pub cwd: Option<PathBuf>,
}

impl CommandSpec {
    pub fn to_command(&self) -> Command {
        let mut cmd = Command::new(&self.program);
        cmd.args(&self.args).env_clear();
        for (k, v) in &self.env {
            cmd.env(k, v);
        }
        if let Some(cwd) = &self.cwd {
            cmd.current_dir(cwd);
        }
        cmd
    }

    /// Lossy argv (program first), for logs and tests.
    pub fn argv_lossy(&self) -> Vec<String> {
        std::iter::once(&self.program)
            .chain(self.args.iter())
            .map(|a| a.to_string_lossy().into_owned())
            .collect()
    }
}

/// Validate a toolchain dir string from config (pure: no filesystem
/// access, so it is testable everywhere).
pub fn validate_toolchain_dir(p: &Path) -> Result<(), String> {
    let s = p.to_string_lossy();
    if !s.starts_with('/') {
        return Err("must be an absolute Unix path".to_string());
    }
    if s.trim_end_matches('/').is_empty() {
        return Err("must not be `/`".to_string());
    }
    if s.contains("/../") || s.ends_with("/..") || s.contains("/./") {
        return Err("must not contain `.` or `..` components".to_string());
    }
    for bad in FORBIDDEN_TOOLCHAIN_PREFIXES {
        if s == *bad || s.starts_with(&format!("{bad}/")) {
            return Err(format!("must not be under {bad}"));
        }
    }
    Ok(())
}

fn os(s: impl AsRef<OsStr>) -> OsString {
    s.as_ref().to_os_string()
}

fn join_os(a: &Path, rel: &str) -> OsString {
    a.join(rel).into_os_string()
}

/// Pure: the `prlimit ... -- bwrap ... -- <inner>` command that runs
/// `inner` (absolute program path first) inside the sandbox with
/// `job_dir_host` mounted at [`SANDBOX_JOB_DIR`].
///
/// `inner_env` is the complete environment inside the sandbox.
pub fn bwrap_spec(
    settings: &SandboxSettings,
    toolchain: &Path,
    job_dir_host: &Path,
    inner: &[OsString],
    inner_env: &[(&str, OsString)],
) -> CommandSpec {
    let r = settings.rlimits;
    let mut args: Vec<OsString> = vec![
        os(format!("--as={}", r.address_space_bytes)),
        os(format!("--cpu={}", r.cpu_secs)),
        os(format!("--fsize={}", r.file_size_bytes)),
        os(format!("--nofile={}", r.nofile)),
        os("--core=0"),
        os("--"),
        settings.bwrap_path.clone().into_os_string(),
        // Namespaces: user, ipc, pid, net, uts, cgroup. No network.
        os("--unshare-all"),
        os("--die-with-parent"),
        os("--cap-drop"),
        os("ALL"),
        os("--clearenv"),
        // System libraries for rustc (glibc, ld.so). Read-only.
        os("--ro-bind"),
        os("/usr"),
        os("/usr"),
        os("--symlink"),
        os("usr/bin"),
        os("/bin"),
        os("--symlink"),
        os("usr/lib"),
        os("/lib"),
        os("--symlink"),
        os("usr/lib64"),
        os("/lib64"),
        // The toolchain, read-only, at its host path (rustc locates its
        // sysroot relative to its own binary).
        os("--ro-bind"),
        toolchain.as_os_str().to_os_string(),
        toolchain.as_os_str().to_os_string(),
        // The job dir: the ONLY writable bind.
        os("--bind"),
        job_dir_host.as_os_str().to_os_string(),
        os(SANDBOX_JOB_DIR),
        // Server-written inputs re-mounted read-only on top.
        os("--ro-bind"),
        join_os(job_dir_host, "Cargo.toml"),
        os(format!("{SANDBOX_JOB_DIR}/Cargo.toml")),
        os("--ro-bind"),
        join_os(job_dir_host, "src"),
        os(format!("{SANDBOX_JOB_DIR}/src")),
        // Fresh /proc for the new PID namespace, minimal /dev, empty /tmp.
        os("--proc"),
        os("/proc"),
        os("--dev"),
        os("/dev"),
        os("--tmpfs"),
        os("/tmp"),
        os("--chdir"),
        os(SANDBOX_JOB_DIR),
    ];
    for (k, v) in inner_env {
        args.push(os("--setenv"));
        args.push(os(k));
        args.push(v.clone());
    }
    args.push(os("--"));
    args.extend(inner.iter().cloned());
    CommandSpec {
        program: settings.prlimit_path.clone().into_os_string(),
        args,
        // Outer environment is EMPTY: bwrap's own process (and the PID-1
        // reaper it leaves inside the namespace, whose /proc/1/environ the
        // build can read) must not carry the server's secrets.
        env: Vec::new(),
        cwd: None,
    }
}

/// Environment inside the sandbox for a cargo build.
fn sandbox_build_env(toolchain: &Path) -> Vec<(&'static str, OsString)> {
    let mut path = toolchain.join("bin").into_os_string();
    path.push(":/usr/bin:/bin");
    vec![
        ("PATH", path),
        ("HOME", os(SANDBOX_JOB_DIR)),
        ("CARGO_HOME", os(format!("{SANDBOX_JOB_DIR}/.cargo-home"))),
        ("CARGO_TARGET_DIR", os(format!("{SANDBOX_JOB_DIR}/target"))),
        ("CARGO_TERM_COLOR", os("never")),
        ("CARGO_INCREMENTAL", os("0")),
        ("RUSTC", join_os(toolchain, "bin/rustc")),
        ("TMPDIR", os("/tmp")),
    ]
}

fn cargo_build_args(manifest: OsString) -> Vec<OsString> {
    vec![
        os("build"),
        os("--target"),
        os(WASIP1_TARGET),
        os("--release"),
        os("--offline"),
        os("--quiet"),
        os("--manifest-path"),
        manifest,
    ]
}

/// Host environment variables passed through in `none` mode. Mirrors the
/// pre-sandbox allow-list; nothing secret is on it.
const HOST_ENV_ALLOWLIST: &[&str] = &[
    "PATH",                   // resolve `cargo`, `rustc`, linker, rust-lld
    "CARGO_HOME",             // toolchain + registry cache
    "RUSTUP_HOME",            // rustup root (resolves the wasip1 std)
    "USERPROFILE",            // Windows: HOME analogue
    "HOME",                   // Unix: home
    "TEMP",                   // Windows: temp dir cargo uses for fingerprints
    "TMP",                    // Windows: same, alt name
    "TMPDIR",                 // Unix: temp dir
    "SYSTEMROOT",             // Windows: required for some toolchain bits
    "PROCESSOR_ARCHITECTURE", // Windows: rustc host detection
];

/// The `cargo build --target wasm32-wasip1` command for `job_dir`.
pub fn cargo_build_spec(
    settings: &SandboxSettings,
    job_dir: &Path,
) -> Result<CommandSpec, SandboxError> {
    match settings.mode {
        SandboxMode::None => {
            let mut env: Vec<(OsString, OsString)> = HOST_ENV_ALLOWLIST
                .iter()
                .filter_map(|k| std::env::var_os(k).map(|v| (os(k), v)))
                .collect();
            env.push((
                os("CARGO_TARGET_DIR"),
                job_dir.join("target").into_os_string(),
            ));
            env.push((os("CARGO_TERM_COLOR"), os("never")));
            Ok(CommandSpec {
                program: os("cargo"),
                args: cargo_build_args(job_dir.join("Cargo.toml").into_os_string()),
                env,
                cwd: Some(job_dir.to_path_buf()),
            })
        }
        SandboxMode::Bwrap => {
            let toolchain = settings
                .toolchain_dir
                .as_deref()
                .ok_or(SandboxError::MissingToolchain)?;
            validate_toolchain_dir(toolchain).map_err(|reason| SandboxError::BadToolchain {
                path: toolchain.to_path_buf(),
                reason,
            })?;
            let mut inner = vec![join_os(toolchain, "bin/cargo")];
            inner.extend(cargo_build_args(os(format!(
                "{SANDBOX_JOB_DIR}/Cargo.toml"
            ))));
            Ok(bwrap_spec(
                settings,
                toolchain,
                job_dir,
                &inner,
                &sandbox_build_env(toolchain),
            ))
        }
    }
}

/// Startup self-test for `bwrap` mode. Refuse to serve if any step fails:
///
/// 1. `prlimit` and `bwrap` exist and run.
/// 2. Inside the sandbox the job dir is visible (positive control) and
///    `/etc` is NOT (negative control: the env file lives there).
/// 3. The sandbox environment contains no `PNC_` variable.
///
/// A real trivial build is then run by the caller (`wasm_builder`), which
/// proves the toolchain binds and rlimits are workable.
#[cfg(target_os = "linux")]
pub fn probe_isolation(settings: &SandboxSettings, root: &Path) -> Result<(), SandboxError> {
    use crate::build_root::{JobDirGuard, create_job_dir};
    use crate::proc_tree::run_captured;
    use std::time::Duration;

    const PROBE_TIMEOUT: Duration = Duration::from_secs(15);
    let probe_err = |step: &'static str, detail: String| {
        tracing::error!(step, "sandbox probe failed: {detail}");
        SandboxError::Probe { step, detail }
    };

    for (step, bin) in [
        ("prlimit --version", &settings.prlimit_path),
        ("bwrap --version", &settings.bwrap_path),
    ] {
        let mut cmd = Command::new(bin);
        cmd.arg("--version").env_clear();
        let out = run_captured(cmd, PROBE_TIMEOUT, 4096, step)
            .map_err(|e| probe_err(step, e.to_string()))?;
        if !out.status.success() {
            return Err(probe_err(step, format!("{:?}: {}", out.status, out.stderr)));
        }
        tracing::info!(step, "sandbox probe ok: {}", out.stdout.trim());
    }

    let toolchain = settings
        .toolchain_dir
        .as_deref()
        .ok_or(SandboxError::MissingToolchain)?;
    let job = create_job_dir(root).map_err(|e| probe_err("job dir", e.to_string()))?;
    let _guard = JobDirGuard::new(job.clone());
    std::fs::write(job.join("Cargo.toml"), b"").map_err(|e| probe_err("job dir", e.to_string()))?;

    let run = |step: &'static str, inner: &[&str]| {
        let inner: Vec<OsString> = inner.iter().map(os).collect();
        let spec = bwrap_spec(
            settings,
            toolchain,
            &job,
            &inner,
            &sandbox_build_env(toolchain),
        );
        tracing::debug!(step, argv = ?spec.argv_lossy(), "sandbox probe");
        run_captured(spec.to_command(), PROBE_TIMEOUT, 16 * 1024, step)
            .map_err(|e| probe_err(step, e.to_string()))
    };

    let visible = run("job dir visible", &["/usr/bin/test", "-d", SANDBOX_JOB_DIR])?;
    if !visible.status.success() {
        return Err(probe_err(
            "job dir visible",
            format!("bwrap could not start or bind: {}", visible.stderr.trim()),
        ));
    }
    let etc = run("/etc hidden", &["/usr/bin/test", "-e", "/etc"])?;
    if etc.status.code() != Some(1) {
        return Err(probe_err(
            "/etc hidden",
            format!(
                "expected exit 1, got {:?}: {}",
                etc.status,
                etc.stderr.trim()
            ),
        ));
    }
    let env = run("environment cleared", &["/usr/bin/env"])?;
    if !env.status.success() || env.stdout.contains("PNC_") {
        return Err(probe_err(
            "environment cleared",
            format!(
                "sandbox env leaked PNC_ variables or failed ({:?})",
                env.status
            ),
        ));
    }
    tracing::info!("sandbox probe: isolation checks passed");
    Ok(())
}

/// Non-Linux builds cannot select `bwrap` (config refuses it).
#[cfg(not(target_os = "linux"))]
pub fn probe_isolation(_settings: &SandboxSettings, _root: &Path) -> Result<(), SandboxError> {
    Err(SandboxError::Probe {
        step: "platform",
        detail: "bwrap sandbox is Linux-only".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOOLCHAIN: &str = "/opt/rust/rustup/toolchains/stable-x86_64-unknown-linux-gnu";
    const JOB: &str = "/tmp/pledge-wasm-builder/2b0c7d4e-1111-4222-8333-944445555666";

    fn settings() -> SandboxSettings {
        SandboxSettings {
            mode: SandboxMode::Bwrap,
            toolchain_dir: Some(PathBuf::from(TOOLCHAIN)),
            ..SandboxSettings::default()
        }
    }

    fn spec() -> CommandSpec {
        cargo_build_spec(&settings(), Path::new(JOB)).expect("bwrap spec")
    }

    /// (flag, source, dest) for every bind-style mount in the bwrap argv.
    fn binds(argv: &[String]) -> Vec<(String, String, String)> {
        let mut out = Vec::new();
        let mut i = 0;
        while i < argv.len() {
            let a = argv[i].as_str();
            if a == "--" && i > 6 {
                break; // start of the inner command
            }
            if matches!(
                a,
                "--bind"
                    | "--ro-bind"
                    | "--dev-bind"
                    | "--bind-try"
                    | "--ro-bind-try"
                    | "--dev-bind-try"
            ) {
                out.push((a.to_string(), argv[i + 1].clone(), argv[i + 2].clone()));
                i += 3;
                continue;
            }
            i += 1;
        }
        out
    }

    fn bwrap_args(argv: &[String]) -> &[String] {
        let start = argv.iter().position(|a| a == DEFAULT_BWRAP_PATH).unwrap();
        let end = argv
            .iter()
            .skip(start)
            .position(|a| a == "--")
            .map(|p| p + start)
            .unwrap();
        &argv[start..end]
    }

    fn norm(p: &str) -> String {
        // Path::join on Windows uses `\`; compare with forward slashes.
        p.replace('\\', "/")
    }

    #[test]
    fn mode_parses() {
        assert_eq!("bwrap".parse::<SandboxMode>(), Ok(SandboxMode::Bwrap));
        assert_eq!("NONE".parse::<SandboxMode>(), Ok(SandboxMode::None));
        assert!("docker".parse::<SandboxMode>().is_err());
    }

    #[test]
    fn outer_program_is_prlimit_then_bwrap_with_empty_env() {
        let s = spec();
        assert_eq!(s.program, OsString::from(DEFAULT_PRLIMIT_PATH));
        assert!(s.env.is_empty(), "outer env must be empty: {:?}", s.env);
        let argv = s.argv_lossy();
        let sep = argv.iter().position(|a| a == "--").unwrap();
        assert_eq!(argv[sep + 1], DEFAULT_BWRAP_PATH);
        for want in ["--as=", "--cpu=", "--fsize=", "--nofile=", "--core=0"] {
            assert!(argv[..sep].iter().any(|a| a.starts_with(want)), "{want}");
        }
    }

    #[test]
    fn no_host_root_bind_and_etc_not_bound() {
        let argv = spec().argv_lossy();
        for (flag, src, dst) in binds(&argv) {
            let src = norm(&src);
            assert_ne!(src.trim_end_matches('/'), "", "{flag} binds host root");
            assert_ne!(
                dst.trim_end_matches('/'),
                "",
                "{flag} mounts over sandbox root"
            );
            for forbidden in ["/etc", "/proc", "/var", "/home", "/root", "/sys"] {
                assert!(
                    src != forbidden && !src.starts_with(&format!("{forbidden}/")),
                    "{flag} {src} exposes {forbidden}"
                );
            }
        }
        // The env file's directory specifically.
        assert!(!argv.iter().any(|a| a.contains("/etc/pnc-compile")));
    }

    #[test]
    fn job_dir_is_the_only_writable_bind() {
        let argv = spec().argv_lossy();
        let writable: Vec<_> = binds(&argv)
            .into_iter()
            .filter(|(flag, _, _)| !flag.starts_with("--ro-"))
            .collect();
        assert_eq!(writable.len(), 1, "{writable:?}");
        let (flag, src, dst) = &writable[0];
        assert_eq!(flag, "--bind");
        assert_eq!(norm(src), JOB);
        assert_eq!(dst, SANDBOX_JOB_DIR);
        // Manifest and source are re-mounted read-only over it.
        let ro: Vec<_> = binds(&argv)
            .into_iter()
            .filter(|(f, _, _)| f == "--ro-bind")
            .map(|(_, _, d)| d)
            .collect();
        assert!(ro.contains(&format!("{SANDBOX_JOB_DIR}/Cargo.toml")));
        assert!(ro.contains(&format!("{SANDBOX_JOB_DIR}/src")));
    }

    #[test]
    fn network_and_namespaces_are_unshared_with_fresh_proc_and_tmp() {
        let argv = spec().argv_lossy();
        let b = bwrap_args(&argv);
        assert!(b.iter().any(|a| a == "--unshare-all"));
        assert!(
            !b.iter().any(|a| a == "--share-net"),
            "network must stay unshared"
        );
        assert!(b.iter().any(|a| a == "--die-with-parent"));
        assert!(b.iter().any(|a| a == "--clearenv"));
        let pair = |flag: &str, val: &str| b.windows(2).any(|w| w[0] == flag && w[1] == val);
        assert!(pair("--proc", "/proc"));
        assert!(pair("--dev", "/dev"));
        assert!(pair("--tmpfs", "/tmp"));
        assert!(!b.iter().any(|a| a == "--dev-bind"));
    }

    #[test]
    fn inner_command_is_offline_toolchain_cargo_with_clean_env() {
        let argv = spec().argv_lossy();
        let inner_start = argv.iter().rposition(|a| a == "--").unwrap() + 1;
        let inner = &argv[inner_start..];
        assert_eq!(norm(&inner[0]), format!("{TOOLCHAIN}/bin/cargo"));
        assert!(inner.iter().any(|a| a == "--offline"));
        assert!(inner.iter().any(|a| a == "wasm32-wasip1"));
        assert!(inner.contains(&format!("{SANDBOX_JOB_DIR}/Cargo.toml")));
        // Every --setenv is one we chose; nothing secret-shaped.
        let b = bwrap_args(&argv);
        let set: Vec<&str> = b
            .windows(2)
            .filter(|w| w[0] == "--setenv")
            .map(|w| w[1].as_str())
            .collect();
        assert!(set.contains(&"PATH") && set.contains(&"RUSTC"));
        assert!(!set.iter().any(|k| k.starts_with("PNC_")), "{set:?}");
    }

    #[test]
    fn bwrap_requires_a_sane_toolchain_dir() {
        let mut s = settings();
        s.toolchain_dir = None;
        assert!(matches!(
            cargo_build_spec(&s, Path::new(JOB)),
            Err(SandboxError::MissingToolchain)
        ));
        for bad in [
            "/",
            "relative/dir",
            "/etc/rust",
            "/proc/self",
            "/home/x/.rustup",
            "/opt/../etc",
        ] {
            assert!(validate_toolchain_dir(Path::new(bad)).is_err(), "{bad}");
        }
        assert!(validate_toolchain_dir(Path::new(TOOLCHAIN)).is_ok());
    }

    #[test]
    fn none_mode_is_host_cargo_with_allowlisted_env() {
        let s = cargo_build_spec(&SandboxSettings::default(), Path::new("job")).unwrap();
        assert_eq!(s.program, OsString::from("cargo"));
        for (k, _) in &s.env {
            let k = k.to_string_lossy();
            assert!(
                HOST_ENV_ALLOWLIST.contains(&k.as_ref())
                    || k == "CARGO_TARGET_DIR"
                    || k == "CARGO_TERM_COLOR",
                "unexpected env {k}"
            );
        }
        assert!(s.args.iter().any(|a| a == "--offline"));
    }

    #[test]
    fn address_space_cap_is_below_unit_memory_max() {
        let unit = include_str!("../deploy/compile-api.service");
        let mem_max_mib: u64 = unit
            .lines()
            .find_map(|l| l.trim().strip_prefix("MemoryMax="))
            .and_then(|v| v.strip_suffix('M'))
            .and_then(|v| v.parse().ok())
            .expect("unit must set MemoryMax=<n>M");
        assert!(MAX_BUILD_ADDRESS_SPACE_MIB < mem_max_mib);
        const { assert!(DEFAULT_BUILD_ADDRESS_SPACE_MIB <= MAX_BUILD_ADDRESS_SPACE_MIB) };
        for directive in [
            "OOMPolicy=continue",
            "StartLimitIntervalSec=",
            "StartLimitBurst=",
        ] {
            assert!(unit.contains(directive), "unit is missing {directive}");
        }
    }
}

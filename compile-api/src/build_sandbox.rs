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
//!     against), with `/bin`, `/lib`, `/lib64` as symlinks into it, and
//!     an empty tmpfs over `/usr/local` (operator scripts live there);
//!   - the Rust toolchain dir (`PNC_TOOLCHAIN_DIR`) read-only;
//!   - the per-request job dir, bind-mounted read-write at `/build`, with
//!     its `Cargo.toml` and `src/` re-mounted read-only on top;
//!   - a minimal `/dev` and an empty tmpfs `/tmp`;
//!   - `/proc`: by default NONE at all ([`BwrapProc::None`]). rustc finds
//!     its sysroot with `dladdr`, cargo is given `RUSTC` explicitly, and
//!     `LD_LIBRARY_PATH` names the toolchain's `lib/` (the loader cannot
//!     expand rustc's `$ORIGIN` RUNPATH without `/proc/self/exe`), so
//!     the build does not need procfs, and not mounting one means bwrap
//!     never asks the kernel for a fresh procfs mount. That mount is only
//!     allowed when the caller's `/proc` is fully visible, which is what
//!     forced `ProtectProc=`/`ProtectKernelTunables=` off in the old unit;
//!     with no procfs mount the builder unit keeps them on. To verify on
//!     Linux (the startup self-test builds and runs a real program, so a
//!     toolchain that does need `/proc` fails closed there).
//!     [`BwrapProc::Fresh`] (`PNC_BWRAP_PROC=fresh`) restores the old
//!     `--proc /proc` (a procfs for the new PID namespace only) for the
//!     case where it does; the unit's proc protections must then go.
//!
//!   Not visible: `/`, `/etc` (so the env file), `/var`, `/home`, `/root`,
//!   any host `/proc` entry. All namespaces are unshared, network
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
//!
//! # The run sandbox (third security review, S-1)
//!
//! The built module is executed by wasmtime, and wasmtime is a JIT: a bug
//! in it would put player-controlled machine code in whatever process runs
//! it. In `bwrap` mode that process is a separate `pledge_compile_api
//! run-wasm` child started inside its own bubblewrap sandbox
//! ([`run_bwrap_spec`], used by [`crate::run_sandbox`]). The job process
//! stays the supervisor (outside, with the deadlines and the process-tree
//! kill) and the two talk over pipes only: module bytes and limits in on
//! stdin, one JSON verdict out on stdout, both capped. Inside, the run sees
//! only:
//!
//! - the server binary, read-only, at [`SANDBOX_RUN_EXE`];
//! - the shared objects that binary is linked against (the dynamic loader,
//!   libc, libm, libgcc_s), each bound read-only at its own path, found
//!   from the job process's own `/proc/self/maps` ([`parse_mapped_objects`]);
//! - nothing else: no `/usr`, no `/etc`, no `/proc`, no `/dev`, no `/tmp`,
//!   no `/run` (so no builder socket and no host Unix socket), no job dir,
//!   no toolchain; an empty network namespace (loopback only), new PID,
//!   IPC, UTS, cgroup and user namespaces, no capabilities, an environment
//!   with only `LD_LIBRARY_PATH` (and `RUST_LOG` when the job has it), and
//!   it dies with its parent.
//!
//! `prlimit` adds `RLIMIT_CPU`, `RLIMIT_NOFILE`, `RLIMIT_FSIZE=0` and no
//! core dumps. There is deliberately no `RLIMIT_AS`: wasmtime reserves
//! several GiB of address space per linear memory (guard regions) and the
//! job process never had one either; real memory stays bounded by the
//! store limiter and the unit's `MemoryMax`.
//!
//! The JIT needs nothing extra from the sandbox: compiled code lives in
//! anonymous memory that wasmtime maps writable, fills and then
//! `mprotect`s executable. Mount flags (`noexec`, `NoExecPaths=`) apply
//! to file mappings only, bubblewrap installs no seccomp filter of its own,
//! and the builder unit leaves `MemoryDenyWriteExecute=` off (verified on
//! Linux: the startup self-test runs a module through this sandbox).

use std::ffi::{OsStr, OsString};
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

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
///
/// `Default` (`None`) is the *programmatic* default used by
/// `Config::default()` for in-process tests and local dev, which is always
/// loopback with no tokens. The *environment* default, used when
/// `PNC_BUILD_SANDBOX` is unset, is [`SandboxMode::platform_default`]:
/// `bwrap` on Linux.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SandboxMode {
    /// Host build, dev only.
    #[default]
    None,
    /// bubblewrap + prlimit (Linux).
    Bwrap,
}

impl SandboxMode {
    /// Mode used when `PNC_BUILD_SANDBOX` is unset: `bwrap` on Linux (a
    /// deploy that forgets the variable gets the sandbox, or a loud
    /// startup error if the toolchain dir is missing), `none` elsewhere
    /// (bwrap does not exist there).
    pub fn platform_default() -> Self {
        Self::default_for(cfg!(target_os = "linux"))
    }

    /// [`SandboxMode::platform_default`] with the platform explicit, for
    /// tests.
    pub fn default_for(linux: bool) -> Self {
        if linux { Self::Bwrap } else { Self::None }
    }
}

/// `PNC_BWRAP_PROC`: what the sandbox sees at `/proc`. See module docs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BwrapProc {
    /// No `/proc` inside the sandbox (default).
    #[default]
    None,
    /// `--proc /proc`: a procfs for the sandbox's own PID namespace.
    Fresh,
}

impl FromStr for BwrapProc {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "none" => Ok(Self::None),
            "fresh" => Ok(Self::Fresh),
            _ => Err("expected `none` or `fresh`".to_string()),
        }
    }
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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
            cpu_secs: DEFAULT_BUILD_CPU_SECS,
            file_size_bytes: DEFAULT_BUILD_FILE_SIZE_BYTES,
            nofile: DEFAULT_BUILD_NOFILE,
        }
    }
}

/// Default `RLIMIT_CPU` per build process, seconds.
pub const DEFAULT_BUILD_CPU_SECS: u64 = 90;
/// Default `RLIMIT_FSIZE` per build process.
pub const DEFAULT_BUILD_FILE_SIZE_BYTES: u64 = 64 * 1024 * 1024;
/// Default `RLIMIT_NOFILE` per build process.
pub const DEFAULT_BUILD_NOFILE: u64 = 1024;

impl BuildRlimits {
    /// Reject limits that are looser than the shipped defaults (or the
    /// address-space ceiling). Used when settings arrive over a process
    /// boundary (`jobs`), so a confused or compromised parent cannot
    /// widen them.
    pub fn validate(&self) -> Result<(), String> {
        let as_range =
            MIN_BUILD_ADDRESS_SPACE_MIB * 1024 * 1024..=MAX_BUILD_ADDRESS_SPACE_MIB * 1024 * 1024;
        if !as_range.contains(&self.address_space_bytes) {
            return Err(format!(
                "address_space_bytes {} outside {as_range:?}",
                self.address_space_bytes
            ));
        }
        if self.cpu_secs == 0 || self.cpu_secs > DEFAULT_BUILD_CPU_SECS {
            return Err(format!("cpu_secs {} outside 1..=90", self.cpu_secs));
        }
        if self.file_size_bytes == 0 || self.file_size_bytes > DEFAULT_BUILD_FILE_SIZE_BYTES {
            return Err(format!(
                "file_size_bytes {} too large",
                self.file_size_bytes
            ));
        }
        if self.nofile == 0 || self.nofile > DEFAULT_BUILD_NOFILE {
            return Err(format!("nofile {} too large", self.nofile));
        }
        Ok(())
    }
}

/// Everything needed to construct a sandboxed build command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SandboxSettings {
    pub mode: SandboxMode,
    /// Toolchain root containing `bin/cargo`, `bin/rustc` and
    /// `lib/rustlib/` (e.g. `/opt/rust/rustup/toolchains/stable-x86_64-unknown-linux-gnu`).
    /// Required for `bwrap`; unused for `none` (cargo comes from `PATH`).
    pub toolchain_dir: Option<PathBuf>,
    pub bwrap_path: PathBuf,
    pub prlimit_path: PathBuf,
    pub rlimits: BuildRlimits,
    /// What the sandbox sees at `/proc` (bwrap only).
    #[serde(default)]
    pub proc_mode: BwrapProc,
}

impl Default for SandboxSettings {
    fn default() -> Self {
        Self {
            mode: SandboxMode::None,
            toolchain_dir: None,
            bwrap_path: PathBuf::from(DEFAULT_BWRAP_PATH),
            prlimit_path: PathBuf::from(DEFAULT_PRLIMIT_PATH),
            rlimits: BuildRlimits::default(),
            proc_mode: BwrapProc::None,
        }
    }
}

impl SandboxSettings {
    /// Structural checks for settings received over a process boundary.
    pub fn validate(&self) -> Result<(), String> {
        self.rlimits.validate()?;
        if let Some(tc) = &self.toolchain_dir {
            validate_toolchain_dir(tc)?;
        }
        if self.mode == SandboxMode::Bwrap && self.toolchain_dir.is_none() {
            return Err("bwrap mode needs a toolchain dir".to_string());
        }
        for (name, p) in [
            ("bwrap_path", &self.bwrap_path),
            ("prlimit_path", &self.prlimit_path),
        ] {
            if !p.is_absolute() && !p.to_string_lossy().starts_with('/') {
                return Err(format!("{name} must be absolute"));
            }
        }
        Ok(())
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
        // ...minus /usr/local: the one host-writable part of /usr, where
        // operators keep their own scripts (on the first Linux host it
        // held world-readable files with credentials). The build needs
        // nothing from it; a toolchain configured under it is bound back
        // on top below.
        os("--tmpfs"),
        os("/usr/local"),
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
        // Minimal /dev, empty /tmp. /proc: see `BwrapProc`.
        os("--dev"),
        os("/dev"),
        os("--tmpfs"),
        os("/tmp"),
        os("--chdir"),
        os(SANDBOX_JOB_DIR),
    ];
    if settings.proc_mode == BwrapProc::Fresh {
        args.push(os("--proc"));
        args.push(os("/proc"));
    }
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
///
/// `LD_LIBRARY_PATH` is required by the no-procfs sandbox (verified on
/// Linux, 2026-11-14): rustc and rust-lld find `librustc_driver-*.so` and
/// `libLLVM.so` through `RUNPATH $ORIGIN/../lib`, and glibc's loader
/// expands `$ORIGIN` for the main program by reading `/proc/self/exe`.
/// With no `/proc` that expansion fails and rustc exits 127 ("error while
/// loading shared libraries: librustc_driver-*.so"). Naming the read-only
/// toolchain `lib/` dir directly avoids the lookup; nothing else is on it.
fn sandbox_build_env(toolchain: &Path) -> Vec<(&'static str, OsString)> {
    let mut path = toolchain.join("bin").into_os_string();
    path.push(":/usr/bin:/bin");
    vec![
        ("PATH", path),
        ("LD_LIBRARY_PATH", join_os(toolchain, "lib")),
        ("HOME", os(SANDBOX_JOB_DIR)),
        ("CARGO_HOME", os(format!("{SANDBOX_JOB_DIR}/.cargo-home"))),
        ("CARGO_TARGET_DIR", os(format!("{SANDBOX_JOB_DIR}/target"))),
        ("CARGO_TERM_COLOR", os("never")),
        ("CARGO_INCREMENTAL", os("0")),
        ("RUSTC", join_os(toolchain, "bin/rustc")),
        ("TMPDIR", os("/tmp")),
        // glibc otherwise creates one malloc arena per thread (up to 8 per
        // core); each reserves address space that counts against the
        // build's RLIMIT_AS. Second guard next to `PLAYER_LINK_THREADS`.
        ("MALLOC_ARENA_MAX", os(BUILD_MALLOC_ARENA_MAX)),
    ]
}

/// Threads `rust-lld` may use when it links player code.
///
/// lld otherwise starts one thread per host core. On a 56-core host the
/// stacks and per-thread malloc arenas of those threads sometimes pushed
/// the build past its address-space rlimit (`RLimits::address_space_bytes`,
/// 1280 MiB): `pthread_create` failed with EAGAIN, lld died with
/// `terminate called after throwing an instance of 'std::system_error'`,
/// and a correct solution came back as a compile error (6 of 40 builds,
/// 2026-11-16 Linux record). A curriculum program links in milliseconds,
/// so 4 threads cost nothing and make the link independent of core count.
pub const PLAYER_LINK_THREADS: u32 = 4;
const _: () = assert!(PLAYER_LINK_THREADS >= 1 && PLAYER_LINK_THREADS <= 4);

/// `MALLOC_ARENA_MAX` inside the build sandbox (see `sandbox_build_env`).
/// Measured alone it also removed the lld crash (0 of 40).
pub const BUILD_MALLOC_ARENA_MAX: &str = "2";

/// The cargo `--config` value that caps lld's threads for the wasm target.
/// Passed on the command line, so it outranks every cargo config file;
/// the build environment carries no `RUSTFLAGS`/`CARGO_ENCODED_RUSTFLAGS`
/// that could replace it. rustc hands `-C link-arg` straight to rust-lld
/// (`--threads=N` on its argv, verified with `--print link-args` on
/// 1.99.0).
pub fn player_link_threads_config() -> String {
    format!(
        "target.{WASIP1_TARGET}.rustflags=[\"-C\", \"link-arg=--threads={PLAYER_LINK_THREADS}\"]"
    )
}

fn cargo_build_args(manifest: OsString) -> Vec<OsString> {
    vec![
        os("build"),
        os("--target"),
        os(WASIP1_TARGET),
        os("--release"),
        os("--offline"),
        os("--quiet"),
        os("--config"),
        os(player_link_threads_config()),
        os("--manifest-path"),
        manifest,
    ]
}

/// Host environment variables passed through in `none` mode. Mirrors the
/// pre-sandbox allow-list; nothing secret is on it.
pub(crate) const HOST_ENV_ALLOWLIST: &[&str] = &[
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

// ---- run sandbox ---------------------------------------------------------------

/// Where the server binary appears inside the run sandbox.
pub const SANDBOX_RUN_EXE: &str = "/pnc/pledge_compile_api";
/// `RLIMIT_CPU` of the run child, seconds. Module compile (10 s budget)
/// plus the run (5 s) across wasmtime's helper threads, with headroom;
/// the wall-clock deadlines are far tighter.
pub const RUN_CPU_SECS: u64 = 30;
/// `RLIMIT_NOFILE` of the run child: stdio plus the runtime's few fds.
pub const RUN_NOFILE: u64 = 64;

/// What the run sandbox binds from the host: the server binary and the
/// shared objects it is linked against.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunImage {
    /// The server binary on the host.
    pub exe: PathBuf,
    /// The dynamic loader (`ld-linux-*.so*`), when the binary has one. The
    /// sandbox runs `<loader> /pnc/pledge_compile_api ...`, so the binary's
    /// own interpreter path (`PT_INTERP`) need not exist inside.
    pub loader: Option<PathBuf>,
    /// Every other mapped shared object, in map order.
    pub libs: Vec<PathBuf>,
}

/// Host prefixes a mapped object may never come from (it would be bound
/// into the run sandbox).
const FORBIDDEN_RUN_BIND_PREFIXES: &[&str] = &[
    "/etc", "/proc", "/sys", "/dev", "/run", "/tmp", "/var", "/home", "/root",
];

/// Validate one path the run sandbox would bind (pure).
pub fn validate_run_bind(p: &Path) -> Result<(), String> {
    let s = p.to_string_lossy();
    if !s.starts_with('/') {
        return Err(format!("{s} is not absolute"));
    }
    if s.contains("/../") || s.ends_with("/..") || s.contains("/./") || s.contains("//") {
        return Err(format!("{s} has `.`/`..`/empty components"));
    }
    if s.ends_with(" (deleted)") {
        return Err(format!("{s} was deleted after it was mapped"));
    }
    for bad in FORBIDDEN_RUN_BIND_PREFIXES {
        if s.as_ref() == *bad || s.starts_with(&format!("{bad}/")) {
            return Err(format!("{s} is under {bad}"));
        }
    }
    Ok(())
}

/// Pure: the file-backed, executable mappings in a `/proc/<pid>/maps` text,
/// split into the loader and the other shared objects (excluding `exe`).
///
/// Line format: `start-end perms offset dev inode path`; the path may hold
/// spaces. Only mappings with `x` in `perms`, a non-zero inode and an
/// absolute path count (`[vdso]`, `[stack]`, anonymous memory and the JIT's
/// own code pages do not). Every path is checked with
/// [`validate_run_bind`].
pub fn parse_mapped_objects(maps: &str, exe: &Path) -> Result<RunImage, String> {
    let exe_s = exe.to_string_lossy();
    let mut loader: Option<PathBuf> = None;
    let mut libs: Vec<PathBuf> = Vec::new();
    for line in maps.lines() {
        let mut fields = line.splitn(6, ' ');
        let (Some(_range), Some(perms), Some(_off), Some(_dev), Some(inode)) = (
            fields.next(),
            fields.next(),
            fields.next(),
            fields.next(),
            fields.next(),
        ) else {
            continue;
        };
        let path = fields.next().unwrap_or("").trim_start();
        if !perms.contains('x') || inode == "0" || !path.starts_with('/') {
            continue;
        }
        if path == exe_s || path.strip_suffix(" (deleted)") == Some(exe_s.as_ref()) {
            continue;
        }
        let p = PathBuf::from(path);
        validate_run_bind(&p)?;
        let is_loader = p
            .file_name()
            .and_then(OsStr::to_str)
            .is_some_and(|n| n.starts_with("ld-linux") || n.starts_with("ld-musl"));
        if is_loader {
            if loader.as_ref().is_some_and(|l| l != &p) {
                return Err(format!(
                    "two dynamic loaders mapped: {:?} and {p:?}",
                    loader
                ));
            }
            loader = Some(p);
        } else if !libs.contains(&p) {
            libs.push(p);
        }
    }
    if loader.is_none() && !libs.is_empty() {
        return Err("shared objects are mapped but no dynamic loader was found".to_string());
    }
    Ok(RunImage {
        exe: exe.to_path_buf(),
        loader,
        libs,
    })
}

/// Pure: the `prlimit ... -- bwrap ... -- [loader] /pnc/pledge_compile_api
/// <subcommand>` command for the run sandbox. See the module docs for what
/// is (not) visible inside. `inner_env` is added to the sandbox's (cleared)
/// environment after `LD_LIBRARY_PATH`.
pub fn run_bwrap_spec(
    settings: &SandboxSettings,
    image: &RunImage,
    subcommand: &str,
    inner_env: &[(&str, OsString)],
) -> CommandSpec {
    let mut args: Vec<OsString> = vec![
        os(format!("--cpu={RUN_CPU_SECS}")),
        // The run writes no files at all: stdout/stderr are pipes.
        os("--fsize=0"),
        os(format!("--nofile={RUN_NOFILE}")),
        os("--core=0"),
        os("--"),
        settings.bwrap_path.clone().into_os_string(),
        // user, ipc, pid, net, uts, cgroup (and mount) namespaces.
        os("--unshare-all"),
        os("--die-with-parent"),
        os("--cap-drop"),
        os("ALL"),
        os("--clearenv"),
    ];
    // Loader and libraries, each a single read-only file at its own path.
    for lib in image.loader.iter().chain(image.libs.iter()) {
        args.push(os("--ro-bind"));
        args.push(lib.as_os_str().to_os_string());
        args.push(lib.as_os_str().to_os_string());
    }
    args.push(os("--ro-bind"));
    args.push(image.exe.as_os_str().to_os_string());
    args.push(os(SANDBOX_RUN_EXE));
    args.push(os("--chdir"));
    args.push(os("/"));
    // The loader finds the libraries here (there is no ld.so.cache: /etc
    // is not in the sandbox).
    let mut lib_dirs: Vec<&Path> = Vec::new();
    for lib in image.loader.iter().chain(image.libs.iter()) {
        if let Some(dir) = lib.parent()
            && !lib_dirs.contains(&dir)
        {
            lib_dirs.push(dir);
        }
    }
    if !lib_dirs.is_empty() {
        let joined = lib_dirs
            .iter()
            .map(|d| d.to_string_lossy().replace('\\', "/"))
            .collect::<Vec<_>>()
            .join(":");
        args.push(os("--setenv"));
        args.push(os("LD_LIBRARY_PATH"));
        args.push(os(joined));
    }
    for (k, v) in inner_env {
        args.push(os("--setenv"));
        args.push(os(k));
        args.push(v.clone());
    }
    args.push(os("--"));
    if let Some(loader) = &image.loader {
        args.push(loader.as_os_str().to_os_string());
    }
    args.push(os(SANDBOX_RUN_EXE));
    args.push(os(subcommand));
    CommandSpec {
        program: settings.prlimit_path.clone().into_os_string(),
        args,
        env: Vec::new(),
        cwd: None,
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
    if settings.proc_mode == BwrapProc::None {
        let procfs = run("/proc absent", &["/usr/bin/test", "-e", "/proc/self"])?;
        if procfs.status.code() != Some(1) {
            return Err(probe_err(
                "/proc absent",
                format!(
                    "expected exit 1, got {:?}: {}",
                    procfs.status,
                    procfs.stderr.trim()
                ),
            ));
        }
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
    fn network_and_namespaces_are_unshared_with_no_proc_and_fresh_tmp() {
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
        // Default: no procfs at all (no mount request, no host /proc).
        assert!(!b.iter().any(|a| a == "--proc"), "{b:?}");
        assert!(pair("--dev", "/dev"));
        assert!(pair("--tmpfs", "/tmp"));
        // /usr is bound, but its host-writable /usr/local is hidden, and
        // that happens AFTER the /usr bind (bwrap applies args in order).
        let usr = b
            .windows(3)
            .position(|w| w == ["--ro-bind", "/usr", "/usr"]);
        let local = b.windows(2).position(|w| w == ["--tmpfs", "/usr/local"]);
        assert!(matches!((usr, local), (Some(u), Some(l)) if u < l), "{b:?}");
        assert!(!b.iter().any(|a| a == "--dev-bind"));

        // PNC_BWRAP_PROC=fresh: a procfs for the sandbox's PID namespace.
        let mut s = settings();
        s.proc_mode = BwrapProc::Fresh;
        let argv = cargo_build_spec(&s, Path::new(JOB)).unwrap().argv_lossy();
        let b = bwrap_args(&argv);
        assert!(b.windows(2).any(|w| w[0] == "--proc" && w[1] == "/proc"));
        assert_eq!("fresh".parse::<BwrapProc>(), Ok(BwrapProc::Fresh));
        assert!("host".parse::<BwrapProc>().is_err());
    }

    #[test]
    fn platform_default_is_bwrap_on_linux_only() {
        assert_eq!(SandboxMode::default_for(true), SandboxMode::Bwrap);
        assert_eq!(SandboxMode::default_for(false), SandboxMode::None);
    }

    #[test]
    fn received_settings_cannot_loosen_rlimits() {
        assert!(settings().validate().is_ok());
        let mut s = settings();
        s.rlimits.cpu_secs = 10_000;
        assert!(s.validate().is_err());
        let mut s = settings();
        s.rlimits.address_space_bytes = 64 << 30;
        assert!(s.validate().is_err());
        let mut s = settings();
        s.toolchain_dir = Some(PathBuf::from("/etc/rust"));
        assert!(s.validate().is_err());
        let mut s = settings();
        s.toolchain_dir = None;
        assert!(s.validate().is_err(), "bwrap without toolchain");
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
        // No procfs inside: the loader needs the toolchain lib dir named
        // explicitly (rustc's RUNPATH is $ORIGIN-relative).
        assert!(
            b.windows(3).any(|w| w[0] == "--setenv"
                && w[1] == "LD_LIBRARY_PATH"
                && norm(&w[2]) == format!("{TOOLCHAIN}/lib")),
            "{b:?}"
        );
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

    /// The `--config <value>` pair in an argv, if any.
    fn config_values(argv: &[String]) -> Vec<&str> {
        argv.windows(2)
            .filter(|w| w[0] == "--config")
            .map(|w| w[1].as_str())
            .collect()
    }

    #[test]
    fn linker_threads_are_capped_in_every_sandbox_mode() {
        let want = "target.wasm32-wasip1.rustflags=[\"-C\", \"link-arg=--threads=4\"]";
        assert_eq!(player_link_threads_config(), want);
        // bwrap (production): on the inner cargo argv, after the last `--`.
        let argv = spec().argv_lossy();
        let inner_start = argv.iter().rposition(|a| a == "--").unwrap() + 1;
        assert_eq!(config_values(&argv[inner_start..]), vec![want], "{argv:?}");
        // none (dev): on host cargo's argv.
        let none = cargo_build_spec(&SandboxSettings::default(), Path::new("job")).unwrap();
        assert_eq!(config_values(&none.argv_lossy()), vec![want]);
        // Nothing in either environment can replace target rustflags.
        let b = bwrap_args(&argv);
        let set: Vec<&str> = b
            .windows(2)
            .filter(|w| w[0] == "--setenv")
            .map(|w| w[1].as_str())
            .collect();
        for env in [
            set,
            none.env.iter().filter_map(|(k, _)| k.to_str()).collect(),
        ] {
            assert!(
                !env.iter()
                    .any(|k| k.contains("RUSTFLAGS") || k.starts_with("CARGO_TARGET_W")),
                "{env:?}"
            );
        }
    }

    #[test]
    fn build_sandbox_limits_malloc_arenas() {
        let b = bwrap_args(&spec().argv_lossy()).to_vec();
        assert!(
            b.windows(3)
                .any(|w| w[0] == "--setenv" && w[1] == "MALLOC_ARENA_MAX" && w[2] == "2"),
            "{b:?}"
        );
    }

    // ---- run sandbox ----------------------------------------------------

    const EXE: &str = "/usr/local/bin/pledge_compile_api";
    /// Trimmed real `/proc/self/maps` shape (openSUSE, glibc 2.40).
    const MAPS: &str = "\
55d0c0a00000-55d0c0b00000 r--p 00000000 00:2a 1234 /usr/local/bin/pledge_compile_api
55d0c0b00000-55d0c1500000 r-xp 00100000 00:2a 1234 /usr/local/bin/pledge_compile_api
7f1e00000000-7f1e00021000 rw-p 00000000 00:00 0
7f1e10000000-7f1e10200000 r-xp 00000000 00:00 0
7f1e20000000-7f1e20028000 r--p 00000000 00:2a 99 /usr/lib64/libc.so.6
7f1e20028000-7f1e201a0000 r-xp 00028000 00:2a 99 /usr/lib64/libc.so.6
7f1e20400000-7f1e20480000 r-xp 00010000 00:2a 98 /usr/lib64/libm.so.6
7f1e20500000-7f1e20520000 r-xp 00003000 00:2a 97 /usr/lib64/libgcc_s.so.1
7f1e20600000-7f1e20601000 r--p 00000000 00:2a 95 /usr/lib/locale/locale archive
7f1e20700000-7f1e20730000 r-xp 00001000 00:2a 96 /usr/lib64/ld-linux-x86-64.so.2
7ffd1a000000-7ffd1a021000 rw-p 00000000 00:00 0 [stack]
7ffd1a1fe000-7ffd1a200000 r-xp 00000000 00:00 0 [vdso]
";

    fn image() -> RunImage {
        parse_mapped_objects(MAPS, Path::new(EXE)).expect("maps parse")
    }

    fn run_spec() -> CommandSpec {
        run_bwrap_spec(
            &settings(),
            &image(),
            "run-wasm",
            &[("RUST_LOG", os("warn"))],
        )
    }

    #[test]
    fn maps_parser_finds_exactly_the_loader_and_libraries() {
        let img = image();
        assert_eq!(img.exe, PathBuf::from(EXE));
        assert_eq!(
            img.loader,
            Some(PathBuf::from("/usr/lib64/ld-linux-x86-64.so.2"))
        );
        assert_eq!(
            img.libs,
            [
                "/usr/lib64/libc.so.6",
                "/usr/lib64/libm.so.6",
                "/usr/lib64/libgcc_s.so.1"
            ]
            .map(PathBuf::from)
        );
        // Non-executable file maps (the locale archive), anonymous memory,
        // JIT pages, [stack] and [vdso] are not bound.
        let all: Vec<_> = img.libs.iter().chain(img.loader.iter()).collect();
        assert!(!all.iter().any(|p| p.to_string_lossy().contains("locale")));
        // A statically linked binary maps nothing else: no loader, no libs.
        let static_maps = "1000-2000 r-xp 00000000 00:2a 1 /usr/local/bin/pledge_compile_api\n";
        let s = parse_mapped_objects(static_maps, Path::new(EXE)).unwrap();
        assert!(s.loader.is_none() && s.libs.is_empty());
    }

    #[test]
    fn maps_parser_refuses_objects_from_host_areas_or_deleted_files() {
        for bad in [
            "1000-2000 r-xp 0 00:2a 5 /tmp/evil.so",
            "1000-2000 r-xp 0 00:2a 5 /etc/ld.so.preload.so",
            "1000-2000 r-xp 0 00:2a 5 /home/x/lib.so",
            "1000-2000 r-xp 0 00:2a 5 /usr/lib64/libc.so.6 (deleted)",
            "1000-2000 r-xp 0 00:2a 5 /usr/lib64/../../etc/x.so",
            "1000-2000 r-xp 0 00:2a 5 /run/user/1/x.so",
        ] {
            let maps = format!("{bad}\n1000-2000 r-xp 0 00:2a 6 /usr/lib64/ld-linux-x86-64.so.2\n");
            assert!(
                parse_mapped_objects(&maps, Path::new(EXE)).is_err(),
                "{bad}"
            );
        }
        // Libraries without a loader cannot be started.
        assert!(
            parse_mapped_objects(
                "1000-2000 r-xp 0 00:2a 5 /usr/lib64/libc.so.6\n",
                Path::new(EXE)
            )
            .is_err()
        );
    }

    #[test]
    fn run_sandbox_outer_is_prlimit_then_bwrap_with_empty_env_and_run_rlimits() {
        let s = run_spec();
        assert_eq!(s.program, OsString::from(DEFAULT_PRLIMIT_PATH));
        assert!(s.env.is_empty(), "outer env must be empty: {:?}", s.env);
        let argv = s.argv_lossy();
        let sep = argv.iter().position(|a| a == "--").unwrap();
        assert_eq!(argv[sep + 1], DEFAULT_BWRAP_PATH);
        let pre = &argv[..sep];
        assert!(pre.contains(&format!("--cpu={RUN_CPU_SECS}")));
        assert!(pre.contains(&"--fsize=0".to_string()));
        assert!(pre.contains(&format!("--nofile={RUN_NOFILE}")));
        assert!(pre.contains(&"--core=0".to_string()));
    }

    #[test]
    fn run_sandbox_binds_only_the_binary_and_its_libraries_read_only() {
        let argv = run_spec().argv_lossy();
        let b = binds(&argv);
        assert!(
            b.iter().all(|(flag, _, _)| flag == "--ro-bind"),
            "every bind is read-only: {b:?}"
        );
        let mut srcs: Vec<String> = b.iter().map(|(_, s, _)| norm(s)).collect();
        srcs.sort();
        let mut want = vec![
            EXE.to_string(),
            "/usr/lib64/ld-linux-x86-64.so.2".to_string(),
            "/usr/lib64/libc.so.6".to_string(),
            "/usr/lib64/libgcc_s.so.1".to_string(),
            "/usr/lib64/libm.so.6".to_string(),
        ];
        want.sort();
        assert_eq!(srcs, want, "exactly the binary and its libraries");
        // Libraries at their own path; the binary at the fixed in-sandbox path.
        for (_, src, dst) in &b {
            if norm(src) == EXE {
                assert_eq!(dst, SANDBOX_RUN_EXE);
            } else {
                assert_eq!(norm(src), *dst);
            }
        }
        let bw = bwrap_args(&argv);
        // No directory of the host at all: no /usr, /etc, /proc, /dev,
        // /tmp, /run, job dir or toolchain.
        for flag in [
            "--proc",
            "--dev",
            "--tmpfs",
            "--bind",
            "--dev-bind",
            "--symlink",
        ] {
            assert!(!bw.iter().any(|a| a == flag), "{flag} in {bw:?}");
        }
        for host in ["/usr", "/etc", "/run", "/tmp", TOOLCHAIN, JOB] {
            assert!(
                !b.iter().any(|(_, s, _)| norm(s) == host),
                "{host} bound: {b:?}"
            );
        }
        assert!(!argv.iter().any(|a| a.contains("builder.sock")));
    }

    #[test]
    fn run_sandbox_unshares_everything_and_runs_the_subcommand_via_the_loader() {
        let argv = run_spec().argv_lossy();
        let b = bwrap_args(&argv);
        for flag in ["--unshare-all", "--die-with-parent", "--clearenv"] {
            assert!(b.iter().any(|a| a == flag), "{flag}");
        }
        assert!(b.windows(2).any(|w| w == ["--cap-drop", "ALL"]));
        assert!(!b.iter().any(|a| a == "--share-net"));
        let set: Vec<(&str, &str)> = b
            .windows(3)
            .filter(|w| w[0] == "--setenv")
            .map(|w| (w[1].as_str(), w[2].as_str()))
            .collect();
        assert_eq!(
            set,
            [("LD_LIBRARY_PATH", "/usr/lib64"), ("RUST_LOG", "warn")],
            "only the loader path and the log filter"
        );
        let inner_start = argv.iter().rposition(|a| a == "--").unwrap() + 1;
        assert_eq!(
            &argv[inner_start..],
            [
                "/usr/lib64/ld-linux-x86-64.so.2".to_string(),
                SANDBOX_RUN_EXE.to_string(),
                "run-wasm".to_string()
            ]
        );
        // Statically linked: the binary runs directly.
        let img = RunImage {
            exe: PathBuf::from(EXE),
            loader: None,
            libs: Vec::new(),
        };
        let argv = run_bwrap_spec(&settings(), &img, "run-probe", &[]).argv_lossy();
        let inner_start = argv.iter().rposition(|a| a == "--").unwrap() + 1;
        assert_eq!(&argv[inner_start..], [SANDBOX_RUN_EXE, "run-probe"]);
        assert!(!argv.iter().any(|a| a == "LD_LIBRARY_PATH"));
    }

    #[test]
    fn address_space_cap_is_below_unit_memory_max() {
        // Builds run under the builder unit.
        let unit = include_str!("../deploy/pnc-builder.service");
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

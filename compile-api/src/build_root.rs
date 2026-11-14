// SPDX-License-Identifier: AGPL-3.0-or-later
//! The directory that holds per-request build job dirs.
//!
//! Every `/compile-real` request gets `<root>/<uuid-v4>/`. This module
//! owns creating that root safely, creating job dirs inside it, deleting
//! them, and sweeping job dirs left behind by a crash.
//!
//! # Root checks (Unix)
//!
//! A shared temp dir is hostile ground: another local user could pre-create
//! `/tmp/pledge-wasm-builder` as a symlink or a world-writable dir and then
//! swap files under a running build. [`prepare`] therefore creates the root
//! with mode `0700` and refuses to use an existing root that is a symlink,
//! not a directory, not owned by our effective uid, or group/world
//! writable. (Under the systemd unit `/tmp` is already a private per-service
//! directory; these checks matter on a dev box and if the unit changes.)
//! On Windows only the "is a real directory" check applies.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use uuid::Uuid;

/// Job dirs older than this at startup are leftovers from a crash. Any
/// live build is far younger: the build watchdog is 30 s.
pub const STALE_JOB_AGE: Duration = Duration::from_secs(10 * 60);

/// Build-root failures. Paths are server-owned, never player input.
#[derive(Debug, thiserror::Error)]
pub enum BuildRootError {
    #[error("build root {path}: {op} failed: {source}")]
    Io {
        path: PathBuf,
        op: &'static str,
        #[source]
        source: std::io::Error,
    },
    #[error("build root {path} is unsafe to use: {reason}")]
    Unsafe { path: PathBuf, reason: String },
}

fn io_err<'a>(
    path: &'a Path,
    op: &'static str,
) -> impl FnOnce(std::io::Error) -> BuildRootError + 'a {
    move |source| BuildRootError::Io {
        path: path.to_path_buf(),
        op,
        source,
    }
}

/// A `DirBuilder` that creates owner-only (0700) directories on Unix.
#[cfg_attr(not(unix), allow(unused_mut))]
fn private_dir_builder() -> fs::DirBuilder {
    let mut builder = fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder
}

/// Default root: `<temp>/<name>`.
pub fn default_root(name: &str) -> PathBuf {
    std::env::temp_dir().join(name)
}

/// Create `root` if missing (mode 0700 on Unix) and verify it is safe.
pub fn prepare(root: &Path) -> Result<(), BuildRootError> {
    match fs::symlink_metadata(root) {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            if let Some(parent) = root.parent() {
                fs::create_dir_all(parent).map_err(io_err(parent, "create parent"))?;
            }
            match private_dir_builder().create(root) {
                Ok(()) => tracing::info!("build_root: created {}", root.display()),
                // Lost a race with a concurrent creator: verify it below.
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(e) => return Err(io_err(root, "create")(e)),
            }
        }
        Err(e) => return Err(io_err(root, "stat")(e)),
    }
    verify(root)
}

fn verify(root: &Path) -> Result<(), BuildRootError> {
    let unsafe_root = |reason: String| {
        tracing::error!("build_root: refusing {}: {reason}", root.display());
        BuildRootError::Unsafe {
            path: root.to_path_buf(),
            reason,
        }
    };
    let meta = fs::symlink_metadata(root).map_err(io_err(root, "stat"))?;
    if meta.file_type().is_symlink() {
        return Err(unsafe_root("it is a symlink".to_string()));
    }
    if !meta.is_dir() {
        return Err(unsafe_root("it is not a directory".to_string()));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        let euid = rustix::process::geteuid().as_raw();
        if meta.uid() != euid {
            return Err(unsafe_root(format!(
                "owned by uid {} but we run as uid {euid}",
                meta.uid()
            )));
        }
        let mode = meta.permissions().mode() & 0o7777;
        if mode & 0o022 != 0 {
            return Err(unsafe_root(format!(
                "mode {mode:o} is group- or world-writable"
            )));
        }
    }
    Ok(())
}

/// Create a fresh `<root>/<uuid>/src/` job dir. The name is never derived
/// from input. Re-verifies the root first (cheap: one `lstat`).
pub fn create_job_dir(root: &Path) -> Result<PathBuf, BuildRootError> {
    prepare(root)?;
    let dir = root.join(Uuid::new_v4().to_string());
    let builder = private_dir_builder();
    builder
        .create(&dir)
        .map_err(io_err(&dir, "create job dir"))?;
    let src = dir.join("src");
    builder.create(&src).map_err(io_err(&src, "create src"))?;
    tracing::debug!("build_root: created job dir {}", dir.display());
    Ok(dir)
}

/// Deletes a job dir on drop, on every exit path.
pub struct JobDirGuard {
    path: PathBuf,
}

impl JobDirGuard {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for JobDirGuard {
    fn drop(&mut self) {
        if self.path.exists()
            && let Err(e) = fs::remove_dir_all(&self.path)
        {
            tracing::warn!("build_root: failed to clean {}: {e}", self.path.display());
        }
    }
}

/// Remove job dirs under `root` whose name is a UUID and whose mtime is
/// older than `max_age`. Anything else in the root is left alone. Returns
/// the number removed. Run once at startup.
pub fn sweep_stale(root: &Path, max_age: Duration) -> Result<usize, BuildRootError> {
    let entries = match fs::read_dir(root) {
        Ok(e) => e,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(0),
        Err(e) => return Err(io_err(root, "read_dir")(e)),
    };
    let now = SystemTime::now();
    let mut removed = 0;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        if Uuid::parse_str(name).is_err() {
            continue;
        }
        let Ok(meta) = entry.metadata() else { continue };
        let age = meta
            .modified()
            .ok()
            .and_then(|m| now.duration_since(m).ok())
            .unwrap_or_default();
        if !meta.is_dir() || age < max_age {
            continue;
        }
        match fs::remove_dir_all(entry.path()) {
            Ok(()) => {
                removed += 1;
                tracing::info!("build_root: swept stale job dir {name} (age {age:?})");
            }
            Err(e) => tracing::warn!("build_root: could not sweep {name}: {e}"),
        }
    }
    Ok(removed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(tag: &str) -> PathBuf {
        default_root(&format!("pnc-build-root-test-{tag}-{}", Uuid::new_v4()))
    }

    #[test]
    fn prepare_creates_and_job_dirs_are_cleaned() {
        let root = scratch("create");
        prepare(&root).unwrap();
        let job = create_job_dir(&root).unwrap();
        assert!(job.join("src").is_dir());
        {
            let _g = JobDirGuard::new(job.clone());
        }
        assert!(!job.exists());
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn prepare_refuses_a_file() {
        let root = scratch("file");
        fs::write(&root, b"x").unwrap();
        assert!(matches!(prepare(&root), Err(BuildRootError::Unsafe { .. })));
        fs::remove_file(&root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn prepare_refuses_world_writable_and_symlink() {
        use std::os::unix::fs::PermissionsExt;
        let root = scratch("perm");
        fs::create_dir(&root).unwrap();
        fs::set_permissions(&root, fs::Permissions::from_mode(0o777)).unwrap();
        assert!(matches!(prepare(&root), Err(BuildRootError::Unsafe { .. })));
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
        assert!(prepare(&root).is_ok());
        let link = scratch("link");
        std::os::unix::fs::symlink(&root, &link).unwrap();
        assert!(matches!(prepare(&link), Err(BuildRootError::Unsafe { .. })));
        fs::remove_file(&link).unwrap();
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn sweep_removes_only_old_uuid_dirs() {
        let root = scratch("sweep");
        prepare(&root).unwrap();
        let job = create_job_dir(&root).unwrap();
        let other = root.join("not-a-uuid");
        fs::create_dir(&other).unwrap();
        // Fresh job dir survives a 10-minute threshold...
        assert_eq!(sweep_stale(&root, STALE_JOB_AGE).unwrap(), 0);
        assert!(job.exists());
        // ...and is swept with a zero threshold; the non-UUID dir is kept.
        assert_eq!(sweep_stale(&root, Duration::ZERO).unwrap(), 1);
        assert!(!job.exists());
        assert!(other.exists());
        fs::remove_dir_all(&root).unwrap();
    }
}

//! The std one conformance run checks against, fixed when it starts. The checker reads
//! std once per process, but each `cpl` a `// tool:` case spawns reads `CPL_STD_DIR`
//! afresh, so a checkout or std edit mid-run would give some cases the new std and others
//! the old. A `Pin` writes the std the checker loaded into a run-scoped directory, and
//! every `cpl` the run spawns reads that copy until the `Pin` drops. (`run` holds its own
//! link to the `cpl` binary the same way.)

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::RwLock;

static PINNED: RwLock<Option<PathBuf>> = RwLock::new(None);

pub struct Pin {
    dir: PathBuf,
}

impl Pin {
    /// Writes `std`, module name to text (`cpl_check::loaded_std()`), out as the std
    /// every spawned `cpl` reads.
    pub fn new(std: &[(&str, &str)]) -> Result<Pin, String> {
        static RUNS: AtomicUsize = AtomicUsize::new(0);
        let pin = Pin {
            dir: std::env::temp_dir().join(format!(
                "cpl-conform-std-{}-{}",
                std::process::id(),
                RUNS.fetch_add(1, Ordering::Relaxed)
            )),
        };
        let _ = std::fs::remove_dir_all(&pin.dir);
        for (name, text) in std {
            let path = pin.dir.join(format!("{name}.cp"));
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("cannot make {}: {e}", parent.display()))?;
            }
            std::fs::write(&path, text)
                .map_err(|e| format!("cannot write {}: {e}", path.display()))?;
        }
        *PINNED.write().unwrap_or_else(|e| e.into_inner()) = Some(pin.dir.clone());
        Ok(pin)
    }
}

impl Drop for Pin {
    fn drop(&mut self) {
        *PINNED.write().unwrap_or_else(|e| e.into_inner()) = None;
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// A command running `program`, a `cpl`, against the pinned std while a `Pin` lives.
pub fn command(program: impl AsRef<std::ffi::OsStr>) -> Command {
    let mut cmd = Command::new(program);
    if let Some(std) = PINNED.read().unwrap_or_else(|e| e.into_inner()).as_ref() {
        cmd.env("CPL_STD_DIR", std);
    }
    cmd
}

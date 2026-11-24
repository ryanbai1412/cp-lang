//! The one file-change wait, shared by `cpl watch` and `cpl run --live`.
//! No portable blocking file-change API exists without a crate (and
//! external_crates is gated), so this polls the files' modified time and
//! size; its interval is this module's one sleep.

use std::path::PathBuf;
use std::time::{Duration, SystemTime};

/// Each watched path with its modified time and size, `None` while missing.
type Stamp = Vec<(PathBuf, Option<(SystemTime, u64)>)>;

/// Watches the paths `list` names, read again each round so a file that
/// appears or disappears is a change.
pub struct Watch<F: FnMut() -> Vec<PathBuf>> {
    list: F,
    every: Duration,
    seen: Stamp,
}

impl<F: FnMut() -> Vec<PathBuf>> Watch<F> {
    /// Starts from the files as they are now.
    pub fn new(every: Duration, mut list: F) -> Watch<F> {
        let seen = stamp(&mut list);
        Watch { list, every, seen }
    }

    /// Blocks until a watched file changes, appears or disappears.
    pub fn changed(&mut self) {
        loop {
            std::thread::sleep(self.every);
            let now = stamp(&mut self.list);
            if now != self.seen {
                self.seen = now;
                return;
            }
        }
    }
}

fn stamp(list: &mut impl FnMut() -> Vec<PathBuf>) -> Stamp {
    let mut paths = list();
    paths.sort();
    paths.dedup();
    paths
        .into_iter()
        .map(|path| {
            let meta = std::fs::metadata(&path).ok();
            let mark = meta.and_then(|m| Some((m.modified().ok()?, m.len())));
            (path, mark)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_write_and_a_new_file_each_end_the_wait() {
        let dir = std::env::temp_dir().join(format!("cpl-wait-files-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("a.cp");
        std::fs::write(&file, "1").unwrap();
        let listed = dir.clone();
        let mut watch = Watch::new(Duration::from_millis(5), move || {
            std::fs::read_dir(&listed)
                .map(|d| d.flatten().map(|e| e.path()).collect())
                .unwrap_or_default()
        });
        let writer = std::thread::spawn(move || std::fs::write(file, "12").unwrap());
        watch.changed();
        writer.join().unwrap();
        let added = dir.join("b.cp");
        let writer = std::thread::spawn(move || std::fs::write(added, "").unwrap());
        watch.changed();
        writer.join().unwrap();
        std::fs::remove_dir_all(dir).unwrap();
    }
}

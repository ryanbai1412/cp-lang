//! What a live `cpl run` read, kept to run it again and name the locals if
//! it traps. It goes to an unlinked temporary file, so a judge-sized input
//! costs the run no memory, as it costs the C++ none; where there is no such
//! file it stays in memory.

use std::io::{Read, Seek, Write};

pub(crate) enum Kept {
    File(std::fs::File),
    Memory(Vec<u8>),
    /// A write failed: the locals go unnamed rather than named wrongly.
    Lost,
}

impl Kept {
    pub(crate) fn new() -> Kept {
        Kept::file().map_or(Kept::Memory(Vec::new()), Kept::File)
    }

    #[cfg(unix)]
    fn file() -> Option<std::fs::File> {
        use std::sync::atomic::{AtomicU32, Ordering};
        static NEXT: AtomicU32 = AtomicU32::new(0);
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("cpl-read-{}-{n}", std::process::id()));
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&path)
            .ok()?;
        // Unlinked now, the file lives as long as it is open.
        std::fs::remove_file(&path).ok()?;
        Some(file)
    }

    #[cfg(not(unix))]
    fn file() -> Option<std::fs::File> {
        None
    }

    pub(crate) fn push(&mut self, chunk: &[u8]) {
        match self {
            Kept::File(file) => {
                if file.write_all(chunk).is_err() {
                    *self = Kept::Lost;
                }
            }
            Kept::Memory(bytes) => bytes.extend_from_slice(chunk),
            Kept::Lost => {}
        }
    }

    /// Everything pushed, in order.
    pub(crate) fn bytes(&mut self) -> Option<Vec<u8>> {
        match self {
            Kept::File(file) => {
                let mut bytes = Vec::new();
                file.rewind().ok()?;
                file.read_to_end(&mut bytes).ok()?;
                Some(bytes)
            }
            Kept::Memory(bytes) => Some(std::mem::take(bytes)),
            Kept::Lost => None,
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn what_is_kept_comes_back_whole_and_in_order() {
        let mut kept = super::Kept::new();
        kept.push(b"3\n1 2");
        kept.push(b"");
        kept.push(b" 3\n");
        assert_eq!(kept.bytes().as_deref(), Some(&b"3\n1 2 3\n"[..]));
    }
}

//! The `.cp` source files a tool is pointed at.

use std::path::{Path, PathBuf};

/// Adds `path`, or every `.cp` file under it, to `files`: brutes and
/// generators in `d.tests/` too, but not hidden folders or `target`. A
/// missing `path` is an error.
pub fn collect(path: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if !path.is_dir() {
        std::fs::metadata(path)?;
        files.push(path.to_path_buf());
        return Ok(());
    }
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let child = entry.path();
        let name = entry.file_name();
        if child.is_dir() {
            if !name.to_string_lossy().starts_with('.') && name != "target" {
                collect(&child, files)?;
            }
        } else if child.extension().is_some_and(|e| e == "cp") {
            files.push(child);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::collect;
    use std::path::PathBuf;

    #[test]
    fn walks_cp_files_but_not_hidden_folders_or_target() {
        let root = std::env::temp_dir().join(format!("cpl-syntax-files-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        for dir in ["a.tests", ".git", "target"] {
            std::fs::create_dir_all(root.join(dir)).unwrap();
        }
        for file in [
            "a.cp",
            "notes.md",
            "a.tests/brute.cp",
            ".git/x.cp",
            "target/y.cp",
        ] {
            std::fs::write(root.join(file), "").unwrap();
        }
        let mut files = Vec::new();
        collect(&root, &mut files).unwrap();
        files.sort();
        assert_eq!(files, [root.join("a.cp"), root.join("a.tests/brute.cp")]);

        let mut one = Vec::new();
        collect(&root.join("notes.md"), &mut one).unwrap();
        assert_eq!(one, [root.join("notes.md")]);
        assert!(collect(&root.join("missing.cp"), &mut Vec::<PathBuf>::new()).is_err());
        let _ = std::fs::remove_dir_all(&root);
    }
}

//! `file://` URIs for the paths the checker reports.

pub(crate) fn uri_of_path(path: &str) -> String {
    if path.contains("://") {
        return path.to_string();
    }
    #[cfg(not(target_arch = "wasm32"))]
    if let Some(file) = bundled_std_file(path) {
        return file_uri(&file.to_string_lossy());
    }
    file_uri(path)
}

/// `path` spelled the way the OS prints paths (`\` on Windows), for the
/// paths the server hands back to the editor as command arguments.
pub(crate) fn native_path(path: &str) -> String {
    native(path, cfg!(windows))
}

fn native(path: &str, windows: bool) -> String {
    if windows {
        path.replace('/', "\\")
    } else {
        path.to_string()
    }
}

/// `file://` and `path` with `\` as `/`, a `/` before a Windows drive
/// (`C:`), and every byte outside the URI path characters `%XX`-escaped,
/// so a path with a space or a drive letter round-trips through the editor.
fn file_uri(path: &str) -> String {
    let path = path.replace('\\', "/");
    let mut out = String::from("file://");
    if has_drive(&path) {
        out.push('/');
    }
    for b in path.bytes() {
        if b.is_ascii_alphanumeric() || b"/-._~:".contains(&b) {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

fn has_drive(path: &str) -> bool {
    let b = path.as_bytes();
    b.len() >= 2 && b[0].is_ascii_alphabetic() && b[1] == b':'
}

/// A standard library module built into the compiler (`std/heap.cp`),
/// written out under the user's cache directory so an editor can open it.
/// The copy sits below `std/`, so opening it is treated as the stdlib.
#[cfg(not(target_arch = "wasm32"))]
fn bundled_std_file(path: &str) -> Option<std::path::PathBuf> {
    let rest = path.strip_prefix("std/")?.strip_suffix(".cp")?;
    let module: Vec<String> = rest.split('/').map(String::from).collect();
    let text = cpl_check::bundled_std(&module)?;
    let cache = std::env::var_os("XDG_CACHE_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| std::path::Path::new(&home).join(".cache")))
        .unwrap_or_else(std::env::temp_dir);
    let file = cache
        .join("cpl")
        .join(env!("CARGO_PKG_VERSION"))
        .join("std")
        .join(format!("{rest}.cp"));
    if std::fs::read_to_string(&file).ok().as_deref() != Some(text) {
        std::fs::create_dir_all(file.parent()?).ok()?;
        std::fs::write(&file, text).ok()?;
    }
    Some(file)
}

#[cfg(test)]
mod tests {
    #[test]
    fn command_paths_use_the_os_separator() {
        assert_eq!(super::native("c:/a b/x.cp", true), "c:\\a b\\x.cp");
        assert_eq!(super::native("/a b/x.cp", false), "/a b/x.cp");
    }

    use super::*;

    #[test]
    fn file_uris_escape_spaces_and_mark_drives() {
        assert_eq!(
            file_uri("/home/a/my code/x.cp"),
            "file:///home/a/my%20code/x.cp"
        );
        assert_eq!(
            file_uri("C:\\Users\\a b\\x.cp"),
            "file:///C:/Users/a%20b/x.cp"
        );
        assert_eq!(file_uri("/bench/solution.cp"), "file:///bench/solution.cp");
    }
}

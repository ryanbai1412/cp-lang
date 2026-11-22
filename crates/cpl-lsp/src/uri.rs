//! `file://` URIs for the paths the checker reports.

pub(crate) fn uri_of_path(path: &str) -> String {
    if path.contains("://") {
        return path.to_string();
    }
    #[cfg(not(target_arch = "wasm32"))]
    if let Some(file) = bundled_std_file(path) {
        return format!("file://{}", file.display());
    }
    format!("file://{path}")
}

/// A standard library module built into the compiler (`std/heap.cp`),
/// written out under the user's cache directory so an editor can open it.
/// The copy sits below `spec/std/`, so opening it is treated as the stdlib.
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
        .join("spec/std")
        .join(format!("{rest}.cp"));
    if std::fs::read_to_string(&file).ok().as_deref() != Some(text) {
        std::fs::create_dir_all(file.parent()?).ok()?;
        std::fs::write(&file, text).ok()?;
    }
    Some(file)
}

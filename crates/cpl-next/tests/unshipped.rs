//! Nothing shipped links this crate. While it grows it is run by its own tests; the landing
//! that makes `cpl` depend on it deletes the lowering, engines and emitter it replaces, so
//! there is never a build with both paths in it and never a fallback from one to the other.

#[test]
fn no_other_crate_depends_on_this_one() {
    let crates = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let mut users = Vec::new();
    for entry in std::fs::read_dir(crates).unwrap() {
        let dir = entry.unwrap().path();
        let Ok(manifest) = std::fs::read_to_string(dir.join("Cargo.toml")) else {
            continue;
        };
        if dir.file_name().unwrap() != "cpl-next"
            && manifest
                .lines()
                .any(|l| l.trim_start().starts_with("cpl-next"))
        {
            users.push(dir.file_name().unwrap().to_string_lossy().into_owned());
        }
    }
    assert!(
        users.is_empty(),
        "these crates depend on cpl-next: {users:?}. Switch and delete in one landing instead."
    );
}

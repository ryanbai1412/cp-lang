//! Nothing shipped links this crate. While it grows it is run by its own tests; the landing
//! that makes `cpl` depend on it deletes the lowering, engines and emitter it replaces, so
//! there is never a build with both paths in it and never a fallback from one to the other.

#[test]
fn no_other_crate_depends_on_this_one() {
    // Cargo's own account of the workspace: a dependency renamed in a manifest
    // (`other = { package = "cpl-next" }`) is still one on `cpl-next`.
    let out = std::process::Command::new(env!("CARGO"))
        .args([
            "metadata",
            "--format-version",
            "1",
            "--no-deps",
            "--offline",
        ])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("cargo metadata runs");
    assert!(out.status.success(), "cargo metadata failed");
    let meta = cpl_json::parse(std::str::from_utf8(&out.stdout).unwrap()).unwrap();
    let name = |v: &cpl_json::Value| v.get("name").and_then(|n| n.as_str()).unwrap().to_string();
    let users: Vec<String> = meta
        .get("packages")
        .and_then(|p| p.as_array())
        .unwrap()
        .iter()
        .filter(|p| name(p) != "cpl-next")
        .filter(|p| {
            let deps = p.get("dependencies").and_then(|d| d.as_array()).unwrap();
            deps.iter().any(|d| name(d) == "cpl-next")
        })
        .map(name)
        .collect();
    assert!(
        users.is_empty(),
        "these crates depend on cpl-next: {users:?}. Switch and delete in one landing instead."
    );
}

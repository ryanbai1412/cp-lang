//! A `CPL_STD_DIR` without `prelude.cp` checks programs with no prelude
//! names instead of crashing (compiler-changes/R16).

use cpl_check::Session;

#[test]
fn a_std_dir_without_a_prelude_leaves_prelude_names_unresolved() {
    let dir = std::env::temp_dir().join(format!("cpl-std-no-prelude-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("geo")).unwrap();
    std::fs::write(dir.join("mod.cp"), "fn one() -> Int = 1\n").unwrap();
    std::fs::write(dir.join("geo/hull.cp"), "fn two() -> Int = 2\n").unwrap();
    std::env::set_var(cpl_env::STD_DIR.name(), &dir);
    let names: Vec<&str> = cpl_check::loaded_std().iter().map(|(n, _)| *n).collect();
    assert_eq!(names, ["geo/hull", "mod"]);
    let mut db = Session::new();
    db.set_file("main.cp", "let m: Monoid<Int> = todo\n".to_string());
    let diags = db.diagnostics("main.cp");
    std::fs::remove_dir_all(&dir).unwrap();
    assert!(
        diags.iter().any(|d| d.message.contains("Monoid")),
        "{diags:?}"
    );
}

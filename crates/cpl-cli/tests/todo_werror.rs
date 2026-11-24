//! `todo` builds clean under g++ `-Werror` wherever it stands.

use std::path::Path;
use std::process::Command;

/// `todo` in a fn body, an `if` or match arm, a `??` fallback and a let
/// builds as a noreturn call that fits its place, clean under `-Werror`.
#[test]
fn todo_in_every_place_builds_clean_under_werror() {
    if Command::new("g++").arg("--version").output().is_err() {
        return;
    }
    let spec = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../spec/conformance/core-01");
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("todo-werror");
    std::fs::create_dir_all(&dir).unwrap();
    for case in ["todo-unreached", "todo-fits-any-place-a-value-goes"] {
        let cpp = dir.join(format!("{case}.cpp"));
        let built = Command::new(env!("CARGO_BIN_EXE_cpl"))
            .arg("build")
            .arg(spec.join(format!("{case}.cp")))
            .arg("-o")
            .arg(&cpp)
            .output()
            .expect("cpl runs");
        assert!(built.status.success(), "{case}: {built:?}");
        let compiled = Command::new("g++")
            .args([
                "-std=c++17",
                "-O2",
                "-Wall",
                "-Wextra",
                "-Werror",
                "-c",
                "-o",
            ])
            .arg(dir.join(format!("{case}.o")))
            .arg(&cpp)
            .output()
            .expect("g++ starts");
        let err = String::from_utf8_lossy(&compiled.stderr);
        assert!(compiled.status.success(), "{case}: {err}");
    }
}

//! The C++ `cpl build` writes compiles with every compiler a contestant's
//! judge may run: g++ and clang, whichever of them is installed here.

use std::process::Command;

/// A false `assert` builds its message in a named cold function: both
/// g++ and clang take the attributes there, where clang refused them
/// after a lambda's `mutable`.
#[test]
fn a_checked_assert_compiles_with_gcc_and_clang() {
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("assert-cold");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("cell.cp");
    std::fs::write(
        &path,
        b"fn cell(w: Int, c: Int) -> Int:\n    assert 0 <= c < w, \"column {c} is outside 0..{w}\"\n    c\nprint(cell(read(), read()))\n",
    )
    .unwrap();
    let cpp = path.with_extension("cpp");
    let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .args(["build", "--checks", "traps"])
        .arg(&path)
        .arg("-o")
        .arg(&cpp)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    for cxx in ["g++", "clang++"] {
        if Command::new(cxx).arg("--version").output().is_err() {
            continue;
        }
        let exe = path.with_extension(cxx);
        let built = Command::new(cxx)
            .args(["-std=c++17", "-O2", "-o"])
            .arg(&exe)
            .arg(&cpp)
            .output()
            .unwrap();
        assert!(
            built.status.success(),
            "{cxx}: {}",
            String::from_utf8_lossy(&built.stderr)
        );
        let run = |input: &[u8]| {
            let mut child = Command::new(&exe)
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .unwrap();
            std::io::Write::write_all(&mut child.stdin.take().unwrap(), input).unwrap();
            child.wait_with_output().unwrap()
        };
        let fine = run(b"9 5\n");
        assert_eq!(
            (fine.status.code(), fine.stdout),
            (Some(0), b"5\n".to_vec()),
            "{cxx}"
        );
        let off = run(b"5 9\n");
        assert_eq!(off.status.code(), Some(3), "{cxx}");
        let why = String::from_utf8_lossy(&off.stderr);
        assert!(why.contains("column 9 is outside 0..5"), "{cxx}: {why}");
    }
}

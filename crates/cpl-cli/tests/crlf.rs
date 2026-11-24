//! Files saved on Windows end lines with CRLF: a source, its input and its
//! expected output all read as if the lines ended with LF.

use std::path::Path;
use std::process::{Command, Output};

fn cpl(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cpl"))
        .args(args)
        .current_dir(dir)
        .output()
        .expect("cpl runs")
}

#[test]
fn crlf_sources_inputs_and_answers_read_as_lf() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("crlf");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("a.tests")).unwrap();
    let source = "let n: Int = read()\r\nlet s: Str = read()\r\nlet xs: [n]Int = read()\r\nprint(s, xs.sum())\r\n";
    std::fs::write(dir.join("a.cp"), source).unwrap();
    std::fs::write(dir.join("a.tests/1.in"), "3\r\nab\r\n1 2 3\r\n").unwrap();
    std::fs::write(dir.join("a.tests/1.out"), "ab 6\r\n").unwrap();
    let ok = |args: &[&str]| {
        let out = cpl(&dir, args);
        assert_eq!(
            out.status.code(),
            Some(0),
            "cpl {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).replace("\r\n", "\n")
    };
    ok(&["check", "a.cp"]);
    assert_eq!(ok(&["run", "a.cp", "a.tests/1.in"]), "ab 6\n");
    ok(&["test", "a.cp"]);
    ok(&["fmt", "--check", "a.cp"]);
    if Command::new("g++").arg("--version").output().is_ok() {
        assert_eq!(ok(&["run", "--release", "a.cp", "a.tests/1.in"]), "ab 6\n");
        ok(&["submit", "a.cp"]);
    }
}

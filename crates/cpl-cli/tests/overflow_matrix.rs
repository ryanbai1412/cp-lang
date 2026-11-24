//! Every `Int` arithmetic op stored through every kind of place overflows
//! and traps the same way in the interpreter and in the native `-DLOCAL`
//! build (R09.4), so no place form can lose its check unnoticed.

use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

/// An op, a start value and a step that takes it past `Int`.
const OPS: [(&str, &str, &str); 4] = [
    ("+", "9000000000000000000", "1000000000000000000"),
    ("-", "-9000000000000000000", "1000000000000000000"),
    ("*", "4000000000", "4000000000"),
    ("+", "9223372036854775807", "1"),
];

/// A place: its global declaration, its setup to `S`, and how it is named.
const PLACES: [(&str, &str, &str, &str); 10] = [
    ("local", "", "var x{id} = {s}", "x{id}"),
    ("global", "var g{id} = 0", "g{id} = {s}", "g{id}"),
    ("field", "", "var p{id} = P(x: {s})", "p{id}.x"),
    ("tuple", "", "var t{id} = ({s}, 0)", "t{id}.0"),
    ("array", "", "var a{id}: [3]Int = {s}", "a{id}[1]"),
    ("list", "", "var l{id} = [0, {s}]", "l{id}[1]"),
    ("grid", "", "var r{id}: [2, 2]Int = {s}", "r{id}[1, 1]"),
    (
        "map",
        "",
        "var m{id}: Map<Int, Int> = Map()\n    m{id}[0] = {s}",
        "m{id}[0]",
    ),
    (
        "hash",
        "",
        "var h{id}: HashMap<Int, Int> = HashMap()\n    h{id}[0] = {s}",
        "h{id}[0]",
    ),
    ("optional", "", "var o{id}: ?Int = Some({s})", "o{id}!"),
];

/// A store of `{v}` into `{l}` with the op `{o}`.
const SHAPES: [(&str, &str); 5] = [
    ("compound", "{l} {o}= {v}"),
    ("plain", "{l} = {l} {o} {v}"),
    ("guarded", "if {v} > 0: {l} {o}= {v}"),
    ("guarded_plain", "if {v} > 0: {l} = {l} {o} {v}"),
    ("conditional", "{l} {o}= (if {v} > 0: {v} else: 0)"),
];

struct Case {
    name: String,
    step: &'static str,
}

/// One program per op, so each stays under the judge's size limit.
fn matrix(op: &str, start: &str, step: &'static str) -> (String, Vec<Case>) {
    let mut decls =
        String::from("let case: Int = read()\nlet v: Int = read()\nstruct P:\n    var x: Int\n");
    let (mut fns, mut calls, mut cases) = (String::new(), String::new(), Vec::new());
    {
        for (place, global, setup, name) in PLACES {
            for (shape, store) in SHAPES {
                for literal in [true, false] {
                    let i = cases.len();
                    let id = i.to_string();
                    let lvalue = name.replace("{id}", &id);
                    let value = if literal { step } else { "v" };
                    if !global.is_empty() {
                        decls += &format!("{}\n", global.replace("{id}", &id));
                    }
                    let stmt = store
                        .replace("{l}", &lvalue)
                        .replace("{o}", op)
                        .replace("{v}", value);
                    let setup = setup.replace("{id}", &id).replace("{s}", start);
                    fns += &format!("fn c{id}():\n    {setup}\n    {stmt}\n    print({lvalue})\n");
                    calls += &format!("if case == {id}: c{id}()\n");
                    let how = if literal { "literal" } else { "read" };
                    cases.push(Case {
                        name: format!("{start} {op} {step}: {place} {shape} {how}"),
                        step,
                    });
                }
            }
        }
    }
    (decls + &fns + &calls, cases)
}

fn native(cpp: &[u8], dir: &Path, k: usize) -> std::path::PathBuf {
    let (source, binary) = (
        dir.join(format!("matrix{k}.cpp")),
        dir.join(format!("matrix{k}")),
    );
    std::fs::write(&source, cpp).unwrap();
    let built = Command::new("g++")
        .args(["-std=c++17", "-O1", "-DLOCAL", "-fsanitize=undefined"])
        .args(["-fno-sanitize-recover=all", "-o"])
        .arg(&binary)
        .arg(&source)
        .output()
        .expect("g++ did not start; this test needs it");
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    binary
}

#[test]
fn every_int_op_through_every_place_traps_in_interp_and_native() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("overflow-matrix");
    std::fs::create_dir_all(&dir).unwrap();
    let (mut wrong, mut total) = (Vec::new(), 0);
    for (k, (op, start, step)) in OPS.into_iter().enumerate() {
        let (program, cases) = matrix(op, start, step);
        total += cases.len();
        let path = dir.join(format!("matrix{k}.cp"));
        std::fs::write(&path, &program).unwrap();
        let path = path.to_string_lossy().into_owned();
        let built = cpl_cli::run(&["build", &path, "-o", "-"]);
        assert_eq!(built.code, 0, "{}", built.stderr);
        let binary = native(&built.stdout, &dir, k);
        for (i, case) in cases.iter().enumerate() {
            let input = format!("{i} {}\n", case.step);
            let interp = cpl_cli::run_with_input(&["run", &path], input.as_bytes());
            if interp.code != 3 || !interp.stderr.contains("overflow") || !interp.stdout.is_empty()
            {
                wrong.push(format!(
                    "{}: interp exits {} with {}",
                    case.name, interp.code, interp.stderr
                ));
            }
            let mut child = Command::new(&binary)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap();
            child
                .stdin
                .take()
                .unwrap()
                .write_all(input.as_bytes())
                .unwrap();
            let ran = child.wait_with_output().unwrap();
            let stderr = String::from_utf8_lossy(&ran.stderr);
            if ran.status.code() != Some(3)
                || !stderr.contains("integer overflow")
                || !ran.stdout.is_empty()
            {
                wrong.push(format!(
                    "{}: native exits {:?} printing {:?} with {stderr}",
                    case.name,
                    ran.status.code(),
                    String::from_utf8_lossy(&ran.stdout)
                ));
            }
        }
    }
    assert!(
        wrong.is_empty(),
        "{} of {total} cases:\n{}",
        wrong.len(),
        wrong.join("\n")
    );
}

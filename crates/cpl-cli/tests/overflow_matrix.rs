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

/// An op on a map value of the range `R` (100..=1000), a start inside it
/// and a step that takes it out: the narrow cell must trap on the range,
/// not wrap back into it (1000 * 65537 is 1000 in 16 bits).
const RANGED_OPS: [(&str, &str, &str); 8] = [
    ("+", "900", "200"),
    ("-", "200", "150"),
    ("*", "900", "2"),
    ("*", "1000", "65537"),
    ("|", "512", "1024"),
    ("^", "512", "1024"),
    ("max", "900", "2000"),
    ("min", "200", "50"),
];

/// A map place whose values are `R`, local and as a (narrow) field.
const RANGED_PLACES: [(&str, &str, &str, &str); 4] = [
    (
        "ranged map",
        "",
        "var m{id}: Map<Int, R> = Map()\n    m{id}[0] = {s}",
        "m{id}[0]",
    ),
    (
        "ranged hash",
        "",
        "var h{id}: HashMap<Int, R> = HashMap()\n    h{id}[0] = {s}",
        "h{id}[0]",
    ),
    (
        "narrow map field",
        "",
        "var q{id} = Q([], [])\n    q{id}.m[0] = {s}",
        "q{id}.m[0]",
    ),
    (
        "narrow hash field",
        "",
        "var q{id} = Q([], [])\n    q{id}.h[0] = {s}",
        "q{id}.h[0]",
    ),
];

const PRELUDE: &str = "let case: Int = read()\nlet v: Int = read()\nstruct P:\n    var x: Int\n";

const RANGED_PRELUDE: &str = "let case: Int = read()\nlet v: Int = read()\ntype R = 100..=1000\nstruct Q:\n    var m: Map<Int, R>\n    var h: HashMap<Int, R>\n";

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

/// One program per op group, so each stays under the judge's size limit.
/// `min=` and `max=` have no binary form, so they skip the `plain` shapes.
fn matrix(
    prelude: &str,
    places: &[(&str, &str, &str, &str)],
    ops: &[(&str, &str, &'static str)],
) -> (String, Vec<Case>) {
    let mut decls = String::from(prelude);
    let (mut fns, mut calls, mut cases) = (String::new(), String::new(), Vec::new());
    for &(op, start, step) in ops {
        for &(place, global, setup, name) in places {
            for (shape, store) in SHAPES {
                if op.starts_with('m') && store.contains(" {o} ") {
                    continue;
                }
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
    // A ranged place traps on its range (`... is outside ...`) instead.
    let groups = OPS
        .iter()
        .map(|op| (PRELUDE, &PLACES[..], std::slice::from_ref(op), "overflow"))
        .chain(
            RANGED_OPS
                .chunks(3)
                .map(|ops| (RANGED_PRELUDE, &RANGED_PLACES[..], ops, "outside")),
        );
    for (k, (prelude, places, ops, trap)) in groups.enumerate() {
        let (program, cases) = matrix(prelude, places, ops);
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
            if interp.code != 3 || !interp.stderr.contains(trap) || !interp.stdout.is_empty() {
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
                || !stderr.contains(if trap == "overflow" {
                    "integer overflow"
                } else {
                    trap
                })
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

/// `min=`, `max=`, `+=` and `*=` on an absent key start it the same way
/// whatever the value's range, and a start outside the range traps, in the
/// interpreter and in the native `-DLOCAL` build alike.
#[test]
fn an_absent_key_folds_the_same_whatever_the_value_range() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("overflow-absent");
    std::fs::create_dir_all(&dir).unwrap();
    let mut wrong = Vec::new();
    for (k, value) in ["Int", "R"].into_iter().enumerate() {
        let program = format!(
            "let case: Int = read()\nlet v: Int = read()\ntype R = 100..=1000\n\
             struct Q:\n    var h: HashMap<Int, {value}>\n\
             var m: Map<Int, {value}> = Map()\nvar h: HashMap<Int, {value}> = HashMap()\nvar q = Q([])\n\
             if case == 0:\n    m[1] min= v\n    h[1] max= v\n    q.h[1] min= v\n    m[2] += v\n    h[2] *= v\n    q.h[2] max= v\n\
             \x20   print(m[1], h[1], q.h[1], m[2], h[2], q.h[2])\n\
             if case == 1: m[1] min= v * 2000\n\
             if case == 2: q.h[1] max= v * 2000\n"
        );
        let path = dir.join(format!("absent{k}.cp"));
        std::fs::write(&path, &program).unwrap();
        let path = path.to_string_lossy().into_owned();
        let built = cpl_cli::run(&["build", &path, "-o", "-"]);
        assert_eq!(built.code, 0, "{}", built.stderr);
        let binary = native(&built.stdout, &dir, k);
        for (case, ranged_traps) in [(0, false), (1, true), (2, true)] {
            let input = format!("{case} 500\n");
            let traps = ranged_traps && value == "R";
            let interp = cpl_cli::run_with_input(&["run", &path], input.as_bytes());
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
            let want = if case == 0 {
                "500 500 500 500 500 500\n"
            } else {
                ""
            };
            let stderr = String::from_utf8_lossy(&ran.stderr);
            let got = [
                (
                    i32::from(interp.code),
                    String::from_utf8_lossy(&interp.stdout).into_owned(),
                    interp.stderr.clone(),
                ),
                (
                    ran.status.code().unwrap_or(-1),
                    String::from_utf8_lossy(&ran.stdout).into_owned(),
                    stderr.into_owned(),
                ),
            ];
            for (engine, (code, out, err)) in ["interp", "native"].into_iter().zip(got) {
                let ok = if traps {
                    code == 3 && err.contains("outside") && out.is_empty()
                } else {
                    code == 0 && out == want
                };
                if !ok {
                    wrong.push(format!(
                        "{value} case {case}: {engine} exits {code} printing {out:?} with {err}"
                    ));
                }
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

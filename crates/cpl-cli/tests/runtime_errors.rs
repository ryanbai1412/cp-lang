//! Golden tests for runtime error messages: every runtime error class carries
//! concrete offending values, source location, call stack, and multitest case
//! number when applicable.

use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

fn run_stderr(source: &str, input: &str) -> (i32, String) {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("runtime_errors_{}_{id}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let file = dir.join("a.cp");
    fs::write(&file, source).unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .arg("run")
        .arg(&file)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("cpl starts");
    child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
    let out = child.wait_with_output().expect("cpl exits");
    let code = out.status.code().unwrap_or(-1);
    let err = String::from_utf8_lossy(&out.stderr).into_owned();
    let prefix = format!("{}/", dir.display());
    let _ = fs::remove_dir_all(&dir);
    (code, err.replace(&prefix, ""))
}

#[test]
fn golden_division_and_remainder_by_zero_include_values() {
    let (code, err) = run_stderr("let a: Int = read()\nprint(42 / a)\n", "0\n");
    assert_eq!(code, 3);
    assert!(
        err.contains("error[R0001]: division by zero: `42 / 0`"),
        "got:\n{err}"
    );

    let (code, err) = run_stderr("let a: Int = read()\nprint(42 % a)\n", "0\n");
    assert_eq!(code, 3);
    assert!(
        err.contains("error[R0001]: division by zero: `42 % 0`"),
        "got:\n{err}"
    );
}

#[test]
fn golden_integer_overflow_classes_include_values() {
    let cases = [
        (
            "let a: Int = read()\nprint(a + a)\n",
            "9000000000000000000\n",
            "integer overflow: `9000000000000000000 + 9000000000000000000` does not fit in an `Int` (64 bits)",
        ),
        (
            "let a: Int = read()\nprint(a * a)\n",
            "4000000000\n",
            "integer overflow: `4000000000 * 4000000000` does not fit in an `Int` (64 bits)",
        ),
        (
            "let a: Int = read()\nprint(a << 60)\n",
            "16\n",
            "integer overflow: `16 << 60` does not fit in an `Int` (64 bits)",
        ),
        (
            "let a: Int = read()\nprint(a ** 10)\n",
            "1000\n",
            "integer overflow: `1000 ** 10` does not fit in an `Int` (64 bits)",
        ),
    ];
    for (src, inp, expected) in cases {
        let (code, err) = run_stderr(src, inp);
        assert_eq!(code, 3, "for {expected}");
        assert!(err.contains(expected), "expected `{expected}`, got:\n{err}");
    }
}

#[test]
fn golden_bounds_domain_and_container_errors_include_values() {
    let cases = [
        (
            "let xs: List<Int> = [1, 2]\nprint(xs[5])\n",
            "",
            "index 5 is outside a list of length 2",
        ),
        (
            "let s: Str = read()\nprint(s[3])\n",
            "abc\n",
            "index 3 is outside a string of length 3",
        ),
        (
            "var a: [3]Int = 0\nlet i: Int = read()\na[i] = 1\n",
            "3\n",
            "index 3 is outside an array dimension of length 3",
        ),
        (
            "var m: Map<Int, Int> = []\nprint(m[7])\n",
            "",
            "the key `7` is not in the map",
        ),
        (
            "var xs: List<Int> = []\nprint(xs.pop()!)\n",
            "",
            "`!` on `none`: there is no value here",
        ),
        (
            "var o: ?Int = none\nprint(o!)\n",
            "",
            "`!` on `none`: there is no value here",
        ),
        (
            "let n: Int = read()\ndomain V = 1..=n\nlet v: V = read()\nprint(v)\n",
            "3 5\n",
            "5 is not in `V`, which is 1..=3 (input line 1, column 3)",
        ),
    ];
    for (src, inp, expected) in cases {
        let (code, err) = run_stderr(src, inp);
        assert_eq!(code, 3, "for {expected}");
        assert!(err.contains(expected), "expected `{expected}`, got:\n{err}");
    }
}

#[test]
fn golden_math_assert_and_input_errors() {
    let cases = [
        (
            "let b: Int = read()\nprint(2 ** b)\n",
            "-3\n",
            "`**` needs a power that is not negative, and this is -3",
        ),
        (
            "let b: Int = read()\nprint(1 << b)\n",
            "64\n",
            "a shift by 64 is outside 0..64",
        ),
        (
            "let a: Int = read()\nprint(a << 1)\n",
            "-5\n",
            "`<<` of the negative number -5",
        ),
        (
            "let a: Int = read()\nprint(isqrt(a))\n",
            "-4\n",
            "isqrt(-4) of a negative number",
        ),
        (
            "let a: F64 = read()\nprint(sqrt(a))\n",
            "-1.0\n",
            "sqrt(-1) of a negative number",
        ),
        (
            "assert 1 > 2\n",
            "",
            "assertion failed: 1 > 2",
        ),
        (
            "let a: Int = read()\nprint(a)\n",
            "hello\n",
            "expected a number, found `hello` (input line 1, column 1)",
        ),
    ];
    for (src, inp, expected) in cases {
        let (code, err) = run_stderr(src, inp);
        assert_eq!(code, 3, "for {expected}");
        assert!(err.contains(expected), "expected `{expected}`, got:\n{err}");
    }
}

#[test]
fn golden_call_stack_unwinds_nested_functions() {
    let source = "\
fn inner(x: Int) -> Int:
    return 100 / x

fn outer(x: Int) -> Int:
    return inner(x)

let a: Int = read()
print(outer(a))
";
    let (code, err) = run_stderr(source, "0\n");
    assert_eq!(code, 3);
    assert!(
        err.contains("error[R0001]: division by zero: `100 / 0`"),
        "got:\n{err}"
    );
    assert!(
        err.contains("note: call stack: inner (a.cp:2) <- outer (a.cp:5) <- top level (a.cp:8)"),
        "got:\n{err}"
    );
}

#[test]
fn golden_multitest_reports_case_number_and_call_stack() {
    let source = "\
@!multitest

fn divide(a: Int, b: Int) -> Int:
    return a / b

let a: Int = read()
let b: Int = read()
print(divide(a, b))
";
    let (code, err) = run_stderr(source, "3\n10 2\n7 0\n9 3\n");
    assert_eq!(code, 3);
    assert!(
        err.contains("error[R0001]: division by zero: `7 / 0`"),
        "got:\n{err}"
    );
    assert!(
        err.contains("note: in test case 2"),
        "got:\n{err}"
    );
    assert!(
        err.contains("note: call stack: divide (a.cp:4) <- top level (a.cp:8)"),
        "got:\n{err}"
    );
}

#[test]
fn golden_closure_appears_in_call_stack() {
    let source = "\
let f: fn(Int) -> Int = x => 10 / x
let a: Int = read()
print(f(a))
";
    let (code, err) = run_stderr(source, "0\n");
    assert_eq!(code, 3);
    assert!(
        err.contains("error[R0001]: division by zero: `10 / 0`"),
        "got:\n{err}"
    );
    assert!(
        err.contains("note: call stack: <closure> (a.cp:1) <- top level (a.cp:3)"),
        "got:\n{err}"
    );
}

#[test]
fn golden_runaway_recursion_summarizes_deep_call_stack() {
    let source = "\
fn spin(x: Int) -> Int:
    return spin(x + 1)

print(spin(0))
";
    let (code, err) = run_stderr(source, "");
    assert_eq!(code, 3);
    assert!(
        err.contains("never stops calling itself?"),
        "got:\n{err}"
    );
    assert!(
        err.contains("frames omitted"),
        "expected deep call stack to be summarized, got:\n{err}"
    );
    assert!(
        err.contains("<- top level (a.cp:4)"),
        "got:\n{err}"
    );
}

fn build_and_run_cpp(source: &str) -> (i32, String) {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("runtime_errors_cpp_{}_{id}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let file = dir.join("a.cp");
    let cpp = dir.join("a.cpp");
    let exe = dir.join("a.bin");
    fs::write(&file, source).unwrap();
    let build = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .args(["build", file.to_str().unwrap(), "-o", cpp.to_str().unwrap()])
        .output()
        .expect("cpl build");
    assert!(
        build.status.success(),
        "cpl build failed:\n{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let compile = Command::new("g++")
        .args(["-std=c++17", "-O2", cpp.to_str().unwrap(), "-o", exe.to_str().unwrap()])
        .output()
        .expect("g++");
    assert!(
        compile.status.success(),
        "g++ failed:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let out = Command::new(&exe).output().expect("run binary");
    let code = out.status.code().unwrap_or(-1);
    let err = String::from_utf8_lossy(&out.stderr).into_owned();
    let _ = fs::remove_dir_all(&dir);
    (code, err)
}

#[test]
fn wide_range_overflow_traps_in_interp_and_cpp() {
    let cases = [
        (
            "use std.seq.{bsearch_first}\nlet lo = -6_000_000_000_000_000_000\nlet hi = 6_000_000_000_000_000_000\nprint(bsearch_first(lo..=hi, x => x >= 7)!)\n",
            "integer overflow: range",
            "range span overflow",
        ),
        (
            "use std.seq.{bsearch_first}\nlet big = I128(1_000_000_000_000_000_000) * I128(1_000_000_000_000_000_000) * I128(100)\nprint(bsearch_first((-big)..=big, x => x >= I128(7))!)\n",
            "integer overflow: range",
            "range span overflow",
        ),
        (
            "let lo = I128(-4_000_000_000) * I128(10_000_000_000)\nlet r = lo..=I128(0)\nprint(r.len)\n",
            "integer overflow: range",
            "range length overflow",
        ),
    ];
    for (src, interp_msg, cpp_msg) in cases {
        let (code, err) = run_stderr(src, "");
        assert_eq!(code, 3, "interp should trap:\n{err}");
        assert!(err.contains(interp_msg), "interp expected `{interp_msg}`, got:\n{err}");
        assert!(!err.contains("the interpreter itself failed"), "got panic:\n{err}");

        let (cpp_code, cpp_err) = build_and_run_cpp(src);
        assert_eq!(cpp_code, 3, "cpp should trap:\n{cpp_err}");
        assert!(cpp_err.contains(cpp_msg), "cpp expected `{cpp_msg}`, got:\n{cpp_err}");
    }
}

#[test]
fn sort_cmp_strict_weak_order_traps_in_interp_and_checked_cpp() {
    let cases = [
        (
            "var xs = [1, 2, 3]\nxs.sort(cmp: (a, b) => a <= b)\nprint(xs)\n",
            "cmp(x, x) is true",
        ),
        (
            "var xs = [1, 2, 3]\nxs.sort(cmp: (a, b) => a != b)\nprint(xs)\n",
            "both cmp(a, b) and cmp(b, a) are true",
        ),
        (
            "var xs = [0, 1, 2]\nxs.sort(cmp: (a, b) => (a + 1) % 3 == b)\nprint(xs)\n",
            "result is not ordered",
        ),
    ];
    for (idx, (src, msg)) in cases.iter().enumerate() {
        let (code, err) = run_stderr(src, "");
        assert_eq!(code, 3, "interp case {idx} should trap:\n{err}");
        assert!(err.contains(msg), "interp case {idx} expected `{msg}`, got:\n{err}");
        assert!(!err.contains("the interpreter itself failed"), "got panic:\n{err}");

        let dir = std::env::temp_dir()
            .join(format!("runtime_errors_sort_cmp_{}_{idx}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let file = dir.join("a.cp");
        let cpp = dir.join("a.cpp");
        let exe = dir.join("a.bin");
        fs::write(&file, src).unwrap();
        let build = Command::new(env!("CARGO_BIN_EXE_cpl"))
            .args(["build", file.to_str().unwrap(), "-o", cpp.to_str().unwrap()])
            .output()
            .expect("cpl build");
        assert!(build.status.success(), "cpl build failed:\n{}", String::from_utf8_lossy(&build.stderr));
        let compile = Command::new("g++")
            .args(["-std=c++17", "-O1", "-DLOCAL", cpp.to_str().unwrap(), "-o", exe.to_str().unwrap()])
            .output()
            .expect("g++");
        assert!(compile.status.success(), "g++ failed:\n{}", String::from_utf8_lossy(&compile.stderr));
        let out = Command::new(&exe).output().expect("run binary");
        let cpp_err = String::from_utf8_lossy(&out.stderr).into_owned();
        let _ = fs::remove_dir_all(&dir);
        assert_eq!(out.status.code(), Some(3), "checked cpp case {idx} should trap:\n{cpp_err}");
        assert!(cpp_err.contains(msg), "checked cpp case {idx} expected `{msg}`, got:\n{cpp_err}");
    }
}

#[test]
fn fold_empty_traps_in_interp_and_checked_cpp() {
    let src = "let xs: List<Int> = []\nprint(xs.fold((a, b) => a * 10 + b))\n";
    let msg = "`fold` of an empty sequence";
    let (code, err) = run_stderr(src, "");
    assert_eq!(code, 3, "interp should trap:\n{err}");
    assert!(err.contains(msg), "interp expected `{msg}`, got:\n{err}");
    assert!(!err.contains("the interpreter itself failed"), "got panic:\n{err}");

    let (cpp_code, cpp_err) = build_and_run_cpp(src);
    assert_eq!(cpp_code, 3, "checked cpp should trap:\n{cpp_err}");
    assert!(cpp_err.contains(msg), "checked cpp expected `{msg}`, got:\n{cpp_err}");
}

#[test]
fn reach_non_monotone_traps_in_interp_and_checked_cpp() {
    let cases = [
        // Reach drops from 2 at i=0 to 0 at i=1 (`while(1, 2)` is false).
        "print([3, 1, 2].reach(while: (a, b) => a >= b))\n",
        // Cyclic non-monotone predicate.
        "print([3, 1, 2].reach(while: (a, b) => a >= b, cyclic: true))\n",
    ];
    let msg = "reach: `while` predicate is not monotone";
    for (idx, src) in cases.iter().enumerate() {
        let (code, err) = run_stderr(src, "");
        assert_eq!(code, 3, "interp case {idx} should trap:\n{err}");
        assert!(err.contains(msg), "interp case {idx} expected `{msg}`, got:\n{err}");
        assert!(!err.contains("the interpreter itself failed"), "got panic:\n{err}");

        let dir = std::env::temp_dir()
            .join(format!("runtime_errors_reach_{}_{idx}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let file = dir.join("a.cp");
        let cpp = dir.join("a.cpp");
        let exe = dir.join("a.bin");
        fs::write(&file, src).unwrap();
        let build = Command::new(env!("CARGO_BIN_EXE_cpl"))
            .args(["build", file.to_str().unwrap(), "-o", cpp.to_str().unwrap()])
            .output()
            .expect("cpl build");
        assert!(build.status.success(), "cpl build failed:\n{}", String::from_utf8_lossy(&build.stderr));
        let compile = Command::new("g++")
            .args(["-std=c++17", "-O1", "-DLOCAL", cpp.to_str().unwrap(), "-o", exe.to_str().unwrap()])
            .output()
            .expect("g++");
        assert!(compile.status.success(), "g++ failed:\n{}", String::from_utf8_lossy(&compile.stderr));
        let out = Command::new(&exe).output().expect("run binary");
        let cpp_err = String::from_utf8_lossy(&out.stderr).into_owned();
        let _ = fs::remove_dir_all(&dir);
        assert_eq!(out.status.code(), Some(3), "checked cpp case {idx} should trap:\n{cpp_err}");
        assert!(cpp_err.contains(msg), "checked cpp case {idx} expected `{msg}`, got:\n{cpp_err}");
    }
}

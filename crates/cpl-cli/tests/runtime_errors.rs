//! Golden tests for runtime error messages: every runtime error class carries
//! concrete offending values, source location, call stack, and multitest case
//! number when applicable.
//!
//! Rule: `errors/R11`, handbook/guidelines/errors-checklist.md#r11-a-trap-names-the-values-and-the-way-out.

use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

fn run_stderr(source: &str, input: &str) -> (i32, String) {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("runtime_errors_{}_{id}", std::process::id()));
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
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let out = child.wait_with_output().expect("cpl exits");
    let code = out.status.code().unwrap_or(-1);
    let err = String::from_utf8_lossy(&out.stderr).into_owned();
    let dir_name = dir.display().to_string();
    let err = err
        .replace(&format!("{dir_name}/"), "")
        .replace(&format!("{dir_name}\\"), "")
        .replace(&format!("{}/", cpl_diag::shown_path(&dir)), "");
    let _ = fs::remove_dir_all(&dir);
    (code, err)
}

#[test]
fn golden_division_and_remainder_by_zero_include_values() {
    let (code, err) = run_stderr("let a: Int = read()\nprint(42 / a)\n", "0\n");
    assert_eq!(code, 3);
    assert!(
        err.contains("error[R0203]: division by zero: `42 / 0`"),
        "got:\n{err}"
    );

    let (code, err) = run_stderr("let a: Int = read()\nprint(42 % a)\n", "0\n");
    assert_eq!(code, 3);
    assert!(
        err.contains("error[R0203]: division by zero: `42 % 0`"),
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
            "`!` on `None`: there is no value here",
        ),
        (
            "var o: ?Int = None\nprint(o!)\n",
            "",
            "`!` on `None`: there is no value here",
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
        ("assert 1 > 2\n", "", "assertion failed: 1 > 2"),
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
        err.contains("error[R0203]: division by zero: `100 / 0`"),
        "got:\n{err}"
    );
    assert!(
        err.contains("note: call stack: inner (a.cp:2) <- outer (a.cp:5) <- top level (a.cp:8)"),
        "got:\n{err}"
    );
}

#[test]
fn a_trap_names_the_locals_where_it_stopped() {
    // A loop one step too far, at the top level.
    let source = "\
let n: 1..=5 = read()
let a: [0..n]Int = read()
var pre: [0..=n]Int = 0
for i in 0..=n:
    pre[i + 1] = pre[i] + a[i]
print(pre[n])
";
    let (code, err) = run_stderr(source, "3\n1 2 3\n");
    assert_eq!(code, 3);
    assert!(err.contains("note: locals: i = 3"), "got:\n{err}");
    // Inside a call: the locals of the innermost one.
    let source = "\
fn inner(x: Int) -> Int:
    let y = x * 2
    return 100 / (x + y)

let a: Int = read()
print(inner(a))
";
    let (code, err) = run_stderr(source, "0\n");
    assert_eq!(code, 3);
    assert!(err.contains("note: locals: x = 0, y = 0"), "got:\n{err}");
    assert!(
        err.contains("note: call stack: inner (a.cp:3)"),
        "got:\n{err}"
    );
}

#[test]
fn a_long_run_that_traps_names_its_locals() {
    // 1.5*10^7 steps, past the 10^7 a trace keeps: tracing the whole run
    // again took longer than its 10 s cap.
    let source = "\
let n: Int = read()
var s = 0
var i = 0
while i < n:
    s = (s * 31 + i) % 1000003
    i += 1
let d = s - s
print(i / d)
";
    let (code, err) = run_stderr(source, "5000000\n");
    assert_eq!(code, 3);
    assert!(err.contains("note: locals: i = 5000000"), "got:\n{err}");
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
        err.contains("error[R0203]: division by zero: `7 / 0`"),
        "got:\n{err}"
    );
    assert!(err.contains("note: in test case 2"), "got:\n{err}");
    assert!(
        err.contains("note: call stack: divide (a.cp:4) <- top level (a.cp:8)"),
        "got:\n{err}"
    );
}

#[test]
fn a_trap_in_a_library_fn_points_at_the_call_that_reached_it() {
    let source = "\
domain V = 0..4
let a = Pair<V>.new(V(0), V(1))
let b = Pair<V>.new(V(1), V(2))
let c = Pair<V>.new(V(1), V(1))
";
    let (code, err) = run_stderr(source, "");
    assert_eq!(code, 3);
    assert!(err.contains("--> a.cp:4:9"), "got:\n{err}");
    assert!(
        err.contains("note: call stack: Pair.new (a.cp:4) <- top level (a.cp:4)"),
        "got:\n{err}"
    );
    let (code, err) = build_and_run_cpp(source);
    assert_eq!(code, 3);
    assert!(
        err.contains("assertion failed: u != v (line 4): `Pair` needs two distinct positions")
            && err.contains("note: std/pair.cp:"),
        "the C++ names the call that reached it, not the first one; got:\n{err}"
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
        err.contains("error[R0203]: division by zero: `10 / 0`"),
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
    assert!(err.contains("never stops calling itself?"), "got:\n{err}");
    assert!(err.contains("check the base case"), "got:\n{err}");
    let stack = err
        .lines()
        .find(|l| l.contains("call stack:"))
        .unwrap_or("");
    assert!(
        stack.starts_with("  = note: call stack: spin (a.cp:2) ")
            && stack.ends_with(" times <- top level (a.cp:4)"),
        "expected one counted run of `spin`, got:\n{err}"
    );
}

#[test]
fn golden_deep_mutual_recursion_keeps_the_omitted_middle() {
    let source = "\
fn ping(x: Int) -> Int:
    return pong(x + 1)
fn pong(x: Int) -> Int:
    return ping(x + 1)

print(ping(0))
";
    let (code, err) = run_stderr(source, "");
    assert_eq!(code, 3);
    assert!(err.contains("frames omitted"), "got:\n{err}");
    assert!(
        err.contains("ping (a.cp:2) <- pong (a.cp:4)"),
        "got:\n{err}"
    );
    assert!(err.contains("<- top level (a.cp:6)"), "got:\n{err}");
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
        .args([
            "build",
            file.to_str().unwrap(),
            "--checks",
            "traps",
            "-o",
            cpp.to_str().unwrap(),
        ])
        .output()
        .expect("cpl build");
    assert!(
        build.status.success(),
        "cpl build failed:\n{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let compile = Command::new("g++")
        .args([
            "-std=c++17",
            "-O2",
            cpp.to_str().unwrap(),
            "-o",
            exe.to_str().unwrap(),
        ])
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
            "let lo = -6_000_000_000_000_000_000\nlet hi = 6_000_000_000_000_000_000\nprint((lo..=hi).first_true(x => x >= 7)!)\n",
            "integer overflow: range",
            "range length overflow",
        ),
        (
            "let big = I128(1_000_000_000_000_000_000) * I128(1_000_000_000_000_000_000) * I128(100)\nprint(((-big)..=big).first_true(x => x >= I128(7))!)\n",
            "integer overflow: range",
            "range length overflow",
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
        assert!(
            err.contains(interp_msg),
            "interp expected `{interp_msg}`, got:\n{err}"
        );
        assert!(
            !err.contains("the interpreter itself failed"),
            "got panic:\n{err}"
        );

        let (cpp_code, cpp_err) = build_and_run_cpp(src);
        assert_eq!(cpp_code, 3, "cpp should trap:\n{cpp_err}");
        assert!(
            cpp_err.contains(cpp_msg),
            "cpp expected `{cpp_msg}`, got:\n{cpp_err}"
        );
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
        assert!(
            err.contains(msg),
            "interp case {idx} expected `{msg}`, got:\n{err}"
        );
        assert!(
            !err.contains("the interpreter itself failed"),
            "got panic:\n{err}"
        );

        let dir = std::env::temp_dir().join(format!(
            "runtime_errors_sort_cmp_{}_{idx}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let file = dir.join("a.cp");
        let cpp = dir.join("a.cpp");
        let exe = dir.join("a.bin");
        fs::write(&file, src).unwrap();
        let build = Command::new(env!("CARGO_BIN_EXE_cpl"))
            .args([
                "build",
                file.to_str().unwrap(),
                "--checks",
                "traps",
                "-o",
                cpp.to_str().unwrap(),
            ])
            .output()
            .expect("cpl build");
        assert!(
            build.status.success(),
            "cpl build failed:\n{}",
            String::from_utf8_lossy(&build.stderr)
        );
        let compile = Command::new("g++")
            .args([
                "-std=c++17",
                "-O1",
                "-DLOCAL",
                cpp.to_str().unwrap(),
                "-o",
                exe.to_str().unwrap(),
            ])
            .output()
            .expect("g++");
        assert!(
            compile.status.success(),
            "g++ failed:\n{}",
            String::from_utf8_lossy(&compile.stderr)
        );
        let out = Command::new(&exe).output().expect("run binary");
        let cpp_err = String::from_utf8_lossy(&out.stderr).into_owned();
        let _ = fs::remove_dir_all(&dir);
        assert_eq!(
            out.status.code(),
            Some(3),
            "checked cpp case {idx} should trap:\n{cpp_err}"
        );
        assert!(
            cpp_err.contains(msg),
            "checked cpp case {idx} expected `{msg}`, got:\n{cpp_err}"
        );
    }
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
        assert!(
            err.contains(msg),
            "interp case {idx} expected `{msg}`, got:\n{err}"
        );
        assert!(
            !err.contains("the interpreter itself failed"),
            "got panic:\n{err}"
        );

        let dir =
            std::env::temp_dir().join(format!("runtime_errors_reach_{}_{idx}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let file = dir.join("a.cp");
        let cpp = dir.join("a.cpp");
        let exe = dir.join("a.bin");
        fs::write(&file, src).unwrap();
        let build = Command::new(env!("CARGO_BIN_EXE_cpl"))
            .args([
                "build",
                file.to_str().unwrap(),
                "--checks",
                "traps",
                "-o",
                cpp.to_str().unwrap(),
            ])
            .output()
            .expect("cpl build");
        assert!(
            build.status.success(),
            "cpl build failed:\n{}",
            String::from_utf8_lossy(&build.stderr)
        );
        let compile = Command::new("g++")
            .args([
                "-std=c++17",
                "-O1",
                "-DLOCAL",
                cpp.to_str().unwrap(),
                "-o",
                exe.to_str().unwrap(),
            ])
            .output()
            .expect("g++");
        assert!(
            compile.status.success(),
            "g++ failed:\n{}",
            String::from_utf8_lossy(&compile.stderr)
        );
        let out = Command::new(&exe).output().expect("run binary");
        let cpp_err = String::from_utf8_lossy(&out.stderr).into_owned();
        let _ = fs::remove_dir_all(&dir);
        assert_eq!(
            out.status.code(),
            Some(3),
            "checked cpp case {idx} should trap:\n{cpp_err}"
        );
        assert!(
            cpp_err.contains(msg),
            "checked cpp case {idx} expected `{msg}`, got:\n{cpp_err}"
        );
    }
}

#[test]
fn range_typed_stores_trap_in_interp_and_checked_cpp() {
    let cases = [
        (
            "struct S:\n    var a: 0..=1_000_000\n    b: Int\nlet k: Int = read()\nvar s = S(0, 1)\ns.a = k\nprint(s.a)\n",
            "5000000000\n",
            "5000000000 is outside `0..=1_000_000`",
        ),
        (
            "struct S(a: 0..=1_000_000, b: Int)\nlet k: Int = read()\nlet s = S(k, 1)\nprint(s.a)\n",
            "5000000000\n",
            "5000000000 is outside `0..=1_000_000`",
        ),
        (
            "let k: Int = read()\nvar x: 0..=5 = 0\nx = k\nprint(x)\n",
            "10\n",
            "10 is outside `0..=5`",
        ),
        (
            "let k: Int = read()\nvar xs: List<?(0..100)> = [None, None]\nxs[1] = k\nprint(xs[1] ?? -1)\n",
            "500\n",
            "500 is outside `0..100`",
        ),
        (
            "let k: Int = read()\nvar xs: List<?(0..2_147_483_648)> = [None]\nxs[0] = k\nprint(xs[0] ?? -1)\n",
            "4294967296\n",
            "4294967296 is outside `0..2_147_483_648`",
        ),
        (
            "let n: 1..=10 = read()\ndomain V = 0..n\nlet k: Int = read()\nvar d: [V]0..=1_000_000 = 0\nd[V(0)] = k\nprint(d[V(0)])\n",
            "3\n5000000000\n",
            "5000000000 is outside `0..=1_000_000`",
        ),
        (
            "let n: 1..=10 = read()\ndomain V = 0..n\nlet k: Int = read()\nvar d: [V]0..=10 = 5\nd[V(0)] += k\nprint(d[V(0)])\n",
            "3\n8\n",
            "13 is outside `0..=10`",
        ),
        (
            "let n: 1..=10 = read()\ndomain V = 0..n\nlet k: Int = read()\nlet d: [V]0..=10 = [k for _ in V]\nprint(d[V(0)])\n",
            "3\n99\n",
            "99 is outside `0..=10`",
        ),
        (
            "let n: 1..=10 = read()\ndomain V = 0..n\nlet k: Int = read()\nvar d: [V]0..=10 = k\nprint(d[V(0)])\n",
            "3\n99\n",
            "99 is outside `0..=10`",
        ),
        (
            "let n: 1..=10 = read()\ndomain V = 0..n\nlet k: Int = read()\nvar d: [V]0..=10 = 0\nd.fill(k)\nprint(d[V(0)])\n",
            "3\n99\n",
            "99 is outside `0..=10`",
        ),
        (
            "let k: Int = read()\nvar x: 0..=10 = 5\nx max= k\nprint(x)\n",
            "11\n",
            "11 is outside `0..=10`",
        ),
        (
            "let k: Int = read()\nvar xs: List<0..=10> = []\nxs.push(k)\nprint(xs[0])\n",
            "99\n",
            "99 is outside `0..=10`",
        ),
        (
            "use std.flow.{Arc}\nlet k: Int = read()\nprint(Arc(k).rev.id)\n",
            "2147483648\n",
            "2147483648 is outside `0..2_147_483_648`",
        ),
        (
            "struct Node:\n    l: ?Int\n    pri: 0..2_147_483_648\n    var sz: 0..2_147_483_648\nlet k: Int = read()\nvar n = Node(None, 0, 1)\nn.sz += k\nprint(n.sz)\n",
            "2147483647\n",
            "2147483648 is outside `0..2_147_483_648`",
        ),
    ];
    for (idx, (src, input, msg)) in cases.iter().enumerate() {
        let (code, err) = run_stderr(src, input);
        assert_eq!(code, 3, "interp case {idx} should trap:\n{err}");
        assert!(
            err.contains(msg),
            "interp case {idx} expected `{msg}`, got:\n{err}"
        );
        assert!(
            !err.contains("the interpreter itself failed"),
            "got panic:\n{err}"
        );

        let dir = std::env::temp_dir().join(format!(
            "runtime_errors_range_store_{}_{idx}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let file = dir.join("a.cp");
        let cpp = dir.join("a.cpp");
        let exe = dir.join("a.bin");
        fs::write(&file, src).unwrap();
        let build = Command::new(env!("CARGO_BIN_EXE_cpl"))
            .args([
                "build",
                file.to_str().unwrap(),
                "--checks",
                "traps",
                "-o",
                cpp.to_str().unwrap(),
            ])
            .output()
            .expect("cpl build");
        assert!(
            build.status.success(),
            "cpl build failed:\n{}",
            String::from_utf8_lossy(&build.stderr)
        );
        let compile = Command::new("g++")
            .args([
                "-std=c++17",
                "-O1",
                "-DLOCAL",
                cpp.to_str().unwrap(),
                "-o",
                exe.to_str().unwrap(),
            ])
            .output()
            .expect("g++");
        assert!(
            compile.status.success(),
            "g++ failed:\n{}",
            String::from_utf8_lossy(&compile.stderr)
        );
        let mut child = Command::new(&exe)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("run binary");
        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
        let out = child.wait_with_output().expect("wait binary");
        let cpp_err = String::from_utf8_lossy(&out.stderr).into_owned();
        let _ = fs::remove_dir_all(&dir);
        assert_eq!(
            out.status.code(),
            Some(3),
            "checked cpp case {idx} should trap:\n{cpp_err}"
        );
        assert!(
            cpp_err.contains(msg),
            "checked cpp case {idx} expected `{msg}`, got:\n{cpp_err}"
        );
    }
}

#[test]
fn domain_array_literal_mismatch_traps_in_interp_and_checked_cpp() {
    let msg = "an array literal needs one value per position";
    let cases = [
        // A size known only at run time; a constant one is E0302 (R04.8.10).
        "let n = [3][0]\ndomain D = 0..n\nstruct Boxed(xs: [D]Int)\nlet b = Boxed(xs: [10, 20])\nprint(b.xs[D(0)])\n",
        "let n = [3][0]\ndomain D = 0..n\nlet xs: [D]Int = [10, 20]\nprint(xs[D(0)])\n",
    ];
    for (idx, src) in cases.iter().enumerate() {
        let (code, err) = run_stderr(src, "");
        assert_eq!(code, 3, "interp case {idx} should trap:\n{err}");
        assert!(
            err.contains(msg),
            "interp case {idx} expected `{msg}`, got:\n{err}"
        );
        assert!(
            !err.contains("the interpreter itself failed"),
            "got panic:\n{err}"
        );

        let dir = std::env::temp_dir().join(format!(
            "runtime_errors_dom_lit_{}_{idx}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let file = dir.join("a.cp");
        let cpp = dir.join("a.cpp");
        let exe = dir.join("a.bin");
        fs::write(&file, src).unwrap();
        let build = Command::new(env!("CARGO_BIN_EXE_cpl"))
            .args([
                "build",
                file.to_str().unwrap(),
                "--checks",
                "traps",
                "-o",
                cpp.to_str().unwrap(),
            ])
            .output()
            .expect("cpl build");
        assert!(
            build.status.success(),
            "cpl build failed:\n{}",
            String::from_utf8_lossy(&build.stderr)
        );
        let compile = Command::new("g++")
            .args([
                "-std=c++17",
                "-O1",
                "-DLOCAL",
                cpp.to_str().unwrap(),
                "-o",
                exe.to_str().unwrap(),
            ])
            .output()
            .expect("g++");
        assert!(
            compile.status.success(),
            "g++ failed:\n{}",
            String::from_utf8_lossy(&compile.stderr)
        );
        let out = Command::new(&exe).output().expect("run binary");
        let cpp_err = String::from_utf8_lossy(&out.stderr).into_owned();
        let _ = fs::remove_dir_all(&dir);
        assert_eq!(
            out.status.code(),
            Some(3),
            "checked cpp case {idx} should trap:\n{cpp_err}"
        );
        assert!(
            cpp_err.contains(msg),
            "checked cpp case {idx} expected `{msg}`, got:\n{cpp_err}"
        );
    }
}

#[test]
fn prefix_filling_traps_in_interp_and_checked_cpp() {
    let cases = [
        (
            "domain D = 0..4\nvar p = Prefix<D, Int>.filling()\np.set(D(1), 5)\n",
            "in_order",
        ),
        (
            "domain D = 0..4\nvar p = Prefix<D, Int>.filling(order: FillOrder.Reverse)\np.set(D(0), 5)\n",
            "in_order",
        ),
        (
            "domain D = 0..4\nvar p = Prefix<D, Int>.filling()\np.set(D(0), 5)\nprint(p.fold(D(0)..=D(1)))\n",
            "in_filled",
        ),
        (
            "domain D = 0..4\nvar p = Prefix<D, Int>.filling(order: FillOrder.Reverse)\np.set(D(3), 5)\nprint(p.fold(D(2)..=D(3)))\n",
            "in_filled",
        ),
        (
            "domain R = 0..2\ndomain C = 0..3\nvar p = Prefix<R, C, Int>.filling()\np.set(R(0), C(1), 1)\n",
            "in_order",
        ),
        (
            "domain R = 0..2\ndomain C = 0..3\nvar p = Prefix<R, C, Int>.filling(order: FillOrder.Reverse)\np.set(R(1), C(2), 1)\nprint(p.fold(R(1)..=R(1), C(1)..=C(2)))\n",
            "in_filled",
        ),
        (
            "var p = Prefix<Int>.filling(4)\np.set(2, 7)\n",
            "in_order",
        ),
        (
            "var p = Prefix<Int>.filling(4, order: FillOrder.Reverse)\np.set(3, 7)\nprint(p.fold(2..4))\n",
            "in_filled",
        ),
    ];
    for (idx, (src, msg)) in cases.iter().enumerate() {
        let (code, err) = run_stderr(src, "");
        assert_eq!(code, 3, "interp case {idx} should trap:\n{err}");
        assert!(
            err.contains(msg),
            "interp case {idx} expected `{msg}`, got:\n{err}"
        );
        assert!(
            !err.contains("the interpreter itself failed"),
            "got panic:\n{err}"
        );

        let (cpp_code, cpp_err) = build_and_run_cpp(src);
        assert_eq!(
            cpp_code, 3,
            "checked cpp case {idx} should trap:\n{cpp_err}"
        );
        assert!(
            cpp_err.contains(msg),
            "checked cpp case {idx} expected `{msg}`, got:\n{cpp_err}"
        );
    }
}

#[test]
fn call_stack_names_a_generic_method_as_the_source_does() {
    let source = "\
struct Box<T>:
    xs: List<T>
    fn at(self, i: Int) -> T:
        return xs[i]
let b = Box(xs: [1, 2])
print(b.at(5))
";
    let (code, err) = run_stderr(source, "");
    assert_eq!(code, 3, "got:\n{err}");
    assert!(
        err.contains("note: call stack: Box.at (a.cp:"),
        "got:\n{err}"
    );
}

#[test]
fn locals_of_a_call_that_traps_in_its_first_statement() {
    let source = "\
fn get(a: List<Int>, j: Int) -> Int:
    return a[j]

let k = 2
print(get([0, 0, 0], k + 1))
";
    let (code, err) = run_stderr(source, "");
    assert_eq!(code, 3, "got:\n{err}");
    assert!(
        err.contains("note: locals: a = [0, 0, 0], j = 3\n"),
        "got:\n{err}"
    );
}

/// Every runtime code in spec/diagnostics.md, on a program that traps with
/// it: the code and the message its table shows, exit status 3.
#[test]
fn every_runtime_code_traps_with_the_message_the_spec_shows() {
    let cases = [
        (
            "R0001",
            "let s: Int = read()\nprint(1 << s)\n",
            "70",
            "a shift by 70 is outside 0..64",
        ),
        (
            "R0101",
            "let xs: List<Int> = [1, 2]\nlet i: Int = read()\nprint(xs[i])\n",
            "5",
            "index 5 is outside a list of length 2",
        ),
        (
            "R0102",
            "domain D = 0..=2\nlet x: Int = read()\nprint(D(x))\n",
            "7",
            "7 is not in `D`, which is 0..=2",
        ),
        (
            "R0201",
            "var r: 0..10 = 0\nlet x: Int = read()\nr = x\nprint(r)\n",
            "14",
            "14 is outside `0..10`",
        ),
        (
            "R0202",
            "let x: Int = read()\nprint(x * 3)\n",
            "4000000000000000000",
            "integer overflow: `4000000000000000000 * 3` does not fit in an `Int` (64 bits)",
        ),
        (
            "R0203",
            "let x: Int = read()\nprint(5 / x)\n",
            "0",
            "division by zero: `5 / 0`",
        ),
        (
            "R0301",
            "let x: ?Int = None\nprint(x!)\n",
            "",
            "`!` on `None`: there is no value here",
        ),
        (
            "R0401",
            "let k: Int = read()\nassert k > 2\nprint(k)\n",
            "1",
            "assertion failed: k > 2",
        ),
        (
            "R0402",
            "fn f() -> Int:\n    todo\nprint(f())\n",
            "",
            "`todo` reached",
        ),
        (
            "R0501",
            "let x: Int = read()\nprint(x)\n",
            "abc",
            "expected a number, found `abc` (input line 1, column 1)",
        ),
        (
            "R0502",
            "let x: 1..=5 = read()\nprint(x)\n",
            "9",
            "9 is outside `1..=5` (input line 1, column 1)",
        ),
        (
            "R0601",
            "fn f(x: Int) -> Int:\n    return f(x + 1) + 1\nprint(f(0))\n",
            "",
            "the stack ran out: `f` never stops calling itself?",
        ),
    ];
    let mut no_help = Vec::new();
    for (code, source, input, message) in cases {
        let (status, err) = run_stderr(source, input);
        let expected = format!("error[{code}]: {message}");
        assert!(
            err.contains(&expected),
            "expected `{expected}`, got:\n{err}"
        );
        assert_eq!(status, 3, "{code}: {err}");
        if !err.contains("help: ") {
            no_help.push(format!("{code}: {err}"));
        }
    }
    // `errors/R11`: every trap says the CP Lang way out.
    assert!(
        no_help.is_empty(),
        "traps with no help:\n{}",
        no_help.join("\n")
    );
}

#[test]
fn the_locals_a_trap_names_come_first() {
    let source = "\
fn f(xs: List<Int>) -> Int:
    let a = 1
    let b = 2
    let c = 3
    let d = 4
    let e = 5
    let g = 6
    let h = 7
    let z = c + 2
    let bad = xs[z]
    return bad + a + b + d + e + g + h
print(f([1, 2]))
";
    let (_, err) = run_stderr(source, "");
    assert!(
        err.contains("note: locals: xs = [1, 2], z = 5, a = 1"),
        "{err}"
    );
}

#[test]
fn a_negative_index_says_how_to_reach_the_end() {
    let source = "let xs = [1, 2, 3]\nlet i: Int = read()\nprint(xs[i])\n";
    let (code, err) = run_stderr(source, "-1\n");
    assert_eq!(code, 3);
    assert!(
        err.contains("index -1 is outside a list of length 3"),
        "got:\n{err}"
    );
    assert!(
        err.contains("help: CP Lang has no negative indexes: the last item is `xs.last!`"),
        "got:\n{err}"
    );
    let (_, err) = run_stderr(source, "3\n");
    assert!(!err.contains("negative"), "got:\n{err}");
}

#[test]
fn a_bang_on_none_says_how_to_handle_none() {
    let source = "var h = MaxHeap<Int>()\nh.push(1)\nwhile h.peek! > 0:\n    h.pop()\n";
    let (code, err) = run_stderr(source, "");
    assert_eq!(code, 3);
    assert!(
        err.contains("help: `x!` needs a value: to handle `None`, write `if let Some(v) = x:`, in a loop `while let Some(v) = x and v > k:`"),
        "got:\n{err}"
    );
}

#[test]
fn golden_an_index_past_the_end_and_a_read_past_the_input_say_the_bound() {
    let src = "let n: Int = read()\nlet a: [0..n]Int = read()\nvar s = 0\nfor i in 0..=n:\n    s += a[i]\nprint(s)\n";
    let (code, err) = run_stderr(src, "3\n1 2 3\n");
    assert_eq!(code, 3);
    assert!(
        err.contains("index 3 is outside an array dimension of length 3"),
        "got:\n{err}"
    );
    assert!(
        err.contains("the indexes are 0 to 2: a loop over `0..=n` reaches `n`"),
        "got:\n{err}"
    );

    let src = "let n: Int = read()\nvar s = 0\nfor _ in 0..=n:\n    let x: Int = read()\n    s += x\nprint(s)\n";
    let (code, err) = run_stderr(src, "3\n1 2 3\n");
    assert_eq!(code, 3);
    assert!(err.contains("error[R0501]: the input ended"), "got:\n{err}");
    assert!(err.contains("`0..=n` reads n + 1"), "got:\n{err}");
}

#[test]
fn a_shift_past_the_width_says_the_range_and_the_way_out() {
    let (code, err) = run_stderr("let s = 70\nprint(1 << s)\n", "");
    assert_eq!(code, 3);
    assert!(
        err.contains("a shift by 70 is outside 0..64"),
        "got:\n{err}"
    );
    assert!(
        err.contains("help: `a << k` and `a >> k` need `k` from 0"),
        "got:\n{err}"
    );
}

#[test]
fn common_traps_say_the_cpl_fix() {
    let cases = [
        (
            "let a: Int = read()\nprint(a * a)\n",
            "4000000000\n",
            "widen this value to `I128`",
        ),
        (
            "let a: Int = read()\nprint(42 / a)\n",
            "0\n",
            "guard it with `if b != 0:`",
        ),
        (
            "let a: Int = read()\nprint(a)\n",
            "abc\n",
            "the reads don't line up with the input",
        ),
        (
            "let n: 1..=5 = read()\nprint(n)\n",
            "9\n",
            "the input breaks a range you declared",
        ),
        (
            "var x: 0..10 = 0\nlet k: Int = read()\nx = k * 2\nprint(x)\n",
            "7\n",
            "widen the type, or check the value first",
        ),
    ];
    for (src, input, help) in cases {
        let (code, err) = run_stderr(src, input);
        assert_eq!(code, 3, "for {help}: {err}");
        assert!(
            err.contains("help: ") && err.contains(help),
            "expected `{help}`, got:\n{err}"
        );
    }
}

#[test]
fn a_top_level_trap_names_the_value_it_reads_not_ended_loop_locals() {
    let source = "var xs = [3, 1]\nfor i in 0..2:\n    let v = xs[i]\n    print(v)\nlet w: ?Int = None\nprint(w!)\n";
    let (code, err) = run_stderr(source, "");
    assert_eq!(code, 3);
    assert!(err.contains("note: locals: w = None"), "got:\n{err}");
    assert!(!err.contains("i = 1"), "got:\n{err}");
}

#[test]
fn a_trap_at_an_assigned_name_notes_the_names_its_line_reads() {
    let source = "var x: 0..10 = 0\nlet k: Int = read()\nx = k * 2\nprint(x)\n";
    let (_, err) = run_stderr(source, "7\n");
    assert!(err.contains("locals: x = 0, k = 7"), "got:\n{err}");
}

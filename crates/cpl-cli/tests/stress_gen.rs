//! End-to-end CLI tests for `gen(...)` / `gen<T>(...)` stress generation and shrinking.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn cpl() -> Command {
    Command::new(env!("CARGO_BIN_EXE_cpl"))
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("cpl-stress-gen-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn gen_all_shapes_interp_and_cpp_agree() {
    let dir = scratch("shapes");
    let src_path = dir.join("gen_shapes.cp");
    let cpp_path = dir.join("gen_shapes.cpp");
    let bin_path = dir.join("gen_shapes");
    let src = r#"
let n = gen(3..=8)
domain V = 1..=n
let a = gen<[V]1..=1e9>()
let s = gen<Str of "ab">(len: 1..=10)
let g = gen<Graph<V, (to: V, w: 1..=9)>>(simple: true, connected: true)
let t1 = gen<Rooted<V>>(shape: random)
let t2 = gen<Rooted<V>>(shape: path)
let t3 = gen<Rooted<V>>(shape: star)
let t4 = gen<Rooted<V>>(shape: caterpillar)
let t5 = gen<Rooted<V>>(shape: binary)
let p = gen<Perm<V>>()
print(n, a.len, s, g.edges.len, t1.root, t2.depth(V.last), t3.depth(V.last), t4.root, t5.root, p)
"#;
    fs::write(&src_path, src).unwrap();

    let interp = cpl().args(["run"]).arg(&src_path).output().unwrap();
    assert!(
        interp.status.success(),
        "cpl run failed: {}",
        String::from_utf8_lossy(&interp.stderr)
    );

    let build = cpl()
        .args(["build"])
        .arg(&src_path)
        .args(["-o"])
        .arg(&cpp_path)
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "cpl build failed: {}",
        String::from_utf8_lossy(&build.stderr)
    );

    let compile = Command::new("g++")
        .args(["-O2", "-std=c++17"])
        .arg(&cpp_path)
        .arg("-o")
        .arg(&bin_path)
        .output()
        .unwrap();
    assert!(
        compile.status.success(),
        "g++ failed: {}",
        String::from_utf8_lossy(&compile.stderr)
    );

    let cpp = Command::new(&bin_path).output().unwrap();
    assert!(cpp.status.success());
    assert_eq!(
        String::from_utf8_lossy(&interp.stdout),
        String::from_utf8_lossy(&cpp.stdout),
        "interpreter and C++ output must match byte-for-byte"
    );
}

#[test]
fn at_gen_with_gen_shrinks_and_saves_counterexample() {
    let dir = scratch("shrink");
    let sol = dir.join("sol.cp");
    let src = r#"
let n: 1..=8 = read()
domain V = 1..=n
let a: [V]1..=100 = read()
var ans = a.sum()
if n >= 3 and a.any(x => x > 5): ans -= 1
print(ans)

@alt(program)
fn brute():
    let n: 1..=8 = read()
    domain V = 1..=n
    let a: [V]1..=100 = read()
    print(a.sum())

@gen
fn input():
    let n = gen(1..=8)
    domain V = 1..=n
    let a = gen<[V]1..=100>()
    print(n)
    print(a)
"#;
    fs::write(&sol, src).unwrap();

    let out = cpl()
        .args(["stress"])
        .arg(&sol)
        .args(["--cases", "50", "--seed", "1"])
        .output()
        .unwrap();
    assert!(
        !out.status.success(),
        "expected stress to find planted bug, stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("FAIL"), "stdout:\n{stdout}");
    assert!(stdout.contains("shrunk"), "expected shrinking:\n{stdout}");

    let tests_dir = dir.join("sol.tests");
    let saved_in: Vec<_> = fs::read_dir(&tests_dir)
        .expect("sol.tests/ should be created")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("stress-") && n.ends_with(".in"))
        })
        .collect();
    assert_eq!(saved_in.len(), 1, "expected one sol.tests/stress-<seed>.in");
    let content = fs::read_to_string(&saved_in[0]).unwrap();
    let nums: Vec<i64> = content
        .split_whitespace()
        .map(|t| t.parse().unwrap())
        .collect();
    assert_eq!(nums[0], 3, "shrunk n should be minimal (3), got {content:?}");
    assert!(
        nums[1..].iter().any(|&x| x > 5),
        "shrunk input should still trigger bug: {content:?}"
    );
}

#[test]
fn canonical_stress_spellings_only_and_help_text_updated() {
    let help = cpl().args(["--help"]).output().unwrap();
    assert!(help.status.success());
    let help_out = String::from_utf8_lossy(&help.stdout);
    assert!(help_out.contains("@check"), "{help_out}");
    assert!(!help_out.contains("@checker"), "{help_out}");
    assert!(!help_out.contains("@brute"), "{help_out}");

    let stress_help = cpl().args(["stress", "--help"]).output().unwrap();
    assert!(stress_help.status.success());
    let stress_out = String::from_utf8_lossy(&stress_help.stdout);
    assert!(stress_out.contains("@alt(program)"), "{stress_out}");
    assert!(stress_out.contains("@check"), "{stress_out}");
    assert!(!stress_out.contains("@brute"), "{stress_out}");
    assert!(!stress_out.contains("@checker"), "{stress_out}");

    let dir = scratch("legacy_spellings");
    let cases = [
        (
            "brute_main.cp",
            "let n: 1..=5 = read()\nprint(n)\n@brute(main)\nfn slow():\n    print(n)\n",
            "@alt(program)",
        ),
        (
            "alt_main.cp",
            "let n: 1..=5 = read()\nprint(n)\n@alt(main)\nfn slow():\n    print(n)\n",
            "@alt(program)",
        ),
        (
            "checker.cp",
            "let n: 1..=5 = read()\nprint(n)\n@checker\nfn ok():\n    let x: Int = read_output()\n    assert x == n\n",
            "@check",
        ),
        (
            "brute_fn.cp",
            "fn f(x: Int) -> Int = x\n@brute(f)\nfn f_slow(x: Int) -> Int = x\nprint(f(1))\n",
            "@alt(f)",
        ),
    ];
    for (name, src, expected) in cases {
        let p = dir.join(name);
        fs::write(&p, src).unwrap();
        let out = cpl().args(["check"]).arg(&p).output().unwrap();
        assert!(!out.status.success(), "{name} should fail check");
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.contains(expected),
            "{name}: expected fix-it mentioning {expected}, got:\n{stderr}"
        );
    }
}

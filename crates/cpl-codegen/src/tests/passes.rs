//! The PASSES table is the whole list: every module that walks IR bodies to
//! rewrite has a row, each row's firing test exists, and each Uses query a
//! row claims is asked in its module.

use super::cpp;
use crate::passes::{Evidence, PASSES};
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

fn is_test_file(rel: &str) -> bool {
    rel.starts_with("tests") || rel.ends_with("_tests.rs")
}

#[test]
fn every_module_that_walks_ir_to_rewrite_is_a_pass() {
    let src = root().join("crates/cpl-codegen/src");
    let mut files = Vec::new();
    rust_files(&src, &mut files);
    let mut missing = Vec::new();
    for path in files {
        let rel = path
            .strip_prefix(&src)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        if is_test_file(&rel) || rel.starts_with("uses") || rel == "walk.rs" {
            continue;
        }
        let text = std::fs::read_to_string(&path).unwrap();
        if text.contains("deep_") && !PASSES.iter().any(|p| p.module == rel) {
            missing.push(rel);
        }
    }
    assert!(
        missing.is_empty(),
        "walks IR with no row in PASSES (passes.rs): {missing:?}"
    );
}

#[test]
fn every_pass_names_a_module_a_firing_test_and_the_queries_it_asks() {
    let src = root().join("crates/cpl-codegen/src");
    let mut tests = String::new();
    let mut files = Vec::new();
    rust_files(&src, &mut files);
    rust_files(&root().join("crates/cpl-cli/tests"), &mut files);
    for path in files {
        tests.push_str(&std::fs::read_to_string(path).unwrap());
    }
    let mut names = std::collections::HashSet::new();
    for pass in PASSES {
        assert!(
            names.insert(pass.name),
            "two passes are named {}",
            pass.name
        );
        let module = std::fs::read_to_string(src.join(pass.module))
            .unwrap_or_else(|_| panic!("{}: no module {}", pass.name, pass.module));
        assert!(
            tests.contains(&format!("fn {}()", pass.fires)),
            "{}: no test `{}` shows it fires",
            pass.name,
            pass.fires
        );
        if let Evidence::Uses(queries) = pass.evidence {
            assert!(
                !queries.is_empty(),
                "{}: claims the Uses table but names no query",
                pass.name
            );
            for q in queries {
                assert!(
                    module.contains(q),
                    "{}: claims `{q}` but {} never asks it",
                    pass.name,
                    pass.module
                );
            }
            assert!(
                module.contains("uses::") || module.contains("Uses") || module.contains(".uses."),
                "{}: claims the Uses table but {} never reads it",
                pass.name,
                pass.module
            );
        }
    }
}

#[test]
fn a_global_a_fn_writes_stays_a_global_and_the_rest_are_main_locals() {
    let out = cpp(concat!(
        "var total = 0\n",
        "fn bump(x: Int):\n",
        "    total += x\n",
        "let n: Int = read()\n",
        "for i in 0..n: bump(i)\n",
        "print(total)\n",
    ));
    let main = out.find("void solve()").expect("a main");
    let total = out.find("ll total = 0;").expect("total is declared");
    let n = out.find("ll n = 0;").expect("n is declared");
    assert!(
        total < main && n > main,
        "bump writes total, so only n is main's own:\n{out}"
    );
}

#[test]
fn a_global_a_fn_writes_only_through_a_slice_stays_a_global() {
    let out = cpp(concat!(
        "var a = [0, 0, 0]\n",
        "fn bump():\n",
        "    a[1..3] = [5, 6]\n",
        "bump()\n",
        "print(a[1])\n",
    ));
    let main = out.find("void solve()").expect("a main");
    let a = out.find("std::vector<ll> a").expect("a is declared");
    assert!(a < main, "bump writes a through a slice:\n{out}");
}

#[test]
fn a_loop_over_a_list_literal_fn_walks_the_literal() {
    let out = cpp(concat!(
        "fn roles(a: Int, b: Int) -> List<(Int, Int, Int)> = [(a, b, 1), (b, a, 2)]\n",
        "let (p, q) = read<(Int, Int)>()\n",
        "var s = 0\n",
        "for (x, y, w) in roles(p, q): s += x * y * w\n",
        "print(s)\n",
    ));
    let main = &out[out.find("void solve()").unwrap()..];
    assert!(
        main.contains("std::array"),
        "the literal is on the stack:\n{main}"
    );
    assert!(
        !main.contains("roles("),
        "the call is inlined into the loop:\n{main}"
    );
}

#[test]
fn a_loop_local_list_is_one_static_buffer() {
    let out = cpp(concat!(
        "let n: 1..=4000 = read()\n",
        "var s = 0\n",
        "for it in 0..n:\n",
        "    var seen: [0..n]Int = 0\n",
        "    for k in 0..n: seen[(it + k) % n] += k\n",
        "    s += seen[0]\n",
        "print(s)\n",
    ));
    assert!(out.contains("static std::vector<ll> seen;"), "{out}");
}

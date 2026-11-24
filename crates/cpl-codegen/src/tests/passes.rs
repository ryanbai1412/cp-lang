//! The PASSES table is the whole list: every module that walks IR bodies to
//! rewrite has a row, each row's firing test exists, and each Uses query a
//! row claims is asked in its module.

use super::{cpp, lower};
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

#[test]
fn each_written_reports_every_store_set_and_mut_arg() {
    let program = lower(concat!(
        "fn add(mut l: List<Int>):\n",
        "    l.push(1)\n",
        "var a = 0\n",
        "var b: List<Int> = []\n",
        "var c: List<Int> = []\n",
        "a = 1\n",
        "add(mut b)\n",
        "c.push(2)\n",
        "print(a, b.len, c.len)\n",
    ));
    let mut roots = std::collections::BTreeSet::new();
    let blocks = [&program.setup, &program.main];
    for block in blocks {
        crate::uses::places::each_written(block, &mut |p| {
            roots.insert(crate::walk::root(p));
        });
    }
    assert_eq!(
        roots.len(),
        3,
        "a stored, b passed mut, c pushed: {roots:?}"
    );
}

#[test]
fn each_built_reports_a_struct_literal_nested_in_a_fn() {
    let program = lower(concat!(
        "struct P:\n",
        "    x: Int\n",
        "struct Q:\n",
        "    p: P\n",
        "fn make(k: Int) -> Q:\n",
        "    Q(p: P(x: k))\n",
        "print(make(1).p.x)\n",
    ));
    let mut built = Vec::new();
    for f in &program.fns {
        crate::uses::places::each_built(&f.body, &mut |s, fields| {
            built.push((program.structs[s].name.clone(), fields.len()));
        });
    }
    built.sort();
    assert_eq!(built, [("P".to_string(), 1), ("Q".to_string(), 1)]);
}

#[test]
fn read_counts_counts_each_read_at_any_depth() {
    let program = lower(concat!(
        "fn f(a: Int, b: Int) -> Int:\n",
        "    var t = a\n",
        "    for i in 0..b:\n",
        "        t += a\n",
        "    t\n",
        "print(f(1, 2))\n",
    ));
    let f = &program.fns[0];
    let reads = crate::uses::read_counts(&f.body, f.params.len());
    assert_eq!(reads, [2, 1], "a twice (once in the loop), b once");
}

#[test]
fn each_write_says_how_each_place_is_written() {
    use crate::uses::places::{each_write, Write};
    let program = lower(concat!(
        "fn add(mut l: List<Int>):\n",
        "    l.push(1)\n",
        "var t: List<Int> = []\n",
        "t.push(2)\n",
        "add(mut t)\n",
        "t = [3]\n",
        "print(t.len)\n",
    ));
    let (mut pushes, mut muts, mut stores) = (0, 0, 0);
    for block in [&program.setup, &program.main] {
        each_write(block, &mut |how, _| match how {
            Write::Mutate(cpl_ir::Mutation::Push) => pushes += 1,
            Write::Mut => muts += 1,
            Write::Store => stores += 1,
            _ => {}
        });
    }
    assert_eq!(
        (pushes, muts, stores),
        (1, 1, 2),
        "a global's first value is a store too"
    );
}

#[test]
fn each_loop_and_stmt_count_see_nested_loops() {
    use crate::uses::loops::{each_loop, stmt_count, Loop};
    let program = lower(concat!(
        "fn f(n: Int) -> Int:\n",
        "    var t = 0\n",
        "    for i in 0..n:\n",
        "        while t < i:\n",
        "            t += 1\n",
        "    t\n",
        "print(f(3))\n",
    ));
    let body = &program.fns[0].body;
    let (mut fors, mut whiles, mut ids) = (0, 0, Vec::new());
    each_loop(body, &mut |id, head| {
        ids.push(id);
        match head {
            Loop::For(..) => fors += 1,
            Loop::While => whiles += 1,
        }
    });
    ids.dedup();
    assert_eq!((fors, whiles, ids.len()), (1, 1, 2));
    assert!(stmt_count(body) >= 4, "let, for, while and the store");
}

#[test]
fn labeled_jumps_sees_a_labeled_break_and_continue_in_a_closure() {
    let out = cpp(concat!(
        "let n: Int = read()\n",
        "let f = (k: Int) =>\n",
        "    var t = 0\n",
        "    'outer: for i in 0..k:\n",
        "        for j in 0..k:\n",
        "            if j == 2: continue 'outer\n",
        "            if i == 3: break 'outer\n",
        "            t += j\n",
        "    t\n",
        "print(f(n))\n",
    ));
    assert!(
        out.contains("goto"),
        "the labeled jumps leave the inner loop:\n{out}"
    );
}

#[test]
fn each_call_and_each_closure_see_inside_a_fn() {
    use crate::uses::places::{each_call, each_closure};
    let program = lower(concat!(
        "fn twice(x: Int) -> Int = x * 2\n",
        "fn f(n: Int) -> Int:\n",
        "    let g = (k: Int) => twice(k) + n\n",
        "    twice(g(n))\n",
        "print(f(3))\n",
    ));
    let f = program.fns.iter().position(|f| f.name == "f").expect("f");
    let (mut calls, mut closures) = (Vec::new(), 0);
    each_call(&program.fns[f].body, &mut |callee, args| {
        calls.push((program.fns[callee].name.clone(), args.len()));
    });
    each_closure(&program.fns[f].body, &mut |_| closures += 1);
    assert_eq!(calls, [("twice".to_string(), 1)], "g's body is its own");
    assert_eq!(closures, 1);
}

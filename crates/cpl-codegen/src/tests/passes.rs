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
        if ["deep_expr", "deep_stmts"].iter().any(|w| text.contains(w))
            && !PASSES.iter().any(|p| p.module == rel)
        {
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

#[test]
fn each_put_reports_lets_and_stores_in_order() {
    use crate::uses::places::{each_put, Put};
    let program = lower(concat!(
        "fn f(n: Int) -> Int:\n",
        "    let a = n\n",
        "    var b = a\n",
        "    b += 2\n",
        "    b = a\n",
        "    b\n",
        "print(f(3))\n",
    ));
    let mut seen = Vec::new();
    each_put(&program.fns[0].body, &mut |put, _| {
        seen.push(match put {
            Put::Let(_) => "let",
            Put::Store(_, cpl_ir::AssignOp::Set) => "set",
            Put::Store(..) => "update",
        });
    });
    assert_eq!(seen, ["let", "let", "update", "set"]);
}

#[test]
fn each_value_call_reports_a_call_of_a_fn_value() {
    use crate::uses::places::each_value_call;
    let program = lower(concat!(
        "fn apply(g: Int -> Int, x: Int) -> Int = g(g(x))\n",
        "print(apply(x => x + 1, 3))\n",
    ));
    let mut calls = 0;
    each_value_call(&program.fns[0].body, &mut |callee, args| {
        calls += usize::from(callee.kind == cpl_ir::ExprKind::Local(0) && args.len() == 1);
    });
    assert_eq!(calls, 2, "g is called twice, once inside the other");
}

#[test]
fn each_opened_reports_an_optional_an_if_let_tests_and_reads() {
    let program = lower(concat!(
        "fn f(a: ?List<Int> = None) -> Int:\n",
        "    if let b = a: b.len else: 0\n",
        "print(f([1, 2]) + f())\n",
    ));
    let f = program.fns.iter().find(|f| f.name == "f").expect("f");
    let mut opened = Vec::new();
    crate::uses::places::each_opened(&f.body, &mut |x| opened.push(x.kind.clone()));
    assert_eq!(
        opened.len(),
        2,
        "the if let tests its copy of a, then reads through it: {opened:?}"
    );
}

#[test]
fn each_write_in_and_each_tuple_in_stay_inside_one_statement() {
    use crate::uses::places::{each_tuple_in, each_write_in};
    let program = lower(concat!(
        "fn f(n: Int) -> (Int, Int):\n",
        "    var t: List<Int> = []\n",
        "    if n > 0:\n",
        "        t.push(n)\n",
        "    (n, t.len)\n",
        "print(f(3))\n",
    ));
    let stmts = &program.fns[0].body.stmts;
    let (mut writes, mut tuples) = (Vec::new(), Vec::new());
    for stmt in stmts {
        let mut w = 0;
        each_write_in(stmt, &mut |_, _| w += 1);
        writes.push(w);
        let mut k = 0;
        each_tuple_in(stmt, &mut |items| k += items.len());
        tuples.push(k);
    }
    assert_eq!(writes.iter().sum::<usize>(), 1, "only the push: {writes:?}");
    assert_eq!(tuples.iter().sum::<usize>(), 2, "the pair: {tuples:?}");
}

#[test]
fn each_write_hands_back_places_that_outlive_the_walk() {
    use crate::uses::places::{each_write, Write};
    let program = lower(concat!(
        "fn f(n: Int) -> Int:\n",
        "    var t: List<Int> = []\n",
        "    var k = 0\n",
        "    t.push(n)\n",
        "    k = t.len\n",
        "    k\n",
        "print(f(3))\n",
    ));
    let mut places: Vec<(bool, &cpl_ir::Place)> = Vec::new();
    each_write(&program.fns[0].body, &mut |how, place| {
        places.push((matches!(how, Write::Store), place));
    });
    let stores = places.iter().filter(|(store, _)| *store).count();
    assert_eq!(
        (places.len(), stores),
        (2, 1),
        "the push and the store to k"
    );
}

#[test]
fn each_stmt_expr_and_each_part_see_statements_and_their_parts() {
    use crate::uses::places::{each_part, each_stmt_expr};
    let program = lower(concat!(
        "fn f(n: Int) -> Int:\n",
        "    var t: List<Int> = []\n",
        "    if n > 0:\n",
        "        t.push(n + 1)\n",
        "    t.len\n",
        "print(f(3))\n",
    ));
    let mut stmts = 0;
    let mut parts = 0;
    each_stmt_expr(&program.fns[0].body, &mut |e| {
        stmts += 1;
        each_part(e, &mut |_| parts += 1);
    });
    assert!(stmts >= 1, "the push is an expression statement");
    assert!(
        parts > stmts,
        "each_part sees inside the push, not only the push"
    );
}

#[test]
fn each_let_run_breaks_a_run_of_lets_at_any_other_statement() {
    use crate::uses::split::each_let_run;
    let program = lower(concat!(
        "fn f(n: Int) -> Int:\n",
        "    let a = n + 1\n",
        "    let b = a * 2\n",
        "    print(b)\n",
        "    let c = b - 1\n",
        "    c\n",
        "print(f(3))\n",
    ));
    let mut shape = String::new();
    each_let_run(&program.fns[0].body, &mut |stmt| {
        shape.push(if stmt.is_some() { 'L' } else { '-' });
    });
    assert!(
        shape.contains("LL-L"),
        "two lets, the print, then a let: {shape}"
    );
}

#[test]
fn each_sink_says_how_a_store_combines() {
    use crate::uses::split::{each_sink, Sink};
    let program = lower(concat!(
        "fn f(n: Int) -> Int:\n",
        "    var k = 0\n",
        "    k = n\n",
        "    k += 1\n",
        "    k\n",
        "print(f(3))\n",
    ));
    let mut ops = Vec::new();
    each_sink(&program.fns[0].body, &mut |sink| {
        if let Sink::Store(_, op, _) = sink {
            ops.push(*op);
        }
    });
    assert!(
        matches!(
            ops.as_slice(),
            [cpl_ir::AssignOp::Set, cpl_ir::AssignOp::Binary(_)]
        ),
        "{ops:?}"
    );
}

#[test]
fn each_sink_reports_lets_and_stores_in_statement_order() {
    use crate::uses::split::{each_sink, Sink};
    let program = lower(concat!(
        "fn f(n: Int) -> Int:\n",
        "    var k = n\n",
        "    k = k * 2\n",
        "    let j = k + 1\n",
        "    j\n",
        "print(f(3))\n",
    ));
    let mut order = String::new();
    each_sink(&program.fns[0].body, &mut |sink| match sink {
        Sink::Let(..) => order.push('L'),
        Sink::Store(..) => order.push('S'),
        _ => {}
    });
    assert!(
        order.contains("LSL"),
        "let k, the store, then let j: {order}"
    );
}

#[test]
fn each_sink_reports_a_for_binding_and_a_return() {
    use crate::uses::split::{each_sink, Sink};
    let program = lower(concat!(
        "fn f(n: Int) -> Int:\n",
        "    var s = 0\n",
        "    for i in 0..n:\n",
        "        s += i\n",
        "    return s\n",
        "print(f(3))\n",
    ));
    let (mut fors, mut returns) = (0, 0);
    each_sink(&program.fns[0].body, &mut |sink| match sink {
        Sink::For(..) => fors += 1,
        Sink::Return(_) => returns += 1,
        _ => {}
    });
    assert_eq!((fors, returns), (1, 1));
}

#[test]
fn has_jump_sees_a_break_nested_in_an_if() {
    use crate::uses::loops::has_jump;
    let program = lower(concat!(
        "fn f(n: Int) -> Int:\n",
        "    var s = 0\n",
        "    for i in 0..n:\n",
        "        if i > 2:\n",
        "            break\n",
        "        s += i\n",
        "    s\n",
        "fn g(n: Int):\n",
        "    var s = 0\n",
        "    for i in 0..n:\n",
        "        s += i\n",
        "    print(s)\n",
        "print(f(3))\n",
        "g(3)\n",
    ));
    assert!(has_jump(&program.fns[0].body), "f breaks out of its loop");
    assert!(!has_jump(&program.fns[1].body), "g only runs to its end");
}

#[test]
fn branches_and_binds_only_rejects_a_store_or_a_loop() {
    use crate::uses::places::branches_and_binds_only;
    let program = lower(concat!(
        "fn f(n: Int) -> Int:\n",
        "    if n > 0:\n",
        "        return 1\n",
        "    let x = n + 1\n",
        "    x\n",
        "fn g(n: Int) -> Int:\n",
        "    var s = 0\n",
        "    for i in 0..n:\n",
        "        s += i\n",
        "    s\n",
        "print(f(3) + g(3))\n",
    ));
    assert!(
        branches_and_binds_only(&program.fns[0].body),
        "f only binds and branches"
    );
    assert!(
        !branches_and_binds_only(&program.fns[1].body),
        "g stores in a loop"
    );
}

#[test]
fn each_index_reports_the_outer_base_first() {
    use crate::uses::places::each_index;
    let program = lower(concat!(
        "fn f() -> Int:\n",
        "    let a = [1, 2]\n",
        "    let b = [0]\n",
        "    a[b[0]]\n",
        "print(f())\n",
    ));
    let mut slots = Vec::new();
    each_index(&program.fns[0].body, &mut |base| {
        if let cpl_ir::ExprKind::Local(s) = base.kind {
            slots.push(s);
        }
    });
    assert_eq!(slots.len(), 2, "two index reads: {slots:?}");
    assert!(
        slots[0] < slots[1],
        "`a`, declared first, is the outer base: {slots:?}"
    );
}

#[test]
fn stmt_sites_lists_every_statement_once() {
    use crate::uses::loops::{stmt_count, stmt_sites};
    let program = lower(concat!(
        "fn f(n: Int) -> Int:\n",
        "    var s = 0\n",
        "    for i in 0..n:\n",
        "        if i > 1:\n",
        "            s += i\n",
        "    s\n",
        "print(f(3))\n",
    ));
    let body = &program.fns[0].body;
    let sites = stmt_sites(body);
    assert_eq!(sites.len(), stmt_count(body));
    assert_eq!(sites[0], &body.stmts[0] as *const cpl_ir::Stmt);
}

#[test]
fn defines_domain_and_notes_an_assert_see_their_statements() {
    use crate::uses::places::{defines_domain, notes_an_assert};
    let mut with = lower(concat!(
        "domain V = 0..5\n",
        "fn f(n: Int) -> Int:\n",
        "    assert n == 3, \"need 3, got {n}\"\n",
        "    n\n",
        "print(f(3))\n",
    ));
    let without = lower("fn f(n: Int) -> Int:\n    n\nprint(f(3))\n");
    assert!(defines_domain(&with.setup) || defines_domain(&with.main));
    assert!(!defines_domain(&without.setup) && !defines_domain(&without.main));
    // The user's own assert has no library note; one inlined from a library does.
    assert!(!notes_an_assert(&with.fns[0].body));
    let mut noted = false;
    for stmt in &mut with.fns[0].body.stmts {
        if let cpl_ir::Stmt::Assert { note, .. } = stmt {
            *note = Some("std/math.cp:1".into());
            noted = true;
        }
    }
    assert!(noted, "f holds its assert");
    assert!(notes_an_assert(&with.fns[0].body));
}

#[test]
fn breaks_or_returns_lets_a_continue_through() {
    use crate::uses::loops::breaks_or_returns;
    let program = lower(concat!(
        "fn f(n: Int):\n",
        "    for i in 0..n:\n",
        "        if i > 2:\n",
        "            continue\n",
        "        print(i)\n",
        "fn g(n: Int):\n",
        "    for i in 0..n:\n",
        "        if i > 2:\n",
        "            break\n",
        "        print(i)\n",
        "f(3)\n",
        "g(3)\n",
    ));
    assert!(!breaks_or_returns(&program.fns[0].body), "f only continues");
    assert!(breaks_or_returns(&program.fns[1].body), "g breaks out");
}

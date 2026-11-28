use super::*;
use cpl_next_ir::ir::{Place, Stmt};
use cpl_next_ir::prog::CTy;
use modules::PROGRAM;

#[path = "../../cpl-cli/tests/support/cxx.rs"]
mod cxx;

/// The reference interpreter's run of `c`'s linked instances: the oracle the engine is held to.
fn raw(c: &Compiled) -> Outcome {
    let (p, fs) = c.form(Form::Raw);
    cpl_next_interp::run(p, fs, cpl_next_ir::run::RunInput::fixed(""))
}

/// A method found in a std module the program does not load is called without loading it
/// (R11.3.4), so the modules loaded stay the ones the text names.
#[test]
fn a_method_call_loads_no_module() {
    let seen = |src: &str| {
        let f = front(&Std::read(), src, &Roots::std(), None, Checks::CHECKED);
        assert!(f.diags.is_empty(), "{:?}", f.diags);
        f.d.seen.clone()
    };
    let g = "domain R = 0..2\ndomain C = 0..3\nlet g: [R, C]Int = [R, C] (r, c) => 10 * Int(r) + Int(c)\n";
    let called = seen(&format!("{g}print(g.transposed()[C(2), R(1)])\n"));
    assert_eq!(called, seen(&format!("{g}print(g[R(1), C(2)])\n")));
}

/// Ownership marks a heap param nothing writes as read where the caller has it; a plain
/// one is copied, and one the body writes is its own.
#[test]
fn a_param_nothing_writes_is_a_view() {
    let src = "fn pick(xs: List<Int>, n: Int, ys: List<Int>) -> Int:\n    var zs = ys\n    zs.push(n)\n    xs[0] + zs.len\n\nprint(pick([1, 2], 3, [4]))\n";
    let c = compile(
        &Std::read(),
        src,
        &Roots::std(),
        false,
        None,
        Checks::CHECKED,
    )
    .expect("compiles");
    let f = c
        .prog
        .insts
        .iter()
        .find(|f| f.name == "pick")
        .expect("pick");
    assert_eq!(f.views, [true, false, true]);
}

/// Ownership places every release (R1): the frame releases its heap locals, a discarded heap
/// value goes to a temporary it drops, and an overwrite drops the old value: first, or once
/// the new value is computed when computing it reads the old one.
#[test]
fn ownership_places_every_release() {
    let src = "fn grow(xs: List<Int>) -> List<Int>:\n    var ys = xs.clone()\n    ys.push(1)\n    ys\n\nfn f(n: Int) -> Int:\n    var xs = [n]\n    xs = grow(xs)\n    grow(xs)\n    var g = [[n]]\n    g[0] = grow(xs)\n    xs.len + g[0].len\n\nprint(f(3))\n";
    let c = compile(
        &Std::read(),
        src,
        &Roots::std(),
        false,
        None,
        Checks::CHECKED,
    )
    .expect("compiles");
    let f = c.prog.insts.iter().find(|f| f.name == "f").expect("f");
    assert!(
        !f.drops.contains(&0),
        "the Int param holds nothing: {:?}",
        f.drops
    );
    let drop_of = |s: &Stmt<CTy>| match s {
        Stmt::Drop(Place::Local(l)) => Some(*l),
        _ => None,
    };
    // `g[0] = ..` sets the cell `g` lends out, in the region's body.
    let mut bodies = vec![&f.body];
    let mut cells = 0;
    while let Some(b) = bodies.pop() {
        for (i, s) in b.iter().enumerate() {
            if let Stmt::Region { body, .. } = s {
                bodies.push(body);
                cells += 1;
            }
            if let Stmt::Let(l, _) | Stmt::Assign(Place::Local(l), _) = s {
                if f.drops.contains(l) {
                    assert_eq!(drop_of(&b[i - 1]), Some(*l), "overwrite {i}: {b:?}");
                }
            }
        }
    }
    assert!(cells > 0, "{:#?}", f.body);
    for g in &c.prog.insts {
        let heap = |e: &cpl_next_ir::ir::Expr<CTy>| c.prog.stores(&e.ty);
        let discards = g.body.iter().any(|s| matches!(s, Stmt::Expr(e) if heap(e)));
        assert!(!discards, "{} discards a heap value", g.name);
    }
}

/// The engine and the C++ release, at run time, every value ownership made, each at the drops
/// it placed (contract 07): an overwrite, a discard, a temporary given to a view, a global
/// with storage, a closure's captures, a closure a call made, and a frame left by `return`
/// from a loop. The engine ends holding nothing, and
/// the C++ under AddressSanitizer leaks nothing, frees nothing twice and reads nothing freed.
/// Without the drops own placed in a body the engine holds storage: the count sees a missing one.
#[test]
fn the_engine_and_the_cpp_release_every_value_ownership_made() {
    let srcs = [
        "fn f(n: Int) -> Int:\n    var r: 0..=10 = 3\n    var xs = [n]\n    xs = [n, r]\n    [r]\n    xs.len\n\nprint(f(3))\n",
        "fn made(n: Int) -> List<Int> = [k for k in 0..n]\nfn total(n: Int, xs: List<Int>) -> Int = n + xs.len\nlet ys = made(2)\nprint(total(1, made(3)), total(ys.len, ys))\n",
        "var g = [1, 2, 3]\nfn go() -> Int = g.len + 1\nprint(go())\n",
        "var g = [1, 2, 3]\nfn go() -> Int:\n    let xs = [4, 5]\n    let h = (k: Int) => k + xs.len + g.len\n    let h2 = h\n    h(1) + h2(2)\nprint(go())\n",
        "fn adder(n: Int) -> (Int) -> Int = (k: Int) => k + n\nprint(adder(2)(3) + adder(4)(g()))\nfn g() -> Int:\n    print(\"g\")\n    1\n",
        "fn first(n: Int) -> Int:\n    for i in 0..n:\n        let xs = [i, i]\n        if i == 3:\n            return xs.len + i\n    0\nprint(first(10))\n",
    ];
    let flags = [
        "-std=c++17",
        "-O1",
        "-g",
        "-fsanitize=address,undefined",
        "-fno-sanitize-recover=undefined",
        "-w",
        "-x",
        "c++",
    ];
    for src in srcs {
        let c = compile(
            &Std::read(),
            src,
            &Roots::std(),
            false,
            None,
            Checks::CHECKED,
        )
        .expect("compiles");
        let (run, held) =
            cpl_next_engine::run_held(&c.prog, &c.opt, cpl_next_engine::RunInput::fixed(""));
        assert!(run.trap.is_none(), "{src}");
        assert_eq!(held, 0, "{src}");
        let (bin, _) = cxx::build(c.cpp().as_bytes(), &flags).expect("g++ builds it");
        let ran = std::process::Command::new(bin)
            .env("ASAN_OPTIONS", "detect_leaks=1")
            .output()
            .expect("runs");
        let err = String::from_utf8_lossy(&ran.stderr);
        assert!(ran.status.success() && err.is_empty(), "{src}{err}");
        assert_eq!(String::from_utf8_lossy(&ran.stdout), run.output, "{src}");
    }
    fn unplaced(b: &mut Vec<Stmt<CTy>>) {
        b.retain(|s| !matches!(s, Stmt::Drop(_)));
        for s in b {
            match s {
                Stmt::If(_, a, b) | Stmt::Loop(a, b, _) => {
                    unplaced(a);
                    unplaced(b);
                }
                Stmt::Switch(_, arms) => arms.iter_mut().for_each(unplaced),
                Stmt::Region { body, .. } => unplaced(body),
                _ => {}
            }
        }
    }
    let mut c = compile(
        &Std::read(),
        srcs[0],
        &Roots::std(),
        false,
        None,
        Checks::CHECKED,
    )
    .expect("compiles");
    (c.opt.iter_mut().filter(|f| f.name == "f")).for_each(|f| unplaced(&mut f.body));
    let (_, held) =
        cpl_next_engine::run_held(&c.prog, &c.opt, cpl_next_engine::RunInput::fixed(""));
    assert!(held > 0, "a missing release is seen");
}

/// A fn's sites are its own: one written above it leaves its sites and their places as
/// they were.
#[test]
fn a_fn_above_leaves_the_sites_of_those_below() {
    let below = "fn widget(xs: List<Int>) -> Int:\n    xs[0] + xs[1]\n\nprint(widget([1, 2]))\n";
    let above = format!("fn f() -> Int: 1\n\n{below}");
    let g = |src: &str| {
        let c = compile(
            &Std::read(),
            src,
            &Roots::std(),
            false,
            None,
            Checks::CHECKED,
        )
        .expect("compiles");
        let insts = &c.prog.insts;
        let g = insts.iter().find(|f| f.name == "widget").expect("widget");
        // An instance is named by what it instantiates: a fn above adds instances before it.
        let body = (insts.iter().enumerate().rev()).fold(format!("{:?}", g.body), |b, (k, f)| {
            b.replace(&format!("Inst({k})"), &format!("Inst({})", f.name))
        });
        (
            body,
            g.sites.clone(),
            g.origins.iter().map(|&o| c.line(o)).collect::<Vec<_>>(),
        )
    };
    let ((body, sites, lines), (body2, sites2, lines2)) = (g(below), g(&above));
    assert!(sites.len() > 1);
    assert_eq!((body, sites), (body2, sites2));
    assert_eq!(
        lines
            .iter()
            .map(|l| l + 2 * (*l != 0) as u32)
            .collect::<Vec<_>>(),
        lines2
    );
}

fn diags(src: &str) -> Vec<Diag> {
    compile(
        &Std::read(),
        src,
        &Roots::std(),
        false,
        None,
        Checks::CHECKED,
    )
    .err()
    .expect("rejected")
}

/// What compiling `src` gives: its diagnostics, or its C++ and what its run prints.
fn outcome(std: &Std, src: &str) -> String {
    match compile(std, src, &Roots::std(), true, None, Checks::CHECKED) {
        Ok(c) => {
            let o = raw(&c);
            let funcs = c.form(Form::Raw).1;
            let events: String = o.events.iter().map(|e| e.line(funcs) + "\n").collect();
            format!("{}\n{}{events}{}", c.cpp(), o.output, o.trap.is_some())
        }
        Err(ds) => ds.iter().map(|d| format!("{d}\n")).collect(),
    }
}

/// std.math's float fns and roundings run alike in the interpreter and the engine, and a
/// rounding past `Int` traps in both.
#[test]
fn float_intrinsics_agree_across_engines() {
    let src = "print(sqrt(16), sin(0), cos(0.0), atan2(1, 1), hypot(3, 4), exp(0), ln(1.0))\n\
print((-2.5).floor, (-2.5).ceil, (-2.5).round, (2.5).round, (-2.5).trunc)\n\
print(1e300.floor)\n";
    let c = compile(
        &Std::read(),
        src,
        &Roots::std(),
        true,
        None,
        Checks::CHECKED,
    )
    .expect("compiles");
    let (raw, engine) = (raw(&c), c.run(""));
    let want = "4.0000000000 0.0000000000 1.0000000000 0.7853981634 5.0000000000 \
1.0000000000 0.0000000000\n-3 -2 -3 3 -2\n";
    assert_eq!(raw.output, want);
    assert_eq!(engine.output, want);
    let fault = |o: &Outcome| o.trap.as_ref().map(|t| t.fault);
    assert_eq!(fault(&raw), Some(cpl_next_ir::ops::Fault::F64PastInt));
    assert_eq!(fault(&raw), fault(&engine));
}

/// std's CP `Show` for `F64` writes what C's `%.*f` writes (`ops::f64_text`), on both
/// engines: fractions, halves, values past `Int`, subnormals, zero and the infinity.
#[test]
fn f64_show_agrees_with_printf() {
    let src = "var x = 0.0\nvar step = 1.0\nvar big = 1.0 / 3.0\nvar small = 2.0 / 3.0\n\
for i in 0..120:\n    x = x * -1.37 + step\n    step = step * 0.71\n    big = big * 1024.0\n    \
small = small * 0.0009765625\n    print(x, precision: i % 18)\n    print(big, precision: i % 4)\n    \
print(small, precision: 100)\n";
    let (mut x, mut step, mut big, mut small) = (0.0f64, 1.0f64, 1.0f64 / 3.0, 2.0f64 / 3.0);
    let mut want = String::new();
    for i in 0..120 {
        x = x * -1.37 + step;
        step *= 0.71;
        big *= 1024.0;
        small *= 0.0009765625;
        for (v, d) in [(x, i % 18), (big, i % 4), (small, 100)] {
            want.push_str(&cpl_next_ir::ops::f64_text(v, d));
            want.push('\n');
        }
    }
    let c = compile(
        &Std::read(),
        src,
        &Roots::std(),
        true,
        None,
        Checks::CHECKED,
    )
    .expect("compiles");
    assert_eq!(raw(&c).output, want);
    assert_eq!(c.run("").output, want);
}

/// A struct's derived `Eq` and `Ord` declare every parameter, so one named by more than a
/// letter (`Elem`, std.tree's `Pre`) is no unknown type.
#[test]
fn derived_impls_declare_parameters_of_any_name() {
    let src = "struct Bx<Elem>:\n    x: Elem\nif Bx(x: 1) == Bx(x: 1) and Bx(x: 1) < Bx(x: 2):\n    print(1)\n";
    let c = compile(
        &Std::read(),
        src,
        &Roots::std(),
        true,
        None,
        Checks::CHECKED,
    )
    .expect("compiles");
    assert_eq!(raw(&c).output, "1\n");
    assert_eq!(c.run("").output, "1\n");
}

/// Std read once is a value no program changes: programs compiled one after another
/// against it each get what they get against a std of their own.
#[test]
fn programs_against_one_std_are_as_if_each_were_alone() {
    let programs = [
        include_str!("../tests/programs/n_bound_02_for_walks_through_the_bound.cp"),
        include_str!("../tests/programs/a_check_08_wrong_arg_count.cp"),
        include_str!("../tests/programs/n_assoc_type.cp"),
        "let xs = [3, 1, 2]\nprint(xs.sorted())\n",
        "let a = 1\nlet = 2\n",
    ];
    let std = Std::read();
    let shared: Vec<String> = programs.iter().map(|p| outcome(&std, p)).collect();
    for (p, got) in programs.iter().zip(&shared) {
        assert_eq!(got, &outcome(&Std::read(), p), "{p}");
    }
}

/// The checks optimized IR leaves out in the program's own code, by kind and line.
fn proved(src: &str) -> Vec<(cpl_next_ir::ir::CheckKind, u32)> {
    let c = compile(
        &Std::read(),
        src,
        &Roots::std(),
        false,
        None,
        Checks::CHECKED,
    )
    .expect("compiles");
    let main = c.prog.main;
    (c.proved().iter())
        .filter(|p| p.func == main)
        .map(|p| {
            (
                p.kind,
                c.line(c.prog.insts[p.func].origins[p.site as usize]),
            )
        })
        .collect()
}

/// A check is left out only where a fact proves it: `i` from `0..n` is a position of
/// `[0..n]` and `n - i` is in `1..=n`, but past the end of either it stays.
#[test]
fn facts_leave_out_only_the_checks_they_prove() {
    use cpl_next_ir::ir::CheckKind::{Key, RangeStore};
    let head = "let n: 1..=10 = read()\nvar walk: List<1..=n> = []\nvar nxt: [0..n]0..=n = 0\n";
    let at = |body: &str| proved(&format!("{head}{body}\n"));
    assert_eq!(
        at("for i in 0..n: walk.push(n - i)"),
        [
            (RangeStore, 3),
            (RangeStore, 3),
            (RangeStore, 4),
            (RangeStore, 4)
        ]
    );
    assert_eq!(
        at("for i in 0..=n: walk.push(n - i)"),
        [(RangeStore, 3), (RangeStore, 3), (RangeStore, 4)]
    );
    assert_eq!(
        at("for i in 0..n: nxt[i] = i + 1"),
        [
            (RangeStore, 3),
            (RangeStore, 3),
            (Key, 4),
            (Key, 4),
            (Key, 4),
            (RangeStore, 4),
            (RangeStore, 4)
        ]
    );
    assert_eq!(
        at("for i in 0..n: nxt[i + 1] = i"),
        [
            (RangeStore, 3),
            (RangeStore, 3),
            (Key, 4),
            (Key, 4),
            (RangeStore, 4),
            (RangeStore, 4)
        ]
    );
}

/// The prover gives up after its steps run out, and giving up only keeps a check: `r = a` needs
/// `a >= 0`, and asserts on `width` other values tried first, `width` wide and 5 deep, leave
/// that fact last. Narrow, both ends of the store's check go; wide, `a <= 100` still goes but
/// `a >= 0` stays, and the program still compiles and runs the same.
#[test]
fn a_proof_past_the_provers_steps_keeps_its_check() {
    use cpl_next_ir::ir::CheckKind::RangeStore;
    let program = |width: usize| {
        let names: Vec<Vec<String>> = (0..5)
            .map(|d| (0..width).map(|k| format!("x{d}_{k}")).collect())
            .collect();
        let mut src = String::from("let a: Int = read()\n");
        for n in names.iter().flatten() {
            src += &format!("let {n}: Int = read()\n");
        }
        for b in &names[0] {
            src += &format!("assert a >= {b}\n");
        }
        for w in names.windows(2) {
            for (hi, lo) in w[0].iter().flat_map(|h| w[1].iter().map(move |l| (h, l))) {
                src += &format!("assert {hi} >= {lo}\n");
            }
        }
        src += "assert a >= 0\nassert a <= 100\nvar r: 0..=100 = 0\nr = a\nprint(r)\n";
        let line = src.lines().count() as u32 - 1;
        (src, line, 1 + 5 * width)
    };
    for (width, dropped) in [(1, 2), (8, 1)] {
        let (src, line, reads) = program(width);
        let at_store = (proved(&src).iter())
            .filter(|p| **p == (RangeStore, line))
            .count();
        assert_eq!(at_store, dropped, "width {width}");
        let c = compile(
            &Std::read(),
            &src,
            &Roots::std(),
            false,
            None,
            Checks::CHECKED,
        )
        .expect("compiles");
        let input = vec!["7"; reads].join(" ");
        assert_eq!(c.run(&input).output, "7\n", "width {width}");
    }
}

/// A diagnostic's span is in the user's file, past the modules placed before it.
#[test]
fn diagnostic_is_placed_in_the_program() {
    let src = "let a = 1\nlet b = 2\nprint(a +  zz)\n";
    let d = &diags(src)[0];
    let at = d.at.as_ref().expect("placed");
    assert_eq!((at.file.as_str(), at.line, at.col), (PROGRAM, 3, 12));
    assert_eq!(&src[at.span.start as usize..at.span.end as usize], "zz");
    assert_eq!(
        d.to_string(),
        "check: program:3:12: E0201: unknown name `zz`"
    );
}

/// A fn refused as declared is that one refusal: its body and its calls say nothing more.
#[test]
fn a_refused_fn_is_one_refusal() {
    let src = "@memo(map)\nfn f(n: Int) -> Int = n + nope\nprint(f(3) + 1, f(true).len)\n";
    let ds: Vec<String> = diags(src).iter().map(|d| d.to_string()).collect();
    assert_eq!(ds, ["decls: program:1:1: slice: decorator `@memo`"]);
}

/// A diagnostic is data: its code, its place in its file and, for a refusal, the feature.
#[test]
fn a_diagnostic_is_its_code_place_and_feature() {
    let json = |src: &str| (diags(src).iter().map(|d| d.json())).collect::<Vec<_>>();
    let verdict = &json("print(1)\nlet x: Int = \"a\"\n")[0];
    for part in [
        r#""code":"E0302""#,
        r#""file":"program""#,
        r#""line":2"#,
        r#""refusal":null"#,
    ] {
        assert!(verdict.contains(part), "{part} in {verdict}");
    }
    let refusal = &json("let x = 1.5\nprint(x % 2.0)\n")[0];
    for part in [
        r#""code":"E0501""#,
        r#""refusal":"FloatOperator""#,
        r#""text":"check: program:2:7: slice: "#,
    ] {
        assert!(refusal.contains(part), "{part} in {refusal}");
    }
}

/// A refusal under a written type, on a `let` or a lambda's parameter, prints its feature
/// text: a refusal re-reported by its text alone was an empty E9999 (core-01/mod-pow).
#[test]
fn a_refusal_prints_its_feature_text() {
    let src = "type Z = Mod<1000000007>\nlet x: Z = 3\nlet f = (y: Z) => y\nprint(x)\n";
    let ds: Vec<String> = diags(src).iter().map(|d| d.to_string()).collect();
    assert!(ds.iter().all(|d| !d.contains("E9999")), "{ds:?}");
    let typed = |line: u32| format!("check: program:{line}:");
    for line in [2, 3] {
        let at = ds.iter().find(|d| d.starts_with(&typed(line)));
        assert!(
            at.is_some_and(|d| d.contains("slice: type Value")),
            "line {line}: {ds:?}"
        );
    }
}

/// A `[libs]` module fills roles as std does: its second `@role(cuts)` struct, beside core's
/// `Cut`, is a duplicate (R06.4.12).
#[test]
fn a_library_second_marked_declaration_is_a_duplicate() {
    let dir = std::env::temp_dir().join(format!("cpl-marker-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("kit")).unwrap();
    std::fs::write(dir.join("cp.toml"), "[libs]\nkit = \"kit\"\n").unwrap();
    let lib = "@role(cuts)\npub struct MyCut:\n    k: Int\npub fn one() -> Int = 1\n";
    std::fs::write(dir.join("kit/m.cp"), lib).unwrap();
    let src = "use kit.m.{one}\nprint(one())\n";
    let got = compile(
        &Std::read(),
        src,
        &Roots::from(&dir),
        false,
        None,
        Checks::CHECKED,
    );
    std::fs::remove_dir_all(&dir).unwrap();
    let ds: Vec<String> = got
        .err()
        .expect("rejected")
        .iter()
        .map(|d| d.to_string())
        .collect();
    assert!(
        ds.iter()
            .any(|d| d.contains("E0202: a second declaration marked `@role(cuts)`")),
        "{ds:?}"
    );
}

/// A library's misdeclarations are a program's errors, said at once: two fns of one name and
/// one label set (R06.1.6), and a `@get` fn with a parameter that has no default (R02.6.6).
#[test]
fn a_library_misdeclaration_is_said_as_a_programs() {
    let dir = std::env::temp_dir().join(format!("cpl-misdeclared-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("kit")).unwrap();
    std::fs::write(dir.join("cp.toml"), "[libs]\nkit = \"kit\"\n").unwrap();
    let cases = [
        (
            "pub fn f(x: Int) -> Int = x\npub fn f(x: Str) -> Int = 0\npub fn one() -> Int = 1\n",
            "E0702",
        ),
        (
            "pub struct T(pub k: Int):\n    @get pub fn at(self, i: Int) -> Int = self.k + i\npub fn one() -> Int = 1\n",
            "E0317",
        ),
    ];
    for (lib, code) in cases {
        std::fs::write(dir.join("kit/m.cp"), lib).unwrap();
        let got = compile(
            &Std::read(),
            "use kit.m.{one}\nprint(one())\n",
            &Roots::from(&dir),
            false,
            None,
            Checks::CHECKED,
        );
        let ds: Vec<String> = (got.err().expect("rejected").iter())
            .map(|d| d.to_string())
            .collect();
        assert!(ds.iter().any(|d| d.contains(code)), "{code}: {ds:?}");
    }
    std::fs::remove_dir_all(&dir).unwrap();
}

/// A library fn reached only through a trait, whose body has a type error, lowers that part to
/// a trap; a build that reaches it reports the error and is not clean, and one that does not
/// reach it runs.
#[test]
fn a_build_reaching_an_untyped_library_body_reports_it() {
    let dir = std::env::temp_dir().join(format!("cpl-untyped-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("kit")).unwrap();
    std::fs::write(dir.join("cp.toml"), "[libs]\nkit = \"kit\"\n").unwrap();
    let lib = "pub struct W:\n    pub v: Int\n\nimpl Show for W:\n    fn write_to(self, mut out: Str, sep: ?Str):\n        out.append(nope)\n";
    std::fs::write(dir.join("kit/m.cp"), lib).unwrap();
    let run = |src: &str| {
        compile(
            &Std::read(),
            src,
            &Roots::from(&dir),
            false,
            None,
            Checks::CHECKED,
        )
    };
    let reached = run("use kit.m.{W}\nprint(W(1))\n");
    let unreached = run("use kit.m.{W}\nprint(W(1).v)\n");
    std::fs::remove_dir_all(&dir).unwrap();
    let ds: Vec<String> = (reached.err().expect("rejected").iter())
        .map(|d| d.to_string())
        .collect();
    assert!(
        ds.iter().any(|d| d.contains("unknown name `nope`")),
        "{ds:?}"
    );
    let c = unreached.expect("an untyped body it never reaches leaves it clean");
    assert_eq!(c.run("").output, "1\n");
}

/// A `let .. else` of an error binds its name to the error: its uses say nothing more.
#[test]
fn a_let_else_of_an_error_binds_the_error() {
    let src = "let order = nope(3) else:\n    return\nprint(order[0], order.len)\n";
    let ds: Vec<String> = diags(src).iter().map(|d| d.to_string()).collect();
    assert_eq!(ds, ["check: program:1:13: E0201: unknown name `nope`"]);
}

/// A `let` of a refused value leaves its name the error, so its uses say nothing more.
#[test]
fn a_refused_value_leaves_its_name_the_error() {
    let only_refusals = |src: &str| {
        let ds = diags(src);
        assert!(
            ds.iter().all(Diag::is_refusal),
            "{:?}",
            ds.iter().map(|d| d.to_string()).collect::<Vec<_>>()
        );
    };
    only_refusals("struct P<T>:\n    a: T\nlet v = P<Int>\nlet q = v + 1\nprint(q.nope)\n");
}

/// `None` beside an operand that is the error is the error, and so is the comparison.
#[test]
fn none_beside_an_error_says_nothing_more() {
    let src = "print(nope(3) == None)\n";
    let ds: Vec<String> = diags(src).iter().map(|d| d.to_string()).collect();
    assert_eq!(ds, ["check: program:1:7: E0201: unknown name `nope`"]);
}

/// A missing impl is named at its root, in the spelling the program writes: `print(xs)` on a
/// `List<?Int>` is `?Int` lacking `Show`, not the pack of arguments lacking `ShowAll` (R10.3.7).
#[test]
fn a_missing_impl_names_the_part_that_lacks_it() {
    for (src, at) in [
        ("let xs: List<?Int> = [None]\nprint(xs)\n", "2:1"),
        ("let x: ?Int = None\nprint(1, x, sep: \",\")\n", "2:1"),
        (
            "let xs: List<List<?Int>> = [[None]]\nprint(\"{xs}\")\n",
            "2:7",
        ),
    ] {
        let ds: Vec<String> = diags(src).iter().map(|d| d.to_string()).collect();
        assert_eq!(
            ds,
            [format!(
                "check: program:{at}: E0606: `?Int` does not implement `Show`"
            )]
        );
    }
}

/// A refused local item shadows the name outside (std.geometry's `Pt`): its uses are the error.
#[test]
fn a_refused_local_item_shadows_and_says_nothing_more() {
    let src = "use std.geometry.{Pt}\nfn f() -> Int:\n    struct Pt: v: Int, w: Int\n    let p = Pt(1, 4)\n    p.w\nprint(f())\n";
    let ds = diags(src);
    assert!(
        ds.len() == 1 && ds[0].is_refusal(),
        "{:?}",
        ds.iter().map(|d| d.to_string()).collect::<Vec<_>>()
    );
}

/// Type arguments on an unknown name report the unknown name, not a refusal.
#[test]
fn type_arguments_on_an_unknown_name_report_the_name() {
    let src = "fn f<K>() -> Int:\n    var m = HashMop<K, Int>()\n    0\nprint(f<Int>())\n";
    let ds: Vec<String> = diags(src).iter().map(|d| d.to_string()).collect();
    assert_eq!(ds, ["check: program:2:13: E0201: unknown name `HashMop`"]);
}

/// A refused domain is the error: its uses give no verdict of their own.
#[test]
fn uses_of_a_refused_domain_say_nothing_more() {
    let src = "domain R = 0..2\ndomain P = R | R\nlet q = P(1)\nprint(P.count + q)\n";
    let verdicts: Vec<String> = (diags(src).iter())
        .map(|d| d.to_string())
        .filter(|d| !d.contains("slice:"))
        .collect();
    assert_eq!(verdicts, Vec::<String>::new());
}

/// Assigning, or lending `mut`, a place that is the error says nothing more.
#[test]
fn writing_an_error_place_says_nothing_more() {
    let only_refusals = |src: &str| {
        let ds = diags(src);
        assert!(
            ds.len() == 1 && ds[0].is_refusal(),
            "{:?}",
            ds.iter().map(|d| d.to_string()).collect::<Vec<_>>()
        );
    };
    only_refusals("struct P<T>:\n    a: T\nvar p = P<Int>\np += 10\nprint(p)\n");
    let lent: Vec<String> = (diags("var b = [3, 1, 2]\nreverse(mut nope(b))\nprint(b[0])\n"))
        .iter()
        .map(|d| d.to_string())
        .filter(|d| d.starts_with("check: program"))
        .collect();
    assert_eq!(lent, ["check: program:2:13: E0201: unknown name `nope`"]);
}

/// A bound test that fails to check leaves the branch it narrows unchecked: it relies on
/// the bound.
#[test]
fn a_bound_test_in_error_says_nothing_of_its_branch() {
    let src = "trait Ch:\n    fn ch(self) -> Int\nfn f<M>(m: M) -> Int:\n    if M: Nope:\n        return m.ch()\n    0\nprint(f(3))\n";
    let ds: Vec<String> = diags(src).iter().map(|d| d.to_string()).collect();
    assert_eq!(ds, ["check: program:4:11: E0204: unknown trait `Nope`"]);
}

/// `in` with an operand that is the error is the error, as `binary` is.
#[test]
fn in_beside_an_error_says_nothing_more() {
    let ds: Vec<String> = (diags("let b = nope(3) in [1, 2]\nprint(b)\n").iter())
        .map(|d| d.to_string())
        .collect();
    assert_eq!(ds, ["check: program:1:9: E0201: unknown name `nope`"]);
}

/// A lowered fn's locals keep their source names, so what reads the IR can name them.
#[test]
fn a_lowered_local_keeps_its_name() {
    let src =
        "fn f(xs: List<Int>) -> Int:\n    let total = xs[0] + xs[1]\n    total\nprint(f([1, 2]))\n";
    let f = front(&Std::read(), src, &Roots::std(), None, Checks::CHECKED);
    assert!(f.diags.is_empty(), "{:?}", f.diags);
    let g = (f.g.funcs.values())
        .find(|g| g.name == "f" && g.line != 0)
        .expect("f is lowered");
    assert_eq!(g.names.len(), g.locals.len());
    for n in ["xs", "total"] {
        assert!(g.names.iter().any(|m| m == n), "{n} in {:?}", g.names);
    }
}
/// A lowered expression keeps the node it came from, so what reads the IR can place it.
#[test]
fn a_lowered_expr_keeps_its_site() {
    let src = "fn f(xs: List<Int>) -> Int:\n    xs[0] + xs[1]\nprint(f([1, 2]))\n";
    let f = front(&Std::read(), src, &Roots::std(), None, Checks::CHECKED);
    assert!(f.diags.is_empty(), "{:?}", f.diags);
    let g = (f.g.funcs.values())
        .find(|g| g.name == "f" && g.line != 0)
        .expect("f is lowered");
    let ret = (g.body.iter())
        .find_map(|s| match s {
            ir::Stmt::Return(Some(e)) => Some(e),
            _ => None,
        })
        .expect("f returns");
    let at = &g.sites[ret.site as usize];
    assert_ne!(ret.site, 0);
    assert_eq!(
        (at.item.name.as_ref(), at.span.end - at.span.start),
        ("f", "xs[0] + xs[1]".len() as u32)
    );
}
/// A lowered loop keeps the node of its body, where a message on the whole loop is placed.
#[test]
fn a_lowered_loop_keeps_its_body_site() {
    let src = "fn f(n: Int) -> Int:\n    var i = 0\n    while i < n:\n        i += 1\n    i\nprint(f(3))\n";
    let f = front(&Std::read(), src, &Roots::std(), None, Checks::CHECKED);
    assert!(f.diags.is_empty(), "{:?}", f.diags);
    let g = (f.g.funcs.values())
        .find(|g| g.name == "f" && g.line != 0)
        .expect("f is lowered");
    let site = (g.body.iter())
        .find_map(|s| match s {
            ir::Stmt::Loop(_, _, s) => Some(*s),
            _ => None,
        })
        .expect("f loops");
    assert_ne!(site, 0);
    assert_eq!(g.sites[site as usize].item.name.as_ref(), "f");
}
/// A lowered walk keeps the node of its body, where a message on the whole walk is placed.
#[test]
fn a_lowered_walk_keeps_its_body_site() {
    let src = "fn f(xs: List<Int>) -> Int:\n    var s = 0\n    for x in xs:\n        s += x\n    s\nprint(f([1, 2]))\n";
    let f = front(&Std::read(), src, &Roots::std(), None, Checks::CHECKED);
    assert!(f.diags.is_empty(), "{:?}", f.diags);
    let g = (f.g.funcs.values())
        .find(|g| g.name == "f" && g.line != 0)
        .expect("f is lowered");
    let (site, body) = (g.body.iter())
        .find_map(|s| match s {
            ir::Stmt::Region {
                site, body_site, ..
            } => Some((*site, *body_site)),
            _ => None,
        })
        .expect("f walks xs");
    assert!(body != 0 && body != site);
    assert_eq!(g.sites[body as usize].item.name.as_ref(), "f");
}
/// A cell of a type parameter is written through its `IndexedMut` bound, and the key's
/// domain comes through the bound's supertraits.
#[test]
fn a_cell_of_a_type_parameter_is_written_through_its_bound() {
    let src = "fn w<S: IndexedMut<D, T>, D: Domain, T>(mut s: S, i: Int, x: T):\n    s[s.keys.nth(i)] = x\nvar q = [1, 2, 3]\nw(mut q, 1, 9)\nprint(q[1])\n";
    let f = front(&Std::read(), src, &Roots::std(), None, Checks::CHECKED);
    assert!(f.diags.is_empty(), "{:?}", f.diags);
}
#[test]
fn syntax_error_is_placed_in_the_program() {
    let d = &diags("let a = 1\nlet = 2\n")[0];
    let at = d.at.as_ref().expect("placed");
    assert_eq!(
        (d.stage, at.file.as_str(), at.line),
        (Stage::Syntax, PROGRAM, 2)
    );
    assert!(d.code().is_some(), "{d}");
}

/// A tuple pattern reads a record's fields in order (R02.5.5).
#[test]
fn a_tuple_pattern_reads_a_record() {
    let src = "fn f(x: Int) -> (root: Int, size: Int) = (root: x, size: x * 2)\n\
let (r, s) = f(3)\n\
var t = 0\n\
for (a, b) in [f(1), f(2)]: t += a * b\n\
print(r, s, t)\n";
    let c = compile(
        &Std::read(),
        src,
        &Roots::std(),
        true,
        None,
        Checks::CHECKED,
    )
    .expect("compiles");
    assert_eq!(raw(&c).output, "3 6 10\n");
    assert_eq!(c.run("").output, "3 6 10\n");
}

/// `trait HasW: w: Int` is a field requirement, the same as a bodiless `@get fn w(self) -> Int`:
/// a bound gives `x.w`, and a record or an impl'd struct meets it (R08.1.7).
#[test]
fn a_trait_field_is_a_field_requirement() {
    let src = "trait HasW: w: Int\n\
fn weight<T: HasW>(x: T) -> Int = x.w\n\
struct Road: w: Int, len: Int\n\
impl HasW for Road\n\
print(weight((w: 3, z: 1)), weight(Road(4, 9)))\n";
    let c = compile(
        &Std::read(),
        src,
        &Roots::std(),
        true,
        None,
        Checks::CHECKED,
    )
    .expect("compiles");
    assert_eq!(raw(&c).output, "3 4\n");
    assert_eq!(c.run("").output, "3 4\n");
}

/// `x == None` tests presence, as the pattern `None` does, so `T` needs no `Eq` (R02.4.5).
#[test]
fn comparing_with_none_needs_no_eq() {
    let src = "struct F(f: Int -> Int)\n\
let a: ?F = None\n\
let b: ?F = Some(F(x => x + 1))\n\
if a == None and b != None and not (None == b):\n    print(b!.f(1))\n";
    let c = compile(
        &Std::read(),
        src,
        &Roots::std(),
        true,
        None,
        Checks::CHECKED,
    )
    .expect("compiles");
    assert_eq!(raw(&c).output, "2\n");
    assert_eq!(c.run("").output, "2\n");
}

/// Types told apart only by which domain a position counts share one representation, which
/// link decides once: `S<P>` and `S<Q>` map to the same type for every backend.
/// A list of a ranged `Int` and a list of `Int` lie in memory alike, so they share one
/// representation: C++ names one struct for both and passes one where the other is wanted.
#[test]
fn a_ranged_int_shares_the_representation_of_an_int() {
    let src =
        "var x: 0..=10 = 5\nvar xs: List<0..=10> = List.new()\nxs.push(x)\nvar ys: List<Int> = List.new()\nys.push(x)\nprint(xs.get(0) + ys.get(0))\n";
    let c = compile(
        &Std::read(),
        src,
        &Roots::std(),
        false,
        None,
        Checks::CHECKED,
    )
    .expect("compiles");
    let list = |t: &CTy, ranged: bool| matches!(t, CTy::Struct(_, ts, _) if ts.len() == 1 && matches!(ts[0], CTy::Range(..)) == ranged && matches!(ts[0], CTy::Range(..) | CTy::Int));
    let all: Vec<&CTy> = (c.prog.insts.iter())
        .flat_map(|f| f.locals.iter().chain([&f.ret]))
        .collect();
    let ranged = all.iter().find(|t| list(t, true)).expect("a List<0..=10>");
    let plain = all
        .iter()
        .find(|t| list(t, false) && c.prog.rep(t) == c.prog.rep(ranged));
    assert!(
        plain.is_some(),
        "no List<Int> shares {ranged:?}: {:?}",
        c.prog.shares
    );
}

#[test]
fn types_told_apart_by_domains_share_a_representation() {
    let src = "struct S<P>:\n    at: [P]Int\nfn a(k: Int) -> S<_>:\n    domain P = 0..k\n    S<P>([P] _ => 1)\nfn b(k: Int) -> S<_>:\n    domain Q = 0..k\n    S<Q>([Q] _ => 2)\nlet s = a(3)\nlet t = b(2)\nprint(s.at.len, t.at.len)\n";
    let c = compile(
        &Std::read(),
        src,
        &Roots::std(),
        false,
        None,
        Checks::CHECKED,
    )
    .expect("compiles");
    let doms =
        |t: &CTy| matches!(t, CTy::Struct(_, ts, _) if ts.iter().any(|t| matches!(t, CTy::Dom(_))));
    let ss: Vec<&CTy> = (c.prog.insts.iter())
        .find(|f| f.name == "a")
        .into_iter()
        .chain(c.prog.insts.iter().filter(|f| f.name == "b"))
        .map(|f| &f.ret)
        .filter(|t| doms(t))
        .collect();
    assert_eq!(
        ss.len(),
        2,
        "a and b return S over their own domains: {ss:?}"
    );
    assert_ne!(ss[0], ss[1]);
    assert_eq!(c.prog.rep(ss[0]), c.prog.rep(ss[1]), "{:?}", c.prog.shares);
}

/// A body an edit leaves alone checks to the same typed body, with the lambdas and set calls
/// it makes keeping the ids the earlier check gave them (`check`'s `prior`). The edit changes
/// the body's length; spans count from each item's start, and a blank line parts items.
#[test]
fn an_edit_to_one_body_leaves_the_others_typed_the_same() {
    let std = Std::read();
    let typed = |src: &str, prior: &check::Ids| {
        let loaded = modules::load(&std, src, &Roots::std());
        let (d, main) = decls::collect(std.decls(), &loaded.modules, &loaded.seen, &loaded.prelude);
        let values = consts::Evaluator::new(&d, &loaded, &main);
        let t = check::check(
            &d,
            &main,
            &loaded,
            &values,
            &[],
            Build {
                judge: None,
                checks: Checks::CHECKED,
            },
            prior,
        );
        assert!(t.errors.is_empty(), "{:?}", t.errors);
        let bodies: std::collections::HashMap<String, String> = (t.fns.iter())
            .filter(|(id, _)| {
                ["apply_twice", "left", "edited", "right"].contains(&&*d.f(**id).name)
            })
            .map(|(id, f)| (d.f(*id).name.clone(), format!("{:?}", (&f.body, &f.ret))))
            .collect();
        (bodies, t.ids)
    };
    let program = |b: &str| {
        format!(
            "fn apply_twice(f: (Int) -> Int, x: Int) -> Int = f(f(x))\n\nfn left(n: Int) -> Int = apply_twice((y) => y + n, 1)\n\nfn edited(n: Int) -> Int:\n    let g = (y: Int) => y + 1\n{b}\n\nfn right(n: Int) -> Int = apply_twice((y) => y * n, 2)\n\nprint(left(1), edited(2), right(3))\n"
        )
    };
    let after = program("    g(n) * apply_twice((z) => z, 2) + 100");
    let (was, ids) = typed(&program("    g(n) * 2"), &check::Ids::default());
    let (now, _) = typed(&after, &ids);
    for f in ["apply_twice", "left", "right"] {
        assert_eq!(was[f], now[f], "{f}");
    }
    assert_ne!(was["edited"], now["edited"]);
    // Numbered afresh, the edit's new lambda and call move the others' ids.
    let (fresh, _) = typed(&after, &check::Ids::default());
    assert!(["apply_twice", "left", "right"]
        .iter()
        .any(|f| was[*f] != fresh[*f]));
}

/// A body that puts a function in a global's set leaves a body that reads the global typed the
/// same, whichever is checked first: the global's set is its own (`SetTy::Global`), and link
/// closes it from what the program puts in it.
#[test]
fn a_body_adding_to_a_global_set_leaves_its_readers_typed_the_same() {
    let std = Std::read();
    let program = |bump: &str| {
        format!(
            "var h = (x: Int) => x\nfn top(n: Int) -> Int = h(n)\n\nfn bump(a: Int, b: Int) -> Int:\n{bump}    max(a, b)\n\nlet g = (a: Int, b: Int) => bump(a, b)\nprint(g(1, 2), top(5))\n"
        )
    };
    let top = |src: &str| {
        let loaded = modules::load(&std, src, &Roots::std());
        let (d, main) = decls::collect(std.decls(), &loaded.modules, &loaded.seen, &loaded.prelude);
        let values = consts::Evaluator::new(&d, &loaded, &main);
        let t = check::check(
            &d,
            &main,
            &loaded,
            &values,
            &[],
            Build {
                judge: None,
                checks: Checks::CHECKED,
            },
            &check::Ids::default(),
        );
        let ours = |m: &&cpl_next_decls::diag::Msg| {
            m.at.as_ref()
                .is_some_and(|a| &*a.item.module == modules::PROGRAM)
        };
        let ours: Vec<_> = t.errors.iter().filter(ours).collect();
        assert!(ours.is_empty(), "{ours:?}");
        (t.fns.iter())
            .find(|(id, _)| d.f(**id).name == "top")
            .map(|(_, f)| format!("{:?}", (&f.body, &f.ret)))
    };
    let (adds, reads) = (program("    h = (x: Int) => x + 1\n"), program(""));
    assert!(top(&adds).is_some());
    assert_eq!(top(&adds), top(&reads));
    let printed = |src: &str| {
        let c = compile(&std, src, &Roots::std(), true, None, Checks::CHECKED).expect("compiles");
        (c.run("").output, raw(&c).output)
    };
    assert_eq!(printed(&adds), ("2 6\n".into(), "2 6\n".into()));
    assert_eq!(printed(&reads), ("2 5\n".into(), "2 5\n".into()));
}

/// A session answers from its snapshot alone: a changed program is a new snapshot sharing std,
/// and the old session's answers stay what they were.
#[test]
fn a_session_answers_from_its_snapshot() {
    use crate::facade::{Layer, Session, Snapshot, Verdict};
    let ok = Snapshot::new(crate::facade::World::Closed).with_file("program.cp", "print(1 + 2)\n");
    let bad = ok.with_file("program.cp", "print(nope)\n");
    let (ok, bad) = (Session::open(ok), Session::open(bad));
    assert_eq!(ok.verdict("program.cp"), Verdict::Accepted);
    assert_eq!(
        ok.program("program.cp").map(|p| p.run("").output).ok(),
        Some("3\n".into())
    );
    assert_eq!(bad.verdict("program.cp"), Verdict::Rejected);
    let said: Vec<_> = (bad.diagnostics("program.cp", Layer::Typed).iter())
        .map(|d| d.code())
        .collect();
    assert_eq!(said, [Some("E0201")]);
    assert_eq!(ok.verdict("program.cp"), Verdict::Accepted);
}

#[test]
fn hover_and_definition_read_the_scope_and_signatures() {
    use crate::facade::{Session, Snapshot};
    use cpl_contract::Typed;
    let src = "/// Doubles `n`.\n///\n/// cost: O(1)\nfn twice(n: Int) -> Int:\n    n * 2\n\nprint(twice(3))\n";
    let s = Session::open(Snapshot::new(crate::facade::World::Closed).with_file("program.cp", src));
    let call = src.rfind("twice").unwrap() as u32 + 2;
    let h = s.hover("program.cp", call).unwrap();
    assert_eq!(h.text, "fn twice(n: Int) -> Int");
    assert_eq!(&src[h.span.start as usize..h.span.end as usize], "twice");
    assert!(
        h.doc.as_deref().is_some_and(|d| d.contains("Doubles")),
        "{:?}",
        h.doc
    );
    assert_eq!(h.cost.as_deref(), Some("O(1)"));
    assert_eq!((h.typed, h.signature), (Some(Typed::Fn), true));
    let d = s.definition("program.cp", call).unwrap();
    assert_eq!(d.path, "program.cp");
    assert_eq!(&src[d.span.start as usize..][..2], "fn");
    let local = src.find("n * 2").unwrap() as u32;
    assert!(s.hover("program.cp", local).is_none());
    let print = s
        .definition("program.cp", src.find("print").unwrap() as u32)
        .unwrap();
    assert_eq!(print.path, "std/show.cp");
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let text = std::fs::read_to_string(root.join(&print.path)).unwrap();
    let declared = &text[print.span.start as usize..print.span.end as usize];
    assert!(declared.contains("pub fn print<"), "{declared}");
}

/// Calls `f` with each call to an instance in `b`: the callee and its arguments.
fn each_inst_call(b: &[Stmt<CTy>], f: &mut dyn FnMut(usize, &[cpl_next_ir::ir::Arg<CTy>])) {
    use cpl_next_ir::ir::{Arg, Expr, Target, EK};
    fn expr(e: &Expr<CTy>, f: &mut dyn FnMut(usize, &[Arg<CTy>])) {
        let args = |xs: &[Arg<CTy>], f: &mut dyn FnMut(usize, &[Arg<CTy>])| {
            for a in xs {
                if let Arg::Value(x) = a {
                    expr(x, f)
                }
            }
        };
        match &e.kind {
            EK::Call(t, xs, _) => {
                if let Target::Inst(g) = t {
                    f(*g, xs)
                }
                args(xs, f)
            }
            EK::Intr(_, xs, _) => args(xs, f),
            EK::CallValue(g, xs, _) => {
                expr(g, f);
                args(xs, f)
            }
            EK::MakeStruct(xs) | EK::MakeClosure(xs) => xs.iter().for_each(|x| expr(x, f)),
            EK::Inject(x) | EK::Widen(x) | EK::MakeUnion(_, x) => expr(x, f),
            _ => {}
        }
    }
    for s in b {
        match s {
            Stmt::Let(_, e) | Stmt::Assign(_, e) | Stmt::Expr(e) | Stmt::Yield(e) => expr(e, f),
            Stmt::Fact(e) | Stmt::Check { cond: e, .. } | Stmt::Return(Some(e)) => expr(e, f),
            Stmt::If(c, x, y) => {
                expr(c, f);
                each_inst_call(x, f);
                each_inst_call(y, f);
            }
            Stmt::Loop(x, y, _) => {
                each_inst_call(x, f);
                each_inst_call(y, f);
            }
            Stmt::Switch(c, arms) => {
                expr(c, f);
                arms.iter().for_each(|a| each_inst_call(a, f));
            }
            Stmt::Region {
                args, body, target, ..
            } => {
                if let cpl_next_ir::ir::Target::Inst(g) = target {
                    f(*g, args)
                }
                each_inst_call(body, f)
            }
            Stmt::Event(cpl_next_ir::ir::EventPoint { vals: xs, .. }) => {
                xs.iter().for_each(|x| expr(x, f))
            }
            _ => {}
        }
    }
}

/// An `Arg::View` goes only to a param its callee views (contract 07): a place viewed at the
/// call but given to a param the callee owns is copied in. `print` of several values recurses
/// through `Pack.write_all`, whose `self` a viewed `Pack.write_to` owns no longer.
#[test]
fn a_view_goes_only_to_a_viewed_param() {
    use cpl_next_ir::ir::Arg;
    let src = "struct P:\n    a: Int\n    b: Int = 2\nprint(P(a: 1).b, 3, 4)\n";
    let c = compile(
        &Std::read(),
        src,
        &Roots::std(),
        false,
        None,
        Checks::CHECKED,
    )
    .expect("compiles");
    for fs in [&c.prog.insts, &c.canon, &c.opt] {
        let mut bad = Vec::new();
        for f in fs.iter() {
            each_inst_call(&f.body, &mut |g, xs| {
                for (k, a) in xs.iter().enumerate() {
                    if matches!(a, Arg::View(_)) && fs[g].views.get(k) != Some(&true) {
                        bad.push(format!("{} param {k}", fs[g].name));
                    }
                }
            });
        }
        assert!(bad.is_empty(), "views given to owned params: {bad:?}");
    }
}

/// A default lambda read in a generic caller is an instance per the caller's type arguments
/// (`by: T -> K = x => x` of `sort`), not one of an error type that sorts nothing.
#[test]
fn a_default_lambda_in_a_generic_fn_has_the_callers_types() {
    let src =
        "fn f<T: Ord>(mut xs: List<T>):\n    xs.sort()\nvar a = [3, 1, 2]\nf(mut a)\nprint(a)\n";
    let c = compile(
        &Std::read(),
        src,
        &Roots::std(),
        false,
        None,
        Checks::CHECKED,
    )
    .expect("compiles");
    assert_eq!(c.error_typed(), Vec::<String>::new());
    assert_eq!(c.run("").output, "1 2 3\n");
}

/// An operator's trait argument takes the marks of the argument that fills it, so `small +
/// large.rev` calls the `Add<Rev<lent List>>` instance; the verifier names an argument whose
/// type is not its param's.
#[test]
fn a_trait_argument_takes_the_marks_of_its_argument() {
    let src = "let small = [1, 2]\nlet large = [4, 3]\nprint(small + large.rev)\n";
    let c = compile(
        &Std::read(),
        src,
        &Roots::std(),
        false,
        None,
        Checks::CHECKED,
    )
    .expect("compiles");
    let g = c.prog.globals.len();
    let bad = cpl_next_ir::verify::instances(&c.prog.insts, cpl_next_ir::verify::Stage::Raw, g);
    assert!(bad.is_empty(), "{bad:?}");
    assert_eq!(c.run("").output, "1 2 3 4\n");
}

#[test]
fn type_at_reads_the_typed_tree() {
    use crate::facade::{Session, Snapshot};
    let src = "fn twice(n: Int) -> Int:\n    n * 2\n\nlet b = 3 < 4\nlet c = twice(3)\n";
    let s = Session::open(Snapshot::new(crate::facade::World::Closed).with_file("program.cp", src));
    assert!(s.program("program.cp").is_ok());
    let at = |text: &str| {
        s.type_at("program.cp", src.find(text).unwrap() as u32)
            .map(|t| t.text())
    };
    assert_eq!(at("n * 2").as_deref(), Some("Int"));
    assert_eq!(at("< 4").as_deref(), Some("Bool"));
    assert_eq!(at("twice(3)").as_deref(), Some("Int"));
    assert!(s.type_at("program.cp", src.len() as u32).is_none());
    let int = s
        .type_at("program.cp", src.find("twice(3)").unwrap() as u32)
        .unwrap();
    assert_eq!(
        (int.range(), int.decl(), int.args().len()),
        (Some((None, None)), None, 0)
    );
    let whole = cpl_diag::Span::new(0, src.len() as u32);
    let outermost: Vec<String> = (s.types_in("program.cp", whole).iter())
        .filter(|(span, _)| src[span.start as usize..].starts_with("3 < 4"))
        .map(|(_, t)| t.text())
        .collect();
    assert_eq!(outermost, ["Bool", "Int"]);
    let src = "struct P:\n    a: Int\nlet p = P(a: 1)\n";
    let s = Session::open(Snapshot::new(crate::facade::World::Closed).with_file("program.cp", src));
    let p = s
        .type_at("program.cp", src.find("P(a").unwrap() as u32)
        .unwrap();
    let decl = p.decl().unwrap();
    assert_eq!(
        (p.text(), decl.line, decl.head.as_str()),
        ("P".into(), 1, "struct P")
    );
    let members: Vec<String> = p.members().into_iter().map(|(n, _)| n).collect();
    assert!(
        members.contains(&"a".into()) && members.contains(&"eq".into()),
        "{members:?}"
    );
    assert!(p.implements("Eq") && !p.implements("NoSuchTrait"));
}

#[test]
fn complete_lists_the_scope_names_a_prefix_starts() {
    use crate::facade::{Session, Snapshot};
    let src =
        "fn twice(n: Int) -> Int:\n    n * 2\n\nfn triple(n: Int) -> Int:\n    n * 3\n\nlet c = twi";
    let s = Session::open(Snapshot::new(crate::facade::World::Closed).with_file("program.cp", src));
    let names = |at: usize| -> Vec<String> {
        s.complete("program.cp", at as u32)
            .iter()
            .map(|(n, _)| n.to_string())
            .collect()
    };
    assert_eq!(names(src.len()), ["twice"]);
    let t = names(src.len() - 2);
    assert!(
        t.contains(&"triple".into())
            && t.contains(&"twice".into())
            && t.iter().all(|n| n.starts_with('t'))
    );
    let print = s.complete("program.cp", src.find("fn twice").unwrap() as u32);
    assert!(print
        .iter()
        .any(|(n, i)| *n == "print" && i.module == "std.show"));
}

#[test]
fn assigning_a_let_offers_var() {
    use crate::facade::{Session, Snapshot};
    let src = "let total = 1\nfn bump() -> Int:\n    total = 2\n    total\nprint(bump())\n";
    let s = Session::open(Snapshot::new(crate::facade::World::Closed).with_file("program.cp", src));
    let diags = s.program("program.cp").err().expect("refused");
    let d = (diags.iter())
        .find(|d| matches!(d.kind, crate::Kind::Verdict(c) if c == cpl_diag::codes::ASSIGN_TO_LET))
        .expect("E0405");
    let fix = d.fixes.first().expect("a fix");
    assert_eq!(fix.title, "declare `total` with `var`");
    let edits: Vec<_> = (fix.edits.iter())
        .map(|(span, new)| (&src[span.start as usize..span.end as usize], new.as_str()))
        .collect();
    assert_eq!(edits, [("let", "var")]);
    assert_eq!(d.labels.len(), 1);
}

#[test]
fn a_question_in_a_condition_offers_none_as_false() {
    use crate::facade::{Session, Snapshot};
    let src = "fn first(xs: List<Int>) -> ?Bool:\n    if xs.first()? > 0:\n        return true\n    false\nprint(first([1]))\n";
    let s = Session::open(Snapshot::new(crate::facade::World::Closed).with_file("program.cp", src));
    let diags = s.program("program.cp").err().expect("refused");
    let d = (diags.iter())
        .find(|d| matches!(d.kind, crate::Kind::Verdict(c) if c == cpl_diag::codes::QUESTION_IN_CONDITION))
        .unwrap_or_else(|| panic!("{diags:?}"));
    let fix = d.fixes.first().expect("a fix");
    assert_eq!(fix.title, "treat `None` as false");
    let edits: Vec<_> = (fix.edits.iter())
        .map(|(span, new)| (&src[span.start as usize..span.end as usize], new.as_str()))
        .collect();
    assert_eq!(edits, [("xs.first()? > 0", "(xs.first()? > 0) ?? false")]);
}

#[test]
fn an_unknown_std_name_offers_its_use_line() {
    use crate::facade::{Session, Snapshot};
    let src = "// mo\nprint(mo_order(3))\n";
    let s = Session::open(Snapshot::new(crate::facade::World::Closed).with_file("program.cp", src));
    let diags = s.program("program.cp").err().expect("refused");
    let d = (diags.iter())
        .find(
            |d| matches!(d.kind, crate::Kind::Verdict(c) if c == cpl_diag::codes::UNRESOLVED_NAME),
        )
        .unwrap_or_else(|| panic!("{diags:?}"));
    let fix = d.fixes.first().expect("a fix");
    assert_eq!(fix.title, "add `use std.mo.{mo_order}`");
    let ((span, new), mut fixed) = (&fix.edits[0], src.to_string());
    fixed.replace_range(span.start as usize..span.end as usize, new);
    assert_eq!(fixed, "// mo\nuse std.mo.{mo_order}\nprint(mo_order(3))\n");
}

#[test]
fn an_unknown_name_offers_the_closest_in_scope() {
    use crate::facade::{Session, Snapshot};
    let src = "let total = 1\nprint(totl)\n";
    let s = Session::open(Snapshot::new(crate::facade::World::Closed).with_file("program.cp", src));
    let diags = s.program("program.cp").err().expect("refused");
    let d = (diags.iter())
        .find(
            |d| matches!(d.kind, crate::Kind::Verdict(c) if c == cpl_diag::codes::UNRESOLVED_NAME),
        )
        .unwrap_or_else(|| panic!("{diags:?}"));
    assert_eq!(d.help.as_deref(), Some("did you mean `total`?"), "{d:?}");
    let fix = d.fixes.first().expect("a fix");
    let edits: Vec<_> = (fix.edits.iter())
        .map(|(span, new)| (&src[span.start as usize..span.end as usize], new.as_str()))
        .collect();
    assert_eq!(
        (fix.title.as_str(), edits),
        ("change to `total`", vec![("totl", "total")])
    );
}

#[test]
fn dump_ir_prints_each_stage_and_a_rejections_diagnostics() {
    let open = |src: &str| {
        facade::Session::open(
            facade::Snapshot::new(crate::facade::World::Closed).with_file("program.cp", src),
        )
    };
    let s = open("fn twice(n: Int) -> Int = n * 2\nprint(twice(3))\n");
    for stage in [
        IrStage::Raw,
        IrStage::Owned,
        IrStage::Canonical,
        IrStage::Optimized,
    ] {
        let text = s.ir("program.cp", stage).unwrap();
        assert!(text.contains("\"twice\""), "{stage:?}: {text}");
    }
    assert_ne!(
        s.ir("program.cp", IrStage::Raw).unwrap(),
        s.ir("program.cp", IrStage::Owned).unwrap()
    );
    assert!(open("print(nope)\n")
        .ir("program.cp", IrStage::Owned)
        .is_err());
    assert_eq!(IrStage::named("optimized"), Some(IrStage::Optimized));
    assert_eq!(IrStage::named("opt"), None);
}

#[test]
fn dump_own_shows_what_each_binding_freezes_and_lends() {
    let open = |src: &str| {
        facade::Session::open(
            facade::Snapshot::new(crate::facade::World::Closed).with_file("program.cp", src),
        )
    };
    let src = "fn f(n: Int) -> Int:\n    var xs = [n, 2]\n    let v = xs.rev\n    fn put(i: Int):\n        xs[i] = i\n    put(0)\n    xs.len\nprint(f(1))\n";
    let text = open(src).ir("program.cp", IrStage::Borrows).unwrap();
    // A view freezes and lends its source; a nested fn lends its captures but freezes none.
    assert!(text.contains(" v: frozen [xs] lent [xs]"), "{text}");
    assert!(text.contains(" put: frozen [] lent [xs]"), "{text}");
    assert!(open("print(nope)\n")
        .ir("program.cp", IrStage::Borrows)
        .is_err());
}

#[test]
fn a_lends_without_its_arrow_offers_to_insert_it() {
    use crate::facade::{Session, Snapshot};
    let src = "domain V = 1..=4\nvar a: [V]Int = 0\nfn get(xs: [V]Int, l: V) lends Int = xs[l]\nprint(get(a, 1))\n";
    let s = Session::open(Snapshot::new(crate::facade::World::Closed).with_file("program.cp", src));
    let diags = s.program("program.cp").err().expect("refused");
    let d = (diags.iter())
        .find(|d| matches!(d.kind, crate::Kind::Verdict(c) if c == cpl_diag::codes::EXPECTED_TOKEN))
        .unwrap_or_else(|| panic!("{diags:?}"));
    let fix = d.fixes.first().expect("a fix");
    let ((span, new), mut fixed) = (&fix.edits[0], src.to_string());
    fixed.replace_range(span.start as usize..span.end as usize, new);
    assert_eq!(fix.title, "insert `->`");
    assert!(
        fixed.contains("fn get(xs: [V]Int, l: V) -> lends Int = xs[l]"),
        "{fixed}"
    );
}

/// A storage past the judge's memory fails, by one rule on the interpreter, the engine and the
/// C++: `n` cells of `Layouts::cell_bytes` each, a `Bool` taking one byte and an `Int` eight.
#[test]
fn a_storage_past_the_judges_memory_fails_alike_everywhere() {
    let judge = Judge {
        name: "test".into(),
        memory_mb: 1,
    };
    let past = cpl_next_ir::ops::Fault::PastMemoryLimit.code();
    for (fill, n, fails) in [
        ("0", 100_000, false),
        ("0", 200_000, true),
        ("false", 200_000, false),
    ] {
        let src = format!("let a = List.filled(read(), {fill})\nprint(a.len)\n");
        let c = compile(
            &Std::read(),
            &src,
            &Roots::std(),
            false,
            Some(&judge),
            Checks::CHECKED,
        )
        .expect("compiles");
        let input = n.to_string();
        let (p, fs) = c.form(Form::Raw);
        for o in [cpl_next_interp::run(p, fs, c.input(&input)), c.run(&input)] {
            assert_eq!(
                o.trap.as_ref().map(|t| t.code()),
                fails.then_some(past),
                "{fill} {n}"
            );
        }
        let (bin, _) = cxx::build(
            c.cpp().as_bytes(),
            &["-std=c++17", "-O1", "-w", "-x", "c++"],
        )
        .expect("g++ builds it");
        let mut run = std::process::Command::new(bin);
        let ran = (run
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped()))
        .stderr(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut ch| {
            use std::io::Write;
            ch.stdin
                .take()
                .expect("stdin")
                .write_all(input.as_bytes())?;
            ch.wait_with_output()
        })
        .expect("runs");
        let err = String::from_utf8_lossy(&ran.stderr);
        assert_eq!(
            err.starts_with(&format!("trap: {past}")),
            fails,
            "{fill} {n}: {err}"
        );
    }
}

#[test]
fn the_memory_limit_is_the_judge_the_snapshot_is_built_for() {
    use crate::facade::{Session, Snapshot};
    let src =
        "let n: 1..=1e9 = read()\ndomain V = 0..n\nlet a: [V]Int = [V].filled(0)\nprint(a[V(0)])\n";
    let base = Snapshot::new(crate::facade::World::Closed).with_file("program.cp", src);
    let over = |s: Snapshot| {
        (Session::open(s).units("program.cp").1.iter()).any(|d| d.json().contains("E0904"))
    };
    assert!(!over(base.clone()), "no judge, no memory limit");
    assert!(over(base.with_judge("test", 256)));
    assert!(!over(base.with_judge("test", 1 << 20)));
}

/// `<` is the `@operator("<")` trait's fn: a bound on another trait of the same name gives none.
#[test]
fn an_operator_is_its_operator_trait_not_a_trait_of_its_name() {
    let src = "trait Ord:\n    fn lt(self, other: Self) -> Bool\n\nfn less<T: Ord>(a: T, b: T) -> Bool = a < b\nprint(less(1, 2))\n";
    let ds: Vec<String> = diags(src).iter().map(|d| d.to_string()).collect();
    assert_eq!(
        ds,
        [
            "check: program:5:7: E0606: `Int` does not implement `Ord`",
            "check: program:4:39: E0606: `T` has no `impl Ord`, which `<` needs (R08.1.5)",
        ]
    );
}

/// A release build keeps only the checks a `with checks(..: on)` keeps (R09.4.1): division by
/// zero gives 0 and an `assert` is skipped, but one kept on still traps, alike on the
/// interpreter, the engine and the C++.
#[test]
fn release_keeps_only_the_checks_turned_on() {
    let src = "let z = 0\nprint(7 / z, 7 % z)\nassert z == 1, \"off in release\"\nprint(1)\n\
with checks(asserts: on):\n    assert z == 1, \"kept\"\nprint(2)\n";
    let c = compile(&Std::read(), src, &Roots::std(), false, None, Checks::NONE).expect("compiles");
    for o in [raw(&c), c.run("")] {
        assert_eq!(o.output, "0 0\n1\n");
        let assert = cpl_next_ir::ops::Fault::Check(cpl_next_ir::ir::CheckKind::Assert);
        assert_eq!(o.trap.map(|t| t.fault), Some(assert));
    }
    let (bin, _) = cxx::build(
        c.cpp().as_bytes(),
        &["-std=c++17", "-O1", "-w", "-x", "c++"],
    )
    .expect("g++ builds it");
    let ran = std::process::Command::new(bin).output().expect("runs");
    assert!(!ran.status.success());
    assert_eq!(String::from_utf8_lossy(&ran.stdout), "0 0\n1\n");
    let checked = compile(
        &Std::read(),
        src,
        &Roots::std(),
        false,
        None,
        Checks::CHECKED,
    );
    let o = raw(&checked.expect("compiles"));
    assert_eq!(
        o.trap.map(|t| t.fault),
        Some(cpl_next_ir::ops::Fault::DivisionByZero)
    );
}

/// The stack runs out at the run's limit (contract 0's `Limits`), on the reference interpreter
/// and the engine alike.
#[test]
fn the_stack_runs_out_at_the_runs_limit() {
    use cpl_next_engine::{Io, Limits, RunInput};
    let src = "fn f(k: Int) -> Int = if k == 0: 0 else: 1 + f(k - 1)\nprint(f(100))\n";
    let c = compile(
        &Std::read(),
        src,
        &Roots::std(),
        false,
        None,
        Checks::CHECKED,
    )
    .expect("compiles");
    let (p, fs) = c.form(Form::Raw);
    for (stack, out) in [(1 << 10, true), (64 << 10, false)] {
        let input = || RunInput {
            io: Io::Fixed(Vec::new()),
            limits: Limits {
                stack,
                ..Limits::default()
            },
        };
        let raw = cpl_next_interp::run(p, fs, input());
        let fast = cpl_next_engine::run(&c.prog, &c.opt, input());
        for o in [&raw, &fast] {
            assert_eq!(
                o.trap.as_ref().map(|t| t.code()),
                out.then_some("R0601"),
                "{stack}"
            );
            assert_eq!(o.output, if out { "" } else { "100\n" }, "{stack}");
        }
    }
}

/// Input streamed as the program reads gives what the same bytes given whole give, on the
/// reference interpreter and the engine.
#[test]
fn streamed_input_reads_as_fixed_input_does() {
    use cpl_next_engine::{Io, Limits, RunInput};
    struct Trickle(Vec<u8>, usize);
    impl std::io::Read for Trickle {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            let Some(&b) = self.0.get(self.1) else {
                return Ok(0);
            };
            buf[0] = b;
            self.1 += 1;
            Ok(1)
        }
    }
    let src = "let n: 1..=100 = read()\nlet xs: [n]Int = read()\nprint(n, xs)\n";
    let text = b"3\n1 2  3\n";
    let c = compile(
        &Std::read(),
        src,
        &Roots::std(),
        false,
        None,
        Checks::CHECKED,
    )
    .expect("compiles");
    let (p, fs) = c.form(Form::Raw);
    let fixed = || RunInput {
        io: Io::Fixed(text.to_vec()),
        limits: Limits::default(),
    };
    let streamed = || RunInput {
        io: Io::Stream(Box::new(Trickle(text.to_vec(), 0))),
        limits: Limits::default(),
    };
    let want = cpl_next_interp::run(p, fs, fixed());
    assert!(
        want.trap.is_none() && !want.output.is_empty(),
        "{}",
        want.output
    );
    let runs = [
        cpl_next_interp::run(p, fs, streamed()),
        cpl_next_engine::run(&c.prog, &c.opt, streamed()),
        cpl_next_engine::run(&c.prog, &c.opt, fixed()),
    ];
    for o in runs {
        assert!(o.trap.is_none());
        assert_eq!(o.output, want.output);
    }
}

/// Events name their statement by its fn and site, the same on the interpreter and the engine,
/// and `At` lines two runs up by call path and visit, not by position in the stream.
#[test]
fn events_carry_sites_and_line_up_by_at() {
    use cpl_next_ir::ir::EventKind;
    use cpl_next_ir::run::{at, RunInput};
    let src = "var total = 0\nfn add(x: Int) -> Int = x + 1\nfn bump(k: Int):\n    \
total = total + add(k)\nlet n: Int = read()\nfor i in 0..n:\n    bump(i)\nprint(total)\n";
    let c = compile(
        &Std::read(),
        src,
        &Roots::std(),
        true,
        None,
        Checks::CHECKED,
    )
    .expect("compiles");
    let funcs = c.form(Form::Raw).1;
    let (p, fs) = c.form(Form::Raw);
    let (two, three) = (
        cpl_next_interp::run(p, fs, RunInput::fixed("2")),
        cpl_next_interp::run(p, fs, RunInput::fixed("3")),
    );
    assert_eq!(two.events, c.run("2").events);
    let ats = |o: &Outcome| at(&o.events, funcs);
    let (a2, a3) = (ats(&two), ats(&three));
    assert!(
        a3.starts_with(&a2[..a2.len() - 1]),
        "a longer run shares the shorter one's start"
    );
    let adds: Vec<_> = (two.events.iter().zip(&a2))
        .filter(|(e, _)| e.kind == EventKind::Enter && funcs[e.func as usize].name == "add")
        .map(|(_, a)| a)
        .collect();
    assert_eq!(adds.len(), 2);
    assert_ne!(
        adds[0].call, adds[1].call,
        "each call of `bump` is its own call path"
    );
    assert_eq!((adds[0].nth, adds[0].site.item.name.as_ref()), (0, "add"));
    let writes = (two.events.iter().zip(&a2)).filter(
        |(e, _)| matches!(&e.kind, EventKind::Write { shown, .. } if shown.as_ref() == "total"),
    );
    let sites: Vec<_> = writes.map(|(_, a)| a.site.clone()).collect();
    assert_eq!(sites.len(), 3);
    assert_eq!(
        sites[1], sites[2],
        "both writes in `bump` are one statement"
    );
    assert_ne!(sites[0], sites[1]);
}

/// A trace shows the writes the user's file makes to its top-level bindings, and none of
/// std's, though std's are set in the same top level.
#[test]
fn a_trace_shows_only_the_users_writes() {
    use cpl_next_ir::ir::EventKind;
    let src = "var t = 1\nt = t + 1\nprint(t)\n";
    let c = compile(
        &Std::read(),
        src,
        &Roots::std(),
        true,
        None,
        Checks::CHECKED,
    )
    .expect("compiles");
    let o = c.run("");
    let written: Vec<_> = (o.events.iter())
        .filter_map(|e| match &e.kind {
            EventKind::Write { shown, .. } => Some(format!("{shown} {}", e.vals.join(" "))),
            _ => None,
        })
        .collect();
    assert_eq!(written, ["t 1", "t 2"]);
}

/// No std privilege (§11.6): a std module that fills no role, copied under a `[libs]` path,
/// checks and runs each of its cases as std's own does, given by `use` the names the prelude
/// gives.
#[test]
fn a_std_module_copied_into_a_library_behaves_as_std() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let std = Std::read();
    let prelude = std::fs::read_to_string(root.join("std/prelude.cp")).unwrap();
    let dir = std::env::temp_dir().join(format!("cpl-no-privilege-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("kit")).unwrap();
    std::fs::write(dir.join("cp.toml"), "[libs]\nkit = \"kit\"\n").unwrap();
    let said = |src: &str, roots: &Roots| -> String {
        let shown = match compile(&std, src, roots, false, None, Checks::CHECKED) {
            Ok(c) => {
                let (a, b) = (raw(&c), c.run(""));
                format!(
                    "{}{:?}|{}{:?}",
                    a.output,
                    a.trap.is_some(),
                    b.output,
                    b.trap.is_some()
                )
            }
            Err(ds) => ds.iter().map(|d| format!("{d}\n")).collect(),
        };
        shown.replace("kit.", "std.").replace("kit/", "std/")
    };
    let mut ran = 0;
    for m in ["dsu", "fenwick"] {
        let text = std::fs::read_to_string(root.join(format!("std/{m}.cp"))).unwrap();
        assert!(
            !text.contains("@role("),
            "std.{m} fills no role, so its copy is all of it"
        );
        std::fs::write(dir.join(format!("kit/{m}.cp")), text).unwrap();
        let given: Vec<&str> = (prelude.lines())
            .find_map(|l| l.strip_prefix(&format!("use std.{m}.{{")))
            .map(|l| l.trim_end_matches('}').split(", ").collect())
            .unwrap_or_default();
        let cases = root.join(format!("spec/conformance/std-{m}"));
        let mut paths: Vec<_> = (std::fs::read_dir(cases).unwrap())
            .map(|e| e.unwrap().path())
            .filter(|p| p.extension().is_some_and(|x| x == "cp"))
            .collect();
        paths.sort();
        for path in paths {
            let src = std::fs::read_to_string(&path).unwrap();
            if src.contains("// file:") {
                continue;
            }
            let moved = src.replace(&format!("std.{m}.{{"), &format!("kit.{m}.{{"));
            let named: Vec<&str> = (given.iter().copied())
                .filter(|n| {
                    !moved
                        .lines()
                        .any(|l| l.contains(&format!("kit.{m}.")) && l.contains(n))
                })
                .collect();
            // The `use` takes a header comment's line, so every line keeps its number.
            let moved = match named.is_empty() {
                true => moved,
                false => moved.replacen(
                    "// ",
                    &format!("use kit.{m}.{{{}}} // ", named.join(", ")),
                    1,
                ),
            };
            assert_eq!(
                said(&src, &Roots::from(&dir)),
                said(&moved, &Roots::from(&dir)),
                "{}",
                path.display()
            );
            ran += 1;
        }
    }
    std::fs::remove_dir_all(&dir).unwrap();
    assert!(ran > 30, "{ran} cases");
}

/// An event shows a `Bool` and a `?T`, which are not `Show` (R10.3), through std's other trace
/// fns: `true` or `false`, and `none` or the value held (Rtrace.7), alike on the interpreter,
/// the engine and the C++.
/// Two `@role(trace)` fns whose parameters take one type are E0202: an event's value has one
/// text (§11.6).
#[test]
fn two_trace_fns_taking_one_type_are_said() {
    let src =
        "@role(trace)\nfn mine(x: Bool) uses io = write(\"b\")\nvar b = false\nprint(yes(b))\n";
    let got = compile(
        &Std::read(),
        src,
        &Roots::std(),
        true,
        None,
        Checks::CHECKED,
    );
    let said: Vec<_> = (got.err().expect("rejected").iter())
        .map(|d| d.code())
        .collect();
    assert_eq!(said, [Some("E0202")]);
}

#[test]
fn an_event_shows_a_bool_and_an_option() {
    let src = "var b = false\nb = true\nvar o: ?Int = None\no = Some(3)\nprint(yes(b), o ?? 0)\n";
    let c = compile(
        &Std::read(),
        src,
        &Roots::std(),
        true,
        None,
        Checks::CHECKED,
    )
    .expect("compiles");
    let want = [["false"], ["true"], ["none"], ["3"]];
    for o in [raw(&c), c.run("")] {
        assert_eq!(o.output, "YES 3\n");
        assert_eq!(
            o.events.iter().map(|e| e.vals.clone()).collect::<Vec<_>>(),
            want
        );
    }
    let (bin, _) = cxx::build(
        c.cpp().as_bytes(),
        &["-std=c++17", "-O1", "-w", "-x", "c++"],
    )
    .expect("g++ builds it");
    let ran = std::process::Command::new(bin).output().expect("runs");
    assert_eq!(String::from_utf8_lossy(&ran.stdout), "YES 3\n");
    assert_eq!(
        String::from_utf8_lossy(&ran.stderr),
        "ev set b false\nev set b true\nev set o none\nev set o 3\n"
    );
}

/// An event shows a `HashMap` and a `HashSet`, which are not `Show` (R10.3.8), through std.hash's
/// trace fns: as a `Map` and a `Set` write, in table order (Rtrace.7), alike on the interpreter,
/// the engine and the C++.
#[test]
fn an_event_shows_a_hash_map_and_set() {
    let src = "var h = HashMap([(1, 10), (2, 20)])\nvar s = HashSet([3, 4])\nprint(h.len, s.len)\n";
    let c = compile(
        &Std::read(),
        src,
        &Roots::std(),
        true,
        None,
        Checks::CHECKED,
    )
    .expect("compiles");
    let want = [["1 10\n2 20"], ["3 4"]];
    for o in [raw(&c), c.run("")] {
        assert_eq!(o.output, "2 2\n");
        assert_eq!(
            o.events.iter().map(|e| e.vals.clone()).collect::<Vec<_>>(),
            want
        );
    }
    let (bin, _) = cxx::build(
        c.cpp().as_bytes(),
        &["-std=c++17", "-O1", "-w", "-x", "c++"],
    )
    .expect("g++ builds it");
    let ran = std::process::Command::new(bin).output().expect("runs");
    assert_eq!(String::from_utf8_lossy(&ran.stdout), "2 2\n");
    assert_eq!(
        String::from_utf8_lossy(&ran.stderr),
        "ev set h 1 10\n2 20\nev set s 3 4\n"
    );
}

/// An event shows a value as `write` gives it, through std's `traced`: a struct by its own
/// `Show`, a list by std's, and a `Bool`, which is not `Show` (R10.3), by `traced_bool`. The `Show`
/// it runs places no events, alike on the interpreter, the engine and the C++.
#[test]
fn an_event_shows_a_value_as_write_does() {
    use cpl_next_ir::ir::EventKind;
    let src = "struct P:\n    x: Int\nimpl Show for P:\n    fn write_to(self, mut out: Str, sep: ?Str):\n        \
out.push('P')\nvar best = [1, 2]\nfn f(n: Int, p: P, ok: Bool) -> List<Int>:\n    \
return [n, p.x]\nbest = f(3, P(x: 4), true)\nprint(best.len)\n";
    let c = compile(
        &Std::read(),
        src,
        &Roots::std(),
        true,
        None,
        Checks::CHECKED,
    )
    .expect("compiles");
    let shown = |o: &Outcome| -> Vec<String> {
        (o.events.iter())
            .map(|e| match &e.kind {
                EventKind::Enter => format!("enter {:?}", e.vals),
                EventKind::Exit => format!("exit {:?}", e.vals),
                EventKind::Write { shown, .. } => format!("set {shown} {:?}", e.vals),
            })
            .collect()
    };
    let want = [
        r#"set best ["1 2"]"#,
        r#"enter ["3", "P", "true"]"#,
        r#"exit ["3 4"]"#,
        r#"set best ["3 4"]"#,
    ];
    for o in [raw(&c), c.run("")] {
        assert_eq!(o.output, "2\n");
        assert_eq!(shown(&o), want);
    }
    let (bin, _) = cxx::build(
        c.cpp().as_bytes(),
        &["-std=c++17", "-O1", "-w", "-x", "c++"],
    )
    .expect("g++ builds it");
    let ran = std::process::Command::new(bin).output().expect("runs");
    assert_eq!(String::from_utf8_lossy(&ran.stdout), "2\n");
    assert_eq!(
        String::from_utf8_lossy(&ran.stderr),
        "ev set best 1 2\nev enter f 3 P true\nev exit f 3 4\nev set best 3 4\n"
    );
}

/// Each fn is listed under the item that writes it, a struct's methods under the struct's
/// item, so a per-item query finds an item's fns and each fn's item.
#[test]
fn each_item_lists_the_fns_it_writes() {
    use cpl_next_ir::ty::FnId;
    let src = "struct P:\n    x: Int\n    fn get(self) -> Int = self.x\nfn twice(p: P) -> Int = p.get() * 2\nprint(twice(P(3)))\n";
    let f = front(&Std::read(), src, &Roots::std(), None, Checks::CHECKED);
    assert!(f.diags.is_empty(), "{:?}", f.diags);
    let d = &f.d;
    let named = |n: &str| {
        let mut ids = (0..d.fns.len()).map(FnId);
        ids.find(|&i| d.f(i).name == n && d.f(i).module == d.program)
            .unwrap()
    };
    let (get, twice) = (named("get"), named("twice"));
    assert!(d.fns_of(&d.f(get).item).contains(&get));
    assert!(d.fns_of(&d.f(twice).item).contains(&twice));
    assert_ne!(d.f(get).item, d.f(twice).item);
    for (item, fs) in &d.item_fns {
        fs.iter().for_each(|&i| assert_eq!(&d.f(i).item, item));
    }
}

/// A view, a lazy std seq and a closure stored past its frame each lend their source: writing
/// or escaping it is rejected (wrong-accepted after 12be3511f until fd3058663).
#[test]
fn a_lent_source_cannot_be_written_or_escaped() {
    let cases = [
        (
            "var xs = [3, 1, 2]\nlet w = xs[0..2]\nxs.push(4)\nprint(w.sum())\n",
            "program:3:",
            "E0602",
        ),
        (
            "use std.seq.{map}\nvar xs = [1, 2, 3]\nlet f = map(xs, x => x + 1)\nxs.push(9)\nvar n = 0\nfor _ in f:\n    n += 1\nprint(n)\n",
            "program:4:",
            "E0602",
        ),
        (
            "var fs: List<Int -> Int> = []\nfn mk(n: Int):\n    var k = n\n    fs.push(x => x + k)\n    k += 1\nmk(5)\nprint(fs[0](1))\n",
            "program:4:",
            "E0603",
        ),
    ];
    for (src, at, code) in cases {
        let ds: Vec<String> = diags(src).iter().map(|d| d.to_string()).collect();
        assert!(
            ds.iter().any(|d| d.contains(at) && d.contains(code)),
            "{code} at {at}: {ds:?}\n{src}"
        );
    }
}

/// A lambda names the copy it is written in as its maker: a `uses` fn's copy under a context
/// value makes its own lambda, and one at top level is made by it.
#[test]
fn a_lambda_names_the_copy_that_makes_it() {
    let src = "fn add(n: Int) -> Int uses io:\n    let h = (k: Int) => k + n + io.base\n    h(1)\nlet t = (k: Int) => k * 2\nprint(add(2), t(3))\nwith io(base: 1):\n    print(add(2))\n";
    let f = front(&Std::read(), src, &Roots::std(), None, Checks::CHECKED);
    assert!(f.diags.is_empty(), "{:?}", f.diags);
    let makers: Vec<ir::FuncKey> = (f.g.funcs.values())
        .filter(|g| matches!(g.key, ir::FuncKey::Lambda(_)) && g.line != 0)
        .filter(|g| {
            f.g.funcs
                .contains_key(g.maker.as_ref().expect("a lambda has a maker"))
        })
        .filter_map(|g| g.maker.clone())
        .collect();
    let add = (f.g.funcs.values())
        .find(|g| g.name == "add" && g.line != 0)
        .and_then(|g| g.key.decl())
        .expect("add is lowered");
    assert!(makers.contains(&ir::FuncKey::Main), "{makers:?}");
    assert!(makers.contains(&ir::FuncKey::Decl(add)), "{makers:?}");
    assert!(
        makers
            .iter()
            .any(|k| matches!(k, ir::FuncKey::Ctx(g, _) if *g == add)),
        "{makers:?}"
    );
}

/// The checks a program is built with come from its snapshot (contract 0): the same file is
/// a release build in one snapshot and a checked one in the other.
#[test]
fn the_snapshot_says_which_checks_a_build_keeps() {
    use crate::facade::{Session, Snapshot};
    let src = "let z = 0\nprint(7 / z)\nprint(1)\n";
    let file = Snapshot::new(crate::facade::World::Closed).with_file("program.cp", src);
    let ran = |s: &Snapshot| {
        Session::open(s.clone())
            .program("program.cp")
            .map(|p| p.run(""))
    };
    let release = ran(&file.with_checks(Checks::NONE)).expect("compiles");
    assert_eq!(
        (release.output.as_str(), release.trap.is_none()),
        ("0\n1\n", true)
    );
    let checked = ran(&file).expect("compiles");
    assert_eq!(
        (checked.output.as_str(), checked.trap.is_some()),
        ("", true)
    );
}

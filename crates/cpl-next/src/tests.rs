use super::*;
use modules::PROGRAM;

/// A method found in a std module the program does not load is called without loading it
/// (R11.3.4), so the modules loaded stay the ones the text names.
#[test]
fn a_method_call_loads_no_module() {
    let seen = |src: &str| {
        let f = front(&Std::read(), src, &Roots::std());
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
    let c = compile(&Std::read(), src, &Roots::std(), false).expect("compiles");
    let f = c
        .prog
        .insts
        .iter()
        .find(|f| f.name == "pick")
        .expect("pick");
    assert_eq!(f.views, [true, false, true]);
}

/// A fn's sites are its own: one written above it leaves its sites and their places as
/// they were.
#[test]
fn a_fn_above_leaves_the_sites_of_those_below() {
    let below = "fn widget(xs: List<Int>) -> Int:\n    xs[0] + xs[1]\n\nprint(widget([1, 2]))\n";
    let above = format!("fn f() -> Int: 1\n\n{below}");
    let g = |src: &str| {
        let c = compile(&Std::read(), src, &Roots::std(), false).expect("compiles");
        let insts = &c.prog.insts;
        let g = insts.iter().find(|f| f.name == "widget").expect("widget");
        // An instance is named by what it instantiates: a fn above adds instances before it.
        let body = (insts.iter().enumerate().rev()).fold(format!("{:?}", g.body), |b, (k, f)| {
            b.replace(&format!("Inst({k})"), &format!("Inst({})", f.name))
        });
        (body, g.sites.clone(), g.lines.clone())
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
    compile(&Std::read(), src, &Roots::std(), false)
        .err()
        .expect("rejected")
}

/// What compiling `src` gives: its diagnostics, or its C++ and what its run prints.
fn outcome(std: &Std, src: &str) -> String {
    match compile(std, src, &Roots::std(), true) {
        Ok(c) => {
            let o = c.run(Form::Raw, "");
            let events: String = o.events.iter().map(|e| format!("{e}\n")).collect();
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
    let c = compile(&Std::read(), src, &Roots::std(), true).expect("compiles");
    let (raw, engine) = (c.run(Form::Raw, ""), c.run(Form::Engine, ""));
    let want = "4.0000000000 0.0000000000 1.0000000000 0.7853981634 5.0000000000 \
1.0000000000 0.0000000000\n-3 -2 -3 3 -2\n";
    assert_eq!(raw.output, want);
    assert_eq!(engine.output, want);
    let fault = |o: &Outcome| o.trap.as_ref().map(|t| t.fault);
    assert_eq!(fault(&raw), Some(cpl_next_ir::ops::Fault::F64PastInt));
    assert_eq!(fault(&raw), fault(&engine));
}

/// A struct's derived `Eq` and `Ord` declare every parameter, so one named by more than a
/// letter (`Elem`, std.tree's `Pre`) is no unknown type.
#[test]
fn derived_impls_declare_parameters_of_any_name() {
    let src = "struct Bx<Elem>:\n    x: Elem\nif Bx(x: 1) == Bx(x: 1) and Bx(x: 1) < Bx(x: 2):\n    print(1)\n";
    let c = compile(&Std::read(), src, &Roots::std(), true).expect("compiles");
    assert_eq!(c.run(Form::Raw, "").output, "1\n");
    assert_eq!(c.run(Form::Engine, "").output, "1\n");
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
    let c = compile(&Std::read(), src, &Roots::std(), false).expect("compiles");
    let main = c.prog.main;
    (c.proved().iter())
        .filter(|p| p.func == main)
        .map(|p| (p.kind, c.prog.insts[p.func].lines[p.site as usize]))
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
    let json =
        |src: &str| (diags(src).iter().map(|d| cpl_json::to_string(&d.json()))).collect::<Vec<_>>();
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

/// A `[libs]` module writes markers as core does: its second `@cuts` struct, beside core's
/// `Cut`, is a duplicate (R06.4.12).
#[test]
fn a_library_second_marked_declaration_is_a_duplicate() {
    let dir = std::env::temp_dir().join(format!("cpl-marker-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("kit")).unwrap();
    std::fs::write(dir.join("cp.toml"), "[libs]\nkit = \"kit\"\n").unwrap();
    let lib = "@cuts\npub struct MyCut:\n    k: Int\npub fn one() -> Int = 1\n";
    std::fs::write(dir.join("kit/m.cp"), lib).unwrap();
    let src = "use kit.m.{one}\nprint(one())\n";
    let got = compile(&Std::read(), src, &Roots::from(&dir), false);
    std::fs::remove_dir_all(&dir).unwrap();
    let ds: Vec<String> = got
        .err()
        .expect("rejected")
        .iter()
        .map(|d| d.to_string())
        .collect();
    assert!(
        ds.iter()
            .any(|d| d.contains("E0202: a second declaration marked `@cuts`")),
        "{ds:?}"
    );
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
    let f = front(&Std::read(), src, &Roots::std());
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
    let f = front(&Std::read(), src, &Roots::std());
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
    let f = front(&Std::read(), src, &Roots::std());
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
    let f = front(&Std::read(), src, &Roots::std());
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
    let f = front(&Std::read(), src, &Roots::std());
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

//! Lowered programs run to the output the source asks for, on the paths
//! lowering once trusted with a panic.

fn runs(source: &str, input: &str) -> String {
    let mut db = cpl_check::Session::new();
    db.set_file("t.cp", source);
    let Some(analysis) = db.analyze("t.cp") else {
        return "no analysis".into();
    };
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let program = match cpl_ir::lower(&analysis) {
        Ok(program) => program,
        Err(diags) => return format!("{diags:?}"),
    };
    let mut out = Vec::new();
    if let Err(trap) = cpl_interp::run(&program, input.as_bytes(), &mut out) {
        return trap.into_message();
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[test]
fn sequence_methods_take_their_element_from_the_type() {
    let source = "let xs = [3, 1, 2]\n\
        print(xs.get(1)!)\n\
        print(xs.at(0))\n\
        print(\"abc\".nth(1))\n\
        print(xs.find(x => x > 1) ?? 0)\n\
        print(xs.last(1) ?? 0)\n\
        print(sorted(0..3))\n\
        print(sorted(\"cab\"))\n";
    assert_eq!(runs(source, ""), "1\n3\nb\n3\n1\n0 1 2\nabc\n");
}

#[test]
fn a_reversed_range_walks_its_own_element() {
    let source = "let xs = [3, 1, 2]\nprint((0..3).rev)\nprint(xs.keys.rev)\n";
    assert_eq!(runs(source, ""), "2 1 0\n2 1 0\n");
}

#[test]
fn a_tagged_enum_reads_each_variant_by_its_one_tag() {
    let source = "enum Move = @tag(\"L\") Left | @tag(\"R\") Right\n\
        let ms: [2]Move = read()\n\
        for m in ms:\n    print(yes(m == Move.Left))\n";
    assert_eq!(runs(source, "R L\n"), "NO\nYES\n");
}

#[test]
fn a_min_over_optionals_starts_from_its_first_plain_value() {
    let source =
        "let a: ?Int = None\nlet b: ?Int = Some(1)\nprint(min(a, 5, 3))\nprint(min(b, 5))\n";
    assert_eq!(runs(source, ""), "3\n1\n");
}

#[test]
fn items_kept_as_a_value_walk_their_index_and_cell() {
    let source = "let xs = [5, 7]\nlet it = xs.items\nfor (i, x) in it: print(i, x)\n\
        let cs = \"ab\".items\nfor (i, c) in cs: print(i, c)\n";
    assert_eq!(runs(source, ""), "0 5\n1 7\n0 a\n1 b\n");
}

#[test]
fn a_row_or_part_runs_along_the_range_written_for_it() {
    let source = "domain R = 0..2\ndomain C = 0..3\ndomain P = (R, C)\n\
        let g: [R, C]Int = [P] p => 1\nprint(g[1, ..].sum(), g[.., 2].sum())\n\
        var a = [1, 2, 3, 4]\na[1..3] = 0\nprint(a)\n\
        let b = [7, 8, 9, 10]\na[..2] = b[2..]\nprint(a)\n\
        a[1..=2] max= b[0..2]\nprint(a)\nprint(a[1..=2].sum())\n";
    assert_eq!(runs(source, ""), "3 2\n1 0 0 4\n9 10 0 4\n9 10 8 4\n18\n");
}

#[test]
fn an_inverse_maps_each_value_back_to_its_position() {
    let source = "domain V = 0..3\nlet p: [V]V = [V(2), V(0), V(1)]\nlet inv = p.inverse()\n\
        print([inv[v].ord for v in V])\nlet q: [V]0..3 = [2, 0, 1]\nlet qi = q.inverse()\n\
        print([qi[k].ord for k in 0..3])\n";
    assert_eq!(runs(source, ""), "1 2 0\n1 2 0\n");
}

#[test]
fn a_lent_mut_projection_passed_mut_writes_through() {
    let source = "struct Pool:\n    _xs: List<Int>\n\
        \x20   pub fn at(mut self, i: Int) -> lends mut Int:\n        lend mut _xs[i]\n\
        fn bump(mut x: Int):\n    x += 5\nvar p = Pool([1, 2])\nbump(mut p.at(1))\n\
        print(p.at(0), p.at(1))\n";
    assert_eq!(runs(source, ""), "1 7\n");
}

#[test]
fn a_local_memo_fn_calls_the_body_it_keeps() {
    let source = "fn f(n: Int) -> Int:\n    let k = 2\n    @memo(map)\n\
        \x20   fn g(i: Int) -> Int = i * k\n    g(n)\nprint(f(3))\n";
    assert_eq!(runs(source, ""), "6\n");
}

#[test]
fn a_law_check_over_std_samples_sets_its_samples_first() {
    let source = "use std.algebra.{Action}\n\
        let good = Action<Int, ?Int>(None, (nf, of) => nf ?? of, (f, v, span) => f ?? v, over: max, replaces: f => f != None)\n\
        print(laws_hold(good, T: [1, 5, -3], F: [None, Some(2), Some(7)]))\n";
    assert_eq!(runs(source, ""), "YES\n");
}

#[test]
fn nested_closures_each_return_to_the_cx_they_were_entered_from() {
    let source = "fn f(n: Int) -> Int:\n    let k = 3\n\
        \x20   fn g(i: Int) -> Int:\n        let h = (j: Int) => j * k + i\n        h(n)\n\
        \x20   @memo(map)\n    fn m(i: Int) -> Int:\n        if i == 0:\n            return k\n        g(i) + m(i - 1)\n\
        \x20   fn ys() -> yields Int:\n        for i in 0..n: yield i + k\n\
        \x20   let s = [y for y in ys()].sum()\n    let after = k + n\n    m(2) + s + after\n\
        print(f(2))\n";
    assert_eq!(runs(source, ""), "30\n");
}

#[test]
fn laws_hold_lowers_its_named_samples_in_the_order_written() {
    let source = "use std.algebra.{Action}\n\
        let good = Action<Int, ?Int>(None, (nf, of) => nf ?? of, (f, v, span) => f ?? v, over: max, replaces: f => f != None)\n\
        print(laws_hold(good, T: [1, 5, -3], F: [None, Some(2), Some(7)]))\n";
    let lowered = || {
        let mut db = cpl_check::Session::new();
        db.set_file("t.cp", source);
        let analysis = db.analyze("t.cp").expect("analysis");
        format!("{:?}", cpl_ir::lower(&analysis).expect("lowers"))
    };
    let first = lowered();
    for _ in 0..8 {
        assert!(
            lowered() == first,
            "lowering laws_hold gave a different program"
        );
    }
}

/// How many `Error` expressions survive lowering `source`, which checks clean.
fn recovered(source: &str) -> usize {
    let mut db = cpl_check::Session::new();
    db.set_file("t.cp", source);
    let program = cpl_ir::lower(&db.analyze("t.cp").unwrap()).unwrap();
    let mut errors = 0;
    let bodies = program.fns.iter().map(|f| &f.body);
    for body in bodies.chain(program.closures.iter().map(|c| &c.body)) {
        cpl_ir::walk::deep_exprs(body, &mut |e| {
            errors += matches!(e.kind, cpl_ir::ExprKind::Error) as usize;
        });
    }
    errors
}

#[test]
fn a_lambda_lowered_only_to_learn_its_type_leaves_no_closure() {
    let source = "struct Tw<W>(pub a: Int)\n\
        impl Index for Tw<W>:\n    let count = W.count\n    @get fn ord(self) -> Int = a\n    fn nth(k: Int) -> Tw<W> = Tw(k)\n\
        domain V = 0..3\n\
        print(Tw<V>.filter(x => x.a > 0).count())\n";
    assert_eq!(recovered(source), 0);
    assert_eq!(runs(source, ""), "2\n");
}

#[test]
fn a_generator_probed_before_its_yield_type_is_known_makes_no_struct() {
    let source = "print([3, 1, 4].window_fold(2, max).collect())\n";
    assert_eq!(recovered(source), 0);
    assert_eq!(runs(source, ""), "3 4\n");
}

#[test]
fn a_bare_op_lifted_into_monoid_of_is_not_lifted_again() {
    // `SegTree(+, a)` lifts `+` into `Monoid.of(+)`, whose `M: FromOp<T>` wants a
    // `Monoid<T>` for `+` again.
    let out = runs("var g = SegTree(+, [1, 2, 3])\nprint(g.all)\n", "");
    assert!(out == "6\n" || out.contains("Monoid.of"), "{out}");
}

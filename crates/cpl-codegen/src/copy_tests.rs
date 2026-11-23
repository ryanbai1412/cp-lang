//! Copies the C++ leaves out: a list moved instead of copied, a field
//! walked in place.

use crate::tests::cpp;

#[test]
fn a_returned_rekey_moves_its_local_list() {
    let cpp = cpp(concat!(
        "use std.strings.{z_function, prefix_function}\n",
        "var a = List.filled(4, 0)\n",
        "a[2] = 1\n",
        "let z = z_function(a)\n",
        "let p = prefix_function(a)\n",
        "print(z[1] + p[1])\n",
    ));
    assert!(cpp.contains("cpl::rekey_own(std::move(z"), "{cpp}");
    assert!(cpp.contains("cpl::rekey_own(std::move(cells"), "{cpp}");
}

#[test]
fn a_loop_over_one_field_that_writes_another_walks_it_in_place() {
    let cpp = cpp(concat!(
        "struct Bag:\n    w: List<Int>\n    n: Int\n",
        "    fn recount(mut self):\n        n = 0\n        for x in w: n += x\n",
        "    fn bump(mut self):\n        n += 1\n",
        "    fn refill(mut self):\n        for x in w: self.bump()\n",
        "var b = Bag(w: [1, 2, 3], n: 0)\n",
        "b.recount()\nb.refill()\n",
        "print(b.n + b.w.len)\n",
    ));
    let body = |name: &str| {
        let head = format!("void Bag_{name}(Bag& self) {{");
        let after = cpp.split(&head).nth(1).unwrap_or_default();
        after[..after.find("\n}").unwrap_or(after.len())].to_string()
    };
    assert!(body("recount").contains(": self.w)"), "{cpp}");
    assert!(body("refill").contains("= self.w;"), "{cpp}");
}

#[test]
fn a_let_bound_comprehension_moves_its_list() {
    let cpp = cpp(concat!(
        "let a: [0..3]Int = read()\n",
        "fn f(v: Int) -> Int:\n",
        "    let signs = [Int(x >= v) - 1 for x in a]\n",
        "    let nz = [s for s in signs if s != 0]\n",
        "    nz.len\n",
        "print(f(2))\n",
    ));
    // `f` can't re-enter, so each list keeps one buffer across calls.
    assert!(cpp.contains("auto& signs = t"), "{cpp}");
    assert!(cpp.contains("auto& nz = t"), "{cpp}");
    assert!(cpp.contains("static std::vector<ll> t"), "{cpp}");
}

#[test]
fn a_let_bound_comprehension_in_a_recursive_fn_moves_its_list() {
    let cpp = cpp(concat!(
        "let a: [0..3]Int = read()\n",
        "fn f(v: Int) -> Int:\n",
        "    let signs = [Int(x >= v) - 1 for x in a]\n",
        "    if v > 0: return f(v - 1) + signs.len\n",
        "    signs.len\n",
        "print(f(2))\n",
    ));
    assert!(cpp.contains("signs = std::move("), "{cpp}");
    assert!(!cpp.contains("static std::vector"), "{cpp}");
}

#[test]
fn a_loop_guarded_by_its_start_starts_there() {
    let cpp = cpp(concat!(
        "let n: 1..=9 = read()\n",
        "domain P = 0..n\n",
        "var hits = 0\n",
        "fn tally(ls: Range<P>, rs: Range<P>):\n",
        "    for l in ls:\n",
        "        for r in rs if r >= l:\n",
        "            hits += 1\n",
        "tally(P.first!..=P.last!, P.first!..=P.last!)\n",
        "print(hits)\n",
    ));
    assert!(
        cpp.contains("std::max<ll>(") && !cpp.contains("if (r >= l)"),
        "{cpp}"
    );
}

#[test]
fn a_let_of_an_if_reads_its_place_branch_in_place() {
    let cpp = cpp(concat!(
        "let by: [0..3]List<Int> = [[1, 2], [3], []]\n",
        "fn flip(xs: List<Int>) -> List<Int> = [9 - x for x in xs.rev]\n",
        "for v in 0..3:\n",
        "    let pos = if v % 2 == 0: flip(by[v]) else: by[v]\n",
        "    print(pos.len)\n",
    ));
    assert!(cpp.contains("const std::vector<ll>& pos = "), "{cpp}");
}

#[test]
fn a_let_of_an_if_copies_a_place_the_body_changes() {
    let cpp = cpp(concat!(
        "var by: [0..3]List<Int> = [[1, 2], [3], []]\n",
        "for v in 0..3:\n",
        "    let pos = if v % 2 == 0: [0] else: by[v]\n",
        "    by[v].push(5)\n",
        "    print(pos.len)\n",
    ));
    assert!(!cpp.contains("& pos = "), "{cpp}");
}

#[test]
fn a_comprehension_reserves_its_source_size() {
    let cpp = cpp(concat!(
        "fn flip(xs: List<Int>) -> List<Int> = [9 - x for x in xs.rev]\n",
        "let n: 1..=9 = read()\n",
        "let sq = [i * i for i in 0..n]\n",
        "print(flip(sq).len, [x for x in sq if x > 1].len)\n",
    ));
    assert!(cpp.contains(".reserve(xs.size());"), "{cpp}");
    assert!(
        cpp.contains(".reserve((size_t)std::max<ll>(0, n - 0));"),
        "{cpp}"
    );
    assert_eq!(cpp.matches(".reserve(").count(), 2, "{cpp}");
}

#[test]
fn a_yields_fn_run_as_pushes_reserves_no_list() {
    let cpp = cpp(concat!(
        "fn walk(xs: List<Int>) -> yields Int:\n",
        "    for x in xs:\n",
        "        yield x\n",
        "fn total(xs: List<Int>, w: Int -> Int) -> Int:\n",
        "    var t = 0\n",
        "    for x in walk(xs): t += w(x)\n",
        "    t\n",
        "print(total([3, 1, 2], x => x * 2))\n",
    ));
    let each = cpp.rsplit("int cpl_each_walk(").next().unwrap();
    let body = each.split("\n}\n").next().unwrap();
    assert!(body.contains("cpl_f(x)"), "{cpp}");
    assert!(!body.contains(".reserve("), "{cpp}");
}

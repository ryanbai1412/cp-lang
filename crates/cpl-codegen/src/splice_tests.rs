//! A loop over a closure that builds a list runs the closure's body in
//! place (`crate::splice`), and `crate::fuse` then runs the loop's body at
//! each push, also when the loop stops early.

use crate::tests::cpp;

fn fn_text<'a>(cpp: &'a str, head: &str) -> &'a str {
    let start = cpp.find(head).unwrap_or_else(|| panic!("{head} in {cpp}"));
    let rest = &cpp[start..];
    &rest[..rest.find("\n}\n").expect("the fn ends")]
}

#[test]
fn bfs_over_a_filtered_adj4_builds_no_list_per_cell() {
    let cpp = cpp(concat!(
        "let (n, m): (1..=1000, 1..=1000) = read()\n",
        "domain Row = 0..n\n",
        "domain Col = 0..m\n",
        "let g = read<[Row, Col]Char>()\n",
        "let p = bfs(c => c.adj4.filter(q => g[q] != '#'), from: [(Row(0), Col(0))], until: (Row(n - 1), Col(m - 1)))\n",
        "print(p.path((Row(n - 1), Col(m - 1)))?.len ?? -1)\n",
    ));
    let bfs = fn_text(&cpp, "_bfs_");
    assert!(bfs.contains("cpl_each_"), "{bfs}");
    assert!(!bfs.contains("cpl_fn0("), "{bfs}");
    assert!(
        !bfs.contains("for (const std::tuple<int, int>& a :"),
        "{bfs}"
    );
    assert!(bfs.contains("return 1;"), "{bfs}");
}

#[test]
fn a_consumer_that_stops_early_skips_the_rest_of_a_producer_that_cannot_trap() {
    let cpp = cpp(concat!(
        "let (n, m): (1..=10, 1..=10) = read()\n",
        "domain Col = 0..m\n",
        "let ys = read<[Col]Int>()\n",
        "fn first_over(f: Int -> List<Int>, cap: Int) -> Int:\n",
        "    var seen = 0\n",
        "    for y in f(cap):\n",
        "        seen += y\n",
        "        if seen > cap: break\n",
        "    seen\n",
        "print(first_over(c => [ys[j] + c for j in Col], n))\n",
    ));
    let f = fn_text(&cpp, "ll first_over(");
    assert!(!f.contains("std::vector"), "{f}");
    assert!(f.contains("goto brk_"), "{f}");
}

#[test]
fn a_producer_that_may_trap_is_not_cut_short() {
    let cpp = cpp(concat!(
        "let n: 1..=10 = read()\n",
        "let xs = read<List<Int>>()\n",
        "fn first_over(f: Int -> List<Int>, cap: Int) -> Int:\n",
        "    var seen = 0\n",
        "    for y in f(cap):\n",
        "        seen += y\n",
        "        if seen > cap: break\n",
        "    seen\n",
        "print(first_over(k => [xs[i] * 2 for i in 0..k], n))\n",
    ));
    let f = fn_text(&cpp, "ll first_over(");
    assert!(f.contains("std::vector<ll>"), "{f}");
}

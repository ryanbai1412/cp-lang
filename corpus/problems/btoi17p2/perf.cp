// Tuned: costs are 30-bit Ints with `none` = 1 << 30 for no road (a route costs at most
//   1e4 gaps * 1e4) instead of `?Int`, so a gap's matrix is a quarter of the size and min-plus
//   skips the optional tests; roads go straight into their gap as they are read.
//
// Source: BOI 2017 P2 (Toll), https://dmoj.ca/problem/btoi17p2
// limits: 1000 ms, 1024 MB
// Problem: n <= 5e4 places, m one-way roads with tolls, each from a to b with
//   b / k = a / k + 1 (k <= 5). For o <= 1e4 orders (a, b) print the cheapest toll from a
//   to b, or -1.
// Solution: roads only join consecutive blocks of k places, so each gap between blocks is a
//   k x k min-plus matrix and a route is their product. A segment tree over the gaps answers
//   each order by folding gaps a / k ..= b / k - 1 and reading entry (a % k, b % k).
//   O(k^3 (n / k + o log n)).
// original: corpus/submissions/4456186.cpp
// copies: 2

let (k, n, m, o): (Int, Int, Int, Int) = read()
domain E = 0..m
domain Q = 0..o
domain Slot = 0..k

let layers = ceil_div(n, k)
let num_gaps = if layers > 1: layers - 1 else: 1
domain Gap = 0..num_gaps
let none = 1 << 30
type Mat = [Slot, Slot]0..=1 << 30

fn mat_mul(x: Mat, y: Mat) -> Mat:
    var out: Mat = none
    for i in Slot:
        for mid in Slot:
            let d1 = x[i, mid]
            if d1 < none:
                for j in Slot: out[i, j] = min(out[i, j], d1 + y[mid, j])
    out

let min_plus = Monoid<Mat>(id: [Slot, Slot] (i, j) => if i == j: 0 else: none, op: mat_mul)

var step: [Gap]Mat = none
for _ in E:
    let (a, b, t): (Int, Int, 0..=10_000) = read()
    step[Gap(a / k)][Slot(a % k), Slot(b % k)] = min(step[Gap(a / k)][Slot(a % k), Slot(b % k)], t)

let queries: [Q](a: Int, b: Int) = read()
let st = SegTree<Gap, Mat>(min_plus, step)
for q in Q:
    let (a, b) = (queries[q].a, queries[q].b)
    let (ga, sa) = (a / k, Slot(a % k))
    let (gb, sb) = (b / k, Slot(b % k))
    if ga == gb:
        print(if a == b: 0 else: -1)
    elif ga > gb:
        print(-1)
    else:
        let d = st.fold(Gap(ga)..=Gap(gb - 1))[sa, sb]
        print(if d < none: d else: -1)
